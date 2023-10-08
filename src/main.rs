use rust_alert::core::csf::{CsfLabel, CsfLanguageEnum, CsfStringtable, CsfVersionEnum};
use rust_alert::core::csf_io::{CsfReader, CsfWriter};

use std::fs::File;

fn main() {
    let mut writer = File::create("stringtable99.csf").unwrap();
    let mut csf = CsfStringtable {
        version: CsfVersionEnum::Cnc,
        language: CsfLanguageEnum::ENUS,
        ..Default::default()
    };
    csf.add_label(CsfLabel::new("Name:Test", "Test Test Test"));
    csf.add_label(CsfLabel::new("Name:Baz", "Zinga"));
    dbg!(&csf);
    CsfWriter::write_file(&csf, &mut writer).unwrap();

    let mut reader = File::open("stringtable99.csf").unwrap();
    let csf = CsfReader::read_file(&mut reader).unwrap();
    dbg!(&csf);
}
