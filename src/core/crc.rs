/// Module for C&C CRC functions used for file indexing.
use crc32fast;
use std::mem::size_of;

pub enum GameEnum {
    TD,
    RA,
    TS,
    YR,
}

/// General CRC function that picks implementation depending on game version.
pub fn crc(value: impl AsRef<str>, game: GameEnum) -> i32 {
    match game {
        GameEnum::TD => crc_td(value),
        GameEnum::RA => crc_td(value),
        _ => crc_ts(value),
    }
}

/// "CRC" function used in TD and RA1.
pub fn crc_td(value: impl AsRef<str>) -> i32 {
    let mut v = value.as_ref().to_uppercase().into_bytes();
    let rem = v.len() % 4;
    // Rust at its finest
    let missing = match rem {
        1 => 3,
        3 => 1,
        x => x,
    };
    v.extend_from_slice(&[0u8, 0, 0, 0][0..missing]);

    v.chunks(size_of::<u32>())
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .fold(0u32, |acc, x| x.wrapping_add(acc.rotate_left(1))) as i32
}

/// CRC function used in TS and YR.
pub fn crc_ts(value: impl AsRef<str>) -> i32 {
    let mut new_value = value.as_ref().to_uppercase();
    let len = new_value.len();
    let magic = (len >> 2) << 2;
    let len2 = len % 4;
    // Magic WW padding.
    if len2 != 0 {
        new_value.push(len2 as u8 as char);
        for _ in 0..(3 - len2) {
            new_value.push(new_value.chars().nth(magic).unwrap());
        }
    }
    crc32fast::hash(new_value.as_bytes()) as i32
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
        // TS and YR use the same implementation.
        assert_eq!(
            crc("cache.mix", GameEnum::TD),
            crc("cache.mix", GameEnum::RA)
        );
        // TD/RA and TS/YR use different implementations.
        assert_ne!(
            crc("cache.mix", GameEnum::TD),
            crc("cache.mix", GameEnum::TS)
        );
    }
}

pub trait ReverseMap {
    fn reversed(self) -> Self;
}

impl<T> ReverseMap for Vec<T> {
    fn reversed(mut self) -> Self {
        self.reverse();
        self
    }
}
