//! INI I/O.

use std::io::{BufRead, Write};

use crate::ini::{IniFile, IniSection};

/// The error type for serialization and deserialization of INI files.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A [`std::io::Error`].
    #[error("{0}")]
    IO(#[from] std::io::Error),
    /// A section is missing the ending bracket.
    #[error("Unclosed section name at line {0}")]
    UnclosedSectionName(usize),
    /// An entry has key, but no value.
    #[error("Missing entry value at line {0}")]
    MissingEntryValue(usize),
    /// An entry has value, but no key.
    #[error("Missing entry key at line {0}")]
    MissingEntryKey(usize),
    /// An entry is missing both key and value (so it's just the `=` character).
    #[error("Missing entry key and value at line {0}")]
    MissingEntryKeyAndValue(usize),
    /// An entry was declared before any section.
    #[error("Entry with no section at line {0}")]
    EntryWithNoSection(usize),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
enum LineParseResultEnum {
    Section(String),
    Entry(String, String),
    Empty,
}

/// Provides static methods for reading INI files.
#[derive(Debug, Default)]
pub struct IniReader {}

impl IniReader {
    /// Read and parse an INI file from input.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::io::IniReader;
    ///
    /// let buf = "[A]\nB=C";
    /// let reader = std::io::BufReader::new(buf.as_bytes());
    ///
    /// let ini = IniReader::read_file(reader);
    /// assert!(ini.is_ok());
    /// let ini = ini.unwrap();
    /// assert_eq!(ini.get_str("A", "B").unwrap(), "C");
    /// ```
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
        let line = line.split(';').next().unwrap_or_else(|| unreachable!());

        // Section
        if line.starts_with('[') {
            return line
                .find(']')
                .map(|end| LineParseResultEnum::Section(line[1..end].to_string()))
                .ok_or(Error::UnclosedSectionName(row));
        }
        // Entry
        let mut iter = line.splitn(2, '=');
        match (iter.next(), iter.next()) {
            // =
            (Some(""), Some("")) => Err(Error::MissingEntryKeyAndValue(row)),
            // key=
            (Some(_), Some("")) => Err(Error::MissingEntryValue(row)),
            // =value
            (Some(""), Some(_)) => Err(Error::MissingEntryKey(row)),
            // key=value
            (Some(k), Some(v)) => Ok(LineParseResultEnum::Entry(k.trim().into(), v.trim().into())),
            // no equals sign
            (Some(_), None) => Ok(LineParseResultEnum::Empty),
            // no other combination possible
            _ => unreachable!(),
        }
    }
}

/// Provides static methods for writing INI files.
pub struct IniWriter {}

impl IniWriter {
    /// Write an INI file to output.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use rust_alert::ini::io::{IniFile, IniWriter};
    ///
    /// let mut ini = IniFile::default();
    /// ini.add_to_section("A", "B", "C");
    /// let mut writer = vec![];
    ///
    /// let result = IniWriter::write_file(&ini, &mut writer);
    /// assert!(result.is_ok());
    /// assert_eq!(writer, "[A]\nB=C\n\n".as_bytes());
    /// ```
    pub fn write_file(ini: &IniFile, writer: &mut impl Write) -> Result<()> {
        for (name, section) in ini.iter() {
            writeln!(writer, "[{name}]")?;
            for (key, entry) in section.iter() {
                writeln!(writer, "{}={}", key, entry.value)?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod examples {
    use crate as rust_alert;

    #[test]
    fn read_file() {
        use rust_alert::ini::io::IniReader;

        let buf = "[A]\nB=C";
        let reader = std::io::BufReader::new(buf.as_bytes());

        let ini = IniReader::read_file(reader);
        assert!(ini.is_ok());
        let ini = ini.unwrap();
        assert_eq!(ini.get_str("A", "B").unwrap(), "C");
    }

    #[test]
    fn write_file() {
        use rust_alert::ini::io::{IniFile, IniWriter};

        let mut ini = IniFile::default();
        ini.add_to_section("A", "B", "C");
        let mut writer = vec![];

        let result = IniWriter::write_file(&ini, &mut writer);
        assert!(result.is_ok());
        assert_eq!(writer, "[A]\nB=C\n\n".as_bytes());
    }
}

#[cfg(test)]
mod tests {
    mod parse_line {
        use crate::{
            ini::io::{Error, IniReader, LineParseResultEnum},
            unwrap_assert,
        };

        #[test]
        fn parse_line_entry_ok() {
            let line = "key=value".to_string();
            let k = "key".to_string();
            let v = "value".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            unwrap_assert!(out, LineParseResultEnum::Entry(k, v));
        }

        #[test]
        fn parse_line_entry_ok_whitespaces() {
            let line = "    b key   =   value   c     ".to_string();
            let k = "b key".to_string();
            let v = "value   c".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            unwrap_assert!(out, LineParseResultEnum::Entry(k, v));
        }

        #[test]
        fn parse_line_entry_ok_comment() {
            let line = "key=value ; comment".to_string();
            let k = "key".to_string();
            let v = "value".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            unwrap_assert!(out, LineParseResultEnum::Entry(k, v));
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
        fn parse_line_entry_ok_equals_sign() {
            let line = "a=b=c".to_string();
            let k = "a".to_string();
            let v = "b=c".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            unwrap_assert!(out, LineParseResultEnum::Entry(k, v));
        }

        #[test]
        fn parse_line_section_ok() {
            let line = "[Section]".to_string();
            let s = "Section".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            unwrap_assert!(out, LineParseResultEnum::Section(s));
        }

        #[test]
        fn parse_line_section_ok_comment() {
            let line = "[Section] ; comment".to_string();
            let s = "Section".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            unwrap_assert!(out, LineParseResultEnum::Section(s));
        }

        #[test]
        fn parse_line_section_err_unclosed() {
            let line = "[Section".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_err());
            assert!(matches!(out, Result::Err(Error::UnclosedSectionName(_))));
        }

        #[test]
        fn parse_line_empty_ok_nothing() {
            let line = "".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            unwrap_assert!(out, LineParseResultEnum::Empty);
        }

        #[test]
        fn parse_line_empty_ok_comment() {
            let line = "; comment".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            unwrap_assert!(out, LineParseResultEnum::Empty);
        }

        #[test]
        fn parse_line_empty_ok_no_entry() {
            let line = "abba".to_string();

            let out = IniReader::parse_line(line, 0);
            assert!(out.is_ok());
            unwrap_assert!(out, LineParseResultEnum::Empty);
        }
    }

    mod read_file {
        use crate::{
            ini::io::{Error, IniFile, IniReader, IniSection},
            unwrap_assert,
        };
        use std::io::BufRead;

        #[test]
        fn read_section_ok() {
            let file = "[Section]\nkey1=value1\n\nkey2=value2";
            let file: &mut dyn BufRead = &mut file.as_bytes();
            let mut expected = IniFile::default();
            let mut s = IniSection::new("Section");
            s.create_entry("key1", "value1");
            s.create_entry("key2", "value2");
            expected.add_section(s);

            let out = IniReader::read_file(file);
            assert!(out.is_ok());
            unwrap_assert!(out, expected);
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
            unwrap_assert!(out, expected);
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
        use crate::{
            ini::{io::IniWriter, IniFile, IniSection},
            unwrap_assert,
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
            unwrap_assert!(std::str::from_utf8(&writer), expected);
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
            unwrap_assert!(std::str::from_utf8(&writer), expected);
        }
    }
}
