//! MIX multitool.

use std::{
    fs::{write, OpenOptions},
    io::BufReader,
    io::Read,
    path::PathBuf,
};

use clap::{Parser, Subcommand};

use rust_alert::{
    converters::ini2db,
    ini::io::IniReader,
    mix::{
        db::{io::LocalMixDbReader, GlobalMixDatabase, LocalMixDatabase, MixDatabase},
        io::{MixReader, MixWriter},
        BlowfishKey, Mix, MixHeaderFlags, LMD_KEY_TD, LMD_KEY_TS,
    },
    printoptionmapln,
};

#[derive(Debug, thiserror::Error)]
enum Error {
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
    /// Build LMD for the MIX file.
    #[arg(short, long, default_value_t = false)]
    lmd: bool,
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
    /// Path to an output MIX file. Same as input by default.
    output: Option<PathBuf>,
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
    /// Recursively extract MIXes from MIXes to subfolders.
    #[arg(short, long, default_value_t = false)]
    recursive: bool,
    /// Path to a MIX database in INI format.
    #[arg(short, long)]
    db: Option<PathBuf>,
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
    /// Path to a MIX database in INI format.
    #[arg(short, long)]
    db: Option<PathBuf>,
}

fn read_db(path: &PathBuf) -> Result<MixDatabase> {
    let reader = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(reader);
    let ini = IniReader::read_file(reader)?;
    let db = ini2db(&ini)?;
    Ok(db)
}

fn read_lmd(mix: &Mix) -> Option<LocalMixDatabase> {
    let lmd = mix.get_file(if mix.is_new_mix {
        LMD_KEY_TS
    } else {
        LMD_KEY_TD
    });
    let lmd = lmd.map(|x| {
        let x: &mut dyn Read = &mut x.as_slice();
        LocalMixDbReader::read_file(x)
    });
    match lmd.transpose() {
        Ok(x) => x,
        Err(x) => {
            println!("Warning: found LMD, but failed to read it. Reason: {}", x);
            None
        }
    }
}

fn prepare_databases(mix: &Mix, gmd_path: &Option<PathBuf>) -> Result<(GlobalMixDatabase, bool)> {
    let mut mixdb = GlobalMixDatabase::default();
    let mut has_lmd = false;
    if let Some(lmd) = read_lmd(mix) {
        mixdb.dbs.push(lmd.db);
        has_lmd = true;
    }
    if let Some(gmd_path) = gmd_path {
        let db = read_db(gmd_path)?;
        mixdb.dbs.push(db);
    }
    Ok((mixdb, has_lmd))
}

fn build(args: &BuildArgs, new_mix: bool) -> Result<()> {
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args.output)?;
    let paths = std::fs::read_dir(&args.input)?;
    let mut mix = Mix::default();
    for res in paths {
        match res {
            Ok(path) => mix.force_file_path(path.path(), false)?,
            Err(e) => Err(e)?,
        }
    }

    let lmd = args.lmd.then(MixDatabase::default);
    mix.recalc();
    MixWriter::write_file(&mut writer, &mix, new_mix)?;
    Ok(())
}

fn checksum(args: &ChecksumArgs, new_mix: bool) -> Result<()> {
    Ok(())
}

fn compact(args: &CompactArgs, new_mix: bool) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let mut mix = MixReader::read_file(&mut reader, new_mix)?;
    mix.recalc_compact();
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.output.as_ref().unwrap_or(&args.input))?;
    MixWriter::write_file(&mut writer, &mix, new_mix)?;
    Ok(())
}

fn corrupt(args: &CorruptArgs, new_mix: bool) -> Result<()> {
    Ok(())
}

fn encrypt(args: &EncryptArgs, new_mix: bool) -> Result<()> {
    Ok(())
}

fn extract(args: &ExtractArgs, new_mix: bool) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    extract_inner(&mut reader, &args.output, args, new_mix)?;

    Ok(())
}

fn extract_inner(
    reader: &mut dyn Read,
    output_dir: &PathBuf,
    args: &ExtractArgs,
    new_mix: bool,
) -> Result<()> {
    let mix = MixReader::read_file(reader, new_mix)?;
    std::fs::create_dir_all(output_dir)?;
    let (mixdb, _) = prepare_databases(&mix, &args.db)?;

    for file in mix.files.values() {
        let filename = mixdb.get_name_or_id(file.index.id);

        if !args.quiet {
            println!("{}, {} bytes", filename, file.index.size);
        }
        if args.recursive && filename.ends_with(".mix") {
            let mix_reader: &mut dyn Read = &mut file.body.as_slice();
            extract_inner(mix_reader, &output_dir.join(filename), args, new_mix)?;
        } else {
            write(output_dir.join(filename), &file.body)?;
        }
    }

    Ok(())
}

fn inspect(args: &InspectArgs, new_mix: bool) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let mix = MixReader::read_file(&mut reader, new_mix)?;
    let (mixdb, has_lmd) = prepare_databases(&mix, &args.db)?;

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
        printoptionmapln!(
            "Blowfish key:       {:x?}",
            mix.blowfish_key,
            |x: BlowfishKey| x.map(|c| format!("{:X?}", c)).concat()
        );
        println!(
            "Checksum:           {:?}",
            mix.flags.contains(MixHeaderFlags::CHECKSUM)
        );
        println!("Has LMD:            {}", has_lmd);
    }

    if !args.no_index {
        println!();
        println!("File Offset Size");
        mix.files.values().for_each(|f| {
            println!(
                "{}: {:?} {:?}",
                mixdb.get_name_or_id(f.index.id),
                f.index.offset,
                f.index.size
            )
        });
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    //let args = Args{ command: Commands::Inspect(InspectArgs{ input: "enc.mix".into(), no_header: false, no_index: false, new_mix: false}) };// DEBUG

    match &args.command {
        Commands::Build(x) => build(x, args.new_mix),
        Commands::Checksum(x) => checksum(x, args.new_mix),
        Commands::Compact(x) => compact(x, args.new_mix),
        Commands::Corrupt(x) => corrupt(x, args.new_mix),
        Commands::Encrypt(x) => encrypt(x, args.new_mix),
        Commands::Extract(x) => extract(x, args.new_mix),
        Commands::Inspect(x) => inspect(x, args.new_mix),
    }
}
