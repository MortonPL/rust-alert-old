use std::{fs::OpenOptions, io::Write, path::PathBuf};

use rust_alert::mix::Mix;

use crate::{
    utils::{encrypt_mix, read_mix, write_mix},
    Error, Result, RunCommand,
};

#[derive(clap::Args)]
pub struct BlowfishCommand {
    /// Mode of operation.
    mode: BlowfishMode,
    /// Path to an input MIX file.
    input: PathBuf,
    /// Path to an output MIX file. Same as input by default.
    output: Option<PathBuf>,
    /// Path to a Blowfish key.
    /// For encryption, 56 bytes of the file will be read and used as the key. Leave empty for a random key.
    /// For key extraction, the key will be written to this file. Leavy empty to write to stdout.
    #[arg(short, long)]
    key: Option<PathBuf>,
}

impl RunCommand for BlowfishCommand {
    /// Encrypt, decrypt MIX or extract the key.
    fn run(self, force_new_format: bool, _safe_mode: bool) -> Result<()> {
        let mut mix = read_mix(&self.input, force_new_format)?;
        match self.mode {
            BlowfishMode::Decrypt => {
                mix.set_blowfish_key(None);
                Ok(())
            }
            BlowfishMode::Encrypt => encrypt_mix(&mut mix, &self.key),
            BlowfishMode::Get => get_mix_key(&mix, &self.key),
        }?;
        write_mix(
            &mut mix,
            &self.output.unwrap_or(self.input),
            force_new_format,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum BlowfishMode {
    /// Decrypt the MIX header/index using attached Blowfish key.
    Decrypt,
    /// Encrypt the MIX header/index using provided or random Blowfish key.
    Encrypt,
    /// Output the Blowfish key attached to the MIX.
    Get,
}

impl std::fmt::Display for BlowfishMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

fn get_mix_key(mix: &Mix, key: &Option<PathBuf>) -> Result<()> {
    let mut writer: Box<dyn std::io::Write> = if let Some(key) = key {
        Box::new(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(key)?,
        )
    } else {
        Box::new(std::io::stdout())
    };
    let key = mix.blowfish_key.ok_or(Error::MissingKey)?;
    writer.write_all(&key)?;
    Ok(())
}
