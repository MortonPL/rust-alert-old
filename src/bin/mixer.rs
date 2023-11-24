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
    core::GameEnum,
    ini::io::IniReader,
    mix::{
        db::{io::LocalMixDbReader, GlobalMixDatabase, LocalMixDatabase, MixDatabase},
        io::{MixReader, MixWriter},
        BlowfishKey, Checksum, Mix, MixHeaderFlags, LMD_KEY_TD, LMD_KEY_TS,
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
    /// Add/Remove/Check MIX checksum.
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
    encryption_key: Option<String>, // todo stricter type
    /// Append SHA1 checksum to the MIX file.
    #[arg(short, long, default_value_t = false)]
    checksum: bool,
    /// Build LMD for the MIX file.
    #[arg(short, long, default_value_t = false)]
    lmd: bool,
}

#[derive(clap::Args)]
struct ChecksumArgs {
    /// Mode of operation.
    #[command(subcommand)]
    mode: ChecksumMode,
    /// Path to an input MIX file.
    input: PathBuf,
    /// Path to an output MIX file. Same as input by default.
    output: Option<PathBuf>,
}

#[derive(Clone, Default, Subcommand)]
enum ChecksumMode {
    /// Add a SHA1 checksum at the end of the MIX.
    #[default]
    Add,
    /// Remove existing checksum from the MIX.
    Remove,
    /// Check if the attached checksum matches MIX body.
    Check,
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
    /// Path to an output MIX file. Same as input by default.
    output: Option<PathBuf>,
    /// Decrypt the MIX file instead.
    #[arg(short, long, default_value_t = false)]
    decrypt: bool,
    /// Blowfish key to use for encryption. Leave empty for a random key.
    #[arg(short, long)]
    encryption_key: Option<String>, // todo stricter type
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
    let lmd = mix.get_file(if mix.is_new_format {
        LMD_KEY_TS
    } else {
        LMD_KEY_TD
    });

    let lmd = if let Some(mut lmd) = lmd {
        let x: &mut dyn Read = &mut lmd;
        Some(LocalMixDbReader::read_file(x))
    } else {
        None
    };

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
            Ok(path) => mix.add_file_path(path.path(), GameEnum::YR, false)?, // todo crc and overwite policy as arg
            Err(e) => Err(e)?,
        }
    }

    if args.checksum {
        mix.calc_checksum();
    }
    if args.encrypt {
        todo!(); // Generate an encryption key.
        mix.flags.insert(MixHeaderFlags::ENCRYPTION);
    }

    let lmd = args.lmd.then(MixDatabase::default); // todo make LMD from files
    mix.recalc();
    MixWriter::write_file(&mut writer, &mut mix, new_mix)?;
    Ok(())
}

fn checksum(args: &ChecksumArgs, new_mix: bool) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let mut mix = MixReader::read_file(&mut reader, new_mix)?;

    match args.mode {
        ChecksumMode::Add => {
            mix.calc_checksum();
            mix.flags.insert(MixHeaderFlags::CHECKSUM);
        }
        ChecksumMode::Remove => {
            mix.checksum = None;
            mix.flags.remove(MixHeaderFlags::CHECKSUM);
        }
        ChecksumMode::Check => {
            let old = mix.checksum.clone().unwrap();
            mix.calc_checksum();
            let new = mix.checksum.unwrap_or_else(|| unreachable!());
            if old.starts_with(&new) {
                println!("Valid checksum.");
            } else {
                println!("Invalid checksum.");
                println!(
                    "Appended to MIX: {}",
                    old.map(|c| format!("{:X}", c)).concat()
                );
                println!("Actual: {}", new.map(|c| format!("{:X}", c)).concat());
            }
        }
    };

    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.output.as_ref().unwrap_or(&args.input))?;
    MixWriter::write_file(&mut writer, &mut mix, new_mix)?;
    Ok(())
}

fn compact(args: &CompactArgs, new_mix: bool) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let mut mix = MixReader::read_file(&mut reader, new_mix)?;
    mix.recalc();
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.output.as_ref().unwrap_or(&args.input))?;
    MixWriter::write_file(&mut writer, &mut mix, new_mix)?;
    Ok(())
}

fn corrupt(args: &CorruptArgs, new_mix: bool) -> Result<()> {
    Ok(())
}

fn encrypt(args: &EncryptArgs, new_mix: bool) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let mut mix = MixReader::read_file(&mut reader, new_mix)?;
    if args.decrypt {
        mix.set_blowfish_key(None);
    } else {
        todo!(); // generate a key
        let blowfish_key = Some([0u8; 56]);
        mix.set_blowfish_key(blowfish_key);
    }
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.output.as_ref().unwrap_or(&args.input))?;
    MixWriter::write_file(&mut writer, &mut mix, new_mix)?;
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

    for file in mix.index.values() {
        let filename = mixdb.get_name_or_id(file.id);

        if !args.quiet {
            println!("{}, {} bytes", filename, file.size);
        }
        if args.recursive && filename.ends_with(".mix") {
            let mix_reader: &mut dyn Read = &mut mix.get_file(file.id).unwrap(); // TODO remove unwrap
            extract_inner(mix_reader, &output_dir.join(filename), args, new_mix)?;
        } else {
            let data = mix.get_file(file.id).unwrap(); // TODO remove unwrap
            write(output_dir.join(filename), data)?;
        }
    }

    Ok(())
}

fn inspect(args: &InspectArgs, new_mix: bool) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let mut mix = MixReader::read_file(&mut reader, new_mix)?;
    let (mixdb, has_lmd) = prepare_databases(&mix, &args.db)?;

    if !args.no_header {
        println!(
            "Mix type:           {}",
            if mix.is_new_format { "new" } else { "old" }
        );
        println!("Mix flags:          {:?}", mix.flags);
        println!("Mix extra flags:    {:?}", mix.extra_flags);
        println!("# of files:         {:?}", mix.index.len());
        println!("Declared body size: {:?} bytes", mix.declared_body_size);
        println!("Actual body size:   {:?} bytes", mix.get_body_size());
        println!("Index size:         {:?} bytes", mix.get_index_size());
        println!("Is compact:         {:?}", mix.is_compact());
        printoptionmapln!(
            "Blowfish key:       {:x?}",
            mix.blowfish_key,
            |x: BlowfishKey| x.map(|c| format!("{:X?}", c)).concat()
        );
        printoptionmapln!("Checksum (SHA1):    {:?}", mix.checksum, |x: Checksum| x
            .map(|c| format!("{:X?}", c))
            .concat());
        println!("Has LMD:            {}", has_lmd);
        if !args.no_index {
            println!();
        }
    }

    mix.sort_by_id();

    if !args.no_index {
        println!(
            "{: <16} {: <8} {: >10} {: >10}",
            "Name", "ID", "Offset", "Size"
        );
        println!("{:=<47}", "");
        for f in mix.index.values() {
            println!(
                "{: <16} {:0<8X} {: >10?} {: >10?}",
                mixdb.get_name(f.id).unwrap_or(&String::default()),
                f.id,
                f.offset,
                f.size
            )
        }
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
