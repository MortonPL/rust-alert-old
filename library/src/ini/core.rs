//! INI structures and manipulation.

use indexmap::IndexMap;

use crate::ini::{Drain, IntoIter, Iter, SectionDrain, SectionIntoIter, SectionIter};

/// An INI file is general purpose, human readable, not standarised configuration format.
/// An INI file organises data into named sections that contain key-value pairs (entries).
/// Sections and entries can be looked-up and their order is maintained.
/// Each section has a unique name.
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IniFile {
    /// Map of sections indexed by their names.
    sections: IndexMap<String, IniSection>,
}

impl IniFile {
    /// Creates an iterator over file's sections and their names.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::ini::IniFile;
    /// 
    /// let mut ini = IniFile::default();
    /// ini.add_to_section("SECTIONA", "SomeKey", "SomeValue");
    /// ini.add_to_section("SECTIONB", "SomeKey", "OtherValue");
    ///
    /// for (_, section) in ini.iter() {
    ///     println!("{}", section.get_entry_str("SomeKey").unwrap());
    /// }
    /// ```
    pub fn iter(&self) -> Iter {
        self.sections.iter().into()
    }

    /// Creates a draining iterator over file's sections and their names.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::ini::IniFile;
    /// 
    /// let mut ini = IniFile::default();
    /// ini.add_to_section("SECTIONA", "SomeKey", "SomeValue");
    /// ini.add_to_section("SECTIONB", "SomeKey", "OtherValue");
    ///
    /// for (_, section) in ini.drain() {
    ///     println!("{}", section.get_entry_str("SomeKey").unwrap());
    /// }
    ///
    /// assert_eq!(ini.len(), 0);
    /// ```
    pub fn drain(&mut self) -> Drain {
        self.sections.drain(..).into()
    }

    /// Sort all sections by their names, alphabetically.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniFile;
    /// 
    /// let mut ini = IniFile::default();
    /// ini.add_to_section("B", "SomeKey", "SomeValue");
    /// ini.add_to_section("A", "SomeKey", "OtherValue");
    /// 
    /// ini.sort();
    /// 
    /// let mut iter = ini.iter();
    /// assert_eq!(iter.next().unwrap().0, "A");
    /// assert_eq!(iter.next().unwrap().0, "B");
    /// ```
    pub fn sort(&mut self) {
        self.sections.sort_keys();
    }

    /// Sort all sections and their entries, alphabetically.
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniFile;
    /// 
    /// let mut ini = IniFile::default();
    /// ini.add_to_section("A", "KeyB", "SomeValue");
    /// ini.add_to_section("A", "KeyA", "OtherValue");
    ///
    /// ini.sort_nested();
    ///
    /// let mut iter = ini.iter();
    /// let (name, section) = iter.next().unwrap();
    /// assert_eq!(name, "A");
    /// let mut iter = section.iter();
    /// assert_eq!(iter.next().unwrap().0, "KeyA");
    /// assert_eq!(iter.next().unwrap().0, "KeyB");
    /// ```
    pub fn sort_nested(&mut self) {
        self.sort();
        for (_, section) in self.sections.iter_mut() {
            section.sort();
        }
    }

    /// Look up section by name.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniFile;
    ///
    /// let mut ini = IniFile::default();
    /// ini.add_to_section("A", "KeyA", "SomeValue");
    /// ini.add_to_section("B", "KeyA", "OtherValue");
    ///
    /// let section = ini.get_section("A");
    /// assert!(section.is_some());
    /// assert_eq!(section.unwrap().get_name(), "A");
    /// ```
    pub fn get_section(&self, name: impl AsRef<str>) -> Option<&IniSection> {
        self.sections.get(name.as_ref())
    }

    /// Look up section by name as mutable.
    ///
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniFile;
    ///
    /// let mut ini = IniFile::default();
    /// ini.add_to_section("A", "KeyA", "SomeValue");
    /// ini.add_to_section("B", "KeyA", "OtherValue");
    ///
    /// let section = ini.get_section("A");
    /// assert!(section.is_some());
    /// assert_eq!(section.unwrap().get_name(), "A");
    /// ```
    pub fn get_section_mut(&mut self, name: impl AsRef<str>) -> Option<&mut IniSection> {
        self.sections.get_mut(name.as_ref())
    }

    /// Look up a value in a section, by entry's key.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniFile;
    ///
    /// let mut ini = IniFile::default();
    /// ini.add_to_section("A", "KeyA", "SomeValue");
    /// ini.add_to_section("A", "KeyB", "OtherValue");
    ///
    /// let value = ini.get_str("A", "KeyA");
    /// assert!(value.is_some());
    /// assert_eq!(value.unwrap(), "SomeValue");
    /// ```
    pub fn get_str(&self, section: impl AsRef<str>, entry: impl AsRef<str>) -> Option<&str> {
        self.sections
            .get(section.as_ref())
            .and_then(|s| s.get_entry_str(entry.as_ref()))
    }

    /// Insert section into file. If there is a section with the same name,
    /// it is replaced and the old value is returned.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniFile;
    ///
    /// let mut ini = IniFile::default();
    ///
    /// assert_eq!(ini.len(), 0);
    /// ini.add_section(IniSection::new("NewSection"));
    ///
    /// assert_eq!(ini.len(), 1);
    /// assert_eq!(ini.get_section("NewSection").unwrap().get_name(), "NewSection");
    /// ```
    pub fn add_section(&mut self, section: IniSection) -> Option<IniSection> {
        self.sections.insert(section.name.clone(), section)
    }

    /// Insert an entry to a section in this file. If there is an entry with the same key,
    /// it is replaced and the old value is returned. If there is no section with set name,
    /// it is created.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniFile;
    ///
    /// let mut ini = IniFile::default();
    ///
    /// ini.add_to_section("NewSection", "MyEntry", "Value");
    ///
    /// let value = ini.get_str("NewSection", "MyEntry");
    /// assert!(value.is_some());
    /// assert_eq!(value.unwrap(), "Value");
    /// ```
    pub fn add_to_section(
        &mut self,
        section: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Option<IniEntry> {
        let name = section.into();
        if let Some(section) = self.sections.get_mut(&name) {
            section.create_entry(key, value)
        } else {
            let mut section = IniSection::new(name);
            section.create_entry(key, value);
            self.add_section(section);
            None
        }
    }

    /// Remove a section from file. Old value or None is returned.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::{IniFile, IniSection};
    ///
    /// let mut ini = IniFile::default();
    /// ini.add_section(IniSection::new("A"));
    /// ini.add_section(IniSection::new("B"));
    /// assert_eq!(ini.len(), 2);
    ///
    /// ini.remove_section("B");
    ///
    /// assert_eq!(ini.len(), 1);
    /// assert!(ini.get_section("B").is_none());
    /// ```
    pub fn remove_section(&mut self, name: impl Into<String>) -> Option<IniSection> {
        self.sections.shift_remove(&name.into())
    }

    /// Get number of sections.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::{IniFile, IniSection};
    ///
    /// let mut ini = IniFile::default();
    /// assert_eq!(ini.len(), 0);
    ///
    /// ini.add_section(IniSection::new("A"));
    /// ini.add_section(IniSection::new("B"));
    /// assert_eq!(ini.len(), 2);
    ///
    /// ini.remove_section("B");
    /// assert_eq!(ini.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.sections.len()
    }
}

impl IntoIterator for IniFile {
    type Item = (String, IniSection);

    type IntoIter = IntoIter;

    /// Creates a consuming iterator that will move sections out of the
    /// INI file.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::ini::{IniFile, IniSection};
    ///
    /// let mut ini = IniFile::default();
    /// ini.add_section(IniSection::new("A"));
    /// ini.add_section(IniSection::new("B"));
    /// // Can't use `ini` after this!
    /// for (name, _) in ini.into_iter() {
    ///     println!("{name}");
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.sections.into_iter().into()
    }
}

/// An INI section is a representation of a named object, described by a collection
/// of key-value pairs.
#[derive(Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IniSection {
    name: String,
    entries: IndexMap<String, IniEntry>,
}

impl IniSection {
    /// Create a new section with given name.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    ///
    /// let section = IniSection::new("SectionName");
    /// assert_eq!(section.get_name(), "SectionName");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Creates an iterator over section's entries.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    /// 
    /// let mut ini = IniSection::default();
    /// ini.create_entry("Key", "Value");
    /// ini.create_entry("Key2", "Value2");
    /// 
    /// for (_, entry) in ini.iter() {
    ///     println!("{}", entry.value);
    /// }
    /// ```
    pub fn iter(&self) -> SectionIter {
        self.entries.iter().into()
    }

    /// Creates a draining iterator over section's entries.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    /// 
    /// let mut ini = IniSection::default();
    /// ini.create_entry("Key", "Value");
    /// ini.create_entry("Key2", "Value2");
    /// 
    /// for (_, entry) in ini.drain() {
    ///     println!("{}", entry.value);
    /// }
    ///
    /// assert_eq!(ini.len(), 0);
    /// ```
    pub fn drain(&mut self) -> SectionDrain {
        self.entries.drain(..).into()
    }

    /// Return the name of this section.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    ///
    /// let ini = IniSection::new("Name");
    /// assert_eq!(ini.get_name(), "Name");
    /// ```
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Sort all entries in this section alphabetically.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    ///
    /// let mut ini = IniSection::default();
    /// ini.create_entry("B", "SomeValue");
    /// ini.create_entry("A", "OtherValue");
    ///
    /// ini.sort();
    /// let mut iter = ini.iter();
    /// assert_eq!(iter.next().unwrap().0, "A");
    /// assert_eq!(iter.next().unwrap().0, "B");
    /// ```
    pub fn sort(&mut self) {
        self.entries.sort_keys();
    }

    /// Look up entry by key.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    ///
    /// let mut ini = IniSection::default();
    /// ini.create_entry("A", "SomeValue");
    /// ini.create_entry("B", "OtherValue");
    ///
    /// let entry = ini.get_entry("A");
    /// assert!(entry.is_some());
    /// assert_eq!(entry.unwrap().value, "SomeValue");
    /// ```
    pub fn get_entry(&self, key: impl Into<String>) -> Option<&IniEntry> {
        self.entries.get(&key.into())
    }

    /// Look up entry's value by key.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    ///
    /// let mut ini = IniSection::default();
    /// ini.create_entry("A", "SomeValue");
    /// ini.create_entry("B", "OtherValue");
    ///
    /// let value = ini.get_entry_str("A");
    /// assert!(value.is_some());
    /// assert_eq!(value.unwrap(), "SomeValue");
    /// ```
    pub fn get_entry_str(&self, key: impl Into<String>) -> Option<&str> {
        self.entries.get(&key.into()).map(|e| e.value.as_str())
    }

    /// Insert entry into section. If there is an entry with the same key,
    /// it is replaced and the old value is returned.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::{IniSection, IniEntry};
    ///
    /// let mut ini = IniSection::default();
    /// ini.add_entry(IniEntry::new("A", "Value"));
    /// ```
    pub fn add_entry(&mut self, entry: IniEntry) -> Option<IniEntry> {
        self.entries.insert(entry.key.clone(), entry)
    }

    /// Create an entry in section. If there is an entry with the same key,
    /// it is replaced and the old value is returned.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    ///
    /// let mut ini = IniSection::default();
    /// ini.create_entry("A", "Value");
    /// ```
    pub fn create_entry(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Option<IniEntry> {
        let key: String = key.into();
        self.entries.insert(
            key.clone(),
            IniEntry {
                key,
                value: value.into(),
            },
        )
    }

    /// Remove an entry from section. Old value or None is returned.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    ///
    /// let mut ini = IniSection::default();
    /// ini.create_entry("A", "Value");
    ///
    /// ini.remove_entry("A");
    /// assert_eq!(ini.len(), 0);
    /// ```
    pub fn remove_entry(&mut self, key: impl Into<String>) -> Option<IniEntry> {
        self.entries.shift_remove(&key.into())
    }

    /// Return the number of entries in this section.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    ///
    /// let mut ini = IniSection::default();
    /// assert_eq!(ini.len(), 0);
    ///
    /// ini.create_entry("A", "Value");
    /// assert_eq!(ini.len(), 1);
    ///
    /// ini.remove_entry("A");
    /// assert_eq!(ini.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl IntoIterator for IniSection {
    type Item = (String, IniEntry);

    type IntoIter = SectionIntoIter;

    /// Creates a consuming iterator that will move entries out of the
    /// section.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::ini::IniSection;
    ///
    /// let mut section = IniSection::default();
    /// section.create_entry("A", "Value");
    /// section.create_entry("B", "Value2");
    /// // Can't use `section` after this!
    /// for (name, _) in section.into_iter() {
    ///     println!("{name}");
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter().into()
    }
}

/// An INI entry is a key-value pair belonging to a section.
/// All data is stored as a string.
#[derive(Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IniEntry {
    /// Key of this entry.
    key: String,
    /// Value of this entry.
    pub value: String,
}

impl IniEntry {
    /// Create a new key-value pair entry.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::IniEntry;
    ///
    /// let entry = IniEntry::new("MyKey", "MyValue");
    /// assert_eq!(entry.value, "MyValue");
    /// ```
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod examples {
    mod ini_file {
        use crate::{self as rust_alert, ini::IniSection};

        #[test]
        fn iter() {
            use rust_alert::ini::IniFile;

            let mut ini = IniFile::default();
            ini.add_to_section("SECTIONA", "SomeKey", "SomeValue");
            ini.add_to_section("SECTIONB", "SomeKey", "OtherValue");

            for (_, section) in ini.iter() {
                println!("{}", section.get_entry_str("SomeKey").unwrap());
            }
        }

        #[test]
        fn drain() {
            use rust_alert::ini::IniFile;

            let mut ini = IniFile::default();
            ini.add_to_section("SECTIONA", "SomeKey", "SomeValue");
            ini.add_to_section("SECTIONB", "SomeKey", "OtherValue");

            for (_, section) in ini.drain() {
                println!("{}", section.get_entry_str("SomeKey").unwrap());
            }

            assert_eq!(ini.len(), 0);
        }

        #[test]
        fn sort() {
            use rust_alert::ini::IniFile;

            let mut ini = IniFile::default();
            ini.add_to_section("B", "SomeKey", "SomeValue");
            ini.add_to_section("A", "SomeKey", "OtherValue");

            ini.sort();

            let mut iter = ini.iter();
            assert_eq!(iter.next().unwrap().0, "A");
            assert_eq!(iter.next().unwrap().0, "B");
        }

        #[test]
        fn sort_nested() {
            use rust_alert::ini::IniFile;

            let mut ini = IniFile::default();
            ini.add_to_section("A", "KeyB", "SomeValue");
            ini.add_to_section("A", "KeyA", "OtherValue");

            ini.sort_nested();

            let mut iter = ini.iter();
            let (name, section) = iter.next().unwrap();
            assert_eq!(name, "A");
            let mut iter = section.iter();
            assert_eq!(iter.next().unwrap().0, "KeyA");
            assert_eq!(iter.next().unwrap().0, "KeyB");
        }

        #[test]
        fn get_section() {
            use rust_alert::ini::IniFile;

            let mut ini = IniFile::default();
            ini.add_to_section("A", "KeyA", "SomeValue");
            ini.add_to_section("B", "KeyA", "OtherValue");

            let section = ini.get_section("A");
            assert!(section.is_some());
            assert_eq!(section.unwrap().get_name(), "A");
        }

        #[test]
        fn get_section_mut() {
            use rust_alert::ini::IniFile;

            let mut ini = IniFile::default();
            ini.add_to_section("A", "KeyA", "SomeValue");
            ini.add_to_section("B", "KeyA", "OtherValue");

            let section = ini.get_section_mut("A");
            assert!(section.is_some());
            assert_eq!(section.unwrap().get_name(), "A");
        }

        #[test]
        fn get_str() {
            use rust_alert::ini::IniFile;

            let mut ini = IniFile::default();
            ini.add_to_section("A", "KeyA", "SomeValue");
            ini.add_to_section("A", "KeyB", "OtherValue");

            let value = ini.get_str("A", "KeyA");
            assert!(value.is_some());
            assert_eq!(value.unwrap(), "SomeValue");
        }

        #[test]
        fn add_section() {
            use rust_alert::ini::IniFile;

            let mut ini = IniFile::default();

            assert_eq!(ini.len(), 0);
            ini.add_section(IniSection::new("NewSection"));

            assert_eq!(ini.len(), 1);
            assert_eq!(ini.get_section("NewSection").unwrap().get_name(), "NewSection");
        }

        #[test]
        fn add_to_section() {
            use rust_alert::ini::IniFile;

            let mut ini = IniFile::default();

            ini.add_to_section("NewSection", "MyEntry", "Value");

            let value = ini.get_str("NewSection", "MyEntry");
            assert!(value.is_some());
            assert_eq!(value.unwrap(), "Value");
        }

        #[test]
        fn remove_section() {
            use rust_alert::ini::{IniFile, IniSection};

            let mut ini = IniFile::default();
            ini.add_section(IniSection::new("A"));
            ini.add_section(IniSection::new("B"));
            assert_eq!(ini.len(), 2);

            ini.remove_section("B");

            assert_eq!(ini.len(), 1);
            assert!(ini.get_section("B").is_none());
        }

        #[test]
        fn len() {
            use rust_alert::ini::{IniFile, IniSection};

            let mut ini = IniFile::default();
            assert_eq!(ini.len(), 0);

            ini.add_section(IniSection::new("A"));
            ini.add_section(IniSection::new("B"));
            assert_eq!(ini.len(), 2);

            ini.remove_section("B");
            assert_eq!(ini.len(), 1);
        }

        #[test]
        fn into_iter() {
            use rust_alert::ini::{IniFile, IniSection};

            let mut ini = IniFile::default();
            ini.add_section(IniSection::new("A"));
            ini.add_section(IniSection::new("B"));
            // Can't use `ini` after this!
            for (name, _) in ini.into_iter() {
                println!("{name}");
            }
        }

    }

    mod ini_section {
        use crate as rust_alert;

        #[test]
        fn new() {
            use rust_alert::ini::IniSection;

            let section = IniSection::new("SectionName");
            assert_eq!(section.get_name(), "SectionName");
        }

        #[test]
        fn iter() {
            use rust_alert::ini::IniSection;
            
            let mut ini = IniSection::default();
            ini.create_entry("Key", "Value");
            ini.create_entry("Key2", "Value2");
            
            for (_, entry) in ini.iter() {
                println!("{}", entry.value);
            }
        }

        #[test]
        fn drain() {
            use rust_alert::ini::IniSection;
            
            let mut ini = IniSection::default();
            ini.create_entry("Key", "Value");
            ini.create_entry("Key2", "Value2");
            
            for (_, entry) in ini.drain() {
                println!("{}", entry.value);
            }

            assert_eq!(ini.len(), 0);
        }

        #[test]
        fn get_name() {
            use rust_alert::ini::IniSection;

            let ini = IniSection::new("Name");
            assert_eq!(ini.get_name(), "Name");
        }

        #[test]
        fn sort() {
            use rust_alert::ini::IniSection;

            let mut ini = IniSection::default();
            ini.create_entry("B", "SomeValue");
            ini.create_entry("A", "OtherValue");

            ini.sort();
            let mut iter = ini.iter();
            assert_eq!(iter.next().unwrap().0, "A");
            assert_eq!(iter.next().unwrap().0, "B");
        }

        #[test]
        fn get_entry() {
            use rust_alert::ini::IniSection;

            let mut ini = IniSection::default();
            ini.create_entry("A", "SomeValue");
            ini.create_entry("B", "OtherValue");

            let entry = ini.get_entry("A");
            assert!(entry.is_some());
            assert_eq!(entry.unwrap().value, "SomeValue");
        }

        #[test]
        fn get_entry_str() {
            use rust_alert::ini::IniSection;

            let mut ini = IniSection::default();
            ini.create_entry("A", "SomeValue");
            ini.create_entry("B", "OtherValue");

            let value = ini.get_entry_str("A");
            assert!(value.is_some());
            assert_eq!(value.unwrap(), "SomeValue");
        }

        #[test]
        fn add_entry() {
            use rust_alert::ini::{IniSection, IniEntry};

            let mut ini = IniSection::default();
            ini.add_entry(IniEntry::new("A", "Value"));
        }

        #[test]
        fn create_entry() {
            use rust_alert::ini::IniSection;

            let mut ini = IniSection::default();
            ini.create_entry("A", "Value");
        }

        #[test]
        fn remove_entry() {
            use rust_alert::ini::IniSection;

            let mut ini = IniSection::default();
            ini.create_entry("A", "Value");

            ini.remove_entry("A");
            assert_eq!(ini.len(), 0);
        }

        #[test]
        fn len() {
            use rust_alert::ini::IniSection;

            let mut ini = IniSection::default();
            assert_eq!(ini.len(), 0);

            ini.create_entry("A", "Value");
            assert_eq!(ini.len(), 1);

            ini.remove_entry("A");
            assert_eq!(ini.len(), 0);
        }

        #[test]
        fn into_iter() {
            use rust_alert::ini::IniSection;

            let mut section = IniSection::default();
            section.create_entry("A", "Value");
            section.create_entry("B", "Value2");
            // Can't use `section` after this!
            for (name, _) in section.into_iter() {
                println!("{name}");
            }
        }
    }

    mod ini_entry {
        use crate as rust_alert;

        #[test]
        fn new() {
            use rust_alert::ini::IniEntry;

            let entry = IniEntry::new("MyKey", "MyValue");
            assert_eq!(entry.value, "MyValue");
        }
    }
}
