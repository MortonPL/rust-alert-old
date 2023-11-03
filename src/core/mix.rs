use std::{
    fs::read,
    path::Path,
};

use bitflags::bitflags;
use clap::ValueEnum;
use indexmap::IndexMap;

use crate::core::{
    crc::{crc, GameEnum},
    mix_io::LMD_HEADER_SIZE,
};

pub const BLOWFISH_KEY_SIZE: usize = 56;
pub const CHECKSUM_SIZE: usize = 20;
pub const LMD_KEY_TD: i32 = 0x54C2D545;
pub const LMD_KEY_TS: i32 = 0x366E051F;

pub type BlowfishKey = [u8; BLOWFISH_KEY_SIZE];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Unknown LMD version found in the LMD: {0}")]
    UnknownLMDVersion(u32),
    #[error("Path {0} doesn't point to a file")]
    NoFileName(Box<Path>),
    #[error("Attempted to overwrite file {0:?}, which is not allowed")]
    FileOverwrite(MixFileEntry),
    #[error("Failed to convert a file path to a string, because it's not valid Unicode")]
    OsStrInvalidUnicode,
}

type Result<T> = std::result::Result<T, Error>;

bitflags! {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct MixHeaderFlags: u16 {
        const NONE = 0x0000;
        const CHECKSUM = 0x0001;
        const ENCRYPTION = 0x0002;

        const _ = !0;
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct MixHeaderExtraFlags: u16 {
        const NONE = 0x0000;

        const _ = !0;
    }
}

impl From<u16> for MixHeaderFlags {
    fn from(value: u16) -> Self {
        Self::from_bits(value).unwrap()
    }
}

impl From<MixHeaderFlags> for u16 {
    fn from(value: MixHeaderFlags) -> Self {
        value.bits()
    }
}

impl From<u16> for MixHeaderExtraFlags {
    fn from(value: u16) -> Self {
        Self::from_bits(value).unwrap()
    }
}

impl From<MixHeaderExtraFlags> for u16 {
    fn from(value: MixHeaderExtraFlags) -> Self {
        value.bits()
    }
}

/// CSF format version.
#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
#[repr(u32)]
pub enum LMDVersionEnum {
    TD = 0,
    RA = 1,
    TS = 2,
    #[default]
    YR = 5,
}

impl TryFrom<u32> for LMDVersionEnum {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            x if x == LMDVersionEnum::TD as u32 => Ok(LMDVersionEnum::TD),
            x if x == LMDVersionEnum::RA as u32 => Ok(LMDVersionEnum::RA),
            x if x == LMDVersionEnum::TS as u32 => Ok(LMDVersionEnum::TS),
            x if x == LMDVersionEnum::YR as u32 => Ok(LMDVersionEnum::YR),
            x => Err(Error::UnknownLMDVersion(x)),
        }
    }
}

impl TryFrom<LMDVersionEnum> for u32 {
    type Error = Error;

    fn try_from(value: LMDVersionEnum) -> Result<Self> {
        Ok(value as u32)
    }
}

#[derive(Debug, Default)]
pub struct Mix {
    /// Helper field; does the MIX have flags in the header?
    pub is_new_mix: bool,
    pub flags: MixHeaderFlags,
    /// Not advised to be non-zero, as some tools may depend on it.
    pub extra_flags: MixHeaderExtraFlags,
    pub files: IndexMap<i32, MixFileEntry>,
    pub body_size: u32,
    pub blowfish_key: Option<BlowfishKey>,
    pub checksum: Option<[u8; CHECKSUM_SIZE]>,
    /// Leftover bytes after the last file in the body.
    pub residue: Vec<u8>,
    pub lmd: Option<LocalMixDatabaseInfo>,
}

impl Mix {
    /// Add a file from path at the end of the MIX. Overwriting a file raises an error.
    pub fn add_file_path(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let data = read(&path)?;
        let len = data.len() as u32;
        let path: &Path = path.as_ref();
        let mut file = MixFileEntry::new(data, vec![], path.file_name().ok_or(Error::NoFileName(path.into()))?.to_str().ok_or(Error::OsStrInvalidUnicode)?.into());
        file.index.offset = self.find_last_offset();
        if let Some(f) =  self.files.insert(file.index.id, file) {
            Err(Error::FileOverwrite(f))?
        }
        self.body_size += len;
        Ok(())
    }

    /// Add a file from raw data at the end of the MIX. Overwriting a file raises an error.
    pub fn add_file_raw(&mut self, data: Vec<u8>, name: String) -> Result<()> {
        let len = data.len() as u32;
        let mut file = MixFileEntry::new(data, vec![], name);
        file.index.offset = self.find_last_offset();
        if let Some(f) =  self.files.insert(file.index.id, file) {
            Err(Error::FileOverwrite(f))?
        }
        self.body_size += len;
        Ok(())
    }

    /// Forcibly add a file from path at the end of the MIX. Overwriting a file may raise an error.
    /// MIX integrity such as body size, index offsets, countigous file data are not guaranteed.
    /// Added file offset will not be set.
    pub fn force_file_path(&mut self, path: impl AsRef<Path>, allow_overwrite: bool) -> Result<()> {
        let data = read(&path)?;
        let path: &Path = path.as_ref();
        let file = MixFileEntry::new(data, vec![], path.file_name().ok_or(Error::NoFileName(path.into()))?.to_str().ok_or(Error::OsStrInvalidUnicode)?.into());
        if let Some(f) = self.files.insert(file.index.id, file) {
            if !allow_overwrite {
                Err(Error::FileOverwrite(f))?
            }
        }
        Ok(())
    }

    /// Recalculate the MIX index and LMD. Previous order of file offsets might not be preserved.
    /// Compactness (or lack thereof) is preserved.
    pub fn recalc(&mut self) {
        let mut offset = 0u32;
        self.body_size = 0;
        if let Some(lmd) = &mut self.lmd {
            lmd.num_names = self.files.values().fold(0, |acc, f| acc + if f.name.is_some() {1} else {0});
            lmd.size = LMD_HEADER_SIZE as u32 + self.files.values().fold(0, |acc, f| acc + 1 + f.name.as_ref().and_then(|s| Some(s.len() as u32)).unwrap_or_default())
        }
        for file in self.files.values_mut() {
            offset += file.residue.len() as u32;
            file.index.offset = offset;
            offset += file.index.size;
            self.body_size += file.index.size;
        }
        self.body_size += self.residue.len() as u32;
    }

    /// Recalculate the MIX index, LMD and compact the MIX. Previous order of file offsets might not be preserved.
    pub fn recalc_compact(&mut self) {
        let mut offset = 0u32;
        self.body_size = 0;
        if let Some(lmd) = &mut self.lmd {
            lmd.num_names = self.files.values().fold(0, |acc, f| acc + if f.name.is_some() {1} else {0});
            lmd.size = LMD_HEADER_SIZE as u32 + self.files.values().fold(0, |acc, f| acc + 1 + f.name.as_ref().and_then(|s| Some(s.len() as u32)).unwrap_or_default())
        }
        for file in self.files.values_mut() {
            file.index.offset = offset;
            offset += file.index.size;
            self.body_size += file.index.size;
            file.residue.clear();
        }
    }

    /// Find the offset *after* the last file ends.
    fn find_last_offset(&self) -> u32 {
        self.files.values().max_by_key(|f| f.index.offset).map_or(0, |f| f.index.offset + f.index.size)
    }
}

#[derive(Debug, Default)]
pub struct MixFileEntry {
    pub index: MixIndexEntry,
    pub body: Vec<u8>,
    pub residue: Vec<u8>,
    pub name: Option<String>,
}

impl MixFileEntry {
    pub fn new(body: Vec<u8>, residue: Vec<u8>, name: String) -> Self {
        // TODO parametrized game ver
        MixFileEntry { index: MixIndexEntry { id: crc(&name, GameEnum::YR), offset: 0, size: body.len() as u32 }, body, residue, name: Some(name) }
    }

    pub fn get_name(&self) -> String {
        self.name.clone().unwrap_or(format!("{:X}", self.index.id))
    }
}

#[derive(Debug, Default)]
/// A MIX index entry identifies and localizes a single file in the MIX body.
pub struct MixIndexEntry {
    pub id: i32,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Default)]
/// LMD header info.
pub struct LocalMixDatabaseInfo {
    pub num_names: u32,
    pub version: LMDVersionEnum,
    pub size: u32,
}
