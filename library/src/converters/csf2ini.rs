use crate::{csf::CsfStringtable, ini::IniFile};

/// The error type for CSF-INI conversions.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The label has no strings.
    #[error("Label {0} contains no strings")]
    EmptyLabel(String),
    /// The label name is not in CATEGORY:NAME format.
    #[error("Label {0} is not in CATEGORY:NAME format, which is required")]
    NoSplit(String),
}

type Result<T> = std::result::Result<T, Error>;

/// Convert a stringtable to an INI file, with CSF categories grouped into sections
/// and strings/their values being entries.
/// 
/// # Examples
/// 
/// ```ignore
/// use rust_alert::{csf::CsfStringtable, ini::IniSection, converters::csf2ini};
/// 
/// let mut csf = CsfStringtable::default();
/// csf.create("BRIEF:ALL01", "Something");
/// let ini = csf2ini(csf);
///
/// let mut expected = IniSection::new("BRIEF");
/// expected.create_entry("ALL01", "Something");
///
/// assert!(ini.is_ok());
/// let ini = ini.unwrap();
/// assert_eq!(ini.get_section("BRIEF"), Some(&expected));
/// ```
pub fn csf2ini(mut csf: CsfStringtable) -> Result<IniFile> {
    let mut ini = IniFile::default();
    for label in csf.drain() {
        let value = &label
            .get_first()
            .ok_or(Error::EmptyLabel(label.name.to_string()))?
            .value
            .replace('\n', "\\n");
        let mut iter = label.name.split(':');
        let kv = match (iter.next(), iter.next()) {
            (Some(k), Some(v)) => Ok((k, v)),
            _ => Err(Error::NoSplit(label.name.to_string())),
        }?;
        ini.add_to_section(kv.0, kv.1, value);
    }
    Ok(ini)
}

/// Convert an INI file to a stringtable, with CSF categories grouped into sections
/// and strings/their values being entries.
/// 
/// # Examples
/// 
/// ```ignore
/// use rust_alert::{ini::IniFile, converters::ini2csf};
/// 
/// let mut ini = IniFile::default();
/// ini.add_to_section("BRIEF", "ALL01", "Something");
/// let csf = ini2csf(ini);
///
/// assert_eq!(csf.get_str("BRIEF:ALL01"), Some("Something"));
/// ```
pub fn ini2csf(mut ini: IniFile) -> CsfStringtable {
    let mut csf = CsfStringtable::default();
    for (name, mut section) in ini.drain() {
        for (key, entry) in section.drain() {
            let value = entry.value.replace("\\n", "\n");
            csf.create(format!("{name}:{key}"), value);
        }
    }
    csf
}

#[cfg(test)]
mod examples {
    use crate as rust_alert;

    #[test]
    fn _csf2ini() {
        use rust_alert::{csf::CsfStringtable, ini::IniSection, converters::csf2ini};

        let mut csf = CsfStringtable::default();
        csf.create("BRIEF:ALL01", "Something");
        let ini = csf2ini(csf);

        let mut expected = IniSection::new("BRIEF");
        expected.create_entry("ALL01", "Something");

        assert!(ini.is_ok());
        let ini = ini.unwrap();
        assert_eq!(ini.get_section("BRIEF"), Some(&expected));
    }

    #[test]
    fn _ini2csf() {
        use rust_alert::{ini::IniFile, converters::ini2csf};

        let mut ini = IniFile::default();
        ini.add_to_section("BRIEF", "ALL01", "Something");
        let csf = ini2csf(ini);

        assert_eq!(csf.get_str("BRIEF:ALL01"), Some("Something"));
    }
}

#[cfg(test)]
mod coverage {
    use crate::csf::CsfStringtable;
    use super::csf2ini;

    #[test]
    fn _csf2ini() {
        let mut csf = CsfStringtable::default();
        csf.create("BRIEFALL01", "Something");
        let ini = csf2ini(csf);
        assert!(ini.is_err());
        matches!(ini, Err(super::Error::NoSplit(_)));
    }
}
