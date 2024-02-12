//! Enums used in the [`CsfStringtable`] struct.

use std::fmt::Display;

use crate::csf::{Error, Result};

/// CSF format version. "Nothing is known about the actual difference between the versions."
///
/// Read more at
/// [ModEnc](https://modenc.renegadeprojects.com/CSF_File_Format#The_Header).
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum CsfVersionEnum {
    /// Used in Nox (2000).
    Nox = 2,
    /// Used in all C&C games with CSF support (so RA2/YR) and Lord of the
    /// Rings: Battle for the Middle-earth.
    #[default]
    Cnc = 3,
}

impl Display for CsfVersionEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<u32> for CsfVersionEnum {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            x if x == CsfVersionEnum::Nox as u32 => Ok(CsfVersionEnum::Nox),
            x if x == CsfVersionEnum::Cnc as u32 => Ok(CsfVersionEnum::Cnc),
            x => Err(Error::UnknownVersion(x)),
        }
    }
}

impl TryFrom<CsfVersionEnum> for u32 {
    type Error = Error;

    fn try_from(value: CsfVersionEnum) -> Result<Self> {
        Ok(value as u32)
    }
}

/// CSF language ID used for localisation.
///
/// Read more at
/// [ModEnc](https://modenc.renegadeprojects.com/CSF_File_Format#Language).
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum CsfLanguageEnum {
    /// English (United States)
    #[default]
    ENUS = 0,
    /// English (United Kingdom)
    ENUK = 1,
    /// German
    DE = 2,
    /// French
    FR = 3,
    /// Spanish
    ES = 4,
    /// Italian
    IT = 5,
    /// Japanese
    JA = 6,
    /// Joke WW entry - allegedly Jabberwockie (sic)
    XX = 7,
    /// Korean
    KO = 8,
    /// Chinese
    ZHCN = 9,
}

impl Display for CsfLanguageEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<u32> for CsfLanguageEnum {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            x if x == CsfLanguageEnum::ENUS as u32 => Ok(CsfLanguageEnum::ENUS),
            x if x == CsfLanguageEnum::ENUK as u32 => Ok(CsfLanguageEnum::ENUK),
            x if x == CsfLanguageEnum::DE as u32 => Ok(CsfLanguageEnum::DE),
            x if x == CsfLanguageEnum::FR as u32 => Ok(CsfLanguageEnum::FR),
            x if x == CsfLanguageEnum::ES as u32 => Ok(CsfLanguageEnum::ES),
            x if x == CsfLanguageEnum::IT as u32 => Ok(CsfLanguageEnum::IT),
            x if x == CsfLanguageEnum::JA as u32 => Ok(CsfLanguageEnum::JA),
            x if x == CsfLanguageEnum::XX as u32 => Ok(CsfLanguageEnum::XX),
            x if x == CsfLanguageEnum::KO as u32 => Ok(CsfLanguageEnum::KO),
            x if x == CsfLanguageEnum::ZHCN as u32 => Ok(CsfLanguageEnum::ZHCN),
            x => Err(Error::UnknownLanguage(x)),
        }
    }
}

impl TryFrom<CsfLanguageEnum> for u32 {
    type Error = Error;

    fn try_from(value: CsfLanguageEnum) -> Result<Self> {
        Ok(value as u32)
    }
}
