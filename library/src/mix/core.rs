//! MIX structures and manipulation.

use std::{fs::read, path::Path};

use indexmap::IndexMap;
use sha1::{Digest, Sha1};

use crate::core::{crc, GameEnum};
use crate::utils::{path_to_filename, PathToStringError};

/// Size of a Blowfish key used in MIX encryption.
pub const BLOWFISH_KEY_SIZE: usize = 56;
/// Size of a MIX checksum.
pub const CHECKSUM_SIZE: usize = 20;
/// MIX index key for "local mix database.dat" for TD/RA mixes.
pub const LMD_KEY_TD: i32 = 0x54C2D545;
/// MIX index key for "local mix database.dat" for TS/FS/RA2/YR mixes.
pub const LMD_KEY_TS: i32 = 0x366E051F;

/// A 56 byte MIX Blowfish key.
pub type BlowfishKey = [u8; BLOWFISH_KEY_SIZE];
/// A 20 byte MIX SHA1 checksum.
pub type Checksum = [u8; CHECKSUM_SIZE];

/// The error type for operations on MIX files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An [`std::io::Error`].
    #[error("{0}")]
    IO(#[from] std::io::Error),
    /// The LMD in this MIX has an unknown version declared in its header.
    #[error("Unknown LMD version found in the LMD: {0}")]
    UnknownLMDVersion(u32),
    /// A file inside the MIX would be overwritten.
    #[error("Attempted to overwrite file {0:?}, which is not allowed")]
    FileOverwrite(MixIndexEntry),
    /// A [`PathToStringError`].
    #[error("{0}")]
    PathToStringError(#[from] PathToStringError),
}

type Result<T> = std::result::Result<T, Error>;

#[cfg(not(tarpaulin_include))]
mod flags {
    bitflags::bitflags! {
        /// MIX header flags containing information about encryption/checksum.
        #[derive(Clone, Copy, Debug, Default)]
        pub struct MixHeaderFlags: u16 {
            /// Plain MIX.
            const NONE = 0x0000;
            /// A MIX with a SHA1 checksum at the end.
            const CHECKSUM = 0x0001;
            /// A MIX with a Blowfish encrypted header and index.
            const ENCRYPTION = 0x0002;
            ///
            const _ = !0;
        }
    
        /// MIX header extra flags, unused in vanilla games.
        #[derive(Clone, Copy, Debug, Default)]
        pub struct MixHeaderExtraFlags: u16 {
            /// Plain MIX.
            const NONE = 0x0000;
            ///
            const _ = !0;
        }
    }
}

pub use flags::*;


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
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let res = mix.add_file_raw(vec![0], 1, false);
    /// assert!(res.is_ok());
    ///
    /// let res = mix.get_file(1);
    /// assert!(res.is_some());
    /// assert_eq!(res.unwrap(), &[0]);
    /// ```
    pub fn get_file(&self, id: i32) -> Option<&[u8]> {
        self.index
            .get(&id)
            .map(|f| &self.body[(f.offset as usize)..(f.offset as usize + f.size as usize)])
    }

    /// Get mutable file contents by ID.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let res = mix.add_file_raw(vec![0], 1, false);
    /// assert!(res.is_ok());
    ///
    /// let res = mix.get_file_mut(1);
    /// assert!(res.is_some());
    /// assert_eq!(res.unwrap(), &[0]);
    /// ```
    pub fn get_file_mut(&mut self, id: i32) -> Option<&mut [u8]> {
        self.index
            .get(&id)
            .map(|f| &mut self.body[(f.offset as usize)..(f.offset as usize + f.size as usize)])
    }

    /// Add a file at the end of the MIX, using raw data. Overwriting a file may raise an error
    /// if `allow_overwrite` is false.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::mix::{Mix, Error};
    ///
    /// let mut mix = Mix::default();
    /// let res = mix.add_file_raw(vec![0], 1, false);
    /// assert!(res.is_ok());
    /// let res = mix.add_file_raw(vec![1], 1, false);
    /// assert!(res.is_err());
    /// assert!(matches!(res.unwrap_err(), Error::FileOverwrite(_)));
    /// let res = mix.add_file_raw(vec![2], 1, true);
    /// assert!(res.is_ok());
    /// ```
    pub fn add_file_raw(
        &mut self,
        mut data: Vec<u8>,
        id: i32,
        allow_overwrite: bool,
    ) -> Result<()> {
        let size = data.len() as u32;
        let file = MixIndexEntry::new(id, self.find_last_offset(), size);
        if let Some(f) = self.index.insert(file.id, file) {
            if !allow_overwrite {
                Err(Error::FileOverwrite(f))?
            }
        }
        self.body.append(&mut data);

        Ok(())
    }

    /// Add a file at the end of the MIX, reading it from path. Overwriting a file may raise an error
    /// if `allow_overwrite` is false.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::{mix::{Error, Mix}, core::GameEnum};
    ///
    /// let path = std::path::Path::new("../test_data/example.csf");
    /// let mut mix = Mix::default();
    /// let res = mix.add_file_from_path(path, GameEnum::YR, false);
    /// assert!(res.is_ok());
    /// let res = mix.add_file_from_path(path, GameEnum::YR, false);
    /// assert!(res.is_err());
    /// assert!(matches!(res.unwrap_err(), Error::FileOverwrite(_)));
    /// let res = mix.add_file_from_path(path, GameEnum::YR, true);
    /// assert!(res.is_ok());
    /// ```
    pub fn add_file_from_path(
        &mut self,
        path: impl AsRef<Path>,
        crc_version: GameEnum,
        allow_overwrite: bool,
    ) -> Result<()> {
        let mut data = read(&path)?;
        let id = crc(path_to_filename(path)?, crc_version);
        let offset = self.get_body_size() as u32;
        let size = data.len() as u32;

        let file = MixIndexEntry::new(id, offset, size);
        if let Some(f) = self.index.insert(file.id, file) {
            if !allow_overwrite {
                Err(Error::FileOverwrite(f))?
            }
        }
        self.body.append(&mut data);

        Ok(())
    }

    /// Removes the file with given ID from the MIX index.
    /// Note: in order to fully remove a file with its contents, use `recalc()` afterwards.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let _ = mix.add_file_raw(vec![0], 1, false);
    /// assert_eq!(mix.len(), 1);
    /// mix.remove_file(1);
    /// assert_eq!(mix.len(), 0);
    /// ```
    pub fn remove_file(&mut self, id: i32) {
        self.index.shift_remove(&id);
    }

    /// Recalculate the MIX index and compact the MIX. Previous order of file offsets might not be preserved.
    /// Any data not covered by indexed files will be lost.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let _ = mix.add_file_raw(vec![0], 1, false);
    /// let _ = mix.add_file_raw(vec![1], 2, false);
    /// assert_eq!(mix.body, &[0, 1]);
    /// mix.remove_file(1);
    /// assert_eq!(mix.body, &[0, 1]);
    /// mix.recalc();
    /// assert_eq!(mix.body, &[1]);
    /// ```
    pub fn recalc(&mut self) {
        // Dynamic array size magic below: we're deleting useless space between files.
        self.sort_by_offset();
        let mut ptr: i64 = 0;
        let mut drained: i64 = 0;
        for file in self.index.values_mut() {
            // How big is the gap between the end of the previous file and the start of the current one?
            let gap = file.offset as i64 - drained - ptr;
            if gap > 0 {
                self.body.drain(((ptr) as usize)..((ptr + gap) as usize));
                drained += gap;
            }
            ptr += file.size as i64 + gap;
            file.offset -= drained as u32;
        }
        self.sort_by_id();
    }

    /// Sort MIX index by ascending ID.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let _ = mix.add_file_raw(vec![0], 2, false);
    /// let _ = mix.add_file_raw(vec![0, 1], 1, false);
    /// assert_eq!(mix.index.first().unwrap().0, &2);
    /// mix.sort_by_id();
    /// assert_eq!(mix.index.first().unwrap().0, &1);
    /// ```
    pub fn sort_by_id(&mut self) {
        self.index.sort_keys();
    }

    /// Sort MIX index by ascending offset.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let _ = mix.add_file_raw(vec![0], 2, false);
    /// let _ = mix.add_file_raw(vec![0, 1], 1, false);
    /// assert_eq!(mix.index.first().unwrap().0, &2);
    /// mix.sort_by_offset();
    /// assert_eq!(mix.index.first().unwrap().0, &2);
    /// ```
    pub fn sort_by_offset(&mut self) {
        self.index.sort_by(|_, f1, _, f2| f1.offset.cmp(&f2.offset));
    }

    /// Sort MIX index by ascending size of files.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let _ = mix.add_file_raw(vec![0], 2, false);
    /// let _ = mix.add_file_raw(vec![0, 1], 1, false);
    /// assert_eq!(mix.index.first().unwrap().0, &2);
    /// mix.sort_by_size();
    /// assert_eq!(mix.index.first().unwrap().0, &2);
    /// ```
    pub fn sort_by_size(&mut self) {
        self.index.sort_by(|_, f1, _, f2| f1.size.cmp(&f2.size));
    }

    /// Get the MIX body SHA1 checksum if available.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let _ = mix.add_file_raw(vec![0], 1, false);
    /// let res = mix.get_checksum();
    /// assert!(res.is_none());
    /// mix.calc_checksum();
    /// let res = mix.get_checksum();
    /// assert!(res.is_some());
    /// ```
    pub fn get_checksum(&self) -> Option<&Checksum> {
        self.checksum.as_ref()
    }

    /// Calculate and set the MIX body SHA1 checksum.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let _ = mix.add_file_raw(vec![0], 1, false);
    /// let res = mix.get_checksum();
    /// assert!(res.is_none());
    /// mix.calc_checksum();
    /// let res = mix.get_checksum();
    /// assert!(res.is_some());
    /// ```
    pub fn calc_checksum(&mut self) {
        let mut hasher = Sha1::new();
        hasher.update(&self.body);
        self.checksum = Some(hasher.finalize().into());
        self.flags.insert(MixHeaderFlags::CHECKSUM);
        self.is_new_format = true;
    }

    /// Set (if Some) or reset (if None) the MIX checksum. Header flags are set appropriately.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let res = mix.get_checksum();
    /// assert!(res.is_none());
    /// let sha = [0u8; 20];
    /// mix.set_checksum(Some(sha));
    /// let res = mix.get_checksum();
    /// assert!(res.is_some());
    /// mix.set_checksum(None);
    /// let res = mix.get_checksum();
    /// assert!(res.is_none());
    /// ```
    pub fn set_checksum(&mut self, checksum: Option<Checksum>) {
        self.checksum = checksum;
        if checksum.is_some() {
            self.flags.insert(MixHeaderFlags::CHECKSUM);
            self.is_new_format = true;
        } else {
            self.flags.remove(MixHeaderFlags::CHECKSUM);
        }
    }

    /// Get the MIX Blowfish key if available.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let res = mix.get_blowfish_key();
    /// assert!(res.is_none());
    /// let key = [0u8; 56];
    /// mix.set_blowfish_key(Some(key));
    /// let res = mix.get_blowfish_key();
    /// assert!(res.is_some());
    /// mix.set_blowfish_key(None);
    /// let res = mix.get_blowfish_key();
    /// assert!(res.is_none());
    /// ```
    pub fn get_blowfish_key(&self) -> Option<&BlowfishKey> {
        self.blowfish_key.as_ref()
    }

    /// Set (if Some) or reset (if None) the MIX Blowfish key. Header flags are set appropriately.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let res = mix.get_blowfish_key();
    /// assert!(res.is_none());
    /// let key = [0u8; 56];
    /// mix.set_blowfish_key(Some(key));
    /// let res = mix.get_blowfish_key();
    /// assert!(res.is_some());
    /// mix.set_blowfish_key(None);
    /// let res = mix.get_blowfish_key();
    /// assert!(res.is_none());
    /// ```
    pub fn set_blowfish_key(&mut self, blowfish_key: Option<BlowfishKey>) {
        self.blowfish_key = blowfish_key;
        if blowfish_key.is_some() {
            self.flags.insert(MixHeaderFlags::ENCRYPTION);
            self.is_new_format = true;
        } else {
            self.flags.remove(MixHeaderFlags::ENCRYPTION);
        }
    }

    /// Check if the MIX is compact, aka if its body contains no extra data beyond files in the index.
    /// This method sorts the MIX index by offset.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// assert!(mix.is_compact());
    /// let _ = mix.add_file_raw(vec![0], 1, false);
    /// let _ = mix.add_file_raw(vec![0], 2, false);
    /// assert!(mix.is_compact());
    /// mix.remove_file(1);
    /// assert!(!mix.is_compact());
    /// mix.recalc();
    /// assert!(mix.is_compact());
    /// ```
    pub fn is_compact(&mut self) -> bool {
        self.sort_by_offset();
        let mut ptr = 0i64;
        for file in self.index.values() {
            // Empty space.
            if file.offset as i64 > ptr {
                return false;
            }
            // Compact or overlapping files.
            ptr += file.size as i64 - (ptr - file.offset as i64);
        }
        self.body.len() <= ptr as usize
    }

    /// Get number of files.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// assert_eq!(mix.len(), 0);
    /// let _ = mix.add_file_raw(vec![0], 1, false);
    /// assert_eq!(mix.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Get MIX index size in bytes.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// assert_eq!(mix.get_index_size(), 0);
    /// let _ = mix.add_file_raw(vec![0], 1, false);
    /// assert_eq!(mix.get_index_size(), 12);
    /// ```
    pub fn get_index_size(&self) -> usize {
        self.index.len() * std::mem::size_of::<MixIndexEntry>()
    }

    /// Get MIX body size in bytes.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// assert_eq!(mix.get_body_size(), 0);
    /// let _ = mix.add_file_raw(vec![0, 1, 2], 1, false);
    /// assert_eq!(mix.get_body_size(), 3);
    /// ```
    pub fn get_body_size(&self) -> usize {
        self.body.len()
    }

    /// Find the offset *after* the last file in the MIX.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::mix::Mix;
    ///
    /// let mut mix = Mix::default();
    /// let _ = mix.add_file_raw(vec![0, 1, 2], 1, false);
    /// let _ = mix.add_file_raw(vec![3], 2, false);
    /// assert_eq!(mix.find_last_offset(), 4);
    /// ```
    fn find_last_offset(&self) -> u32 {
        self.index
            .values()
            .max_by_key(|f| f.offset + f.size)
            .map_or(0, |f| f.offset + f.size)
    }
}

/// A MIX index entry identifies and localizes a single file in the MIX body.
#[derive(Debug, Default, Clone)]
pub struct MixIndexEntry {
    /// ID / CRC of file name.
    pub id: i32,
    /// Offset from the start of the MIX body.
    pub offset: u32,
    /// Size of this file in bytes.
    pub size: u32,
}

impl MixIndexEntry {
    /// Create a new MIX index entry.
    pub fn new(id: i32, offset: u32, size: u32) -> Self {
        Self { id, offset, size }
    }
}

#[cfg(test)]
mod examples {
    use crate as rust_alert;

    #[test]
    fn get_file() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let res = mix.add_file_raw(vec![0], 1, false);
        assert!(res.is_ok());

        let res = mix.get_file(1);
        assert!(res.is_some());
        assert_eq!(res.unwrap(), &[0]);
    }

    #[test]
    fn get_file_mut() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let res = mix.add_file_raw(vec![0], 1, false);
        assert!(res.is_ok());

        let res = mix.get_file_mut(1);
        assert!(res.is_some());
        assert_eq!(res.unwrap(), &[0]);
    }

    #[test]
    fn add_file_raw() {
        use rust_alert::mix::{Error, Mix};

        let mut mix = Mix::default();
        let res = mix.add_file_raw(vec![0], 1, false);
        assert!(res.is_ok());
        let res = mix.add_file_raw(vec![1], 1, false);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::FileOverwrite(_)));
        let res = mix.add_file_raw(vec![2], 1, true);
        assert!(res.is_ok());
    }

    #[test]
    fn add_file_from_path() {
        use rust_alert::{mix::{Error, Mix}, core::GameEnum};

        let path = std::path::Path::new("../test_data/example.csf");
        let mut mix = Mix::default();
        let res = mix.add_file_from_path(path, GameEnum::YR, false);
        assert!(res.is_ok());
        let res = mix.add_file_from_path(path, GameEnum::YR, false);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), Error::FileOverwrite(_)));
        let res = mix.add_file_from_path(path, GameEnum::YR, true);
        assert!(res.is_ok());
    }

    #[test]
    fn remove_file() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let _ = mix.add_file_raw(vec![0], 1, false);
        assert_eq!(mix.len(), 1);
        mix.remove_file(1);
        assert_eq!(mix.len(), 0);
    }

    #[test]
    fn recalc() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let _ = mix.add_file_raw(vec![0], 1, false);
        let _ = mix.add_file_raw(vec![1], 2, false);
        assert_eq!(mix.body, &[0, 1]);
        mix.remove_file(1);
        assert_eq!(mix.body, &[0, 1]);
        mix.recalc();
        assert_eq!(mix.body, &[1]);
    }

    #[test]
    fn sort_by_id() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let _ = mix.add_file_raw(vec![0], 2, false);
        let _ = mix.add_file_raw(vec![0, 1], 1, false);
        assert_eq!(mix.index.first().unwrap().0, &2);
        mix.sort_by_id();
        assert_eq!(mix.index.first().unwrap().0, &1);
    }

    #[test]
    fn sort_by_offset() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let _ = mix.add_file_raw(vec![0], 2, false);
        let _ = mix.add_file_raw(vec![0, 1], 1, false);
        assert_eq!(mix.index.first().unwrap().0, &2);
        mix.sort_by_offset();
        assert_eq!(mix.index.first().unwrap().0, &2);
    }

    #[test]
    fn sort_by_size() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let _ = mix.add_file_raw(vec![0], 2, false);
        let _ = mix.add_file_raw(vec![0, 1], 1, false);
        assert_eq!(mix.index.first().unwrap().0, &2);
        mix.sort_by_size();
        assert_eq!(mix.index.first().unwrap().0, &2);
    }

    #[test]
    fn get_checksum() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let _ = mix.add_file_raw(vec![0], 1, false);
        let res = mix.get_checksum();
        assert!(res.is_none());
        mix.calc_checksum();
        let res = mix.get_checksum();
        assert!(res.is_some());
    }

    #[test]
    fn set_checksum() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let res = mix.get_checksum();
        assert!(res.is_none());
        let sha = [0u8; 20];
        mix.set_checksum(Some(sha));
        let res = mix.get_checksum();
        assert!(res.is_some());
        mix.set_checksum(None);
        let res = mix.get_checksum();
        assert!(res.is_none());
    }

    #[test]
    fn get_blowfish_key() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let res = mix.get_blowfish_key();
        assert!(res.is_none());
        let key = [0u8; 56];
        mix.set_blowfish_key(Some(key));
        let res = mix.get_blowfish_key();
        assert!(res.is_some());
        mix.set_blowfish_key(None);
        let res = mix.get_blowfish_key();
        assert!(res.is_none());
    }

    #[test]
    fn is_compact() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        assert!(mix.is_compact());
        let _ = mix.add_file_raw(vec![0], 1, false);
        let _ = mix.add_file_raw(vec![0], 2, false);
        assert!(mix.is_compact());
        mix.remove_file(1);
        assert!(!mix.is_compact());
        mix.recalc();
        assert!(mix.is_compact());
    }

    #[test]
    fn len() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        assert_eq!(mix.len(), 0);
        let _ = mix.add_file_raw(vec![0], 1, false);
        assert_eq!(mix.len(), 1);
    }

    #[test]
    fn get_index_size() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        assert_eq!(mix.get_index_size(), 0);
        let _ = mix.add_file_raw(vec![0], 1, false);
        assert_eq!(mix.get_index_size(), 12);
    }

    #[test]
    fn get_body_size() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        assert_eq!(mix.get_body_size(), 0);
        let _ = mix.add_file_raw(vec![0, 1, 2], 1, false);
        assert_eq!(mix.get_body_size(), 3);
    }

    #[test]
    fn find_last_offset() {
        use rust_alert::mix::Mix;

        let mut mix = Mix::default();
        let _ = mix.add_file_raw(vec![0, 1, 2], 1, false);
        let _ = mix.add_file_raw(vec![3], 2, false);
        assert_eq!(mix.find_last_offset(), 4);
    }
}

#[cfg(test)]
mod coverage {
    use crate::mix::{MixHeaderFlags, MixHeaderExtraFlags};

    #[test]
    fn header_flags_from() {
        assert!(MixHeaderFlags::from(0x0000).contains(MixHeaderFlags::NONE));
        assert!(MixHeaderFlags::from(0x0001).contains(MixHeaderFlags::CHECKSUM));
        assert!(MixHeaderFlags::from(0x0002).contains(MixHeaderFlags::ENCRYPTION));

        assert!(MixHeaderExtraFlags::from(0x0000).contains(MixHeaderExtraFlags::NONE));
    }

    #[test]
    fn header_flags_into() {
        assert_eq!(u16::from(MixHeaderFlags::NONE), 0x0000);
        assert_eq!(u16::from(MixHeaderFlags::CHECKSUM), 0x0001);
        assert_eq!(u16::from(MixHeaderFlags::ENCRYPTION), 0x0002);

        assert_eq!(u16::from(MixHeaderExtraFlags::NONE), 0x0000);
    }
}
