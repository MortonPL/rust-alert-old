use std::collections::HashMap;

use clap::ValueEnum;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Unknown version number {x}")]
    UnknownVersion { x: u32 },
    #[error("Unknown language number {x}")]
    UnknownLanguage { x: u32 },
}

type Result<T> = std::result::Result<T, Error>;

/// CSF format version.
#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
#[repr(u32)]
pub enum CsfVersionEnum {
    /// Also used in BFME.
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
            x => Err(Error::UnknownVersion { x }),
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
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
#[repr(u32)]
pub enum CsfLanguageEnum {
    #[default]
    /// English (United States)
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

pub struct CsfStringtableIter<'a> {
    iter: std::collections::hash_map::Iter<'a, String, CsfLabel>,
}

impl<'a> Iterator for CsfStringtableIter<'a> {
    type Item = (&'a String, &'a CsfLabel);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
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
    pub fn iter(&self) -> CsfStringtableIter {
        CsfStringtableIter {
            iter: self.labels.iter(),
        }
    }

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
    pub fn remove_label(&mut self, name: impl Into<String>) -> Option<CsfLabel> {
        self.labels.remove(&name.into())
    }

    /// Looks up first string of a label with given name.
    /// Returns value if a label is found and contains any strings, otherwise None.
    pub fn lookup(&self, name: impl Into<String>) -> Option<&String> {
        self.labels
            .get(&name.into())
            .and_then(|l| l.get_first())
            .map(|s| &s.value)
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
    use crate::core::csf::{CsfLabel, CsfStringtable};

    #[test]
    /// Test label creation.
    fn stringtable_create_label() {
        let label = "Label".to_string();
        let string = "String".to_string();

        let mut expected = CsfStringtable::default();
        expected
            .labels
            .insert(label.clone(), CsfLabel::new(&label, &string));
        let mut csf = CsfStringtable::default();
        csf.create_label(label, string);

        assert_eq!(csf, expected);
    }

    #[test]
    /// Test label addition.
    fn stringtable_add_label() {
        let label = "Label".to_string();
        let string = "String".to_string();

        let mut expected = CsfStringtable::default();
        expected
            .labels
            .insert(label.clone(), CsfLabel::new(&label, &string));
        let mut csf = CsfStringtable::default();
        csf.add_label(CsfLabel::new(label, string));

        assert_eq!(csf, expected);
    }

    #[test]
    /// Test label removal.
    fn stringtable_remove_label() {
        let label = "Label".to_string();

        let expected = CsfStringtable::default();
        let mut csf = CsfStringtable::default();
        csf.labels
            .insert(label.clone(), CsfLabel::new(&label, "String"));
        csf.remove_label(&label);

        assert_eq!(csf, expected);
    }

    #[test]
    /// Test label lookup.
    fn stringtable_lookup_label() {
        let label = "Label".to_string();
        let string = "String".to_string();

        let mut csf = CsfStringtable::default();
        csf.labels
            .insert(label.clone(), CsfLabel::new(&label, &string));
        let actual = csf.lookup(&label);

        assert!(actual.is_some());
        assert_eq!(actual.unwrap(), &string);

        let actual = csf.lookup("NoString");
        assert!(actual.is_none());
    }

    #[test]
    /// Test label count.
    fn stringtable_count_labels() {
        let label = "Label".to_string();
        let label2 = "Label2".to_string();

        let expected = 2;
        let mut csf = CsfStringtable::default();
        csf.labels
            .insert(label.clone(), CsfLabel::new(label, "String"));
        csf.labels
            .insert(label2.clone(), CsfLabel::new(label2, "String2"));
        let actual = csf.get_label_count();

        assert_eq!(actual, expected);
    }

    #[test]
    /// Test string count.
    fn stringtable_count_strings() {
        let label = "Label".to_string();
        let string = "String".to_string();
        let string2 = "String2".to_string();

        let expected = 2;
        let mut csf = CsfStringtable::default();
        let mut lbl = CsfLabel::new(&label, string);
        lbl.strings.push(string2.into());
        csf.labels.insert(label, lbl);
        let actual = csf.get_string_count();

        assert_eq!(actual, expected);
    }

    #[test]
    /// Test label access.
    fn label_get_first() {
        let string = "String";

        let label = CsfLabel::new("Label", string);

        let expected = label.strings.first().unwrap();
        let actual = label.get_first();
        assert!(actual.is_some());
        assert_eq!(actual.unwrap(), expected);
    }
}
