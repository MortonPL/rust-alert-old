use std::{fs::OpenOptions, path::PathBuf};

use rust_alert::{converters::csf2ini, csf::io::CsfReader, ini::io::IniWriter};

use crate::{Result, RunCommand};

#[derive(clap::Args)]
pub struct ExtractCommand {
    /// Path to an input CSF file.
    input: PathBuf,
    /// Path to an output INI file.
    output: PathBuf,
    /// Sort all strings.
    #[arg(short, long, default_value_t = false)]
    sort: bool,
}

impl RunCommand for ExtractCommand {
    fn run(self) -> Result<()> {
        let mut reader = OpenOptions::new().read(true).open(&self.input)?;
        let mut writer = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.output)?;
        let csf = CsfReader::read_file(&mut reader)?;
        let mut ini = csf2ini(csf)?;
        if self.sort {
            ini.sort_all();
        }
        IniWriter::write_file(&ini, &mut writer)?;
        Ok(())
    }
}