use std::{
    fs::OpenOptions,
    io::{BufReader, Read},
    path::PathBuf,
};

use rust_alert::{
    converters::ini2db,
    defaultarray,
    ini::io::IniReader,
    mix::{
        db::{io::LocalMixDbReader, GlobalMixDatabase, LocalMixDatabase, MixDatabase},
        io::{generate_blowfish, MixReader, MixWriter},
        BlowfishKey, Mix, LMD_KEY_TD, LMD_KEY_TS,
    },
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

pub fn encrypt_mix(mix: &mut Mix, key: &Option<PathBuf>) -> Result<()> {
    let key = if let Some(key) = key {
        let mut reader = OpenOptions::new().read(true).open(key)?;
        let mut key = defaultarray!(BlowfishKey);
        reader.read_exact(&mut key)?;
        Some(key)
    } else {
        Some(generate_blowfish())
    };
    mix.set_blowfish_key(key);
    Ok(())
}

/// Read a MIX database from an INI file.
pub fn read_db(path: &PathBuf) -> Result<MixDatabase> {
    let reader = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(reader);
    let ini = IniReader::read_file(reader)?;
    let db = ini2db(ini)?;
    Ok(db)
}

/// Read an LMD from inside a MIX.
pub fn read_lmd(mix: &Mix) -> Option<LocalMixDatabase> {
    let lmd = mix.get_file(if mix.is_new_format {
        LMD_KEY_TS
    } else {
        LMD_KEY_TD
    });

    let lmd = if let Some(mut lmd) = lmd {
        let x: &mut dyn Read = &mut lmd;
        Some(LocalMixDbReader::read_file(x))
    } else {
        None
    };

    match lmd.transpose() {
        Ok(x) => x,
        Err(x) => {
            println!("Warning: found LMD, but failed to read it. Reason: {}", x);
            None
        }
    }
}

/// Read GMD & LMD and merge them.
pub fn prepare_databases(
    mix: &Mix,
    gmd: MixDatabase,
    safe_mode: bool,
) -> Result<(GlobalMixDatabase, bool)> {
    let mut mixdb = GlobalMixDatabase::default();
    let mut has_lmd = false;
    if !safe_mode {
        if let Some(lmd) = read_lmd(mix) {
            mixdb.dbs.push(lmd.db);
            has_lmd = true;
        }
    }
    mixdb.dbs.push(gmd);
    Ok((mixdb, has_lmd))
}
