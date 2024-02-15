//! CSF (stringtable) structure definitions and manipulation methods.

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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
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

    /// Looks up the [`CsfLabel`] with given name.
    ///
    /// Returns reference to the value if a label is found, otherwise `None`.
    /// Also see [`get_str`][Self::get_str] to look up just the associated string.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::csf::CsfStringtable;
    ///
    /// let mut csf = CsfStringtable::default();
    /// csf.create("A", "1");
    ///
    /// let result = csf.get("A");
    /// assert_eq!(result, csf.iter().next());
    ///
    /// let result = csf.get("B");
    /// assert_eq!(result, None);
    /// ```
    pub fn get(&self, name: impl Into<String>) -> Option<&CsfLabel> {
        self.labels.get(&CsfLabel {
            name: name.into(),
            strings: vec![],
        })
    }

    /// Looks up the first string of a label with given name.
    ///
    /// Returns reference to the value if a label is found and contains any
    /// strings, otherwise `None`.
    /// Also see [`get`][Self::get] to look up an entire [`CsfLabel`].
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::csf::CsfStringtable;
    ///
    /// let mut csf = CsfStringtable::default();
    /// csf.create("A", "1");
    ///
    /// let result = csf.get_str("A");
    /// assert_eq!(result, Some("1"));
    ///
    /// let result = csf.get_str("B");
    /// assert_eq!(result, None);
    /// ```
    pub fn get_str(&self, name: impl Into<String>) -> Option<&str> {
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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
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
    /// Also see [`get_first_str`][CsfLabel::get_first_str] to just get the
    /// underlying `&str`.
    ///
    /// # Examples
    ///
    /// ```ignore
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
    /// assert_eq!(result, Some(&CsfString::new("1")));
    /// ```
    pub fn get_first(&self) -> Option<&CsfString> {
        self.strings.first()
    }

    /// Returns the first string in a label as a `&str` or `None` if the label
    /// contains no strings.
    ///
    /// Also see [`get_first`][CsfLabel::get_first] to access a whole
    /// [`CsfString`] in a similar manner.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::csf::{CsfLabel, CsfString};
    ///
    /// let label = CsfLabel::default();
    /// let result = label.get_first_str();
    /// assert_eq!(result, None);
    ///
    /// let label = CsfLabel::new("A", "1");
    /// let result = label.get_first_str();
    /// assert_eq!(result, Some("1"));
    ///
    /// let label = CsfLabel { name: "A".to_string(), strings: vec![CsfString::new("1"), CsfString::new("2")] };
    /// let result = label.get_first_str();
    /// assert_eq!(result, Some("1"));
    /// ```
    pub fn get_first_str(&self) -> Option<&str> {
        self.strings.first().and_then(|s| Some(s.value.as_str()))
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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
    /// use std::cmp::Ordering;
    /// use rust_alert::csf::CsfLabel;
    ///
    /// let a = CsfLabel::new("A", "1");
    /// let b = CsfLabel::new("B", "2");
    ///
    /// assert_eq!(a.cmp(&b), Ordering::Less);
    /// assert_eq!(b.cmp(&a), Ordering::Greater);
    /// assert_eq!(a.cmp(&a), Ordering::Equal);
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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
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
    fn from(value: String) -> Self {
        CsfString {
            value,
            ..Default::default()
        }
    }
}

impl From<CsfString> for String {
    fn from(string: CsfString) -> Self {
        string.value
    }
}

impl From<&str> for CsfString {
    fn from(value: &str) -> Self {
        CsfString {
            value: value.to_string(),
            ..Default::default()
        }
    }
}

impl AsRef<str> for CsfString {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod examples {
    mod csf_stringtable {
        use crate as rust_alert;

        #[test]
        fn new() {
            use rust_alert::csf::{CsfLanguageEnum, CsfStringtable, CsfVersionEnum};

            let csf = CsfStringtable::new(CsfVersionEnum::Cnc, CsfLanguageEnum::DE, 42);
            assert_eq!(csf.version, CsfVersionEnum::Cnc);
            assert_eq!(csf.language, CsfLanguageEnum::DE);
            assert_eq!(csf.extra, 42);
            assert_eq!(csf.len(), 0);
        }

        #[test]
        fn iter() {
            use rust_alert::csf::{CsfLabel, CsfStringtable};

            let mut csf = CsfStringtable::default();
            csf.insert(CsfLabel::new("A", "1"));
            csf.insert(CsfLabel::new("B", "2"));

            assert_eq!(csf.len(), 2);

            for x in csf.iter() {
                println!("{x}");
            }

            assert_eq!(csf.len(), 2);
        }

        #[test]
        fn drain() {
            use rust_alert::csf::{CsfLabel, CsfStringtable};

            let mut csf = CsfStringtable::default();
            csf.insert(CsfLabel::new("A", "1"));
            csf.insert(CsfLabel::new("B", "2"));

            assert_eq!(csf.len(), 2);

            for x in csf.drain() {
                println!("{x}");
            }

            assert_eq!(csf.len(), 0);
        }

        #[test]
        fn create() {
            use rust_alert::csf::{CsfLabel, CsfStringtable};

            let mut csf = CsfStringtable::default();
            let result = csf.create("A", "1");
            assert_eq!(result, None);
            assert_eq!(csf.len(), 1);

            let result = csf.create("A", "2");
            assert_eq!(result, Some(CsfLabel::new("A", "1")));
            assert_eq!(csf.len(), 1);
        }

        #[test]
        fn insert() {
            use rust_alert::csf::*;

            let mut csf = CsfStringtable::default();
            let label = CsfLabel::new("A", "1");

            let result = csf.insert(label.clone());
            assert_eq!(result, None);
            assert_eq!(csf.len(), 1);

            let result = csf.insert(CsfLabel::new("A", "2"));
            assert_eq!(result, Some(label));
            assert_eq!(csf.len(), 1);
        }

        #[test]
        fn remove() {
            use rust_alert::csf::{CsfLabel, CsfStringtable};

            let mut csf = CsfStringtable::default();
            csf.create("A", "1");
            csf.create("B", "2");

            assert_eq!(csf.len(), 2);

            let result = csf.remove("A");
            assert_eq!(result, Some(CsfLabel::new("A", "1")));
            assert_eq!(csf.len(), 1);

            let result = csf.remove("A");
            assert_eq!(result, None);
            assert_eq!(csf.len(), 1);
        }

        #[test]
        fn get() {
            use rust_alert::csf::CsfStringtable;

            let mut csf = CsfStringtable::default();
            csf.create("A", "1");

            let result = csf.get("A");
            assert_eq!(result, csf.iter().next());

            let result = csf.get("B");
            assert_eq!(result, None);
        }

        #[test]
        fn get_str() {
            use rust_alert::csf::CsfStringtable;

            let mut csf = CsfStringtable::default();
            csf.create("A", "1");

            let result = csf.get_str("A");
            assert_eq!(result, Some("1"));

            let result = csf.get_str("B");
            assert_eq!(result, None);
        }

        #[test]
        fn len() {
            use rust_alert::csf::CsfStringtable;

            let mut csf = CsfStringtable::default();

            let size = csf.len();
            assert_eq!(size, 0);

            csf.create("A", "1");
            let size = csf.len();
            assert_eq!(size, 1);

            csf.create("B", "2");
            let size = csf.len();
            assert_eq!(size, 2);
        }

        #[test]
        fn strings_len() {
            use rust_alert::csf::CsfStringtable;

            let mut csf = CsfStringtable::default();

            let size = csf.strings_len();
            assert_eq!(size, 0);

            csf.create("A", "1");
            let size = csf.strings_len();
            assert_eq!(size, 1);

            csf.create("B", "2");
            let size = csf.strings_len();
            assert_eq!(size, 2);
        }

        #[test]
        fn reserve() {
            use rust_alert::csf::CsfStringtable;

            let mut csf = CsfStringtable::default();
            csf.reserve(42);
        }

        #[test]
        fn into_iter() {
            use rust_alert::csf::{CsfLabel, CsfStringtable};

            let mut csf = CsfStringtable::default();
            csf.insert(CsfLabel::new("A", "1"));
            csf.insert(CsfLabel::new("B", "2"));

            // Can't use `csf` after this!
            let v: Vec<CsfLabel> = csf.into_iter().collect();
            for x in &v {
                println!("{x}");
            }
        }
    }

    mod csf_label {
        use crate as rust_alert;

        #[test]
        fn new() {
            use rust_alert::csf::{CsfLabel, CsfString};

            let label = CsfLabel::new("A", "1");
            assert_eq!(label.name, "A");
            assert_eq!(label.strings.len(), 1);
            assert_eq!(label.strings[0], CsfString::new("1"));
        }

        #[test]
        fn get_first() {
            use rust_alert::csf::{CsfLabel, CsfString};

            let label = CsfLabel::default();
            let result = label.get_first();
            assert_eq!(result, None);

            let label = CsfLabel::new("A", "1");
            let result = label.get_first();
            assert_eq!(result, Some(&CsfString::new("1")));

            let label = CsfLabel {
                name: "A".to_string(),
                strings: vec![CsfString::new("1"), CsfString::new("2")],
            };
            let result = label.get_first();
            assert_eq!(result, Some(&CsfString::new("1")));
        }

        #[test]
        fn get_first_str() {
            use rust_alert::csf::{CsfLabel, CsfString};

            let label = CsfLabel::default();
            let result = label.get_first_str();
            assert_eq!(result, None);

            let label = CsfLabel::new("A", "1");
            let result = label.get_first_str();
            assert_eq!(result, Some("1"));

            let label = CsfLabel {
                name: "A".to_string(),
                strings: vec![CsfString::new("1"), CsfString::new("2")],
            };
            let result = label.get_first_str();
            assert_eq!(result, Some("1"));
        }

        #[test]
        fn eq() {
            use rust_alert::csf::CsfLabel;

            let a = CsfLabel::new("A", "1");
            let b = CsfLabel::new("B", "2");
            let c = CsfLabel::new("A", "2");

            assert_eq!(a, c);
            assert_ne!(a, b);
            assert_ne!(b, c);
        }

        #[test]
        fn partial_cmp() {
            use rust_alert::csf::CsfLabel;
            use std::cmp::Ordering;

            let a = CsfLabel::new("A", "1");
            let b = CsfLabel::new("B", "2");
            let c = CsfLabel::new("A", "2");

            assert_eq!(a.partial_cmp(&c), Some(Ordering::Equal));
            assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
            assert_eq!(b.partial_cmp(&c), Some(Ordering::Greater));
        }

        #[test]
        fn cmp() {
            use rust_alert::csf::CsfLabel;
            use std::cmp::Ordering;

            let a = CsfLabel::new("A", "1");
            let b = CsfLabel::new("B", "2");

            assert_eq!(a.cmp(&b), Ordering::Less);
            assert_eq!(b.cmp(&a), Ordering::Greater);
            assert_eq!(a.cmp(&a), Ordering::Equal);
        }

        #[test]
        fn hash() {
            use rust_alert::csf::CsfLabel;
            use std::hash::{DefaultHasher, Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            CsfLabel::new("A", "1").hash(&mut hasher);
            let a = hasher.finish();

            let mut hasher = DefaultHasher::new();
            CsfLabel::new("B", "2").hash(&mut hasher);
            let b = hasher.finish();

            let mut hasher = DefaultHasher::new();
            CsfLabel::new("A", "2").hash(&mut hasher);
            let c = hasher.finish();

            assert_eq!(a, c);
            assert_ne!(a, b);
            assert_ne!(b, c);
        }

        #[test]
        fn fmt() {
            use rust_alert::csf::{CsfLabel, CsfString};

            let label = CsfLabel::new("A", "1");
            assert_eq!(format!("{label}"), "A: \"1\"");

            let label = CsfLabel {
                name: "B".to_string(),
                strings: vec![],
            };
            assert_eq!(format!("{label}"), "B: \"\"");

            let label = CsfLabel {
                name: "C".to_string(),
                strings: vec![CsfString::new("1"), CsfString::new("2")],
            };
            assert_eq!(format!("{label}"), "C: \"1\"");
        }
    }

    mod csf_string {
        use crate as rust_alert;

        #[test]
        fn new() {
            use rust_alert::csf::CsfString;

            let string = CsfString::new("A");
            assert_eq!(
                string,
                CsfString {
                    value: "A".to_string(),
                    ..Default::default()
                }
            );
        }

        #[test]
        fn fmt() {
            use rust_alert::csf::CsfString;

            let string = CsfString::new("A");
            assert_eq!(format!("{string}"), "A");

            let string = CsfString::new("");
            assert_eq!(format!("{string}"), "");

            let string = CsfString {
                value: "B".to_string(),
                extra_value: vec![32],
            };
            assert_eq!(format!("{string}"), "B");
        }
    }
}

#[cfg(test)]
mod coverage {
    mod csf_label {
        use crate::csf::CsfString;

        #[test]
        fn from_string() {
            let string = "A".to_string();
            let string: CsfString = string.into();

            assert_eq!(string.value, "A");
            assert!(string.extra_value.is_empty())
        }

        #[test]
        fn into_string() {
            let string = CsfString::new("A");
            let string: String = string.into();
            assert_eq!(string, "A");

            let string = CsfString {
                value: string,
                extra_value: vec![0],
            };
            let string: String = string.into();
            assert_eq!(string, "A");
        }

        #[test]
        fn from_ref_str() {
            let string: &str = "A";
            let string: CsfString = string.into();

            assert_eq!(string.value, "A");
            assert!(string.extra_value.is_empty());
        }

        #[test]
        fn as_ref_str() {
            let string = CsfString::new("A");
            let string_ref: &str = string.as_ref();

            assert_eq!(string_ref, "A");
        }
    }
}
