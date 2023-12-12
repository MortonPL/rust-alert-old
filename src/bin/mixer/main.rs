//! MIX multitool.

use clap::{Parser, Subcommand};

use rust_alert::{utils::PathToStringError, make_app};

mod blowfish;
mod build;
mod checksum;
mod compact;
mod corrupt;
mod extract;
mod inspect;
mod utils;

use blowfish::BlowfishCommand;
use build::BuildCommand;
use checksum::ChecksumCommand;
use compact::CompactCommand;
use corrupt::CorruptCommand;
use extract::ExtractCommand;
use inspect::InspectCommand;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    Mix(#[from] rust_alert::mix::Error),
    #[error("{0}")]
    MixIO(#[from] rust_alert::mix::io::Error),
    #[error("{0}")]
    IniIO(#[from] rust_alert::ini::io::Error),
    #[error("{0}")]
    DatabaseConversionError(#[from] rust_alert::converters::DBConversionError),
    #[error("MIX doesn't contain a checksum")]
    MissingChecksum,
    #[error("Checksum in MIX and actual don't match")]
    InvalidChecksum,
    #[error("Cannot extract key out of a decrypted MIX")]
    MissingKey,
    #[error("{0}")]
    PathToStringError(#[from] PathToStringError),
    #[error("{0}")]
    LMDIOError(#[from] rust_alert::mix::db::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Parser)]
#[command(name = "mixer")]
#[command(author = "MortonPL <bartm12@wp.pl>")]
#[command(version = "1.0")]
#[command(about = "Extract, build, alter and inspect MIX files.", long_about = None)]
struct Args {
    #[command(subcommand)]
    /// Mode of operation.
    command: Commands,
    /// Force new mix format, useful if extra flags are non-0.
    #[arg(long, default_value_t = false)]
    new_mix: bool,
}

/// Modes of operation.
#[derive(Subcommand)]
enum Commands {
    /// Build MIX from files.
    Build(BuildCommand),
    /// Add/Remove/Check MIX checksum.
    Checksum(ChecksumCommand),
    /// Compact (remove unused data from) the MIX body.
    Compact(CompactCommand),
    /// Secure the MIX with a suite of anti-ripper corruptions.
    Corrupt(CorruptCommand),
    /// Encrypt/Decrypt a MIX file or extract stored key.
    Blowfish(BlowfishCommand),
    /// Extract MIX contents to folder.
    Extract(ExtractCommand),
    /// Inspect MIX file. Print general information such as header values,
    /// checksum, encryption key, as well as the file index.
    Inspect(InspectCommand),
}

impl RunCommand for Commands {
    fn run(self, force_new_format: bool) -> Result<()> {
        match self {
            Commands::Build(x) => x.run(force_new_format),
            Commands::Checksum(x) => x.run(force_new_format),
            Commands::Compact(x) => x.run(force_new_format),
            Commands::Corrupt(x) => x.run(force_new_format),
            Commands::Blowfish(x) => x.run(force_new_format),
            Commands::Extract(x) => x.run(force_new_format),
            Commands::Inspect(x) => x.run(force_new_format),
        }
    }
}

trait RunCommand {
    fn run(self, force_new_format: bool) -> Result<()>;
}

make_app!(Args, new_mix);
