use crate::{
    ini::{IniFile, IniSection},
    mix::db::MixDatabase,
    utils::hex2int,
};

/// The error type for DB-INI conversions.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to parse a hex string as integer.
    #[error("{0}")]
    ParseIntError(#[from] crate::utils::ParseIntError),
}

type Result<T> = std::result::Result<T, Error>;

/// Convert a Mix DB file to an INI file, writing filenames and hashes as entries.
/// 
/// # Examples
/// 
/// ```ignore
/// use rust_alert::{mix::db::MixDatabase, converters::db2ini};
/// 
/// let mut db = MixDatabase::default();
/// db.names.insert(1, "a".to_string());
/// let ini = db2ini(db);
///
/// assert_eq!(ini.get_str("MixDatabase", "00000001"), Some("a"));
/// ```
pub fn db2ini(mut db: MixDatabase) -> IniFile {
    let mut ini = IniFile::default();
    let mut section = IniSection::new("MixDatabase");
    for (id, name) in db.names.drain() {
        section.create_entry(format!("{:0>8X}", id), name);
    }
    ini.add_section(section);
    ini
}

/// Convert an INI file to a Mix DB file.
/// 
/// # Examples
/// 
/// ```ignore
/// use rust_alert::{ini::IniFile, converters::ini2db};
/// 
/// let mut ini = IniFile::default();
/// ini.add_to_section("MixDatabase", "00000001", "a");
/// let db = ini2db(ini);
///
/// assert!(db.is_ok());
/// let db = db.unwrap();
/// assert_eq!(db.names.get(&1), Some(&"a".to_string()));
/// ```
pub fn ini2db(mut ini: IniFile) -> Result<MixDatabase> {
    let mut db = MixDatabase::default();
    for (_, mut section) in ini.drain() {
        for (key, entry) in section.drain() {
            let id = hex2int(key.as_str())?;
            db.names.insert(id, entry.value);
        }
    }
    Ok(db)
}

#[cfg(test)]
mod examples {
    use crate as rust_alert;

    #[test]
    fn _db2ini() {
        use rust_alert::{mix::db::MixDatabase, converters::db2ini};

        let mut db = MixDatabase::default();
        db.names.insert(1, "a".to_string());
        let ini = db2ini(db);

        assert_eq!(ini.get_str("MixDatabase", "00000001"), Some("a"));
    }

    #[test]
    fn _ini2db() {
        use rust_alert::{ini::IniFile, converters::ini2db};

        let mut ini = IniFile::default();
        ini.add_to_section("MixDatabase", "00000001", "a");
        let db = ini2db(ini);

        assert!(db.is_ok());
        let db = db.unwrap();
        assert_eq!(db.names.get(&1), Some(&"a".to_string()));
    }
}
