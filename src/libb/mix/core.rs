//! MIX structures and manipulation.

use std::{fs::read, path::Path};

use bitflags::bitflags;
use indexmap::IndexMap;
use sha1::{Digest, Sha1};

use crate::core::{crc, GameEnum};

/// Size of a Blowfish key used in MIX encryption.
pub const BLOWFISH_KEY_SIZE: usize = 56;
/// Size of a MIX checksum.
pub const CHECKSUM_SIZE: usize = 20;
/// MIX index key for "local mix database.dat" for TD/RA mixes.
pub const LMD_KEY_TD: i32 = 0x54C2D545;
/// MIX index key for "local mix database.dat" for TS/FS/RA2/YR mixes.
pub const LMD_KEY_TS: i32 = 0x366E051F;

pub type BlowfishKey = [u8; BLOWFISH_KEY_SIZE];
pub type Checksum = [u8; CHECKSUM_SIZE];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Unknown LMD version found in the LMD: {0}")]
    UnknownLMDVersion(u32),
    #[error("Path {0} doesn't point to a file")]
    NoFileName(Box<Path>),
    #[error("Attempted to overwrite file {0:?}, which is not allowed")]
    FileOverwrite(MixIndexEntry),
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
        Self::from_bits(value).unwrap_or_else(|| unreachable!())
    }
}

impl From<MixHeaderFlags> for u16 {
    fn from(value: MixHeaderFlags) -> Self {
        value.bits()
    }
}

impl From<u16> for MixHeaderExtraFlags {
    fn from(value: u16) -> Self {
        Self::from_bits(value).unwrap_or_else(|| unreachable!())
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
    pub is_new_format: bool,
    /// Contain information whether the MIX is encrypted/checksummed. Used in RA and up.
    pub flags: MixHeaderFlags,
    /// Always zero in vanilla files. Not advised to be non-zero, as some tools may depend on it. Used in RA and up.
    pub extra_flags: MixHeaderExtraFlags,
    /// Map of files in the MIX, indexed by CRC of their names.
    pub index: IndexMap<i32, MixIndexEntry>,
    /// BLOB MIX body.
    pub body: Vec<u8>,
    /// Declared MIX body size (not counting the header/index). Should match reality, even if YR seems to ignore it.
    pub declared_body_size: u32,
    /// Optional, decrypted Blowfish key used to encrypt the MIX header. Always 56 bytes long. Used in RA and up.
    pub blowfish_key: Option<BlowfishKey>,
    /// Optional, SHA1 checksum of the entire MIX body. Always 20 bytes long. Used in RA and up.
    pub checksum: Option<Checksum>,
}

impl Mix {
    /// Get file contents by ID.
    pub fn get_file(&self, id: i32) -> Option<&[u8]> {
        self.index
            .get(&id)
            .map(|f| &self.body[(f.offset as usize)..(f.offset as usize + f.size as usize)])
    }

    /// Get mutable file contents by ID.
    pub fn get_file_mut(&mut self, id: i32) -> Option<&mut [u8]> {
        self.index
            .get(&id)
            .map(|f| &mut self.body[(f.offset as usize)..(f.offset as usize + f.size as usize)])
    }

    /// Add a file from raw data at the end of the MIX. Overwriting a file raises an error.
    pub fn add_file_raw(&mut self, data: Vec<u8>, id: i32) -> Result<()> {
        let size = data.len() as u32;
        let file = MixIndexEntry::new(id, self.find_last_offset(), size);
        if let Some(f) = self.index.insert(file.id, file) {
            Err(Error::FileOverwrite(f))?
        }
        self.body.extend(data);

        Ok(())
    }

    /// Add a file from path at the end of the MIX. Overwriting a file may raise an error.
    pub fn add_file_path(&mut self, path: impl AsRef<Path>, crc_version: GameEnum, allow_overwrite: bool) -> Result<()> {
        let mut data = read(&path)?;
        let path: &Path = path.as_ref();

        let id = crc(
            path.file_name()
                .ok_or(Error::NoFileName(path.into()))?
                .to_str()
                .ok_or(Error::OsStrInvalidUnicode)?,
            crc_version
        );
        let offset = self.get_body_size() as u32;
        let size = data.len() as u32;

        self.body.append(&mut data);
        let file = MixIndexEntry::new(id, offset, size);
        if let Some(f) = self.index.insert(file.id, file) {
            if !allow_overwrite {
                Err(Error::FileOverwrite(f))?
            }
        }

        Ok(())
    }

    /// Removes the file with given ID from the MIX index. Note: in order to fully remove a file with its contents, use `recalc()` afterwards.
    pub fn remove_file(&mut self, id: i32) {
        self.index.remove(&id);
    }

    /// Recalculate the MIX index and compact the MIX. Previous order of file offsets might not be preserved.
    /// Any data not covered by indexed files willbe lost.
    pub fn recalc(&mut self) {



        todo!() // TODO
    }

    /// Sort MIX index by ascending ID.
    pub fn sort_by_id(&mut self) {
        self.index.sort_keys();
    }

    /// Sort MIX index by ascending offset.
    pub fn sort_by_offset(&mut self) {
        self.index.sort_by(|_, f1, _, f2| f1.offset.cmp(&f2.offset));
    }

    /// Get the MIX body SHA1 checksum if available.
    pub fn get_checksum(&self) -> Option<&Checksum> {
        self.checksum.as_ref()
    }

    /// Calculate and set the MIX body SHA1 checksum.
    pub fn calc_checksum(&mut self) {
        let mut hasher = Sha1::new();
        hasher.update(&self.body);
        self.checksum = Some(hasher.finalize().into());
        self.flags.insert(MixHeaderFlags::CHECKSUM);
    }

    /// Set (if Some) or reset (if None) the MIX checksum. Header flags are set appropriately.
    pub fn set_checksum(&mut self, checksum: Option<Checksum>) {
        self.checksum = checksum;
        if checksum.is_some() {
            self.flags.insert(MixHeaderFlags::CHECKSUM);
        } else {
            self.flags.remove(MixHeaderFlags::CHECKSUM);
        }
    }

    /// Get the MIX Blowfish key if available.
    pub fn get_blowfish_key(&self) -> Option<&BlowfishKey> {
        self.blowfish_key.as_ref()
    }

    /// Set (if Some) or reset (if None) the MIX Blowfish key. Header flags are set appropriately.
    pub fn set_blowfish_key(&mut self, blowfish_key: Option<BlowfishKey>) {
        self.blowfish_key = blowfish_key;
        if blowfish_key.is_some() {
            self.flags.insert(MixHeaderFlags::ENCRYPTION);
        } else {
            self.flags.remove(MixHeaderFlags::ENCRYPTION);
        }
    }

    /// Check if the MIX is compact, aka if its body contains no extra data beyond files in the index.
    ///
    /// This method sorts the MIX index by offset.
    pub fn is_compact(&mut self) -> bool {
        self.sort_by_offset();
        let mut ptr = 0;
        for file in self.index.values() {
            // Empty space.
            if file.offset > ptr {
                return false;
            }
            // Compact or overlapping files.
            ptr += file.size - (ptr - file.offset);
        }
        return true;
    }

    /// Get MIX index size in bytes.
    pub fn get_index_size(&self) -> usize {
        self.index.len() * std::mem::size_of::<MixIndexEntry>()
    }

    /// Get MIX body size in bytes.
    pub fn get_body_size(&self) -> usize {
        self.body.len()
    }

    /// Find the offset *after* the last file in the MIX.
    fn find_last_offset(&self) -> u32 {
        self.index
            .values()
            .max_by_key(|f| f.offset)
            .map_or(0, |f| f.offset + f.size)
    }
}

/// A MIX index entry identifies and localizes a single file in the MIX body.
#[derive(Debug, Default)]
pub struct MixIndexEntry {
    pub id: i32,
    pub offset: u32,
    pub size: u32,
}

impl MixIndexEntry {
    pub fn new(id: i32, offset: u32, size: u32) -> Self {
        Self { id, offset, size }
    }
}
