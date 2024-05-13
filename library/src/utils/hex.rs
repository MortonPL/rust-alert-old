//! Hexadecimal-string-to-int helper.

/// The error type for str-int conversion.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An [`std::num::ParseIntError`].
    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

type Result<T> = std::result::Result<T, Error>;

/// A helper that parses a string with hexadecimal value into an i32.
/// 
/// # Example
/// 
/// ```ignore
/// use rust_alert::utils::{hex2int, ParseIntError};
///
/// let res = hex2int("00A1");
/// assert!(res.is_ok());
/// assert_eq!(res.unwrap(), 161);
///
/// let res = hex2int("ZBFJHDL259h");
/// assert!(res.is_err());
/// assert!(matches!(res.unwrap_err(), ParseIntError::ParseIntError(_)));
/// ```
pub fn hex2int(hex: &str) -> Result<i32> {
    let x = u32::from_str_radix(hex, 16)?;
    let x = i32::from_le_bytes(x.to_le_bytes());
    Ok(x)
}

#[cfg(test)]
mod exmaples {
    use crate as rust_alert;

    #[test]
    fn hex2int() {
        use rust_alert::utils::{hex2int, ParseIntError};

        let res = hex2int("00A1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 161);

        let res = hex2int("ZBFJHDL259h");
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), ParseIntError::ParseIntError(_)));
    }
}
