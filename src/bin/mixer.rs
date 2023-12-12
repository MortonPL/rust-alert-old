//! MIX multitool.

use std::{
    fs::{write, OpenOptions},
    io::{BufReader, Read, Write},
    path::PathBuf,
};

use clap::{Parser, Subcommand};

use rust_alert::{
    converters::ini2db,
    core::{crc, GameEnum},
    defaultarray,
    ini::io::IniReader,
    mix::{
        db::{
            io::{LocalMixDbReader, LocalMixDbWriter},
            GlobalMixDatabase, LocalMixDatabase, MixDatabase,
        },
        io::{generate_blowfish, MixReader, MixWriter},
        BlowfishKey, Checksum, Mix, MixHeaderFlags, LMD_KEY_TD, LMD_KEY_TS,
    },
    printoptionmapln,
    utils::{path_to_str, PathToStringError},
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
    Build(BuildArgs),
    /// Add/Remove/Check MIX checksum.
    Checksum(ChecksumArgs),
    /// Compact (remove unused data from) the MIX body.
    Compact(CompactArgs),
    /// Secure the MIX with a suite of anti-ripper corruptions.
    Corrupt(CorruptArgs),
    /// Encrypt/Decrypt a MIX file or extract stored key.
    Blowfish(BlowfishArgs),
    /// Extract MIX contents to folder.
    Extract(ExtractArgs),
    /// Inspect MIX file. Print general information such as header values,
    /// checksum, encryption key, as well as the file index.
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
    /// Recursively build inner MIXes from subfolders.
    #[arg(short, long, default_value_t = false)]
    recursive: bool,
    /// Path to a Blowfish key for encryption.
    /// 56 bytes of the file will be read and used as the key. Leave empty for a random key.
    #[arg(short, long)]
    key: Option<PathBuf>,
    /// Append SHA1 checksum to the MIX file.
    #[arg(short, long, default_value_t = false)]
    checksum: bool,
    /// Build LMD for the MIX file.
    #[arg(short, long, default_value_t = false)]
    lmd: bool,
    /// Use old CRC function (TD/RA).
    #[arg(short, long, default_value_t = false)]
    old_crc: bool,
    /// Allow to overwrite files with the same name.
    #[arg(long, default_value_t = false)]
    overwrite: bool,
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
    /// Add a SHA1 checksum of MIX body at the end of the file and set the checksum header flag.
    #[default]
    Add,
    /// Remove existing checksum from the MIX and clear the checksum header flag.
    Remove,
    /// Check if the attached checksum matches MIX body.
    /// Raises error when checksums don't match or the MIX contains no checksum.
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
struct BlowfishArgs {
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
    /// Do not print the MIX header information.
    #[arg(long, default_value_t = false)]
    no_header: bool,
    /// Do not print the file index.
    #[arg(long, default_value_t = false)]
    no_index: bool,
    /// Path to a MIX database (containing filenames) in INI format.
    #[arg(short, long)]
    db: Option<PathBuf>,
    /// Sort file index (in ascending order) by given column.
    #[arg(short, long, default_value_t = Default::default())]
    sort: InspectSortOrderEnum,
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
enum InspectSortOrderEnum {
    #[default]
    Id,
    Offset,
    Size,
    Name,
}

impl std::fmt::Display for InspectSortOrderEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

fn read_mix(input: &PathBuf, new_mix: bool) -> Result<Mix> {
    let mut reader = OpenOptions::new().read(true).open(input)?;
    let mix = MixReader::read_file(&mut reader, new_mix)?;
    Ok(mix)
}

fn write_mix(
    mix: &mut Mix,
    output: &Option<PathBuf>,
    default: &PathBuf,
    new_mix: bool,
) -> Result<()> {
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output.as_ref().unwrap_or(default))?;
    MixWriter::write_file(&mut writer, mix, new_mix)?;
    Ok(())
}

/// Read a MIX database from an INI file.
fn read_db(path: &PathBuf) -> Result<MixDatabase> {
    let reader = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(reader);
    let ini = IniReader::read_file(reader)?;
    let db = ini2db(&ini)?;
    Ok(db)
}

/// Read an LMD from inside a MIX.
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

/// Read GMD & LMD and merge them.
fn prepare_databases(mix: &Mix, gmd: MixDatabase) -> Result<(GlobalMixDatabase, bool)> {
    let mut mixdb = GlobalMixDatabase::default();
    let mut has_lmd = false;
    if let Some(lmd) = read_lmd(mix) {
        mixdb.dbs.push(lmd.db);
        has_lmd = true;
    }
    mixdb.dbs.push(gmd);
    Ok((mixdb, has_lmd))
}

fn build(args: BuildArgs, new_mix: bool) -> Result<()> {
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args.output)?;
    build_inner(&mut writer, &args.input, &args, new_mix)
}

fn build_inner(
    writer: &mut dyn Write,
    input: &PathBuf,
    args: &BuildArgs,
    new_mix: bool,
) -> Result<()> {
    let paths = std::fs::read_dir(input)?;
    let mut mix = Mix::default();
    let mut lmd = LocalMixDatabase::default();
    let crc_version = match args.old_crc {
        true => GameEnum::TD,
        false => GameEnum::YR,
    };
    for res in paths {
        let path = res?.path();
        let str = path_to_str(&path)?;
        if path.is_dir() {
            let mut temp: Vec<u8> = vec![];
            build_inner(&mut temp, &path, args, new_mix)?;
            mix.add_file_raw(temp, crc(&str, crc_version))?;
        } else {
            mix.add_file_path(path, crc_version, args.overwrite)?;
        }
        if args.lmd {
            lmd.db.names.insert(crc(&str, crc_version), str);
        }
    }
    if args.lmd {
        let mut temp: Vec<u8> = vec![];
        LocalMixDbWriter::write_file(&mut temp, &lmd)?;
        let lmd_id = match args.old_crc {
            true => LMD_KEY_TD,
            false => LMD_KEY_TS,
        };
        mix.add_file_raw(temp, lmd_id)?;
    }
    if args.encrypt {
        encrypt_mix(&mut mix, &args.key)?;
    }
    if args.checksum {
        mix.calc_checksum();
    }
    MixWriter::write_file(writer, &mut mix, new_mix)?;
    Ok(())
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

/// Add checksum to MIX, remove checksum from MIX, or check if checksum in the MIX is true.
fn checksum(args: ChecksumArgs, new_mix: bool) -> Result<()> {
    let mut mix = read_mix(&args.input, new_mix)?;
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
            return compare_checksum(&mut mix);
        }
    };
    write_mix(&mut mix, &args.output, &args.input, new_mix)?;
    match args.mode {
        ChecksumMode::Add => println!("Checksum added successfully."),
        ChecksumMode::Remove => println!("Checksum removed successfully."),
        _ => (),
    };
    Ok(())
}

/// Compact the MIX: remove all data not belonging to any file.
fn compact(args: CompactArgs, new_mix: bool) -> Result<()> {
    let mut mix = read_mix(&args.input, new_mix)?;
    mix.recalc();
    write_mix(&mut mix, &args.output, &args.input, new_mix)?;
    println!("Compacted the MIX successfully.");
    Ok(())
}

fn corrupt(args: CorruptArgs, new_mix: bool) -> Result<()> {
    Ok(())
}

fn encrypt_mix(mix: &mut Mix, key: &Option<PathBuf>) -> Result<()> {
    let key = if let Some(key) = key {
        let mut reader = OpenOptions::new().read(true).open(key)?;
        let mut key = defaultarray!(BlowfishKey);
        reader.read_exact(&mut key)?;
        Some(key)
    } else {
        Some(generate_blowfish())
    };
    mix.set_blowfish_key(key);
    Ok(())
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

/// Encrypt, decrypt MIX or extract the key.
fn encrypt(args: BlowfishArgs, new_mix: bool) -> Result<()> {
    let mut mix = read_mix(&args.input, new_mix)?;
    match args.mode {
        BlowfishMode::Decrypt => {
            mix.set_blowfish_key(None);
            Ok(())
        }
        BlowfishMode::Encrypt => encrypt_mix(&mut mix, &args.key),
        BlowfishMode::Get => get_mix_key(&mix, &args.key),
    }?;
    write_mix(&mut mix, &args.output, &args.input, new_mix)?;
    Ok(())
}

/// Extract all files from a MIX.
fn extract(args: ExtractArgs, new_mix: bool) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let gmd = args
        .db
        .clone()
        .map(|p| read_db(&p))
        .transpose()?
        .unwrap_or_default();
    extract_inner(&mut reader, &args.output, &args, new_mix, &gmd)?;
    Ok(())
}

fn extract_inner(
    reader: &mut dyn Read,
    output_dir: &PathBuf,
    args: &ExtractArgs,
    new_mix: bool,
    gmd: &MixDatabase,
) -> Result<()> {
    let mix = MixReader::read_file(reader, new_mix)?;
    std::fs::create_dir_all(output_dir)?;
    let (mixdb, _) = prepare_databases(&mix, gmd.clone())?;

    for file in mix.index.values() {
        let filename = mixdb.get_name_or_id(file.id);

        if !args.quiet {
            println!("{}, {} bytes", filename, file.size);
        }
        if args.recursive && filename.ends_with(".mix") {
            let mix_reader: &mut dyn Read =
                &mut mix.get_file(file.id).unwrap_or_else(|| unreachable!());
            extract_inner(mix_reader, &output_dir.join(filename), args, new_mix, gmd)?;
        } else {
            let data = mix.get_file(file.id).unwrap_or_else(|| unreachable!());
            write(output_dir.join(filename), data)?;
        }
    }

    Ok(())
}

/// Sort given MIX by names from given GMD.
fn sort_by_name(mix: &mut Mix, db: &GlobalMixDatabase) {
    mix.index.sort_by(|_, f1, _, f2| {
        db.get_name(f1.id)
            .cloned()
            .unwrap_or(String::default())
            .cmp(db.get_name(f2.id).unwrap_or(&String::default()))
    });
}

fn inspect_header(mix: &mut Mix, has_lmd: bool) {
    println!(
        "Mix type:           {}",
        if mix.is_new_format {
            "New (>= RA1)"
        } else {
            "Old (TD, RA1)"
        }
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
        |x: BlowfishKey| x.map(|c| format!("{:02X?}", c)).concat()
    );
    printoptionmapln!("Checksum (SHA1):    {:?}", mix.checksum, |x: Checksum| x
        .map(|c| format!("{:X?}", c))
        .concat());
    println!("Has LMD:            {}", has_lmd);
}

fn inspect_index(mix: &mut Mix, mixdb: &GlobalMixDatabase, sort: InspectSortOrderEnum) {
    match sort {
        InspectSortOrderEnum::Id => mix.sort_by_id(),
        InspectSortOrderEnum::Name => sort_by_name(mix, mixdb),
        InspectSortOrderEnum::Offset => mix.sort_by_offset(),
        InspectSortOrderEnum::Size => mix.sort_by_size(),
    }
    let names: Vec<_> = mix
        .index
        .values()
        .map(|f| mixdb.get_name(f.id).cloned().unwrap_or(String::default()))
        .collect();
    let maxname = names.iter().map(|x| x.len()).max().unwrap_or_default();
    println!(
        "{: <maxname$} {: <8} {: >10} {: >10}",
        "Name",
        "ID",
        "Offset",
        "Size",
        maxname = maxname
    );
    let total_len = maxname + 28 + 3;
    println!("{:=<len$}", "", len = total_len);
    for (f, name) in mix.index.values().zip(names) {
        println!(
            "{: <len$} {:0>8X} {: >10?} {: >10?}",
            name,
            f.id,
            f.offset,
            f.size,
            len = maxname,
        )
    }
}

/// Inspect the MIX, printing useful header information and/or index contents.
fn inspect(args: InspectArgs, new_mix: bool) -> Result<()> {
    let mut reader = OpenOptions::new().read(true).open(&args.input)?;
    let mut mix = MixReader::read_file(&mut reader, new_mix)?;
    let gmd = args
        .db
        .map(|p| read_db(&p))
        .transpose()?
        .unwrap_or_default();
    let (mixdb, has_lmd) = prepare_databases(&mix, gmd)?;
    if !args.no_header {
        inspect_header(&mut mix, has_lmd);
        if !args.no_index {
            println!();
        }
    }
    if !args.no_index {
        inspect_index(&mut mix, &mixdb, args.sort);
    }
    Ok(())
}

fn main() {
    let args = Args::parse();
    let res = match args.command {
        Commands::Build(x) => build(x, args.new_mix),
        Commands::Checksum(x) => checksum(x, args.new_mix),
        Commands::Compact(x) => compact(x, args.new_mix),
        Commands::Corrupt(x) => corrupt(x, args.new_mix),
        Commands::Blowfish(x) => encrypt(x, args.new_mix),
        Commands::Extract(x) => extract(x, args.new_mix),
        Commands::Inspect(x) => inspect(x, args.new_mix),
    };
    match res {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };
}
