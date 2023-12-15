use std::{
    fs::{write, OpenOptions},
    io::Read,
    path::PathBuf,
};

use rust_alert::mix::{db::MixDatabase, io::MixReader};

#[derive(clap::Args)]
pub struct ExtractCommand {
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

use crate::{
    utils::{prepare_databases, read_db},
    Result, RunCommand,
};

impl RunCommand for ExtractCommand {
    /// Extract all files from a MIX.
    fn run(self, force_new_format: bool, safe_mode: bool) -> Result<()> {
        let mut reader = OpenOptions::new().read(true).open(&self.input)?;
        let gmd = self
            .db
            .clone()
            .map(|p| read_db(&p))
            .transpose()?
            .unwrap_or_default();
        extract_inner(
            &mut reader,
            &self.output,
            &self,
            force_new_format,
            &gmd,
            safe_mode,
        )?;
        Ok(())
    }
}

fn extract_inner(
    reader: &mut dyn Read,
    output_dir: &PathBuf,
    args: &ExtractCommand,
    new_mix: bool,
    gmd: &MixDatabase,
    safe_mode: bool,
) -> Result<()> {
    let mix = MixReader::read_file(reader, new_mix)?;
    std::fs::create_dir_all(output_dir)?;
    let (mixdb, _) = prepare_databases(&mix, gmd.clone(), safe_mode)?;

    for file in mix.index.values() {
        let filename = mixdb.get_name_or_id(file.id);

        if !args.quiet {
            println!("{}, {} bytes", filename, file.size);
        }
        if args.recursive && filename.ends_with(".mix") {
            let mix_reader: &mut dyn Read =
                &mut mix.get_file(file.id).unwrap_or_else(|| unreachable!());
            extract_inner(
                mix_reader,
                &output_dir.join(filename),
                args,
                new_mix,
                gmd,
                safe_mode,
            )?;
        } else {
            let data = mix.get_file(file.id).unwrap_or_else(|| unreachable!());
            write(output_dir.join(filename), data)?;
        }
    }

    Ok(())
}
