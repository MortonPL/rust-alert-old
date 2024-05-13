use std::{fs::OpenOptions, io::Write, path::PathBuf};

use rust_alert::{
    core::{crc, GameEnum},
    mix::{
        db::{io::LocalMixDbWriter, LocalMixDatabase},
        io::MixWriter,
        Mix, LMD_KEY_TD, LMD_KEY_TS,
    },
    utils::path_to_filename,
};

use crate::{utils::encrypt_mix, Result, RunCommand};

#[derive(clap::Args)]
pub struct BuildCommand {
    /// Path to an input directory.
    input: PathBuf,
    /// Path to an output MIX file.
    output: PathBuf,
    /// Encrypt the MIX file.
    #[arg(short, long, default_value_t = false)]
    encrypt: bool,
    /// Recursively build inner MIXes from subfolders.
    #[arg(short, long, default_value_t = false)]
    recursive: bool,
    /// Path to a Blowfish key for encryption.
    /// 56 bytes of the file will be read and used as the key. Leave empty for a random key.
    #[arg(short, long)]
    key: Option<PathBuf>,
    /// Append SHA1 checksum to the MIX file.
    #[arg(short, long, default_value_t = false)]
    checksum: bool,
    /// Build LMD for the MIX file.
    #[arg(short, long, default_value_t = false)]
    lmd: bool,
    /// Use old CRC function (TD/RA).
    #[arg(short, long, default_value_t = false)]
    old_crc: bool,
    /// Allow to overwrite files with the same name.
    #[arg(long, default_value_t = false)]
    overwrite: bool,
}

impl RunCommand for BuildCommand {
    /// Build a MIX from files.
    fn run(self, force_new_format: bool, _safe_mode: bool) -> Result<()> {
        let mut writer = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.output)?;
        build_inner(&mut writer, &self.input, &self, force_new_format)
    }
}

fn build_inner(
    writer: &mut dyn Write,
    input: &PathBuf,
    args: &BuildCommand,
    new_mix: bool,
) -> Result<()> {
    let paths = std::fs::read_dir(input)?;
    let mut mix = Mix::default();
    let mut lmd = LocalMixDatabase::default();
    let crc_version = match args.old_crc {
        true => GameEnum::TD,
        false => GameEnum::YR,
    };
    for res in paths {
        let path = res?.path();
        let str = path_to_filename(&path)?;
        if path.is_dir() {
            let mut temp: Vec<u8> = vec![];
            build_inner(&mut temp, &path, args, new_mix)?;
            mix.add_file_raw(temp, crc(&str, crc_version))?;
        } else {
            mix.add_file_path(path, crc_version, args.overwrite)?;
        }
        if args.lmd {
            lmd.db.names.insert(crc(&str, crc_version), str);
        }
    }
    if args.lmd {
        let mut temp: Vec<u8> = vec![];
        LocalMixDbWriter::write_file(&mut temp, &lmd)?;
        let lmd_id = match args.old_crc {
            true => LMD_KEY_TD,
            false => LMD_KEY_TS,
        };
        mix.add_file_raw(temp, lmd_id)?;
    }
    if args.encrypt {
        encrypt_mix(&mut mix, &args.key)?;
    }
    if args.checksum {
        mix.calc_checksum();
    }
    MixWriter::write_file(writer, &mut mix, new_mix)?;
    Ok(())
}
