mod csf;

use crate::csf::{Csf, CsfLabel, CsfLanguageEnum, CsfVersionEnum};

use std::fs::File;

fn main() {
    let mut writer = File::create("stringtable99.csf").unwrap();
    let mut csf = Csf {
        version: CsfVersionEnum::Cnc,
        language: CsfLanguageEnum::ENUS,
        ..Default::default()
    };
    csf.add_label(CsfLabel::new("Name:Test".into(), "Test Test Test".into()));
    csf.add_label(CsfLabel::new("Name:Baz".into(), "Zinga".into()));
    println!("{csf:?}");
    csf.write(&mut writer).unwrap();

    let mut reader = File::open("stringtable99.csf").unwrap();
    let csf = Csf::read(&mut reader).unwrap();
    println!("{csf:?}");
}
