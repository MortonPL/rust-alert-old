use std::{fs::OpenOptions, io::BufReader, path::PathBuf};

use rust_alert::{
    core::{crc, GameEnum},
    ini::{
        io::{IniReader, IniWriter},
        IniFile, IniSection,
    },
};

use crate::{Result, RunCommand};

#[derive(clap::Args)]
pub struct ProcessCommand {
    /// Path to an input INI file.
    input: PathBuf,
    /// Path to an output INI file.
    output: PathBuf,
    /// Use old (TD/RA) CRC algorithm.
    #[arg(short, long, default_value_t = false)]
    old_crc: bool,
}

impl RunCommand for ProcessCommand {
    fn run(self) -> Result<()> {
        let reader = OpenOptions::new().read(true).open(self.input)?;
        let mut reader = BufReader::new(reader);
        let mut ini = IniReader::read_file(&mut reader)?;
        let mut new_ini = IniFile::default();
        let game = if self.old_crc {
            GameEnum::TD
        } else {
            GameEnum::YR
        };
        for (section_name, mut section) in ini.drain() {
            let mut new_section = IniSection::new(section_name);
            for (_, entry) in section.drain() {
                new_section.create_entry(format!("{:0>8X}", crc(&entry.value, game)), entry.value);
            }
            new_ini.add_section(new_section);
        }
        let mut writer = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.output)?;
        IniWriter::write_file(&new_ini, &mut writer)?;
        Ok(())
    }
}
