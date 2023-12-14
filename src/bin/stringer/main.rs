//! CSF to INI and INI to CSF tool.

use clap::{Parser, Subcommand};

mod build;
mod extract;
mod inspect;

use build::BuildCommand;
use extract::ExtractCommand;
use inspect::InspectCommand;
use rust_alert::make_app;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    CsfIO(#[from] rust_alert::csf::io::Error),
    #[error("{0}")]
    IniIO(#[from] rust_alert::ini::io::Error),
    #[error("{0}")]
    Csf(#[from] rust_alert::csf::Error),
    #[error("{0}")]
    Conversion(#[from] rust_alert::converters::CSFConversionError),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Parser)]
#[command(name = "stringer")]
#[command(author = "MortonPL <bartm12@wp.pl>")]
#[command(version = "1.0")]
#[command(about = "Extract CSF to INI and vice versa.", long_about = None)]
struct Args {
    #[command(subcommand)]
    /// Mode of operation.
    command: Commands,
}

/// Modes of operation.
#[derive(Subcommand)]
enum Commands {
    /// Build CSF from INI.
    Build(BuildCommand),
    /// Extract CSF contents to INI.
    Extract(ExtractCommand),
    /// Inspect CSF file.
    Inspect(InspectCommand),
}

trait RunCommand {
    fn run(self) -> Result<()>;
}

impl RunCommand for Commands {
    fn run(self) -> Result<()> {
        match self {
            Commands::Build(x) => x.run(),
            Commands::Extract(x) => x.run(),
            Commands::Inspect(x) => x.run(),
        }
    }
}

make_app!(Args);
