use std::path::PathBuf;

use rust_alert::mix::{
    db::io::{LocalMixDbReader, LMD_HEADER_SIZE},
    Mix, MixHeaderFlags, LMD_KEY_TD, LMD_KEY_TS,
};

use crate::{
    utils::{read_mix, write_mix},
    Result, RunCommand,
};

#[derive(clap::Args)]
pub struct CrackCommand {
    /// Path to an input MIX file.
    input: PathBuf,
    /// Path to an output MIX file. Same as input by default.
    output: Option<PathBuf>,
}

impl RunCommand for CrackCommand {
    fn run(self, force_new_format: bool) -> Result<()> {
        let mut mix = read_mix(&self.input, force_new_format)?;
        mix.flags = mix
            .flags
            .intersection(MixHeaderFlags::ENCRYPTION | MixHeaderFlags::CHECKSUM);
        mix.extra_flags = 0.into();
        mix.declared_body_size = mix.get_body_size() as u32;
        mix.index.retain(|_, entry| {
            (entry.size > 0)
                && ((entry.offset as u64 + entry.size as u64) <= mix.declared_body_size as u64)
        });
        validate_lmd(&mut mix, LMD_KEY_TD);
        validate_lmd(&mut mix, LMD_KEY_TS);
        mix.recalc();
        write_mix(
            &mut mix,
            &self.output.unwrap_or(self.input),
            force_new_format,
        )
    }
}

fn validate_lmd(mix: &mut Mix, id: i32) {
    if let Some(mut lmd) = mix.get_file(id) {
        let too_small = lmd.len() < LMD_HEADER_SIZE;
        let is_err = LocalMixDbReader::read_file(&mut lmd).is_err();
        if too_small || is_err {
            mix.remove_file(id);
        }
    }
}
