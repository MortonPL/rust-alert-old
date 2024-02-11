#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

type Result<T> = std::result::Result<T, Error>;

pub fn hex2int(hex: &str) -> Result<i32> {
    let x = u32::from_str_radix(hex, 16)?;
    let x = i32::from_le_bytes(x.to_le_bytes());
    Ok(x)
}
