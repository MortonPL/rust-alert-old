use bitflags::bitflags;
use indexmap::IndexMap;

pub const BLOWFISH_KEY_SIZE: usize = 80;
pub const CHECKSUM_SIZE: usize = 20;
pub const LMD_KEY_TD: i32 = 0x54C2D545;
pub const LMD_KEY_TS: i32 = 0x366E051F;

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

#[derive(Debug, Default)]
pub struct Mix {
    pub flags: MixHeaderFlags,
    pub extra_flags: MixHeaderExtraFlags,
    pub is_new_mix: bool,
    pub files: IndexMap<i32, MixFileEntry>,
    blowfish_key: Option<[u8; BLOWFISH_KEY_SIZE]>,
    checksum: Option<[u8; CHECKSUM_SIZE]>,
    pub residue: Vec<u8>,
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

#[derive(Debug, Default)]
/// A MIX index entry identifies and localizes a single file in the MIX body.
pub struct MixIndexEntry {
    pub id: i32,
    pub offset: u32,
    pub size: u32,
}
