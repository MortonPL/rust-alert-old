//! MIX security tool.

use clap::{Parser, Subcommand};

use rust_alert::make_app;

mod crack;
mod lock;
mod utils;

use crack::CrackCommand;
use lock::LockCommand;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    Mix(#[from] rust_alert::mix::Error),
    #[error("{0}")]
    MixIO(#[from] rust_alert::mix::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Parser)]
#[command(name = "mixcracker")]
#[command(author = "MortonPL <bartm12@wp.pl>")]
#[command(version = "1.0")]
#[command(about = "Crack or lock MIX files.", long_about = None)]
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
    /// Crack MIX security methods.
    Crack(CrackCommand),
    /// Lock the MIX with anti-ripping methods.
    Lock(LockCommand),
}

impl RunCommand for Commands {
    fn run(self, force_new_format: bool) -> Result<()> {
        match self {
            Commands::Crack(x) => x.run(force_new_format),
            Commands::Lock(x) => x.run(force_new_format),
        }
    }
}

trait RunCommand {
    fn run(self, force_new_format: bool) -> Result<()>;
}

make_app!(Args, new_mix);
