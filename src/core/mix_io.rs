use std::{
    io::{Read, Write},
    mem::size_of,
};

use crate::core::mix::{
    Mix, MixFileEntry, MixHeaderExtraFlags, MixHeaderFlags, MixIndexEntry, BLOWFISH_KEY_SIZE,
    LMD_KEY_TD, LMD_KEY_TS,
};

pub const LMD_PREFIX: &[u8; 32] = b"XCC by Olaf van der Spek\x1a\x04\x17\x27\x10\x19\x80\x00";
pub const LMD_HEADER_SIZE: usize = LMD_PREFIX.len() + 20;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Attempted to read the LMD, but the prefix didn't match")]
    InvalidLMDPrefix,
}

type Result<T> = std::result::Result<T, Error>;

pub struct MixReader {}

impl MixReader {
    pub fn read_file(reader: &mut dyn Read) -> Result<Mix> {
        let mut mix = Mix::default();
        let (is_new_mix, extra_flags, flags, num_files, body_size, remaining) =
            Self::read_header(reader)?;
        mix.is_new_mix = is_new_mix;
        mix.flags = flags;
        mix.extra_flags = extra_flags;

        let index: Vec<MixIndexEntry>;
        if flags.contains(MixHeaderFlags::ENCRYPTION) {
            index = Self::read_index_encrypted(reader, num_files, remaining)?;
        } else {
            index = Self::read_index(reader, num_files)?;
        }

        let (files, pos) = Self::read_bodies(reader, index)?;
        for file in files {
            mix.files.insert(file.index.id, file);
        }

        let residue = body_size - pos;
        let mut buf: Vec<u8> = Vec::with_capacity(residue as usize);
        reader.read_exact(&mut buf)?;
        mix.residue = buf;

        // LMD strings expect order by ID.
        mix.files.sort_keys();
        Self::apply_lmd(&mut mix)?;

        Ok(mix)
    }

    /// Read the MIX header.
    pub fn read_header(
        reader: &mut dyn Read,
    ) -> Result<(bool, MixHeaderExtraFlags, MixHeaderFlags, u16, u32, [u8; 2])> {
        let mut buf = [0u8; size_of::<u16>()];
        reader.read_exact(&mut buf)?;
        let extra_flags = u16::from_le_bytes(buf);
        let new_format = extra_flags == 0;

        let mut flags = MixHeaderFlags::default();
        let num_files: u16;
        let body_size: u32;
        let remaining = [0u8; 2];

        if new_format {
            // New MIX format (>=RA).
            reader.read_exact(&mut buf)?;
            flags = u16::from_le_bytes(buf).into();

            if flags.contains(MixHeaderFlags::ENCRYPTION) {
                // Decrypt and read header.
                let mut buf = [0u8; BLOWFISH_KEY_SIZE];
                reader.read_exact(&mut buf)?;
                todo!() // TODO
            } else {
                // Just read header.
                reader.read_exact(&mut buf)?;
                num_files = u16::from_le_bytes(buf);
                let mut buf = [0u8; size_of::<u32>()];
                reader.read_exact(&mut buf)?;
                body_size = u32::from_le_bytes(buf);
            }
        } else {
            // Old MIX format (TD).
            num_files = extra_flags;
            let mut buf = [0u8; size_of::<u32>()];
            reader.read_exact(&mut buf)?;
            body_size = u32::from_le_bytes(buf);
        }

        Ok((
            new_format,
            extra_flags.into(),
            flags,
            num_files,
            body_size,
            remaining,
        ))
    }

    /// Read the entire MIX index section.
    pub fn read_index(reader: &mut dyn Read, num_files: u16) -> Result<Vec<MixIndexEntry>> {
        (0..num_files)
            .map(|_| Self::read_index_entry(reader))
            .collect()
    }

    /// Read the entire encrypted MIX index section.
    pub fn read_index_encrypted(
        reader: &mut dyn Read,
        num_files: u16,
        remaining: [u8; 2],
    ) -> Result<Vec<MixIndexEntry>> {
        for _ in 0..num_files {
            todo!(); // TODO
        }

        todo!(); // TODO
    }

    /// Read a MIX index entry.
    pub fn read_index_entry(reader: &mut dyn Read) -> Result<MixIndexEntry> {
        let mut buf = [0u8; size_of::<u32>()];
        reader.read_exact(&mut buf)?;
        let id = i32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let offset = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let size = u32::from_le_bytes(buf);
        Ok(MixIndexEntry { id, offset, size })
    }

    /// Read file blobs.
    pub fn read_bodies(
        reader: &mut dyn Read,
        mut index: Vec<MixIndexEntry>,
    ) -> Result<(Vec<MixFileEntry>, u32)> {
        index.sort_by(|a, b| a.offset.cmp(&b.offset));
        let mut files: Vec<MixFileEntry> = Vec::with_capacity(index.len());
        let mut current = 0;
        for entry in index {
            let distance = entry.offset - current;
            let mut residue = Vec::with_capacity(distance as usize);
            reader.read_exact(&mut residue)?;
            let mut body = Vec::with_capacity(entry.size as usize);
            reader.read_exact(&mut body)?;

            current += distance + entry.size;
            files.push(MixFileEntry {
                index: entry,
                body,
                residue,
                name: None,
            });
        }
        Ok((files, current))
    }

    /// Apply LMD data to mixed files.
    fn apply_lmd(mix: &mut Mix) -> Result<()> {
        let key = if mix.is_new_mix {
            LMD_KEY_TS
        } else {
            LMD_KEY_TD
        };
        if let Some(lmd) = mix.files.get(&key) {
            let reader: &mut dyn Read = &mut lmd.body.as_slice();
            Self::read_lmd_header(reader)?;

            let mut buf: Vec<u8> = Vec::with_capacity(lmd.index.size as usize - LMD_HEADER_SIZE);
            reader.read_exact(&mut buf)?;
            buf.split(|x| *x == 0u8)
                .map(|s| String::from_utf8(s.to_vec()))
                .collect::<Result<Vec<String>>>()?
                .iter()
                .zip(mix.files.values_mut())
                .for_each(|(name, file)| file.name = Some(*name));
        }
        Ok(())
    }

    /// Read the LMD header.
    fn read_lmd_header(reader: &mut dyn Read) -> Result<()> {
        let mut buf = [0u8; LMD_PREFIX.len()];
        reader.read_exact(&mut buf)?;
        if buf.ne(LMD_PREFIX) {
            return Err(Error::InvalidLMDPrefix);
        }
        let mut buf = [0u8; size_of::<u32>()];
        reader.read_exact(&mut buf)?;
        let lmd_size = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?; // Skip 4 bytes
        reader.read_exact(&mut buf)?; // Skip 4 bytes
        reader.read_exact(&mut buf)?;
        let lmd_version = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let num_names = u32::from_le_bytes(buf);
        // TODO return something of value?
        Ok(())
    }
}

pub struct MixWriter {}

impl MixWriter {}
