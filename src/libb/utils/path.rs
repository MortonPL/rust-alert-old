use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Path {0} doesn't point to a file")]
    NoFileName(Box<Path>),
    #[error("Failed to convert a file path to a string, because it's not valid Unicode")]
    OsStrInvalidUnicode,
}

type Result<T> = std::result::Result<T, Error>;

pub fn path_to_str(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    path.file_name()
        .ok_or(Error::NoFileName(path.into()))?
        .to_str()
        .ok_or(Error::OsStrInvalidUnicode)
        .map(|s| s.to_string())
}
