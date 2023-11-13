use crate::{csf::CsfStringtable, ini::IniFile};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Label {0} contains no strings")]
    EmptyLabel(String),
    #[error("Label {0} is not in CATEGORY:NAME format, which is required")]
    NoSplit(String),
}

type Result<T> = std::result::Result<T, Error>;

pub fn csf2ini(csf: &CsfStringtable) -> Result<IniFile> {
    let mut ini = IniFile::default();
    for (name, label) in csf.iter() {
        let value = &label
            .get_first()
            .ok_or(Error::EmptyLabel(name.to_string()))?
            .value
            .replace('\n', "\\n");
        let mut iter = name.split(':');
        let kv = match (iter.next(), iter.next()) {
            (Some(k), Some(v)) => Ok((k, v)),
            _ => Err(Error::NoSplit(name.to_string())),
        }?;
        ini.add_to_section(kv.0, kv.1, value);
    }
    Ok(ini)
}

pub fn ini2csf(ini: &IniFile) -> CsfStringtable {
    let mut csf = CsfStringtable::default();
    for (name, section) in ini.iter() {
        for (key, value) in section.iter() {
            let value = &value.value.replace("\\n", "\n");
            csf.create_label(format!("{name}:{key}"), value);
        }
    }
    csf
}
