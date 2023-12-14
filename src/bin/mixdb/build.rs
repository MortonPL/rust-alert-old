use std::{fs::OpenOptions, io::BufReader, path::PathBuf};

use rust_alert::{
    converters::ini2db,
    ini::io::IniReader,
    mix::db::{io::LocalMixDbWriter, LMDVersionEnum, LocalMixDatabase},
};

use crate::{Result, RunCommand};

#[derive(clap::Args)]
pub struct BuildCommand {
    /// Path to an input INI file.
    input: PathBuf,
    /// Path to an output MIX database (.dat) file.
    output: PathBuf,
    /// Name of the LMD version, defaults to YR / 6.
    #[arg(long, default_value_t = LMDVersionEnum::YR)]
    version: LMDVersionEnum,
}

impl RunCommand for BuildCommand {
    fn run(self) -> Result<()> {
        let reader = OpenOptions::new().read(true).open(self.input)?;
        let mut reader = BufReader::new(reader);
        let mut writer = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.output)?;
        let ini = IniReader::read_file(&mut reader)?;
        let mut lmd = LocalMixDatabase::default();
        lmd.db = ini2db(ini)?;
        lmd.version = self.version;
        LocalMixDbWriter::write_file(&mut writer, &lmd)?;
        Ok(())
    }
}
