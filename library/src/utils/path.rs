//! File system path to filename string helper.
use std::path::Path;

/// The error type for path lookup helper.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    ///
    #[error("Path {0} doesn't point to a file or a directory")]
    NoFileName(Box<Path>),
    /// A path is not a valid Unicode string.
    #[error("Failed to convert a file path to a string, because it's not a valid Unicode string")]
    OsStrInvalidUnicode,
}

type Result<T> = std::result::Result<T, Error>;

/// Extracts the final component of a path and converts it to a [`String`]`.
/// 
/// # Examples
/// 
/// ```ignore
/// use rust_alert::utils::path_to_filename;
///
/// let path = std::path::Path::new("/files/assets/palace.shp");
/// let result = path_to_filename(&path);
///
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), "palace.shp");
/// ```
pub fn path_to_filename(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    path.file_name()
        .ok_or(Error::NoFileName(path.into()))?
        .to_str()
        .ok_or(Error::OsStrInvalidUnicode)
        .map(|s| s.to_string())
}

#[cfg(test)]
mod examples {
    use crate as rust_alert;

    #[test]
    fn path_to_filename() {
        use rust_alert::utils::path_to_filename;

        let path = std::path::Path::new("/files/assets/palace.shp");
        let result = path_to_filename(&path);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "palace.shp");
    }
}
