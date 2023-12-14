use std::{fs::OpenOptions, path::PathBuf};

use rust_alert::{converters::db2ini, ini::io::IniWriter, mix::db::io::LocalMixDbReader};

use crate::{Result, RunCommand};

#[derive(clap::Args)]
pub struct ExtractCommand {
    /// Path to an input MIX database (.dat) file.
    input: PathBuf,
    /// Path to an output INI file.
    output: PathBuf,
}

impl RunCommand for ExtractCommand {
    fn run(self) -> Result<()> {
        let mut reader = OpenOptions::new().read(true).open(self.input)?;
        let mut writer = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.output)?;
        let mixdb = LocalMixDbReader::read_file(&mut reader)?;
        let ini = db2ini(mixdb.db);
        IniWriter::write_file(&ini, &mut writer)?;
        Ok(())
    }
}
