//! MIX structures and manipulation.

use std::{fs::read, path::Path};

use bitflags::bitflags;
use indexmap::IndexMap;

use crate::core::{crc, GameEnum};
use crate::mix::{db::LocalMixDatabaseInfo, io::LMD_HEADER_SIZE};

/// Size of a Blowfish key used in MIX encryption.
pub const BLOWFISH_KEY_SIZE: usize = 56;
/// Size of a MIX checksum.
pub const CHECKSUM_SIZE: usize = 20;
/// MIX index key for "local mix database.dat" for TD/RA mixes.
pub const LMD_KEY_TD: i32 = 0x54C2D545;
/// MIX index key for "local mix database.dat" for TS/FS/RA2/YR mixes.
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
    /// MIX header flags containing information about encryption/checksum.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct MixHeaderFlags: u16 {
        const NONE = 0x0000;
        const CHECKSUM = 0x0001;
        const ENCRYPTION = 0x0002;

        const _ = !0;
    }

    /// MIX header extra flags, unused in vanilla games.
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

/// A MIX file is an uncompressed file archive used in C&C games up to Yuri's Revenge.
#[derive(Debug, Default)]
pub struct Mix {
    /// Helper field; does the MIX have flags in the header?
    pub is_new_mix: bool,
    /// Contain information whether the MIX is encrypted/checksummed. Used in RA and up.
    pub flags: MixHeaderFlags,
    /// Always zero in vanilla files. Not advised to be non-zero, as some tools may depend on it. Used in RA and up.
    pub extra_flags: MixHeaderExtraFlags,
    /// Map of files in the MIX, indexed by CRC of their names.
    pub files: IndexMap<i32, MixFileEntry>,
    /// Declared MIX body size (not counting the header/index). Should match reality, even if YR seems to ignore it.
    pub body_size: u32,
    /// Optional, decrypted Blowfish key used to encrypt the MIX header. Always 56 bytes long. Used in RA and up.
    pub blowfish_key: Option<BlowfishKey>,
    /// Optional, SHA1 checksum of the entire MIX body. Always 20 bytes long. Used in RA and up.
    pub checksum: Option<[u8; CHECKSUM_SIZE]>,
    /// Leftover bytes after the last file in the body.
    pub residue: Vec<u8>,
    /// Local Mix Database (not vanilla; introduced in XCC) header info.
    pub lmd: Option<LocalMixDatabaseInfo>,
}

impl Mix {
    /// Add a file from path at the end of the MIX. Overwriting a file raises an error.
    pub fn add_file_path(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let data = read(&path)?;
        let len = data.len() as u32;
        let path: &Path = path.as_ref();
        let mut file = MixFileEntry::new(
            data,
            vec![],
            path.file_name()
                .ok_or(Error::NoFileName(path.into()))?
                .to_str()
                .ok_or(Error::OsStrInvalidUnicode)?
                .into(),
        );
        file.index.offset = self.find_last_offset();
        if let Some(f) = self.files.insert(file.index.id, file) {
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
        if let Some(f) = self.files.insert(file.index.id, file) {
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
        let file = MixFileEntry::new(
            data,
            vec![],
            path.file_name()
                .ok_or(Error::NoFileName(path.into()))?
                .to_str()
                .ok_or(Error::OsStrInvalidUnicode)?
                .into(),
        );
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
            lmd.num_names = self
                .files
                .values()
                .fold(0, |acc, f| acc + if f.name.is_some() { 1 } else { 0 });
            lmd.size = LMD_HEADER_SIZE as u32
                + self.files.values().fold(0, |acc, f| {
                    acc + 1
                        + f.name
                            .as_ref()
                            .map(|s| s.len() as u32)
                            .unwrap_or_default()
                })
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
            lmd.num_names = self
                .files
                .values()
                .fold(0, |acc, f| acc + if f.name.is_some() { 1 } else { 0 });
            lmd.size = LMD_HEADER_SIZE as u32
                + self.files.values().fold(0, |acc, f| {
                    acc + 1
                        + f.name
                            .as_ref()
                            .map(|s| s.len() as u32)
                            .unwrap_or_default()
                })
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
        self.files
            .values()
            .max_by_key(|f| f.index.offset)
            .map_or(0, |f| f.index.offset + f.index.size)
    }
}

/// A MIX file entry contains an index entry used for identification, actual body
/// and residue bytes an optional name obtained from LMD/GMD.
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
        MixFileEntry {
            index: MixIndexEntry {
                id: crc(&name, GameEnum::YR),
                offset: 0,
                size: body.len() as u32,
            },
            body,
            residue,
            name: Some(name),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone().unwrap_or(format!("{:X}", self.index.id))
    }
}

/// A MIX index entry identifies and localizes a single file in the MIX body.
#[derive(Debug, Default)]
pub struct MixIndexEntry {
    pub id: i32,
    pub offset: u32,
    pub size: u32,
}
