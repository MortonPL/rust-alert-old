use bitflags::bitflags;
use clap::ValueEnum;
use indexmap::IndexMap;

pub const BLOWFISH_KEY_SIZE: usize = 80;
pub const CHECKSUM_SIZE: usize = 20;
pub const LMD_KEY_TD: i32 = 0x54C2D545;
pub const LMD_KEY_TS: i32 = 0x366E051F;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown LMD version found in the LMD: {0}")]
    UnknownLMDVersion(u32),
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
    pub blowfish_key: Option<[u8; BLOWFISH_KEY_SIZE]>,
    pub checksum: Option<[u8; CHECKSUM_SIZE]>,
    /// Leftover bytes after the last file in the body.
    pub residue: Vec<u8>,
    pub lmd: Option<LocalMixDatabaseInfo>,
}

impl From<u16> for MixHeaderFlags {
    fn from(value: u16) -> Self {
        Self::from_bits(value).unwrap()
    }
}

impl From<u16> for MixHeaderExtraFlags {
    fn from(value: u16) -> Self {
        Self::from_bits(value).unwrap()
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
