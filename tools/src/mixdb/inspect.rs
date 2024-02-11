use std::{fs::OpenOptions, path::PathBuf};

use rust_alert::mix::db::{io::LocalMixDbReader, MixDatabase};

use crate::{Result, RunCommand};

#[derive(clap::Args)]
pub struct InspectCommand {
    /// Path to an input INI file.
    input: PathBuf,
    /// Do not print the database header information.
    #[arg(long, default_value_t = false)]
    no_header: bool,
    /// Do not print the names inside.
    #[arg(long, default_value_t = false)]
    no_names: bool,
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
    fn run(self) -> Result<()> {
        let mut reader = OpenOptions::new().read(true).open(self.input)?;
        let mut mixdb = LocalMixDbReader::read_file(&mut reader)?;
        if !self.no_header {
            println!(
                "MIX DB version: {:?} ({})",
                mixdb.version, mixdb.version as u32
            );
            println!("# of entries:   {}", mixdb.db.names.len());
            if !self.no_names {
                println!();
            }
        }
        if !self.no_names {
            let names = match self.sort {
                InspectSortOrderEnum::Id => sort(mixdb.db, |(id, _)| *id),
                InspectSortOrderEnum::Name => sort(mixdb.db, |(_, name)| name.to_lowercase()),
                InspectSortOrderEnum::Offset => mixdb.db.names.drain().collect(),
                InspectSortOrderEnum::Size => sort(mixdb.db, |(_, name)| name.len()),
            };
            for (id, name) in names {
                println!("{:0>8X} {}", id, name);
            }
        }
        Ok(())
    }
}

fn sort<F, K>(mut db: MixDatabase, pred: F) -> Vec<(i32, String)>
where
    F: FnMut(&(i32, String)) -> K,
    K: Ord,
{
    let mut vec: Vec<_> = db.names.drain().collect();
    vec.sort_by_key(pred);
    vec
}
