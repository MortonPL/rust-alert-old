use crate::{csf::CsfStringtable, ini::IniFile};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Label {0} contains no strings")]
    EmptyLabel(String),
    #[error("Label {0} is not in CATEGORY:NAME format, which is required")]
    NoSplit(String),
}

type Result<T> = std::result::Result<T, Error>;

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

pub fn ini2csf(mut ini: IniFile) -> CsfStringtable {
    let mut csf = CsfStringtable::default();
    for (name, mut section) in ini.drain() {
        for (key, value) in section.drain() {
            let value = value.value.replace("\\n", "\n");
            csf.create(format!("{name}:{key}"), value);
        }
    }
    csf
}
