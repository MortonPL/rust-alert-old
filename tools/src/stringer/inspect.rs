use std::{fs::OpenOptions, path::PathBuf};

use rust_alert::csf::io::CsfReader;

use crate::RunCommand;

#[derive(clap::Args)]
pub struct InspectCommand {
    /// Path to an input CSF file.
    input: PathBuf,
}

impl RunCommand for InspectCommand {
    fn run(self) -> crate::Result<()> {
        let mut reader = OpenOptions::new().read(true).open(self.input)?;
        let csf = CsfReader::read_file(&mut reader)?;
        println!(
            "Version:      {:?} ({})",
            csf.version,
            TryInto::<u32>::try_into(csf.version)?
        );
        println!(
            "Language:     {:?} ({})",
            csf.language,
            TryInto::<u32>::try_into(csf.language)?
        );
        println!("Extra data:   {:X}", csf.extra);
        println!("# of labels:  {:?}", csf.len());
        println!("# of strings: {:?}", csf.strings_len());
        println!(
            "Contains WSTRs: {:?}",
            csf.iter()
                .any(|l| l.get_first().is_some_and(|s| !s.extra_value.is_empty()))
        );
        Ok(())
    }
}
