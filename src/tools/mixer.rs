// MIX tool.
use std::{
    fs::{write, OpenOptions},
    path::PathBuf,
};

use clap::{Parser, Subcommand};

use rust_alert::core::{
    mix::{Mix, MixHeaderFlags},
    mix_io::{MixReader, MixWriter},
};
use rust_alert::printoptionln;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    MixIO(#[from] rust_alert::core::mix_io::Error),
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
}

/// Modes of operation.
#[derive(Subcommand)]
enum Commands {
    /// Build MIX from files.
    Build(BuildArgs),
    /// Add/Remove MIX checksum.
    Checksum(ChecksumArgs),
    /// Compact the MIX body.
    Compact(CompactArgs),
    /// Secure the MIX with a suite of anti-ripper corruptions.
    Corrupt(CorruptArgs),
    /// Encrypt/Decrypt a MIX file.
    Encrypt(EncryptArgs),
    /// Extract MIX contents to folder.
    Extract(ExtractArgs),
    /// Inspect MIX file.
    Inspect(InspectArgs),
}

#[derive(clap::Args)]
struct BuildArgs {
    /// Path to an input directory.
    input: PathBuf,
    /// Path to an output MIX file.
    output: PathBuf,
    /// Encrypt the MIX file.
    #[arg(short, long, default_value_t = false)]
    encrypt: bool,
    /// Blowfish key to use for encryption. Leave empty for a random key.
    #[arg(short, long)]
    encryption_key: Option<String>,
    /// Append checksum to the MIX file.
    #[arg(short, long, default_value_t = false)]
    checksum: bool,
}

#[derive(clap::Args)]
struct ChecksumArgs {
    /// Path to an input MIX file.
    input: PathBuf,
    /// Remove the checksum instead.
    #[arg(short, long, default_value_t = false)]
    remove: bool,
}

#[derive(clap::Args)]
struct CompactArgs {
    /// Path to an input MIX file.
    input: PathBuf,
}

#[derive(clap::Args)]
struct CorruptArgs {
    /// Path to an input MIX file.
    input: PathBuf,
    /// Remove the LMD from the MIX.
    #[arg(long, default_value_t = false)]
    lmd_purge: bool,
    /// Leave a corrupted index entry for the LMD.
    #[arg(long, default_value_t = false)]
    lmd_corrupt_index: bool,
    /// Corrupt body size in the MIX header.
    #[arg(long, default_value_t = false)]
    header_corrupt_body: bool,
    /// Corrupt flags in the MIX header.
    #[arg(long, default_value_t = false)]
    header_corrupt_flags: bool,
    /// Corrupt extra flags in the MIX header.
    #[arg(long, default_value_t = false)]
    header_corrupt_flags_extra: bool,
}

#[derive(clap::Args)]
struct EncryptArgs {
    /// Path to an input MIX file.
    input: PathBuf,
    /// Decrypt the MIX file instead.
    #[arg(short, long, default_value_t = false)]
    decrypt: bool,
    /// Blowfish key to use for encryption. Leave empty for a random key.
    #[arg(short, long)]
    encryption_key: Option<String>,
}

#[derive(clap::Args)]
struct ExtractArgs {
    /// Path to an input MIX file.
    input: PathBuf,
    /// Path to an output directory.
    output: PathBuf,
    /// Do not print any messages.
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
}

#[derive(clap::Args)]
struct InspectArgs {
    /// Path to an input MIX file.
    input: PathBuf,
    /// Do not print the MIX header.
    #[arg(long, default_value_t = false)]
    no_header: bool,
    /// Do not print the file index.
    #[arg(long, default_value_t = false)]
    no_index: bool,
}

fn build(args: &BuildArgs) -> Result<()> {
    Ok(())
}

fn checksum(args: &ChecksumArgs) -> Result<()> {
    Ok(())
}

fn compact(args: &CompactArgs) -> Result<()> {
    Ok(())
}

fn corrupt(args: &CorruptArgs) -> Result<()> {
    Ok(())
}

fn encrypt(args: &EncryptArgs) -> Result<()> {
    Ok(())
}

fn extract(args: &ExtractArgs) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let mix = MixReader::read_file(&mut reader)?;
    std::fs::create_dir_all(&args.output)?;
    for file in mix.files.values() {
        let filename = file.get_name();
        if !args.quiet {
            println!("{}, {} bytes", filename, file.index.size);
        }
        write(args.output.join(filename), &file.body)?;
    }
    Ok(())
}

fn inspect(args: &InspectArgs) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let mix = MixReader::read_file(&mut reader)?;
    if !args.no_header {
        println!(
            "Mix type:           {}",
            if mix.is_new_mix { "new" } else { "old" }
        );
        println!("Mix flags:          {:?}", mix.flags);
        println!("Mix extra flags:    {:?}", mix.extra_flags);
        println!("# of files:         {:?}", mix.files.len());
        println!("Declared body size: {:?}", mix.body_size);
        println!(
            "Compact:            {:?}",
            !mix.files.values().any(|f| !f.residue.is_empty()) || !mix.residue.is_empty()
        );
        println!(
            "Encrypted:          {:?}",
            mix.flags.contains(MixHeaderFlags::ENCRYPTION)
        );
        printoptionln!("Blowfish key:       {:x?}", mix.blowfish_key);
        println!(
            "Checksum:           {:?}",
            mix.flags.contains(MixHeaderFlags::CHECKSUM)
        );
        printoptionln!("Local Mix Database: {:?}", mix.lmd);
    }
    if !args.no_index {
        println!();
        println!("File Offset Size");
        mix.files
            .values()
            .for_each(|f| println!("{}: {:?} {:?}", f.get_name(), f.index.offset, f.index.size));
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse_from(wild::args());

    //let args = Args{ command: Commands::Inspect(InspectArgs{ input: "expandmd14.mix".into()}) };// DEBUG

    match &args.command {
        Commands::Build(x) => build(x),
        Commands::Checksum(x) => checksum(x),
        Commands::Compact(x) => compact(x),
        Commands::Corrupt(x) => corrupt(x),
        Commands::Encrypt(x) => encrypt(x),
        Commands::Extract(x) => extract(x),
        Commands::Inspect(x) => inspect(x),
    }
}
