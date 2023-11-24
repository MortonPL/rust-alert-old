//! MIX I/O.

use std::{
    io::{Read, Write},
    mem::size_of,
};

use blowfish::{
    cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt},
    Blowfish,
};
use num_bigint::BigUint;

use crate::{
    defaultarray,
    mix::{BlowfishKey, Checksum, Mix, MixHeaderFlags, MixIndexEntry},
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
/// Exponent (e) of Westwood's "fast"/RSA key.
pub const FAST_E: &[u8] = &[1, 0, 1];
/// Modulus (n) of Westwood's "fast"/RSA key.
pub const FAST_N: &[u8] = &[
    21, 127, 67, 170, 61, 79, 251, 209, 230, 193, 176, 248, 106, 14, 221, 171, 74, 176, 130, 102,
    250, 84, 170, 232, 162, 63, 113, 81, 214, 96, 81, 86, 228, 252, 57, 109, 8, 218, 188, 81,
];
/// Modular inverse (d) of Westwood's "fast"/RSA key.
pub const FAST_D: &[u8] = &[
    129, 48, 137, 130, 230, 244, 251, 161, 6, 87, 223, 27, 78, 39, 88, 67, 51, 212, 180, 74, 174,
    174, 208, 219, 91, 94, 16, 84, 124, 198, 34, 196, 71, 156, 19, 153, 188, 55, 86, 10,
];

pub type BlowfishKeyEncrypted = [u8; ENCRYPTED_BLOWFISH_KEY_SIZE];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Attempted to read the LMD, but the prefix didn't match")]
    InvalidLMDPrefix,
    #[error("{0}")]
    MIX(#[from] crate::mix::core::Error),
    #[error("Expected Blowfish key to be 56 bytes long, but was {0}")]
    WrongBlowfishSizeDecrypted(usize),
    #[error("Expected Blowfish key to be 56 bytes long, but was {0}")]
    WrongBlowfishSizeEncrypted(usize),
}

type Result<T> = std::result::Result<T, Error>;

/// Provides static methods for reading MIX files.
pub struct MixReader {}

type HeaderReturnType = (Mix, u16, Option<(BlowfishKey, Blowfish, [u8; 2])>);

impl MixReader {
    /// Create a MIX from given byte input. Note: in order to guard against incorrect
    /// body size declaration, input **will be read until EOF**.
    pub fn read_file(reader: &mut dyn Read, force_new_format: bool) -> Result<Mix> {
        // Read header.
        let (mut mix, num_files, blowfish_data) = Self::read_header(reader, force_new_format)?;
        // Read index.
        let mut index = if let Some((key, cipher, remaining)) = blowfish_data {
            mix.blowfish_key = Some(key);
            Self::read_index_encrypted(reader, num_files, cipher, remaining)
        } else {
            Self::read_index(reader, num_files)
        }?;
        mix.index.extend(index.drain(..).map(|f| (f.id, f)));
        // Read body.
        reader.read_to_end(&mut mix.body)?;
        // Read the checksum if available.
        if mix.flags.contains(MixHeaderFlags::CHECKSUM) {
            let buf = mix.body.split_off(mix.body.len() - size_of::<Checksum>());
            mix.checksum = Some(buf.try_into().unwrap_or_else(|_| unreachable!()));
        }

        Ok(mix)
    }

    /// Read the MIX header.
    pub fn read_header(reader: &mut dyn Read, force_new_format: bool) -> Result<HeaderReturnType> {
        let mut buf = [0u8; size_of::<u16>()];
        let mut flags = MixHeaderFlags::default();
        let mut blowfish_data: Option<(BlowfishKey, Blowfish, [u8; 2])> = None;
        let num_files: u16;
        let declared_body_size: u32;
        // Read flags. 
        reader.read_exact(&mut buf)?;
        let extra_flags = u16::from_le_bytes(buf);

        let is_new_format = force_new_format || extra_flags == 0;
        if is_new_format {
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
                // Read header.
                num_files =
                    u16::from_le_bytes(buf[0..2].try_into().unwrap_or_else(|_| unreachable!()));
                declared_body_size =
                    u32::from_le_bytes(buf[2..6].try_into().unwrap_or_else(|_| unreachable!()));
                let remaining = buf[6..8].try_into().unwrap_or_else(|_| unreachable!());
                blowfish_data = Some((key, cipher, remaining));
            } else {
                // Just read header.
                reader.read_exact(&mut buf)?;
                num_files = u16::from_le_bytes(buf);
                let mut buf = [0u8; size_of::<u32>()];
                reader.read_exact(&mut buf)?;
                declared_body_size = u32::from_le_bytes(buf);
            }
        } else {
            // Old MIX format (TD).
            num_files = extra_flags;
            let mut buf = [0u8; size_of::<u32>()];
            reader.read_exact(&mut buf)?;
            declared_body_size = u32::from_le_bytes(buf);
        }

        let mix = Mix {
            is_new_format,
            flags,
            extra_flags: extra_flags.into(),
            declared_body_size,
            ..Default::default()
        };
        Ok((mix, num_files, blowfish_data))
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

    /// Read the encrypted blowfish key and decrypt it using a handmade RSA algorithm.
    fn read_blowfish(reader: &mut dyn Read) -> Result<BlowfishKey> {
        // Read the encrypted Blowfish key.
        let mut buf: BlowfishKeyEncrypted = defaultarray!(BlowfishKeyEncrypted);
        reader.read_exact(&mut buf)?;
        decrypt_blowfish(&buf)
    }
}

/// Provides static methods for writing MIX files.
pub struct MixWriter {}

impl MixWriter {
    pub fn write_file(writer: &mut dyn Write, mix: &mut Mix, force_new_format: bool) -> Result<()> {
        Self::write_header(writer, mix, force_new_format)?;
        if let Some(key) = mix.blowfish_key {
            Self::write_index_encrypted(writer, mix, &key)?;
        } else {
            Self::write_index(writer, mix)?;
        }
        writer.write_all(&mix.body)?;
        if let Some(checksum) = mix.checksum {
            writer.write_all(&checksum)?;
        }
        Ok(())
    }

    pub fn write_header(writer: &mut dyn Write, mix: &Mix, force_new_format: bool) -> Result<()> {
        let new_format = force_new_format || mix.is_new_format;
        if new_format {
            let extra_flags: u16 = mix.extra_flags.into();
            writer.write_all(&extra_flags.to_le_bytes())?;
            let flags: u16 = mix.flags.into();
            writer.write_all(&flags.to_le_bytes())?;
            // New MIX format (>=RA).
            if let Some(key) = mix.blowfish_key {
                // Write key only; header data and index have to be handled together later.
                MixWriter::write_blowfish(writer, &key)?;
            } else {
                // Just write header.
                writer.write_all(&(mix.index.len() as u16).to_le_bytes())?;
                writer.write_all(&mix.declared_body_size.to_le_bytes())?;
            }
        } else {
            // Old MIX format (TD).
            writer.write_all(&(mix.index.len() as u16).to_le_bytes())?;
            writer.write_all(&mix.declared_body_size.to_le_bytes())?;
        }
        Ok(())
    }

    pub fn write_index_encrypted(
        writer: &mut dyn Write,
        mix: &mut Mix,
        key: &BlowfishKey,
    ) -> Result<()> {
        mix.sort_by_id();
        let size = size_of::<u16>() + size_of::<u32>() + mix.get_index_size();
        let fullsize = size.next_multiple_of(BLOWFISH_BLOCK_SIZE);
        let pad = fullsize - size;
        let mut buf = Vec::with_capacity(fullsize);
        buf.write_all(&(mix.index.len() as u16).to_le_bytes())?;
        buf.write_all(&(mix.get_body_size() as u32).to_le_bytes())?;
        for entry in mix.index.values() {
            Self::write_index_entry(&mut buf, &entry)?;
        }
        buf.extend_from_slice(&[0u8; BLOWFISH_BLOCK_SIZE][0..pad]);
        let mut cipher = Blowfish::bc_init_state();
        cipher.bc_expand_key(key);
        // Cut the header into Blowfish blocks and encrypt.
        let mut blocks: Vec<GenericArray<u8, _>> = buf
            .chunks_exact(BLOWFISH_BLOCK_SIZE)
            .map(|c| GenericArray::from_slice(c).to_owned())
            .collect();
        cipher.encrypt_blocks(blocks.as_mut_slice());
        writer.write_all(&blocks.concat())?;
        Ok(())
    }

    pub fn write_index(writer: &mut dyn Write, mix: &mut Mix) -> Result<()> {
        mix.sort_by_id();
        for entry in mix.index.values() {
            Self::write_index_entry(writer, &entry)?;
        }
        Ok(())
    }

    pub fn write_index_entry(writer: &mut dyn Write, entry: &MixIndexEntry) -> Result<()> {
        writer.write_all(&entry.id.to_le_bytes())?;
        writer.write_all(&entry.offset.to_le_bytes())?;
        writer.write_all(&entry.size.to_le_bytes())?;
        Ok(())
    }

    /// Encrypt and write the Blowfish key.
    fn write_blowfish(writer: &mut dyn Write, key: &BlowfishKey) -> Result<()> {
        let encrypted = encrypt_blowfish(key)?;
        writer.write_all(&encrypted)?;
        Ok(())
    }
}

/// Decrypt the Blowfish key using a handmade RSA algorithm.
pub fn decrypt_blowfish(key: &BlowfishKeyEncrypted) -> Result<BlowfishKey> {
    // Get the RSA/"fast" key from known constants.
    let e = BigUint::from_bytes_le(FAST_E);
    let n = BigUint::from_bytes_le(FAST_N);
    // Decrypt the key in 40 byte chunks.
    let key: Vec<u8> = key
        .chunks_exact(BLOWFISH_KEY_CHUNK_SIZE)
        .flat_map(|x| BigUint::from_bytes_le(x).modpow(&e, &n).to_bytes_le())
        .collect();
    // Ensure that the result is exactly 56 bytes long.
    let len = key.len();
    key.try_into()
        .or(Err(Error::WrongBlowfishSizeDecrypted(len)))
}

/// Encrypt the Blowfish key using a handmade RSA algorithm.
pub fn encrypt_blowfish(key: &BlowfishKey) -> Result<BlowfishKeyEncrypted> {
    // Get the RSA/"fast" key from known constants.
    let d = BigUint::from_bytes_le(FAST_D);
    let n = BigUint::from_bytes_le(FAST_N);
    // Encrypt the key in 40 byte chunks.
    let key: Vec<u8> = key
        .chunks(BLOWFISH_KEY_CHUNK_SIZE - 1)
        .flat_map(|x| BigUint::from_bytes_le(x).modpow(&d, &n).to_bytes_le())
        .collect();
    // Ensure that the result is exactly 80 bytes long.
    let len = key.len();
    key.try_into()
        .or(Err(Error::WrongBlowfishSizeDecrypted(len)))
}

#[cfg(test)]
mod tests {
    use crate::{
        mix::{
            io::{decrypt_blowfish, encrypt_blowfish, BlowfishKeyEncrypted},
            BlowfishKey,
        },
        unwrap_ref_assert,
    };

    // Taken from multimd.mix
    const ENCRYPTED_KEY: &BlowfishKeyEncrypted = &[
        31, 245, 211, 151, 220, 77, 151, 240, 232, 170, 197, 246, 40, 90, 199, 85, 148, 216, 142,
        158, 120, 4, 198, 144, 196, 23, 145, 144, 181, 177, 143, 143, 28, 215, 81, 110, 83, 64, 84,
        41, 42, 194, 69, 188, 141, 96, 189, 202, 60, 66, 183, 76, 236, 123, 9, 8, 42, 37, 44, 85,
        142, 68, 81, 246, 102, 120, 25, 18, 35, 43, 174, 88, 226, 132, 96, 131, 253, 188, 57, 5,
    ];

    const DECRYPTED_KEY: &BlowfishKey = &[
        171, 92, 165, 248, 18, 172, 78, 242, 212, 163, 254, 255, 93, 40, 18, 170, 67, 107, 152, 11,
        192, 215, 163, 33, 232, 190, 204, 198, 24, 194, 53, 84, 185, 26, 134, 104, 114, 41, 79,
        178, 147, 188, 131, 20, 170, 220, 77, 119, 142, 102, 227, 196, 177, 113, 68, 247,
    ];

    #[test]
    /// Test Blowfish key encryption.
    fn blowfish_encrypt() {
        let encrypted = encrypt_blowfish(DECRYPTED_KEY);

        assert!(encrypted.is_ok());
        unwrap_ref_assert!(encrypted, ENCRYPTED_KEY);
    }

    #[test]
    /// Test Blowfish key decryption.
    fn blowfish_decrypt() {
        let decrypted = decrypt_blowfish(ENCRYPTED_KEY);

        assert!(decrypted.is_ok());
        unwrap_ref_assert!(decrypted, DECRYPTED_KEY);
    }
}
