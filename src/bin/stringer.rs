//! CSF to INI and INI to CSF tool.

use std::{fs::OpenOptions, io::BufReader, path::PathBuf};

use clap::{Parser, Subcommand};

use rust_alert::{
    converters::{csf2ini, ini2csf},
    csf::{
        io::{CsfReader, CsfWriter},
        CsfLanguageEnum, CsfVersionEnum,
    },
    ini::io::{IniReader, IniWriter},
};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    CsfIO(#[from] rust_alert::csf::io::Error),
    #[error("{0}")]
    IniIO(#[from] rust_alert::ini::io::Error),
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
    Build(BuildArgs),
    /// Extract CSF contents to INI.
    Extract(ExtractArgs),
    /// Inspect CSF file.
    Inspect(InspectArgs),
}

#[derive(clap::Args)]
struct BuildArgs {
    /// Path to an input INI file.
    input: PathBuf,
    /// Path to an output CSF file.
    output: PathBuf,
    /// CSF language ID.
    #[arg(short, long, value_enum, default_value_t = CsfLanguageEnum::ENUS)]
    language: CsfLanguageEnum,
    /// CSF format version.
    #[arg(short, long, value_enum, default_value_t = CsfVersionEnum::Cnc)]
    version: CsfVersionEnum,
    /// Sort all strings.
    #[arg(short, long, default_value_t = false)]
    sort: bool,
}

#[derive(clap::Args)]
struct ExtractArgs {
    /// Path to an input CSF file.
    input: PathBuf,
    /// Path to an output INI file.
    output: PathBuf,
    /// Sort all strings.
    #[arg(short, long, default_value_t = false)]
    sort: bool,
}

#[derive(clap::Args)]
struct InspectArgs {
    /// Path to an input CSF file.
    input: PathBuf,
}

fn build(args: &BuildArgs) -> Result<()> {
    let reader = OpenOptions::new().read(true).open(&args.input)?;
    let mut reader = BufReader::new(reader);
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args.output)?;
    let mut ini = IniReader::read_file(&mut reader)?;
    if args.sort {
        ini.sort_all()
    }
    let mut csf = ini2csf(&ini);
    csf.language = args.language;
    csf.version = args.version;
    CsfWriter::write_file(&csf, &mut writer)?;
    Ok(())
}

fn extract(args: &ExtractArgs) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args.output)?;
    let csf = CsfReader::read_file(&mut reader)?;
    let mut ini = csf2ini(&csf)?;
    if args.sort {
        ini.sort_all();
    }
    IniWriter::write_file(&ini, &mut writer)?;
    Ok(())
}

fn inspect(args: &InspectArgs) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let csf = CsfReader::read_file(&mut reader)?;
    println!("Version:      {:?}", csf.version);
    println!("Language:     {:?}", csf.language);
    println!("Extra data:   {:X}", csf.extra);
    println!("# of labels:  {:?}", csf.get_label_count());
    println!("# of strings: {:?}", csf.get_string_count());
    println!(
        "Contains WSTRs: {:?}",
        csf.labels
            .values()
            .any(|l| l.get_first().is_some_and(|s| !s.extra_value.is_empty()))
    );
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Commands::Build(x) => build(x),
        Commands::Extract(x) => extract(x),
        Commands::Inspect(x) => inspect(x),
    }
}
