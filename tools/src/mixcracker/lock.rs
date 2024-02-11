use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use rust_alert::mix::{io::MixWriter, Mix, MixIndexEntry, LMD_KEY_TD, LMD_KEY_TS};

use crate::{utils::read_mix, Result, RunCommand};

#[derive(clap::Args)]
pub struct LockCommand {
    /// Path to an input MIX file.
    input: PathBuf,
    /// Path to an output MIX file. Same as input by default.
    output: Option<PathBuf>,
    /// Remove the LMD from the MIX. MIX is still openable, but all filenames will be lost.
    #[arg(long, default_value_t = false)]
    lmd_purge: bool,
    /// Corrupt body size in the MIX header. XCC Mixer will not recognize this MIX.
    #[arg(long, default_value_t = false)]
    hdr_body: bool,
    /// Leave a corrupted index entry for the LMD. XCC Mixer will crash on opening the directory this MIX resides in.
    #[arg(long, default_value_t = false)]
    lmd_index: bool,
}

impl RunCommand for LockCommand {
    fn run(self, force_new_format: bool) -> Result<()> {
        let mut mix = read_mix(&self.input, force_new_format)?;
        if self.lmd_purge {
            lmd_purge(&mut mix);
        }
        if self.lmd_index {
            lmd_index(&mut mix);
        }
        let mut writer = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.output.unwrap_or(self.input))?;
        if self.hdr_body {
            MixWriterUnsafe::write_file(&mut writer, &mut mix, force_new_format)?;
        } else {
            MixWriter::write_file(&mut writer, &mut mix, force_new_format)?;
        }
        Ok(())
    }
}

fn lmd_purge(mix: &mut Mix) {
    mix.remove_file(LMD_KEY_TD);
    mix.remove_file(LMD_KEY_TS);
    mix.recalc();
}

fn lmd_index(mix: &mut Mix) {
    let lmd = MixIndexEntry::new(LMD_KEY_TD, rand::random(), rand::random());
    mix.index.insert(LMD_KEY_TD, lmd);
    let lmd = MixIndexEntry::new(LMD_KEY_TS, rand::random(), rand::random());
    mix.index.insert(LMD_KEY_TS, lmd);
}

pub struct MixWriterUnsafe {}

impl MixWriterUnsafe {
    pub fn write_file(writer: &mut dyn Write, mix: &mut Mix, force_new_format: bool) -> Result<()> {
        mix.is_new_format =
            mix.is_new_format || (!mix.flags.is_empty() && !mix.extra_flags.is_empty());
        let mut body = vec![];
        body.append(&mut mix.body);
        MixWriter::write_header(writer, mix, force_new_format)?;
        if let Some(key) = mix.blowfish_key {
            MixWriter::write_index_encrypted(writer, mix, &key)?;
        } else {
            MixWriter::write_index(writer, mix)?;
        }
        writer.write_all(&body)?;
        if let Some(checksum) = mix.checksum {
            writer.write_all(&checksum)?;
        }
        Ok(())
    }
}
