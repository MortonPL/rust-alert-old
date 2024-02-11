use std::{fs::OpenOptions, path::PathBuf};

use rust_alert::mix::{
    io::{MixReader, MixWriter},
    Mix,
};

use crate::Result;

pub fn read_mix(input: &PathBuf, new_mix: bool) -> Result<Mix> {
    let mut reader = OpenOptions::new().read(true).open(input)?;
    let mix = MixReader::read_file(&mut reader, new_mix)?;
    Ok(mix)
}

pub fn write_mix(mix: &mut Mix, output: &PathBuf, new_mix: bool) -> Result<()> {
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output)?;
    MixWriter::write_file(&mut writer, mix, new_mix)?;
    Ok(())
}
