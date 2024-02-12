//! CSF (stringtable) structures and manipulation.

use std::collections::HashSet;

use crate::csf::{enums::*, iters::*};

/// The error type for operations on CSF stringtables and related constructs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The supplied number doesn't match any known CSF version variants.
    #[error("Unknown version number {0}")]
    UnknownVersion(u32),
    /// The supplied number doesn't match any known CSF language variants.
    #[error("Unknown language number {0}")]
    UnknownLanguage(u32),
}

#[doc(hidden)]
pub type Result<T> = std::result::Result<T, Error>;

/// A stringtable containing key-value pairs for game text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CsfStringtable {
    /// Set of labels.
    labels: HashSet<CsfLabel>,
    /// Format version of the stringtable.
    pub version: CsfVersionEnum,
    /// Language of the stringtable.
    pub language: CsfLanguageEnum,
    /// Extra data attached to the header.
    pub extra: u32,
}

impl CsfStringtable {
    /// Creates a new empty [`CsfStringtable`] with specified header info.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::{CsfStringtable, CsfVersionEnum, CsfLanguageEnum};
    ///
    /// let csf = CsfStringtable::new(CsfVersionEnum::Cnc, CsfLanguageEnum::DE, 42);
    /// assert_eq!(csf.version, CsfVersionEnum::Cnc);
    /// assert_eq!(csf.language, CsfLanguageEnum::DE);
    /// assert_eq!(csf.extra, 42);
    /// assert_eq!(csf.len(), 0);
    /// ```
    pub fn new(version: CsfVersionEnum, language: CsfLanguageEnum, extra: u32) -> Self {
        Self {
            version,
            language,
            extra,
            ..Default::default()
        }
    }

    /// Creates an iterator visiting all labels in an arbitrary order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::{CsfStringtable, CsfLabel};
    ///
    /// let mut csf = CsfStringtable::default();
    /// csf.insert(CsfLabel::new("A", "1"));
    /// csf.insert(CsfLabel::new("B", "2"));
    ///
    /// assert_eq!(csf.len(), 2);
    ///
    /// for x in csf.iter() {
    ///     println!("{x}");
    /// }
    ///
    /// assert_eq!(csf.len(), 2);
    /// ```
    pub fn iter(&self) -> Iter {
        self.labels.iter().into()
    }

    /// Creates a draining iterator visiting all labels in an arbitrary order.
    /// Allocated memory is not freed.
    ///
    /// If the draining iterator is dropped before being fully consumed, all
    /// remaining elements are dropped as well.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::{CsfStringtable, CsfLabel};
    ///
    /// let mut csf = CsfStringtable::default();
    /// csf.insert(CsfLabel::new("A", "1"));
    /// csf.insert(CsfLabel::new("B", "2"));
    ///
    /// assert_eq!(csf.len(), 2);
    ///
    /// for x in csf.drain() {
    ///     println!("{x}");
    /// }
    ///
    /// assert_eq!(csf.len(), 0);
    /// ```
    pub fn drain(&mut self) -> Drain {
        self.labels.drain().into()
    }

    /// Creates a new label from a name and a string, then adds (or replaces)
    /// it to the stringtable.
    ///
    /// Returns the old label with the same name if overwritten, otherwise
    /// `None`.
    /// Also see [`insert`][Self::insert] to insert an existing
    /// [`CsfLabel`] into the stringtable.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::{CsfStringtable, CsfLabel};
    ///
    /// let mut csf = CsfStringtable::default();
    /// let result = csf.create("A", "1");
    /// assert_eq!(result, None);
    /// assert_eq!(csf.len(), 1);
    ///
    /// let result = csf.create("A", "2");
    /// assert_eq!(result, Some(CsfLabel::new("A", "1")));
    /// assert_eq!(csf.len(), 1);
    /// ```
    pub fn create(
        &mut self,
        label: impl Into<String>,
        string: impl Into<String>,
    ) -> Option<CsfLabel> {
        self.insert(CsfLabel::new(label, string))
    }

    /// Inserts (or replaces) a label to the stringtable.
    ///
    /// Returns the old label with the same name if overwritten, otherwise
    /// `None`.
    /// Also see [`create`][Self::create] to create and put a new
    /// [`CsfLabel`] into the stringtable directly.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::*;
    ///
    /// let mut csf = CsfStringtable::default();
    /// let label = CsfLabel::new("A", "1");
    ///
    /// let result = csf.insert(label.clone());
    /// assert_eq!(result, None);
    /// assert_eq!(csf.len(), 1);
    ///
    /// let result = csf.insert(CsfLabel::new("A", "2"));
    /// assert_eq!(result, Some(label));
    /// assert_eq!(csf.len(), 1);
    /// ```
    pub fn insert(&mut self, label: CsfLabel) -> Option<CsfLabel> {
        self.labels.replace(label)
    }

    /// Removes a label with given name from the stringtable.
    ///
    /// Returns removed [`CsfLabel`] or `None` if nothing was removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::{CsfStringtable, CsfLabel};
    ///
    /// let mut csf = CsfStringtable::default();
    /// csf.create("A", "1");
    /// csf.create("B", "2");
    ///
    /// assert_eq!(csf.len(), 2);
    ///
    /// let result = csf.remove("A");
    /// assert_eq!(result, Some(CsfLabel::new("A", "1")));
    /// assert_eq!(csf.len(), 1);
    ///
    /// let result = csf.remove("A");
    /// assert_eq!(result, None);
    /// assert_eq!(csf.len(), 1);
    /// ```
    pub fn remove(&mut self, name: impl Into<String>) -> Option<CsfLabel> {
        self.labels.take(&CsfLabel {
            name: name.into(),
            strings: vec![],
        })
    }

    /// Looks up the first string of a label with given name.
    ///
    /// Returns reference to the value if a label is found and contains any
    /// strings, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::CsfStringtable;
    ///
    /// let mut csf = CsfStringtable::default();
    /// csf.create("A", "1");
    ///
    /// let result = csf.get("A");
    /// assert_eq!(result, Some("1"));
    ///
    /// let result = csf.get("B");
    /// assert_eq!(result, None);
    /// ```
    pub fn get(&self, name: impl Into<String>) -> Option<&str> {
        self.labels
            .get(&CsfLabel {
                name: name.into(),
                strings: vec![],
            })
            .and_then(|l| l.get_first())
            .map(|s| s.value.as_str())
    }

    /// Count all labels in the stringtable.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::CsfStringtable;
    ///
    /// let mut csf = CsfStringtable::default();
    ///
    /// let size = csf.len();
    /// assert_eq!(size, 0);
    ///
    /// csf.create("A", "1");
    /// let size = csf.len();
    /// assert_eq!(size, 1);
    ///
    /// csf.create("B", "2");
    /// let size = csf.len();
    /// assert_eq!(size, 2);
    /// ```
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Count strings in all labels in the stringtable.
    ///
    /// The CSF format allows for multiple strings per label, however the game
    /// doesn't read strings besides the first one, so in 99% of use cases this
    /// is equivalent with [`len`][Self::len].
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::CsfStringtable;
    ///
    /// let mut csf = CsfStringtable::default();
    ///
    /// let size = csf.strings_len();
    /// assert_eq!(size, 0);
    ///
    /// csf.create("A", "1");
    /// let size = csf.strings_len();
    /// assert_eq!(size, 1);
    ///
    /// csf.create("B", "2");
    /// let size = csf.strings_len();
    /// assert_eq!(size, 2);
    /// ```
    pub fn strings_len(&self) -> usize {
        self.labels.iter().fold(0, |acc, l| acc + l.strings.len())
    }

    /// Reserves capacity for at least `additional` more labels.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::CsfStringtable;
    ///
    /// let mut csf = CsfStringtable::default();
    /// csf.reserve(42);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.labels.reserve(additional);
    }
}

impl IntoIterator for CsfStringtable {
    type Item = CsfLabel;

    type IntoIter = IntoIter;

    /// Creates a consuming iterator that will move labels out of the
    /// stringtable in arbitrary order.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::{CsfStringtable, CsfLabel};
    ///
    /// let mut csf = CsfStringtable::default();
    /// csf.insert(CsfLabel::new("A", "1"));
    /// csf.insert(CsfLabel::new("B", "2"));
    ///
    /// // Can't use `csf` after this!
    /// let v: Vec<CsfLabel> = csf.into_iter().collect();
    /// for x in &v {
    ///     println!("{x}");
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.labels.into_iter().into()
    }
}

impl Extend<CsfLabel> for CsfStringtable {
    fn extend<T: IntoIterator<Item = CsfLabel>>(&mut self, iter: T) {
        self.labels.extend(iter);
    }
}

/// A CSF label contains a name and a collection of CSF strings.
///
/// Every label in vanilla RA2/YR files contains only one string.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CsfLabel {
    /// Name of the label. Game rules, GUI and triggers look up this value.
    pub name: String,
    /// List of CSF strings associated with the label.
    pub strings: Vec<CsfString>,
}

impl CsfLabel {
    /// Creates a new [`CsfLabel`] from a label and a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::{CsfLabel, CsfString};
    ///
    /// let label = CsfLabel::new("A", "1");
    /// assert_eq!(label.name, "A");
    /// assert_eq!(label.strings.len(), 1);
    /// assert_eq!(label.strings[0], CsfString::new("1"));
    /// ```
    pub fn new(label: impl Into<String>, string: impl Into<String>) -> Self {
        CsfLabel {
            name: label.into(),
            strings: vec![CsfString::new(string)],
        }
    }

    /// Returns the first [`CsfString`] in a label or `None` if the label
    /// contains no strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::{CsfLabel, CsfString};
    ///
    /// let label = CsfLabel::default();
    /// let result = label.get_first();
    /// assert_eq!(result, None);
    ///
    /// let label = CsfLabel::new("A", "1");
    /// let result = label.get_first();
    /// assert_eq!(result, Some(&CsfString::new("1")));
    ///
    /// let label = CsfLabel { name: "A".to_string(), strings: vec![CsfString::new("1"), CsfString::new("2")] };
    /// let result = label.get_first();
    /// assert_eq!(result, Some(&CsfString::new("1")))
    /// ```
    pub fn get_first(&self) -> Option<&CsfString> {
        self.strings.first()
    }
}

impl PartialEq for CsfLabel {
    /// This method tests for self and other values to be equal, and is used
    /// by ==.
    ///
    /// [`CsfLabel`]s are considered equal if their names are equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::CsfLabel;
    ///
    /// let a = CsfLabel::new("A", "1");
    /// let b = CsfLabel::new("B", "2");
    /// let c = CsfLabel::new("A", "2");
    ///
    /// assert_eq!(a, c);
    /// assert_ne!(a, b);
    /// assert_ne!(b, c);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl std::cmp::Eq for CsfLabel {}

impl PartialOrd for CsfLabel {
    /// This method returns an [`Ordering`][std::cmp::Ordering] between self
    /// and other values if one exists.
    ///
    /// [`CsfLabel`]s ordering is equivalent to their names ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use rust_alert::csf::CsfLabel;
    ///
    /// let a = CsfLabel::new("A", "1");
    /// let b = CsfLabel::new("B", "2");
    /// let c = CsfLabel::new("A", "2");
    ///
    /// assert_eq!(a.partial_cmp(&c), Some(Ordering::Equal));
    /// assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    /// assert_eq!(b.partial_cmp(&c), Some(Ordering::Greater));
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl std::cmp::Ord for CsfLabel {
    /// This method returns an [`Ordering`][std::cmp::Ordering] between self
    /// and other.
    ///
    /// [`CsfLabel`]s ordering is equivalent to their names ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use rust_alert::csf::CsfLabel;
    ///
    /// let a = CsfLabel::new("A", "1");
    /// let b = CsfLabel::new("B", "2");
    ///
    /// assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    /// assert_eq!(b.partial_cmp(&a), Some(Ordering::Greater));
    /// assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));
    /// ```
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl std::hash::Hash for CsfLabel {
    /// Feeds this value into the given [`Hasher`][std::hash::Hasher].
    ///
    /// [`CsfLabel`]'s hash is equivalent to hash of its name.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::hash::{DefaultHasher, Hash, Hasher};
    /// use rust_alert::csf::CsfLabel;
    ///
    /// let mut hasher = DefaultHasher::new();
    /// CsfLabel::new("A", "1").hash(&mut hasher);
    /// let a = hasher.finish();
    ///
    /// let mut hasher = DefaultHasher::new();
    /// CsfLabel::new("B", "2").hash(&mut hasher);
    /// let b = hasher.finish();
    ///
    /// let mut hasher = DefaultHasher::new();
    /// CsfLabel::new("A", "2").hash(&mut hasher);
    /// let c = hasher.finish();
    ///
    /// assert_eq!(a, c);
    /// assert_ne!(a, b);
    /// assert_ne!(b, c);
    /// ```
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl std::fmt::Display for CsfLabel {
    /// Formats the value using the given [`Formatter`][std::fmt::Formatter].
    ///
    /// Only the first string of the label (if any) will be displayed.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::{CsfLabel, CsfString};
    ///
    /// let label = CsfLabel::new("A", "1");
    /// assert_eq!(format!("{label}"), "A: \"1\"");
    ///
    /// let label = CsfLabel { name: "B".to_string(), strings: vec![] };
    /// assert_eq!(format!("{label}"), "B: \"\"");
    ///
    /// let label = CsfLabel { name: "C".to_string(), strings: vec![CsfString::new("1"), CsfString::new("2")] };
    /// assert_eq!(format!("{label}"), "C: \"1\"");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: \"{}\"",
            self.name,
            self.get_first().unwrap_or(&CsfString::default())
        )
    }
}

/// A CSF string is a Unicode string serialized to a LE UTF-16 string. There
/// are two types of CSF strings: normal (prefix ` RTS`) and with extra value
/// (prefix `WRTS`) which can contain an additional ASCII string.
///
/// All vanilla game strings are normal (prefix ` RTS`).
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CsfString {
    /// Content of a string.
    pub value: String,
    /// Extra data associated with the string, serialized as plain ASCII.
    pub extra_value: Vec<u8>,
}

impl CsfString {
    /// Creates a new [CsfString] from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::CsfString;
    ///
    /// let string = CsfString::new("A");
    /// assert_eq!(string, CsfString { value: "A".to_string(), ..Default::default() });
    /// ```
    pub fn new(string: impl Into<String>) -> Self {
        CsfString {
            value: string.into(),
            ..Default::default()
        }
    }
}

impl std::fmt::Display for CsfString {
    /// Formats the value using the given [`Formatter`][std::fmt::Formatter].
    ///
    /// Only actual string contents will be displayed, extra data will be omitted.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_alert::csf::CsfString;
    ///
    /// let string = CsfString::new("A");
    /// assert_eq!(format!("{string}"), "A");
    ///
    /// let string = CsfString::new("");
    /// assert_eq!(format!("{string}"), "");
    ///
    /// let string = CsfString { value: "B".to_string(), extra_value: vec![32] };
    /// assert_eq!(format!("{string}"), "B");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<String> for CsfString {
    fn from(string: String) -> Self {
        CsfString {
            value: string.into(),
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
    use crate::{
        csf::{CsfLabel, CsfStringtable},
        unwrap_assert,
    };

    #[test]
    /// Test label creation.
    fn stringtable_create_label() {
        let label = "Label".to_string();
        let string = "String".to_string();

        let mut expected = CsfStringtable::default();
        expected.labels.insert(CsfLabel::new(&label, &string));
        let mut csf = CsfStringtable::default();
        csf.create(label, string);

        assert_eq!(csf, expected);
    }

    #[test]
    /// Test label addition.
    fn stringtable_add_label() {
        let label = "Label".to_string();
        let string = "String".to_string();

        let mut expected = CsfStringtable::default();
        expected.labels.insert(CsfLabel::new(&label, &string));
        let mut csf = CsfStringtable::default();
        csf.insert(CsfLabel::new(label, string));

        assert_eq!(csf, expected);
    }

    #[test]
    /// Test label removal.
    fn stringtable_remove_label() {
        let label = "Label".to_string();

        let expected = CsfStringtable::default();
        let mut csf = CsfStringtable::default();
        csf.labels.insert(CsfLabel::new(&label, "String"));
        csf.remove(&label);

        assert_eq!(csf, expected);
    }

    #[test]
    /// Test label lookup.
    fn stringtable_lookup_label() {
        let label = "Label".to_string();
        let string = "String".to_string();

        let mut csf = CsfStringtable::default();
        csf.labels.insert(CsfLabel::new(&label, &string));
        let actual = csf.get(&label);

        assert!(actual.is_some());
        unwrap_assert!(actual, &string);

        let actual = csf.get("NoString");
        assert!(actual.is_none());
    }

    #[test]
    /// Test label count.
    fn stringtable_count_labels() {
        let label = "Label".to_string();
        let label2 = "Label2".to_string();

        let expected = 2;
        let mut csf = CsfStringtable::default();
        csf.labels.insert(CsfLabel::new(label, "String"));
        csf.labels.insert(CsfLabel::new(label2, "String2"));
        let actual = csf.len();

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
        let mut lbl = CsfLabel::new(label, string);
        lbl.strings.push(string2.into());
        csf.labels.insert(lbl);
        let actual = csf.strings_len();

        assert_eq!(actual, expected);
    }

    #[test]
    /// Test label access.
    fn label_get_first() {
        let string = "String";

        let label = CsfLabel::new("Label", string);

        let expected = label.strings.first().unwrap_or_else(|| unreachable!());
        let actual = label.get_first();
        assert!(actual.is_some());
        unwrap_assert!(actual, expected);
    }
}
