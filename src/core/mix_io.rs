use std::{
    io::{Read, Write},
    mem::size_of,
};

use blowfish::{
    cipher::{
        generic_array::GenericArray,
        BlockDecrypt,
    },
    Blowfish,
};
use num_bigint::BigUint;

use crate::core::mix::{
    BlowfishKey, Mix, MixFileEntry, MixHeaderExtraFlags,
    MixHeaderFlags, MixIndexEntry,
};

/// Prefix of every LMD header.
pub const LMD_PREFIX: &[u8; 32] = b"XCC by Olaf van der Spek\x1a\x04\x17\x27\x10\x19\x80\x00";
/// Size of the entire LMD header.
pub const LMD_HEADER_SIZE: usize = LMD_PREFIX.len() + 20;
/// Size of an RSA-encryptable Blowfish key chunk.
pub const BLOWFISH_KEY_CHUNK_SIZE: usize = 40;
/// Total size of an RSA encypted Blowfish key.
pub const ENCRYPTED_BLOWFISH_KEY_SIZE: usize = 80;
/// Blowfish block size.
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

/// Provides static methods for reading MIX files.
pub struct MixReader {}

type HeaderReturnType = (bool, MixHeaderExtraFlags, MixHeaderFlags, Option<(BlowfishKey, Blowfish)>, u16, u32, [u8; 2]);

impl MixReader {
    pub fn read_file(reader: &mut dyn Read, force_new_format: bool) -> Result<Mix> {
        let mut mix = Mix::default();
        // Read header.
        let (is_new_mix, extra_flags, flags, blowfish, num_files, body_size, remaining) =
            Self::read_header(reader, force_new_format)?;
        mix.is_new_mix = is_new_mix;
        mix.flags = flags;
        mix.extra_flags = extra_flags;
        mix.body_size = body_size;
        // Read index.
        let index = if let Some((key, cipher)) = blowfish {
            mix.blowfish_key = Some(key);
            Self::read_index_encrypted(reader, num_files, cipher, remaining)
        } else {
            Self::read_index(reader, num_files)
        }?;
        // Read file bodies.
        let (files, pos) = Self::read_bodies(reader, index)?;
        for file in files {
            mix.files.insert(file.index.id, file);
        }
        // Read the final byte residue.
        let residue = body_size - pos;
        let mut buf: Vec<u8> = vec![0u8; residue as usize];
        reader.read_exact(&mut buf)?;
        mix.residue = buf;

        Ok(mix)
    }

    /// Read the MIX header.
    pub fn read_header(
        reader: &mut dyn Read,
        force_new_format: bool,
    ) -> Result<HeaderReturnType> {
        let mut buf = [0u8; size_of::<u16>()];
        let mut flags = MixHeaderFlags::default();
        let mut blowfish: Option<(BlowfishKey, Blowfish)> = None;
        let num_files: u16;
        let body_size: u32;
        let mut remaining = [0u8; 2];
        // Read flags.
        reader.read_exact(&mut buf)?;
        let extra_flags = u16::from_le_bytes(buf);
        let new_format = force_new_format || (extra_flags == 0);
        if new_format {
            // New MIX format (>=RA).
            reader.read_exact(&mut buf)?;
            flags = u16::from_le_bytes(buf).into();
            if flags.contains(MixHeaderFlags::ENCRYPTION) {
                // Decrypt header.
                let key = Self::read_blowfish(reader)?;
                let mut cipher = Blowfish::bc_init_state();
                cipher.bc_expand_key(&key);
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)?;
                let mut block = buf.into();
                cipher.decrypt_block(&mut block);
                let buf = block.as_slice();
                blowfish = Some((key, cipher));
                // Read header.
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
        // Read the encrypted index.
        let size = num_files as usize * size_of::<MixIndexEntry>() - 2;
        let size = size.next_multiple_of(BLOWFISH_BLOCK_SIZE);
        let mut buf = vec![0u8; size];
        reader.read_exact(&mut buf)?;
        // Cut the header into Blowfish blocks and decrypt.
        let mut blocks: Vec<GenericArray<u8, _>> = buf
            .chunks_exact(BLOWFISH_BLOCK_SIZE)
            .map(|c| GenericArray::from_slice(c).to_owned())
            .collect();
        cipher.decrypt_blocks(blocks.as_mut_slice());
        let mut buf2 = blocks.concat();
        // Include two remaining decrypted bytes from the header.
        let mut buf = remaining.to_vec();
        buf.append(&mut buf2);
        let mut decrypted_reader: &mut dyn Read = &mut buf.as_slice();

        // Read decrypted index.
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
        // Read the files from start to finish - so sort by offset.
        index.sort_by(|a, b| a.offset.cmp(&b.offset));
        let mut files: Vec<MixFileEntry> = Vec::with_capacity(index.len());
        let mut current = 0;
        for entry in index {
            // Read residue bytes since the last file.
            let distance = entry.offset - current;
            let mut residue = vec![0u8; distance as usize];
            reader.read_exact(&mut residue)?;
            // Read the actual blob.
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

    /// Read the encrypted blowfish key and decrypt it using a handmade RSA algorithm.
    fn read_blowfish(reader: &mut dyn Read) -> Result<BlowfishKey> {
        // Read the encrypted Blowfish key.
        let mut buf = [0u8; ENCRYPTED_BLOWFISH_KEY_SIZE];
        reader.read_exact(&mut buf)?;
        // Get the RSA/"fast" key from known constants.
        let exponent = BigUint::from_bytes_le(EXPONENT);
        let modulus = BigUint::from_bytes_le(MODULUS);
        // Decrypt the key in 40 byte chunks.
        let blowfish: Vec<u8> = buf
            .chunks_exact(BLOWFISH_KEY_CHUNK_SIZE)
            .flat_map(|x| {
                BigUint::from_bytes_le(x)
                    .modpow(&exponent, &modulus)
                    .to_bytes_le()
            })
            .collect();
        // Ensure that the result is exactly 56 bytes long.
        let len = blowfish.len();
        blowfish.try_into().or(Err(Error::WrongBlowfishSize(len)))
    }
}

/// Provides static methods for writing MIX files.
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
