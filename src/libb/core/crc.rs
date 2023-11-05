//! C&C CRC functions used for file indexing.

use std::mem::size_of;

use crc32fast;

use crate::core::general::GameEnum;

/// General CRC function that picks implementation depending on game version.
pub fn crc(value: impl AsRef<str>, game: GameEnum) -> i32 {
    match game {
        GameEnum::TD => crc_td(value),
        GameEnum::RA => crc_td(value),
        GameEnum::TS => crc_ts(value),
        GameEnum::FS => crc_ts(value),
        GameEnum::RA2 => crc_ts(value),
        GameEnum::YR => crc_ts(value),
    }
}

/// "CRC" function used in TD and RA.
pub fn crc_td(string: impl AsRef<str>) -> i32 {
    let mut string_upper = string.as_ref().to_uppercase().into_bytes();
    if string_upper.is_empty() {
        return 0;
    }
    // Pad the string so that its length is a multiple of 4.
    let missing = match string_upper.len() % 4 {
        1 => 3,
        3 => 1,
        x => x,
    };
    string_upper.extend_from_slice(&[0u8, 0, 0, 0][0..missing]);
    // Algorithm proper; Read 32bit chunks, rotate and sum.
    string_upper
        .chunks_exact(size_of::<u32>())
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .fold(0u32, |acc, x| x.wrapping_add(acc.rotate_left(1))) as i32
}

/// CRC function used in TS, FS, RA2 and YR.
pub fn crc_ts(string: impl AsRef<str>) -> i32 {
    let mut string_upper = string.as_ref().to_uppercase();
    let len = string_upper.len();
    if len == 0 {
        return 0;
    }
    // Magic WW padding.
    let remainder = len % 4;
    if remainder != 0 {
        // First pad with the pad size.
        string_upper.push(remainder as u8 as char);
        // Then pad with the beginning of the last 4-byte chunk.
        let padding_idx = (len >> 2) << 2;
        let padding = string_upper.chars().nth(padding_idx).unwrap();
        for _ in 0..(3 - remainder) {
            string_upper.push(padding);
        }
    }
    // Standard CRC32.
    crc32fast::hash(string_upper.as_bytes()) as i32
}

#[cfg(test)]
mod tests {
    use crate::core::crc::GameEnum;
    use crate::core::crc::{crc, crc_td, crc_ts};

    #[test]
    /// Test TD CRC function.
    fn test_crc_td() {
        // Zero length.
        assert_eq!(crc_td(""), 0);
        // Multiple of 4 length.
        assert_eq!(crc_td("shok.shp"), 0xE6E6E3D4u32 as i32);
        // Not multiple of 4 length.
        assert_eq!(crc_td("a10.shp"), 0x5CB0AAD5u32 as i32);
        // LMD constant.
        assert_eq!(crc_td("local mix database.dat"), 0x54C2D545u32 as i32);
        // Determinism test.
        assert_eq!(crc_td("deterministic"), crc_td("deterministic"));
    }

    #[test]
    /// Test TS CRC function.
    fn test_crc_ts() {
        // Zero length.
        assert_eq!(crc_ts(""), 0);
        // Multiple of 4 length.
        assert_eq!(crc_ts("bomb.shp"), 0x50F0D1EFu32 as i32);
        // Not multiple of 4 length.
        assert_eq!(crc_ts("wrench.shp"), 0x97E9DF77u32 as i32);
        // LMD constant.
        assert_eq!(crc_ts("local mix database.dat"), 0x366E051Fu32 as i32);
        // Determinism test.
        assert_eq!(crc_ts("deterministic"), crc_ts("deterministic"));
    }

    #[test]
    // Test the implementation-picking function.
    fn test_crc_auto() {
        assert_eq!(crc("cache.mix", GameEnum::TD), crc_td("cache.mix"));
        assert_eq!(crc("cache.mix", GameEnum::TS), crc_ts("cache.mix"));
        // TD and RA use the same implementation.
        assert_eq!(
            crc("cache.mix", GameEnum::TD),
            crc("cache.mix", GameEnum::RA)
        );
        // TS/FS and RA2/YR use the same implementation.
        assert_eq!(
            crc("cache.mix", GameEnum::TS),
            crc("cache.mix", GameEnum::FS),
        );
        assert_eq!(
            crc("cache.mix", GameEnum::RA2),
            crc("cache.mix", GameEnum::YR),
        );
        assert_eq!(
            crc("cache.mix", GameEnum::FS),
            crc("cache.mix", GameEnum::YR),
        );
        // TD/RA and TS/FS/RA2/YR use different implementations.
        assert_ne!(
            crc("cache.mix", GameEnum::TD),
            crc("cache.mix", GameEnum::TS)
        );
    }
}
