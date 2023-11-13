use crate::{
    ini::{IniFile, IniSection},
    mix::db::MixDatabase,
    utils::hex2int,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    ParseIntError(#[from] crate::utils::ParseIntError),
}

type Result<T> = std::result::Result<T, Error>;

pub fn db2ini(db: &MixDatabase) -> IniFile {
    let mut ini = IniFile::default();
    let mut section = IniSection::new("MixDatabase");
    for (id, name) in &db.names {
        section.create_entry(format!("{:X}", id), name);
    }
    ini.add_section(section);
    ini
}

pub fn ini2db(ini: &IniFile) -> Result<MixDatabase> {
    let mut db = MixDatabase::default();
    for (id, name) in ini.iter().flat_map(|(_, x)| x.iter()) {
        let id = hex2int(id)?;
        db.names.insert(id, name.value.clone());
    }
    Ok(db)
}
