use std::{
    collections::HashMap,
    string::{FromUtf16Error, FromUtf8Error},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Unknown version number {x}")]
    UnknownVerison { x: u32 },
    #[error("Unknown language number {x}")]
    UnknownLanguage { x: u32 },
    #[error("CSF prefix missing")]
    CsfMissingPrefix,
    #[error("LBL prefix missing")]
    LblMissingPrefix,
    #[error("RTS/WRTS prefix missing!")]
    RtsOrWrtsMissingPrefix,
    #[error("{0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("{0}")]
    Utf16(#[from] FromUtf16Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum CsfVersionEnum {
    /// Also includes BFME.
    #[default]
    Cnc = 3,
    Nox = 2,
}

impl TryFrom<u32> for CsfVersionEnum {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            x if x == CsfVersionEnum::Nox as u32 => Ok(CsfVersionEnum::Nox),
            x if x == CsfVersionEnum::Cnc as u32 => Ok(CsfVersionEnum::Cnc),
            x => Err(Error::UnknownVerison { x }),
        }
    }
}

impl TryFrom<CsfVersionEnum> for u32 {
    type Error = Error;

    fn try_from(value: CsfVersionEnum) -> Result<Self> {
        Ok(value as u32)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum CsfLanguageEnum {
    #[default]
    ENUS = 0,
    ENUK = 1,
    DE = 2,
    FR = 3,
    ES = 4,
    IT = 5,
    JA = 6,
    /// Joke WW entry - allegedly Jabberwockie (sic)
    XX = 7,
    KO = 8,
    ZHCN = 9,
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
            x => Err(Error::UnknownLanguage { x }),
        }
    }
}

impl TryFrom<CsfLanguageEnum> for u32 {
    type Error = Error;

    fn try_from(value: CsfLanguageEnum) -> Result<Self> {
        Ok(value as u32)
    }
}

/// A CSF file contains a header and a list of CSF labels.
/// Labels are stored as a dictionary for easy manipulation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CsfStringtable {
    /// Map of labels with their names as keys.
    pub labels: HashMap<String, CsfLabel>,
    /// Format version of the stringtable.
    pub version: CsfVersionEnum,
    /// Language of the stringtable.
    pub language: CsfLanguageEnum,
    /// Extra data attached to the header.
    pub extra: u32,
}

impl CsfStringtable {
    /// Creates a new label from name and string, then adds it to the stringtable.
    /// Returns old label with the same name if overwritten, otherwise None.
    pub fn create_label(&mut self, label: impl Into<String>, string: impl Into<String>) {
        self.add_label(CsfLabel::new(label, string));
    }

    /// Adds a label to the stringtable.
    /// Returns old label with the same name if overwritten, otherwise None.
    pub fn add_label(&mut self, label: CsfLabel) -> Option<CsfLabel> {
        self.labels.insert(label.name.clone(), label)
    }

    /// Remove a label with given name from the stringtable.
    /// Returns removed CsfLabel or None if nothing was removed.
    pub fn remove_label(&mut self, name: &String) -> Option<CsfLabel> {
        self.labels.remove(name)
    }

    /// Looks up first string of a label with given name.
    /// Returns value if a label is found and contains any strings, otherwise None.
    pub fn lookup(&self, name: &String) -> Option<&String> {
        self.labels.get(name).and_then(|l| l.get_first()).map(|s| &s.value)
    }

    /// Count all labels in the stringtable.
    pub fn get_label_count(&self) -> usize {
        self.labels.len()
    }

    /// Count strings in all labels in the stringtable.
    pub fn get_string_count(&self) -> usize {
        self.labels.values().fold(0, |acc, l| acc + l.strings.len())
    }
}

/// A CSF label contains a name and a collection of CSF strings.
/// Every label in vanilla game files contains only one string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CsfLabel {
    /// Name of the label. Game rules and triggers look up this value.
    pub name: String,
    /// List of CSF strings associated with the label.
    pub strings: Vec<CsfString>,
}

impl CsfLabel {
    pub fn new(label: impl Into<String>, string: impl Into<String>) -> Self {
        CsfLabel {
            name: label.into(),
            strings: vec![CsfString::new(string)],
        }
    }

    /// Returns the first CsfString in a label or None if the label contains no strings.
    pub fn get_first(&self) -> Option<&CsfString> {
        self.strings.first()
    }
}

/// A CSF string contains a LE UTF-16 string. There are two types of CSF strings:
/// normal (prefix RTS) and wide (prefix WRTS) which can contain an extra ASCII string.
/// All vanilla game strings are normal.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CsfString {
    /// Content of a string.
    pub value: String,
    /// Extra data associated with the string, stored as plain ASCII.
    pub extra_value: String,
}

impl CsfString {
    pub fn new(string: impl Into<String>) -> Self {
        CsfString {
            value: string.into(),
            ..Default::default()
        }
    }
}

impl From<String> for CsfString {
    fn from(string: String) -> Self {
        CsfString {
            value: string,
            ..Default::default()
        }
    }
}

impl From<CsfString> for String {
    fn from(string: CsfString) -> Self {
        string.value
    }
}

#[cfg(test)]
mod tests {
    use crate::core::csf::{CsfVersionEnum, CsfLabel, CsfLanguageEnum, CsfStringtable};

    #[test]
    /// Test if CsfLanguageEnum casts to u32.
    fn language_enum_into_u32() {
        let x: u32 = CsfLanguageEnum::ENUS.try_into().unwrap();
        assert_eq!(x, 0);
        let x: u32 = CsfLanguageEnum::ENUK.try_into().unwrap();
        assert_eq!(x, 1);
        let x: u32 = CsfLanguageEnum::DE.try_into().unwrap();
        assert_eq!(x, 2);
        let x: u32 = CsfLanguageEnum::FR.try_into().unwrap();
        assert_eq!(x, 3);
        let x: u32 = CsfLanguageEnum::ES.try_into().unwrap();
        assert_eq!(x, 4);
        let x: u32 = CsfLanguageEnum::IT.try_into().unwrap();
        assert_eq!(x, 5);
        let x: u32 = CsfLanguageEnum::JA.try_into().unwrap();
        assert_eq!(x, 6);
        let x: u32 = CsfLanguageEnum::XX.try_into().unwrap();
        assert_eq!(x, 7);
        let x: u32 = CsfLanguageEnum::KO.try_into().unwrap();
        assert_eq!(x, 8);
        let x: u32 = CsfLanguageEnum::ZHCN.try_into().unwrap();
        assert_eq!(x, 9);
    }

    
    #[test]
    /// Test if u32 casts to CsfLanguageEnum.
    fn language_enum_from_u32() {
        let x: CsfLanguageEnum = 0u32.try_into().unwrap();
        assert_eq!(x, CsfLanguageEnum::ENUS);
        let x: CsfLanguageEnum = 1u32.try_into().unwrap();
        assert_eq!(x, CsfLanguageEnum::ENUK);
        let x: CsfLanguageEnum = 2u32.try_into().unwrap();
        assert_eq!(x, CsfLanguageEnum::DE);
        let x: CsfLanguageEnum = 3u32.try_into().unwrap();
        assert_eq!(x, CsfLanguageEnum::FR);
        let x: CsfLanguageEnum = 4u32.try_into().unwrap();
        assert_eq!(x, CsfLanguageEnum::ES);
        let x: CsfLanguageEnum = 5u32.try_into().unwrap();
        assert_eq!(x, CsfLanguageEnum::IT);
        let x: CsfLanguageEnum = 6u32.try_into().unwrap();
        assert_eq!(x, CsfLanguageEnum::JA);
        let x: CsfLanguageEnum = 7u32.try_into().unwrap();
        assert_eq!(x, CsfLanguageEnum::XX);
        let x: CsfLanguageEnum = 8u32.try_into().unwrap();
        assert_eq!(x, CsfLanguageEnum::KO);
        let x: CsfLanguageEnum = 9u32.try_into().unwrap();
        assert_eq!(x, CsfLanguageEnum::ZHCN);

        let x = TryInto::<CsfLanguageEnum>::try_into(10u32);
        assert!(x.is_err());
        let x = TryInto::<CsfLanguageEnum>::try_into(99u32);
        assert!(x.is_err());
    }

    #[test]
    /// Test if CsfVersionEnum casts to u32.
    fn version_enum_into_u32() {
        let x: u32 = CsfVersionEnum::Nox.try_into().unwrap();
        assert_eq!(x, 2);
        let x: u32 = CsfVersionEnum::Cnc.try_into().unwrap();
        assert_eq!(x, 3);
    }

    #[test]
    /// Test if u32 casts to CsfVersionEnum.
    fn version_enum_from_u32() {
        let x: CsfVersionEnum = 2u32.try_into().unwrap();
        assert_eq!(x, CsfVersionEnum::Nox);
        let x: CsfVersionEnum = 3u32.try_into().unwrap();
        assert_eq!(x, CsfVersionEnum::Cnc);

        let x = TryInto::<CsfVersionEnum>::try_into(0u32);
        assert!(x.is_err());
        let x = TryInto::<CsfVersionEnum>::try_into(1u32);
        assert!(x.is_err());
        let x = TryInto::<CsfVersionEnum>::try_into(4u32);
        assert!(x.is_err());
        let x = TryInto::<CsfVersionEnum>::try_into(2137u32);
        assert!(x.is_err());
    }

    #[test]
    fn stringtable_create_label() {
        let label = "Label".to_string();
        let string = "String".to_string();

        let mut expected = CsfStringtable::default();
        expected.labels.insert(label.clone(), CsfLabel::new(label.clone(), string.clone()));
        let mut csf = CsfStringtable::default();
        csf.create_label(label, string);
        assert_eq!(csf, expected)
    }
}
