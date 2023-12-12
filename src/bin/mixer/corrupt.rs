use std::path::PathBuf;

use crate::{
    utils::{read_mix, write_mix},
    Result, RunCommand,
};

#[derive(clap::Args)]
pub struct CorruptCommand {
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

impl RunCommand for CorruptCommand {
    fn run(self, force_new_format: bool) -> Result<()> {
        todo!()
    }
}
