use std::{fs::OpenOptions, path::PathBuf};

use rust_alert::{
    core::{crc, GameEnum},
    ini::{io::IniWriter, IniFile, IniSection},
    utils::path_to_str,
};

use crate::{Result, RunCommand};

#[derive(clap::Args)]
pub struct ScanCommand {
    /// Path to an input directory.
    input: PathBuf,
    /// Path to an output INI file.
    output: PathBuf,
    /// Use old (TD/RA) CRC algorithm.
    #[arg(short, long, default_value_t = false)]
    old_crc: bool,
}

impl RunCommand for ScanCommand {
    fn run(self) -> Result<()> {
        let game = if self.old_crc {
            GameEnum::TD
        } else {
            GameEnum::YR
        };
        let mut ini = IniFile::default();
        let mut section = IniSection::new("MixDatabase");
        inner(self.input, &mut section, game)?;
        ini.add_section(section);
        let mut writer = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.output)?;
        IniWriter::write_file(&ini, &mut writer)?;
        Ok(())
    }
}

fn inner(path: PathBuf, section: &mut IniSection, game: GameEnum) -> Result<()> {
    let paths = std::fs::read_dir(path)?;
    for res in paths {
        let path = res?.path();
        let str = path_to_str(&path)?;
        if path.is_dir() {
            inner(path, section, game)?;
        } else {
            section.create_entry(format!("{:0>8X}", crc(&str, game)), str);
        }
    }
    Ok(())
}
