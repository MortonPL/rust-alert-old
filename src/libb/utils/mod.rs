//! Helper functions and macros.

mod hash;
mod hex;
pub mod macros;

pub use hash::*;
pub use hex::hex2int;
pub use hex::Error as ParseIntError;
