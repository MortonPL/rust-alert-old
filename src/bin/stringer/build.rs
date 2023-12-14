use std::{fs::OpenOptions, io::BufReader, path::PathBuf};

use rust_alert::{
    converters::ini2csf,
    csf::{io::CsfWriter, CsfLanguageEnum, CsfVersionEnum},
    ini::io::IniReader,
};

use crate::{Result, RunCommand};

#[derive(clap::Args)]
pub struct BuildCommand {
    /// Path to an input INI file.
    input: PathBuf,
    /// Path to an output CSF file.
    output: PathBuf,
    /// CSF language ID.
    #[arg(short, long, value_enum, default_value_t = CsfLanguageEnum::ENUS)]
    language: CsfLanguageEnum,
    /// CSF format version.
    #[arg(short, long, value_enum, default_value_t = CsfVersionEnum::Cnc)]
    version: CsfVersionEnum,
    /// Sort all strings.
    #[arg(short, long, default_value_t = false)]
    sort: bool,
}

impl RunCommand for BuildCommand {
    fn run(self) -> Result<()> {
        let reader = OpenOptions::new().read(true).open(&self.input)?;
        let mut reader = BufReader::new(reader);
        let mut writer = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.output)?;
        let mut ini = IniReader::read_file(&mut reader)?;
        if self.sort {
            ini.sort_all()
        }
        let mut csf = ini2csf(ini);
        csf.language = self.language;
        csf.version = self.version;
        CsfWriter::write_file(&csf, &mut writer)?;
        Ok(())
    }
}
