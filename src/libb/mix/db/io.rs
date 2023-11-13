//! MIX database I/O.

use std::{
    io::{Read, Write},
    mem::size_of,
};

use crate::{
    core::{crc, GameEnum},
    mix::db::{
        GlobalMixDatabase, LMDVersionEnum, LocalMixDatabase, LocalMixDatabaseInfo, MixDatabase,
    },
};

/// Prefix of every LMD header.
pub const LMD_PREFIX: &[u8; 32] = b"XCC by Olaf van der Spek\x1a\x04\x17\x27\x10\x19\x80\x00";
/// Size of the entire LMD header.
pub const LMD_HEADER_SIZE: usize = LMD_PREFIX.len() + 20;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Attempted to read the LMD, but the prefix didn't match")]
    InvalidLMDPrefix,
    #[error("Expected Blowfish key to be 56 bytes long, but was {0}")]
    WrongBlowfishSize(usize),
    #[error("{0}")]
    MixDbError(#[from] crate::mix::db::Error),
    #[error("Expected a null terminated string, but couldn't find null")]
    NoNullTermination(usize),
}

type Result<T> = std::result::Result<T, Error>;

pub struct LocalMixDbReader {}

impl LocalMixDbReader {
    pub fn read_file(reader: &mut dyn Read) -> Result<LocalMixDatabase> {
        // Read the LMD header.
        let info = Self::read_header(reader)?;
        // Read and process the LMD body.
        let strings =
            Self::read_strings(reader, info.size as usize - LMD_HEADER_SIZE, info.version)?;
        let mut lmd = LocalMixDatabase::default();
        lmd.db.names.extend(strings);
        lmd.version = info.version;
        Ok(lmd)
    }

    pub fn read_header(reader: &mut dyn Read) -> Result<LocalMixDatabaseInfo> {
        // Read the mandatory prefix.
        let mut buf = [0u8; LMD_PREFIX.len()];
        reader.read_exact(&mut buf)?;
        if buf.ne(LMD_PREFIX) {
            return Err(Error::InvalidLMDPrefix);
        }
        // Read header data.
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

    /// Read LMD body content and generate filename IDs.
    pub fn read_strings(
        reader: &mut dyn Read,
        size: usize,
        version: LMDVersionEnum,
    ) -> Result<Vec<(i32, String)>> {
        // Map LMD version to CRC version.
        let version = match version {
            LMDVersionEnum::TD => GameEnum::TD,
            LMDVersionEnum::RA => GameEnum::RA,
            LMDVersionEnum::TS => GameEnum::TS,
            LMDVersionEnum::RA2 => GameEnum::RA2,
            LMDVersionEnum::YR => GameEnum::YR,
        };
        // Read and process strings.
        let mut buf: Vec<u8> = vec![0u8; size];
        reader.read_exact(&mut buf)?;
        let pairs = String::from_utf8(buf)?
            .split(|x| x == '\0')
            .map(|s| (crc(s, version), s.to_string()))
            .collect();

        Ok(pairs)
    }
}

pub struct LocalMixDbWriter {}

impl LocalMixDbWriter {
    pub fn write_file(writer: &mut dyn Write, lmd: &LocalMixDatabase) -> Result<()> {
        Self::write_header(writer, lmd)?;
        Self::write_strings(writer, lmd)?;

        Ok(())
    }

    pub fn write_header(writer: &mut dyn Write, lmd: &LocalMixDatabase) -> Result<()> {
        writer.write_all(LMD_PREFIX)?;
        writer.write_all(
            &(LMD_HEADER_SIZE as u32
                + lmd
                    .db
                    .names
                    .values()
                    .fold(0u32, |acc, x| acc + x.len() as u32))
            .to_le_bytes(),
        )?;
        writer.write_all(&[0u8, 0, 0, 0])?;
        writer.write_all(&[0u8, 0, 0, 0])?;
        writer.write_all(&TryInto::<u32>::try_into(lmd.version)?.to_le_bytes())?;
        writer.write_all(&(lmd.db.names.len()).to_le_bytes())?;

        Ok(())
    }

    pub fn write_strings(writer: &mut dyn Write, lmd: &LocalMixDatabase) -> Result<()> {
        let joint = lmd.db.names.values().fold(String::new(), |mut acc, x| {
            acc.reserve(x.len() + 1);
            acc.push_str(x);
            acc.push(0 as char);
            acc
        });
        writer.write_all(joint.as_bytes())?;

        Ok(())
    }
}

pub struct GlobalMixDbReader {}

impl GlobalMixDbReader {
    pub fn read_file(reader: &mut dyn Read) -> Result<GlobalMixDatabase> {
        // TODO: Might want to use BufRead and read_until(), or keep being a moron.
        // NOTE: The XCC format kinda sucks, because we don't know the size in advance
        // and we're reading variable length strings. We have three options:
        // 1) read everything at once, which is ugly, heavy and will break things
        //    if we have more things  in the same reader.
        // 2) read in chunks until we finally find a zero, split, join chunks, continue and so on. We
        //    still risk "overshooting" and going past the end of this file if the reader has more things inside.
        // 3) read byte by byte, which is terribly slow.
        // Right now we roll with the first option.
        let mut gmd = GlobalMixDatabase::default();
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let mut ptr = 0;
        let len = buf.len();
        let versions = [GameEnum::TD, GameEnum::RA, GameEnum::TS];
        let mut i = 0;
        while ptr + 4 < len {
            let (strings, new_ptr) = Self::read_database(&buf, ptr)?;
            ptr = new_ptr;
            // All DBs past the second will use newer CRC.
            let version = versions[i.max(2)];
            let mut db = MixDatabase::default();
            db.names
                .extend(strings.into_iter().map(|s| (crc(&s, version), s)));
            i += 1;
            gmd.dbs.push(db);
        }

        Ok(gmd)
    }

    pub fn read_database(buf: &[u8], mut ptr: usize) -> Result<(Vec<String>, usize)> {
        let mut strings = Vec::<String>::new();
        let num_names = u32::from_le_bytes(buf[ptr..ptr + 4].try_into().unwrap()); // Won't panic.
        ptr += 4;
        for _ in 0..num_names {
            let cut = buf[ptr..]
                .iter()
                .position(|x| *x == 0)
                .ok_or(Error::NoNullTermination(ptr))?;
            strings.push(String::from_utf8(buf[ptr..ptr + cut].try_into().unwrap())?); // Won't panic.
            ptr += cut + 1;
            // Just advance the pointer, we don't need the description.
            ptr += buf[ptr..].iter().position(|x| *x == 0).unwrap() + 1;
        }

        Ok((strings, ptr))
    }
}

pub struct GlobalMixDbWriter {}

impl GlobalMixDbWriter {
    pub fn write_file(writer: &mut dyn Write, gmd: &GlobalMixDatabase) -> Result<()> {
        for db in &gmd.dbs {
            writer.write_all(&db.names.len().to_le_bytes())?;
            let strings = db.names.values().fold(String::default(), |mut acc, s| {
                acc.reserve(s.len() + 2);
                acc.push_str(s);
                acc.push_str("\0\0");
                acc
            });
            writer.write_all(strings.as_bytes())?;
        }

        Ok(())
    }
}
