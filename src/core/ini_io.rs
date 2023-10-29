use std::io::{BufRead, Write};

use crate::core::ini::{IniFile, IniSection};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Unclosed section name at line {0}")]
    UnclosedSectionName(usize),
    #[error("Too many values in an entry at line {0}")]
    TooManyValues(usize),
    #[error("Missing entry value at line {0}")]
    MissingEntryValue(usize),
    #[error("Missing entry key at line {0}")]
    MissingEntryKey(usize),
    #[error("Missing entry key and value at line {0}")]
    MissingEntryKeyAndValue(usize),
    #[error("Entry with no section at line {0}")]
    EntryWithNoSection(usize),
    #[error("Other error at line {0}")]
    Other(usize),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
enum LineParseResultEnum {
    Section(String),
    Entry(String, String),
    Empty,
}

#[derive(Debug, Default)]
struct IniReader {}

impl IniReader {
    /// Read and parse an INI file from input.
    pub fn read_file(reader: impl BufRead) -> Result<IniFile> {
        let mut ini = IniFile::default();
        let mut current_section: Option<IniSection> = None;

        for (row, line) in reader.lines().enumerate() {
            let line = line?;
            match Self::parse_line(line, row)? {
                LineParseResultEnum::Section(s) => {
                    current_section.and_then(|s| ini.add_section(s));
                    current_section = Some(IniSection::new(s));
                }
                LineParseResultEnum::Entry(k, v) => {
                    if let Some(mut s) = current_section {
                        s.create_entry(k, v);
                        current_section = Some(s);
                    } else {
                        return Err(Error::EntryWithNoSection(row));
                    }
                }
                LineParseResultEnum::Empty => (),
            }
        }

        current_section.and_then(|s| ini.add_section(s));
        Ok(ini)
    }

    /// Parse one line of text into a section header, key-value entry or an empty line.
    fn parse_line(line: String, row: usize) -> Result<LineParseResultEnum> {
        let line = line.split(';').next();
        if line.is_none() {
            return Ok(LineParseResultEnum::Empty);
        }
        let line = line.unwrap();
        // Section
        if line.starts_with('[') {
            return line
                .find(']')
                .map(|end| LineParseResultEnum::Section(line[1..end].to_string()))
                .ok_or(Error::UnclosedSectionName(row));
        }
        // Entry
        let mut iter = line.split('=');
        match (iter.next(), iter.next(), iter.next()) {
            // =
            (Some(""), Some(""), _) => Err(Error::MissingEntryKeyAndValue(row)),
            // key=
            (Some(_), Some(""), _) => Err(Error::MissingEntryValue(row)),
            // =value
            (Some(""), Some(_), _) => Err(Error::MissingEntryKey(row)),
            // key=value
            (Some(k), Some(v), None) => {
                Ok(LineParseResultEnum::Entry(k.trim().into(), v.trim().into()))
            }
            // key=value=error
            (_, _, Some(_)) => Err(Error::TooManyValues(row)),
            // no equals sing
            (Some(_), None, _) => Ok(LineParseResultEnum::Empty),
            // other error
            _ => Err(Error::Other(row)),
        }
    }
}

struct IniWriter {}

impl IniWriter {
    /// Write an INI file to output.
    pub fn write_file(ini: &IniFile, writer: &mut impl Write) -> Result<()> {
        for (name, section) in ini.iter() {
            writeln!(writer, "[{name}]")?;
            for (key, entry) in section.iter() {
                writeln!(writer, "{key}={}", entry.value)?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod parse_line {
        use crate::core::ini_io::{Error, IniReader, LineParseResultEnum};

        #[test]
        fn parse_line_entry_ok() {
            let line = "key=value".to_string();
            let k = "key".to_string();
            let v = "value".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            assert_eq!(out.unwrap(), LineParseResultEnum::Entry(k, v));
        }

        #[test]
        fn parse_line_entry_ok_whitespaces() {
            let line = "    b key   =   value   c     ".to_string();
            let k = "b key".to_string();
            let v = "value   c".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            assert_eq!(out.unwrap(), LineParseResultEnum::Entry(k, v));
        }

        #[test]
        fn parse_line_entry_ok_comment() {
            let line = "key=value ; comment".to_string();
            let k = "key".to_string();
            let v = "value".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            assert_eq!(out.unwrap(), LineParseResultEnum::Entry(k, v));
        }

        #[test]
        fn parse_line_entry_err_no_value() {
            let line = "key=".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_err());
            assert!(matches!(out, Result::Err(Error::MissingEntryValue(_))));
        }

        #[test]
        fn parse_line_entry_err_no_key() {
            let line = "=value".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_err());
            assert!(matches!(out, Result::Err(Error::MissingEntryKey(_))));
        }

        #[test]
        fn parse_line_entry_err_nothing() {
            let line = "=".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_err());
            assert!(matches!(
                out,
                Result::Err(Error::MissingEntryKeyAndValue(_))
            ));
        }

        #[test]
        fn parse_line_entry_err_too_many_values() {
            let line = "a=b=c".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_err());
            assert!(matches!(out, Result::Err(Error::TooManyValues(_))));
        }

        #[test]
        fn parse_line_section_ok() {
            let line = "[Section]".to_string();
            let s = "Section".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            assert_eq!(out.unwrap(), LineParseResultEnum::Section(s));
        }

        #[test]
        fn parse_line_section_ok_comment() {
            let line = "[Section] ; comment".to_string();
            let s = "Section".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            assert_eq!(out.unwrap(), LineParseResultEnum::Section(s));
        }

        #[test]
        fn parse_line_section_err_unclosed() {
            let line = "[Section".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_err());
            assert!(matches!(out, Result::Err(Error::UnclosedSectionName(_))));
        }

        #[test]
        fn parse_line_empty_ok() {
            let line = "abba".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            assert_eq!(out.unwrap(), LineParseResultEnum::Empty);
        }
    }

    mod read_file {
        use crate::core::ini_io::{Error, IniFile, IniReader, IniSection};
        use std::io::BufRead;

        #[test]
        fn read_section_ok() {
            let file = "[Section]\nkey1=value1\nkey2=value2";
            let file: &mut dyn BufRead = &mut file.as_bytes();
            let mut expected = IniFile::default();
            let mut s = IniSection::new("Section");
            s.create_entry("key1", "value1");
            s.create_entry("key2", "value2");
            expected.add_section(s);

            let out = IniReader::read_file(file);
            assert!(out.is_ok());
            assert_eq!(out.unwrap(), expected);
        }

        #[test]
        fn read_many_sections_ok() {
            let file = "[A]\nkey1=value1\n[B]\nkey2=value2";
            let file: &mut dyn BufRead = &mut file.as_bytes();
            let mut expected = IniFile::default();
            let mut s = IniSection::new("A");
            s.create_entry("key1", "value1");
            expected.add_section(s);
            let mut s = IniSection::new("B");
            s.create_entry("key2", "value2");
            expected.add_section(s);

            let out = IniReader::read_file(file);
            assert!(out.is_ok());
            assert_eq!(out.unwrap(), expected);
        }

        #[test]
        fn read_global_err() {
            let file = "key1=value1";
            let file: &mut dyn BufRead = &mut file.as_bytes();

            let out = IniReader::read_file(file);
            assert!(out.is_err());
            assert!(matches!(out, Result::Err(Error::EntryWithNoSection(_))));
        }
    }

    mod write_file {
        use crate::core::{
            ini::{IniFile, IniSection},
            ini_io::IniWriter,
        };

        #[test]
        fn write_section_ok() {
            let mut writer: Vec<u8> = Default::default();
            let mut ini = IniFile::default();
            let mut section = IniSection::new("Section");
            section.create_entry("key", "value");
            ini.add_section(section);
            let expected = "[Section]\nkey=value\n\n";

            let out = IniWriter::write_file(&ini, &mut writer);
            assert!(out.is_ok());
            assert_eq!(std::str::from_utf8(&writer).unwrap(), expected);
        }

        #[test]
        fn write_many_sections_ok() {
            let mut writer: Vec<u8> = Default::default();
            let mut ini = IniFile::default();
            let mut section = IniSection::new("Section1");
            section.create_entry("key1", "value1");
            ini.add_section(section);
            let mut section = IniSection::new("Section2");
            section.create_entry("key2", "value2");
            ini.add_section(section);
            let expected = "[Section1]\nkey1=value1\n\n[Section2]\nkey2=value2\n\n";

            let out = IniWriter::write_file(&ini, &mut writer);
            assert!(out.is_ok());
            assert_eq!(std::str::from_utf8(&writer).unwrap(), expected);
        }
    }
}
