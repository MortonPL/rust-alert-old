use std::{
    io::{Read, Write},
    mem::size_of,
    str::FromStr,
};

use blowfish::{
    cipher::{
        generic_array::GenericArray, typenum::UInt, typenum::UTerm, typenum::B0, typenum::B1,
        BlockDecrypt,
    },
    Blowfish,
};
use num_bigint::BigUint;

use crate::core::mix::{
    BlowfishKey, LMDVersionEnum, LocalMixDatabaseInfo, Mix, MixFileEntry, MixHeaderExtraFlags,
    MixHeaderFlags, MixIndexEntry, BLOWFISH_KEY_SIZE, LMD_KEY_TD, LMD_KEY_TS,
};

/// Prefix of every LMD header.
pub const LMD_PREFIX: &[u8; 32] = b"XCC by Olaf van der Spek\x1a\x04\x17\x27\x10\x19\x80\x00";
/// Size of the entire LMD header.
pub const LMD_HEADER_SIZE: usize = LMD_PREFIX.len() + 20;
pub const BLOWFISH_KEY_CHUNK_SIZE: usize = 40;
pub const ENCRYPTED_BLOWFISH_KEY_SIZE: usize = 80;
pub const BLOWFISH_BLOCK_SIZE: usize = 8;
/// Exponent of Westwood's "fast"/RSA key.
pub const EXPONENT: &[u8] = &[1, 0, 1];
/// Modulus of Westwood's "fast"/RSA key.
pub const MODULUS: &[u8] = &[
    21, 127, 67, 170, 61, 79, 251, 209, 230, 193, 176, 248, 106, 14, 221, 171, 74, 176, 130, 102,
    250, 84, 170, 232, 162, 63, 113, 81, 214, 96, 81, 86, 228, 252, 57, 109, 8, 218, 188, 81,
];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Attempted to read the LMD, but the prefix didn't match")]
    InvalidLMDPrefix,
    #[error("{0}")]
    MIX(#[from] crate::core::mix::Error),
    #[error("Expected Blowfish key to be 56 bytes long, but was {0}")]
    WrongBlowfishSize(usize),
}

type Result<T> = std::result::Result<T, Error>;

pub struct MixReader {}

impl MixReader {
    pub fn read_file(reader: &mut dyn Read, force_new_format: bool) -> Result<Mix> {
        let mut mix = Mix::default();
        let (is_new_mix, extra_flags, flags, blowfish, num_files, body_size, remaining) =
            Self::read_header(reader, force_new_format)?;
        mix.is_new_mix = is_new_mix;
        mix.flags = flags;
        mix.extra_flags = extra_flags;
        mix.body_size = body_size;

        let index = if let Some((key, cipher)) = blowfish {
            mix.blowfish_key = Some(key);
            Self::read_index_encrypted(reader, num_files, cipher, remaining)
        } else {
            Self::read_index(reader, num_files)
        }?;

        let (files, pos) = Self::read_bodies(reader, index)?;
        for file in files {
            mix.files.insert(file.index.id, file);
        }

        let residue = body_size - pos;
        let mut buf: Vec<u8> = vec![0u8; residue as usize];
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
        force_new_format: bool,
    ) -> Result<(
        bool,
        MixHeaderExtraFlags,
        MixHeaderFlags,
        Option<(BlowfishKey, Blowfish)>,
        u16,
        u32,
        [u8; 2],
    )> {
        let mut buf = [0u8; size_of::<u16>()];
        reader.read_exact(&mut buf)?;
        let extra_flags = u16::from_le_bytes(buf);
        let new_format = force_new_format || (extra_flags == 0);

        let mut flags = MixHeaderFlags::default();
        let mut blowfish: Option<(BlowfishKey, Blowfish)> = None;
        let num_files: u16;
        let body_size: u32;
        let mut remaining = [0u8; 2];

        if new_format {
            // New MIX format (>=RA).
            reader.read_exact(&mut buf)?;
            flags = u16::from_le_bytes(buf).into();
            if flags.contains(MixHeaderFlags::ENCRYPTION) {
                // Decrypt and read header.
                let key = Self::read_blowfish(reader)?;
                let mut cipher = Blowfish::bc_init_state();
                cipher.bc_expand_key(&key);
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)?;
                let mut block = buf.into();
                cipher.decrypt_block(&mut block);
                let buf = block.as_slice();
                blowfish = Some((key, cipher));

                num_files = u16::from_le_bytes(buf[0..2].try_into().unwrap());
                body_size = u32::from_le_bytes(buf[2..6].try_into().unwrap());
                remaining = buf[6..8].try_into().unwrap();
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
            blowfish,
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
        cipher: Blowfish,
        remaining: [u8; 2],
    ) -> Result<Vec<MixIndexEntry>> {
        let size = num_files as usize * size_of::<MixIndexEntry>() - 2;
        let size = size.next_multiple_of(BLOWFISH_BLOCK_SIZE);
        let mut buf = vec![0u8; size];
        reader.read_exact(&mut buf)?;

        let mut blocks: Vec<GenericArray<u8, _>> = buf
            .chunks_exact(BLOWFISH_BLOCK_SIZE)
            .map(|c| GenericArray::from_slice(c).to_owned())
            .collect();
        cipher.decrypt_blocks(blocks.as_mut_slice());
        let mut buf2 = blocks.concat();
        let mut buf = remaining.to_vec();
        buf.append(&mut buf2);
        let mut decrypted_reader: &mut dyn Read = &mut buf.as_slice();

        Self::read_index(&mut decrypted_reader, num_files)
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
            let mut residue = vec![0u8; distance as usize];
            reader.read_exact(&mut residue)?;
            let mut body = vec![0u8; entry.size as usize];
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
            mix.lmd = Some(Self::read_lmd_header(reader)?);

            let mut buf: Vec<u8> = vec![0u8; lmd.index.size as usize - LMD_HEADER_SIZE];
            reader.read_exact(&mut buf)?;
            String::from_utf8(buf)?
                .split(|x| x == '\0')
                .zip(mix.files.values_mut())
                .for_each(|(name, file)| file.name = Some(name.to_string()));
        }
        Ok(())
    }

    /// Read the LMD header.
    fn read_lmd_header(reader: &mut dyn Read) -> Result<LocalMixDatabaseInfo> {
        let mut buf = [0u8; LMD_PREFIX.len()];
        reader.read_exact(&mut buf)?;
        if buf.ne(LMD_PREFIX) {
            return Err(Error::InvalidLMDPrefix);
        }
        let mut buf = [0u8; size_of::<u32>()];
        reader.read_exact(&mut buf)?;
        let size = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?; // Skip 4 bytes
        reader.read_exact(&mut buf)?; // Skip 4 bytes
        reader.read_exact(&mut buf)?;
        let version: LMDVersionEnum = u32::from_le_bytes(buf).try_into()?;
        reader.read_exact(&mut buf)?;
        let num_names = u32::from_le_bytes(buf);
        let lmd = LocalMixDatabaseInfo {
            num_names,
            version,
            size,
        };
        Ok(lmd)
    }

    /// Read the encrypted blowfish key and decrypt it using a handmade RSA algorithm.
    fn read_blowfish(reader: &mut dyn Read) -> Result<BlowfishKey> {
        let mut buf = [0u8; ENCRYPTED_BLOWFISH_KEY_SIZE];
        reader.read_exact(&mut buf)?;
        let exponent = BigUint::from_bytes_le(EXPONENT);
        let modulus = BigUint::from_bytes_le(MODULUS);
        let blowfish: Vec<u8> = buf
            .chunks_exact(BLOWFISH_KEY_CHUNK_SIZE)
            .map(|x| {
                BigUint::from_bytes_le(x)
                    .modpow(&exponent, &modulus)
                    .to_bytes_le()
            })
            .flatten()
            .collect();
        let len = blowfish.len();
        blowfish.try_into().or(Err(Error::WrongBlowfishSize(len)))
    }
}

pub struct MixWriter {}

impl MixWriter {
    pub fn write_file(writer: &mut dyn Write, mix: &Mix, force_new_format: bool) -> Result<()> {
        Self::write_header(writer, mix, force_new_format)?;

        todo!();
        // prep_lmd();
        // write_index();
        // write_bodies();
    }

    pub fn write_header(writer: &mut dyn Write, mix: &Mix, force_new_format: bool) -> Result<()> {
        let new_format = force_new_format || mix.is_new_mix;

        if new_format {
            let extra_flags: u16 = mix.extra_flags.into();
            writer.write_all(&extra_flags.to_le_bytes())?;
            let flags: u16 = mix.flags.into();
            writer.write_all(&flags.to_le_bytes())?;
            // New MIX format (>=RA).
            if mix.flags.contains(MixHeaderFlags::ENCRYPTION) {
                // Encrypt and write header.
            } else {
                // Just write header.
                writer.write_all(&(mix.files.len() as u16).to_le_bytes())?;
                writer.write_all(&mix.body_size.to_le_bytes())?;
            }
        } else {
            // Old MIX format (TD).
            writer.write_all(&(mix.files.len() as u16).to_le_bytes())?;
            writer.write_all(&mix.body_size.to_le_bytes())?;
        }
        Ok(())
    }
}
