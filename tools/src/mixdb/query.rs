use std::{fs::OpenOptions, path::PathBuf};

use rust_alert::{
    core::crc,
    mix::db::{io::LocalMixDbReader, LocalMixDatabase},
    utils::hex2int,
};

use crate::{Result, RunCommand};

#[derive(clap::Args)]
pub struct QueryCommand {
    /// Path to an input MIX DB file.
    input: PathBuf,
    /// Query name or ID.
    query: String,
    /// Query name by ID instead of ID by name.
    #[arg(long, default_value_t = false)]
    by_id: bool,
    /// Calculate the result if querying ID fails.
    #[arg(short, long, default_value_t = false)]
    calculate: bool,
}

impl RunCommand for QueryCommand {
    fn run(self) -> Result<()> {
        let mut reader = OpenOptions::new().read(true).open(self.input)?;
        let mixdb = LocalMixDbReader::read_file(&mut reader)?;
        let res = if self.by_id {
            let id = hex2int(&self.query)?;
            query_by_id(&mixdb, id)
        } else {
            query_by_name(&mixdb, &self.query, self.calculate)
        };
        if let Some(res) = res {
            println!("{}", res);
        } else {
            println!("Not found");
        }
        Ok(())
    }
}

fn query_by_id(mixdb: &LocalMixDatabase, query: i32) -> Option<String> {
    mixdb.db.names.get(&query).cloned()
}

fn query_by_name(mixdb: &LocalMixDatabase, query: &String, calculate: bool) -> Option<String> {
    let real = crc(query, mixdb.version.into());
    if calculate || mixdb.db.names.get(&real).is_some_and(|res| res == query) {
        Some(format!("{:0>8X}", real))
    } else {
        None
    }
}
