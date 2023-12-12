use std::path::PathBuf;

use clap::Subcommand;

use rust_alert::mix::{Mix, MixHeaderFlags};

use crate::{
    utils::{read_mix, write_mix},
    Error, Result, RunCommand,
};

#[derive(clap::Args)]
pub struct ChecksumCommand {
    /// Mode of operation.
    #[command(subcommand)]
    mode: ChecksumMode,
    /// Path to an input MIX file.
    input: PathBuf,
    /// Path to an output MIX file. Same as input by default.
    output: Option<PathBuf>,
}

impl RunCommand for ChecksumCommand {
    /// Add checksum to MIX, remove checksum from MIX, or check if checksum in the MIX is true.
    fn run(self, force_new_format: bool) -> Result<()> {
        let mut mix = read_mix(&self.input, force_new_format)?;
        match self.mode {
            ChecksumMode::Add => {
                mix.calc_checksum();
                mix.flags.insert(MixHeaderFlags::CHECKSUM);
            }
            ChecksumMode::Remove => {
                mix.checksum = None;
                mix.flags.remove(MixHeaderFlags::CHECKSUM);
            }
            ChecksumMode::Check => {
                return compare_checksum(&mut mix);
            }
        };
        write_mix(&mut mix, &self.output, &self.input, force_new_format)?;
        match self.mode {
            ChecksumMode::Add => println!("Checksum added successfully."),
            ChecksumMode::Remove => println!("Checksum removed successfully."),
            _ => (),
        };
        Ok(())
    }
}

#[derive(Clone, Default, Subcommand)]
enum ChecksumMode {
    /// Add a SHA1 checksum of MIX body at the end of the file and set the checksum header flag.
    #[default]
    Add,
    /// Remove existing checksum from the MIX and clear the checksum header flag.
    Remove,
    /// Check if the attached checksum matches MIX body.
    /// Raises error when checksums don't match or the MIX contains no checksum.
    Check,
}

fn compare_checksum(mix: &mut Mix) -> Result<()> {
    let old = mix.checksum.ok_or(Error::MissingChecksum)?;
    mix.calc_checksum();
    let new = mix.checksum.unwrap_or_else(|| unreachable!());
    println!("In MIX: {}", old.map(|c| format!("{:X}", c)).concat());
    println!("Actual: {}", new.map(|c| format!("{:X}", c)).concat());
    if old.starts_with(&new) {
        println!("Matching checksum.");
        Ok(())
    } else {
        Err(Error::InvalidChecksum)
    }
}
