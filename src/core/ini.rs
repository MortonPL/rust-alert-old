use indexmap::IndexMap;

#[derive(Debug, Default, PartialEq)]
pub struct IniFile {
    sections: IndexMap<String, IniSection>,
}

pub struct IniFileIter<'a> {
    iter: indexmap::map::Iter<'a, String, IniSection>,
}

impl<'a> Iterator for IniFileIter<'a> {
    type Item = (&'a String, &'a IniSection);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// An INI file organises data into named sections that contain key-value pairs (entries).
/// Sections and entries can be looked-up and their order is maintained.
impl IniFile {
    pub fn iter(&self) -> IniFileIter {
        IniFileIter {
            iter: self.sections.iter(),
        }
    }

    /// Sort all sections.
    pub fn sort(&mut self) {
        self.sections.sort_keys();
    }

    /// Sort all sections and their entries.
    pub fn sort_all(&mut self) {
        self.sort();
        for (_, section) in self.sections.iter_mut() {
            section.sort();
        }
    }

    /// Look up section by name.
    pub fn get_section(&self, name: impl Into<String>) -> Option<&IniSection> {
        self.sections.get(&name.into())
    }

    /// Look up section by name as mutable.
    pub fn get_section_mut(&mut self, name: impl Into<String>) -> Option<&mut IniSection> {
        self.sections.get_mut(&name.into())
    }

    /// Insert section into file. If there is a section with the same name,
    /// it is replaced and the old value is returned.
    pub fn add_section(&mut self, section: IniSection) -> Option<IniSection> {
        self.sections.insert(section.name.clone(), section)
    }

    /// Insert an entry to a section in this file. If there is an entry with the same key,
    /// it is replaced and the old value is returned. If there is no section with set name,
    /// it is created.
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
    pub fn remove_section(&mut self, name: impl Into<String>) -> Option<IniSection> {
        self.sections.remove(&name.into())
    }
}

/// An INI section is a representation of a named object, described by a collection
/// of key-value pairs. Each section has a unique name.
#[derive(Debug, Default, PartialEq)]
pub struct IniSection {
    name: String,
    entries: IndexMap<String, IniEntry>,
}

pub struct IniSectionIter<'a> {
    iter: indexmap::map::Iter<'a, String, IniEntry>,
}

impl<'a> Iterator for IniSectionIter<'a> {
    type Item = (&'a String, &'a IniEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl IniSection {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Get section name.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn iter(&self) -> IniSectionIter {
        IniSectionIter {
            iter: self.entries.iter(),
        }
    }

    /// Sort all entries.
    pub fn sort(&mut self) {
        self.entries.sort_keys();
    }

    /// Look up entry by key.
    pub fn get_entry(&self, key: impl Into<String>) -> Option<&IniEntry> {
        self.entries.get(&key.into())
    }

    /// Insert entry into section. If there is an entry with the same key,
    /// it is replaced and the old value is returned.
    pub fn add_entry(&mut self, entry: IniEntry) -> Option<IniEntry> {
        self.entries.insert(entry.key.clone(), entry)
    }

    /// Create an entry in section. If there is an entry with the same key,
    /// it is replaced and the old value is returned.
    pub fn create_entry(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Option<IniEntry> {
        let key: String = key.into();
        let key2 = key.clone();
        self.entries.insert(
            key,
            IniEntry {
                key: key2,
                value: value.into(),
            },
        )
    }

    /// Remove an entry from section. Old value or None is returned.
    pub fn remove_entry(&mut self, key: impl Into<String>) -> Option<IniEntry> {
        self.entries.remove(&key.into())
    }
}

/// An INI entry is a key-value pair belonging to a section.
/// All data is stored as a string.
#[derive(Debug, Default, PartialEq)]
pub struct IniEntry {
    key: String,
    pub value: String,
}

impl IniEntry {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}
