//! MIX database multitool.

use clap::{Parser, Subcommand};

mod build;
mod extract;
mod inspect;
mod process;
mod query;
mod scan;

use build::BuildCommand;
use extract::ExtractCommand;
use inspect::InspectCommand;
use process::ProcessCommand;
use query::QueryCommand;
use rust_alert::make_app;
use scan::ScanCommand;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    IniIO(#[from] rust_alert::ini::io::Error),
    #[error("{0}")]
    LmdIO(#[from] rust_alert::mix::db::io::Error),
    #[error("{0}")]
    DBConversionError(#[from] rust_alert::converters::DBConversionError),
    #[error("{0}")]
    ParseIntError(#[from] rust_alert::utils::ParseIntError),
    #[error("{0}")]
    PathToStringError(#[from] rust_alert::utils::PathToStringError),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Parser)]
#[command(name = "mixdb")]
#[command(author = "MortonPL <bartm12@wp.pl>")]
#[command(version = "1.0")]
#[command(about = "Manipulate MIX database files.", long_about = None)]
struct Args {
    #[command(subcommand)]
    /// Mode of operation.
    command: Commands,
}

/// Modes of operation.
#[derive(Subcommand)]
enum Commands {
    /// Build a database from an INI DB file.
    Build(BuildCommand),
    /// Extract names from the database into an INI file.
    Extract(ExtractCommand),
    /// Process names from an INI file to an INI DB file.
    Process(ProcessCommand),
    /// Inspect the database header contents.
    Inspect(InspectCommand),
    /// Query the database for an index or name.
    Query(QueryCommand),
    /// Scan directory and construct an INI DB file.
    Scan(ScanCommand),
}

trait RunCommand {
    fn run(self) -> Result<()>;
}

impl RunCommand for Commands {
    fn run(self) -> Result<()> {
        match self {
            Commands::Build(x) => x.run(),
            Commands::Extract(x) => x.run(),
            Commands::Process(x) => x.run(),
            Commands::Inspect(x) => x.run(),
            Commands::Query(x) => x.run(),
            Commands::Scan(x) => x.run(),
        }
    }
}

make_app!(Args);
