use std::{fs::OpenOptions, path::PathBuf};

use rust_alert::{
    mix::{db::GlobalMixDatabase, io::MixReader, BlowfishKey, Checksum, Mix},
    printoptionmapln,
};

use crate::{
    utils::{prepare_databases, read_db},
    Result, RunCommand,
};

#[derive(clap::Args)]
pub struct InspectCommand {
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

impl RunCommand for InspectCommand {
    /// Inspect the MIX, printing useful header information and/or index contents.
    fn run(self, force_new_format: bool) -> Result<()> {
        let mut reader = OpenOptions::new().read(true).open(self.input)?;
        let mut mix = MixReader::read_file(&mut reader, force_new_format)?;
        let gmd = self
            .db
            .map(|p| read_db(&p))
            .transpose()?
            .unwrap_or_default();
        let (mixdb, has_lmd) = prepare_databases(&mix, gmd)?;
        if !self.no_header {
            inspect_header(&mut mix, has_lmd);
            if !self.no_index {
                println!();
            }
        }
        if !self.no_index {
            inspect_index(&mut mix, &mixdb, self.sort);
        }
        Ok(())
    }
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
    println!("Is compact:         {:?}", mix.is_compact());
    println!("Index size:         {:?} bytes", mix.get_index_size());
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
