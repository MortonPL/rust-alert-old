use std::path::PathBuf;

use crate::{
    utils::{read_mix, write_mix},
    Result, RunCommand,
};

#[derive(clap::Args)]
pub struct CompactCommand {
    /// Path to an input MIX file.
    input: PathBuf,
    /// Path to an output MIX file. Same as input by default.
    output: Option<PathBuf>,
}

impl RunCommand for CompactCommand {
    /// Compact the MIX: remove all data not belonging to any file.
    fn run(self, force_new_format: bool) -> Result<()> {
        let mut mix = read_mix(&self.input, force_new_format)?;
        mix.recalc();
        write_mix(&mut mix, &self.output, &self.input, force_new_format)?;
        println!("Compacted the MIX successfully.");
        Ok(())
    }
}
