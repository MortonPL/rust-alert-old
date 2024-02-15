//! CSF (stringtable) I/O.

use std::{
    io::{Read, Write},
    mem::size_of,
    string::{FromUtf16Error, FromUtf8Error},
};

use crate::csf::{CsfLabel, CsfString, CsfStringtable};

/// The error type for serialization and deserialization of CSF stringtables.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A [`std::io::Error`].
    #[error("{0}")]
    IO(#[from] std::io::Error),
    /// The "` FSC`" prefix is missing from the file beginning.
    #[error("FSC prefix missing")]
    CsfMissingPrefix,
    /// The "` LBL`" prefix is missing from the label beginning.
    #[error("LBL prefix missing")]
    LblMissingPrefix,
    /// The "` RTS`"/"`WRTS`" prefix is missing from the string beginning.
    #[error("RTS/WRTS prefix missing!")]
    StrOrStrwMissingPrefix,
    /// Data is not a valid UTF-8 string.
    #[error("{0}")]
    Utf8(#[from] FromUtf8Error),
    /// Data is not a valid UTF-16 string.
    #[error("{0}")]
    Utf16(#[from] FromUtf16Error),
    /// A [`rust_alert::csf::Error`][crate::csf::Error].
    #[error("{0}")]
    CSF(#[from] crate::csf::Error),
}

#[doc(hidden)]
type Result<T> = std::result::Result<T, Error>;

/// Storage for static strings.
struct CsfPrefixes {}
impl CsfPrefixes {
    const CSF_PREFIX: &'static [u8] = b" FSC";
    const LBL_PREFIX: &'static [u8] = b" LBL";
    const STR_PREFIX: &'static [u8] = b" RTS";
    const STRW_PREFIX: &'static [u8] = b"WRTS";
}

pub trait CsfRead {
    /// Fixed size of the CSF header in bytes.
    const CSF_HEADER_SIZE: usize = 24;

    /// Reads the entire [`CsfStringtable`] from a byte source.
    ///
    /// This method is auto-implemented using [`read_header`][CsfRead::read_header]
    /// and [`read_label`][CsfRead::read_label].
    ///
    /// Examples
    ///
    /// ```ignore
    /// use rust_alert::csf::io::{CsfRead, CsfReader}; // CsfReader implements CsfRead
    ///
    /// let mut file = std::fs::File::open("test_data/example.csf")?;
    /// let mut csf_reader = CsfReader::default();
    /// let csf = csf_reader.read(&mut file)?;
    ///
    /// assert_eq!(csf.len(), 1);
    /// assert_eq!(csf.get_str("Label"), Some("String"));
    ///
    /// Ok(())
    /// ```
    fn read(&mut self, reader: &mut dyn Read) -> Result<CsfStringtable> {
        let (mut csf, num_labels) = self.read_header(reader)?;
        csf.reserve(num_labels as usize);
        for _ in 0..num_labels {
            csf.insert(self.read_label(reader)?);
        }
        Ok(csf)
    }

    /// Reads a CSF file header and constructs an empty [`CsfStringtable`]
    /// alongside with the declared number of labels.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::csf::io::{CsfRead, CsfReader}; // CsfReader implements CsfRead
    ///
    /// let mut file = std::fs::File::open("test_data/example.csf")?;
    /// let mut csf_reader = CsfReader::default();
    /// let (csf, num_labels) = csf_reader.read_header(&mut file)?;
    ///
    /// assert_eq!(num_labels, 1);
    ///
    /// Ok(())
    /// ```
    fn read_header(&mut self, reader: &mut dyn Read) -> Result<(CsfStringtable, u32)>;

    /// Reads a [`CsfLabel`] from a byte source.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::csf::io::{CsfRead, CsfReader}; // CsfReader implements CsfRead
    ///
    /// let mut file = std::fs::File::open("test_data/example.csf")?;
    /// let mut csf_reader = CsfReader::default();
    ///
    /// let label = csf_reader.read_label(&mut file)?;
    /// file.seek(std::io::SeekFrom::Start(CsfReader::CSF_HEADER_SIZE as u64))?;
    /// assert_eq!(label.name, "Label");
    /// assert_eq!(label.get_first_str(), Some("String"));
    ///
    /// Ok(())
    /// ```
    fn read_label(&mut self, reader: &mut dyn Read) -> Result<CsfLabel>;

    /// Reads a [`CsfString`] from a byte source.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rust_alert::csf::io::{CsfRead, CsfReader}; // CsfReader implements CsfRead
    ///
    /// let vec = vec![b' ', b'R', b'T', b'S', 3u8, 0, 0, 0, 0xAC, 0xFF, 0x8B, 0xFF, 0x8D, 0xFF];
    /// let mut csf_reader = CsfReader::default();
    /// let string = csf_reader.read_string(&mut vec.as_slice())?;
    ///
    /// assert_eq!(string.value, "Str");
    /// assert!(string.extra_value.is_empty());
    ///
    /// Ok(())
    /// ```
    fn read_string(&mut self, reader: &mut dyn Read) -> Result<CsfString>;
}

pub trait CsfWrite {
    /// Writes the entire [`CsfStringtable`] into a byte sink.
    ///
    /// This method is auto-implemented using [`write_header`][CsfWrite::write_header]
    /// and [`write_label`][CsfWrite::write_label].
    ///
    /// Examples
    ///
    /// ```ignore
    /// use rust_alert::csf::{CsfStringtable, io::{CsfWrite, CsfReader}}; // CsfReader implements CsfWrite
    ///
    /// let mut vec = vec![];
    /// let mut csf_reader = CsfReader::default();
    /// let csf = CsfStringtable::default();
    ///
    /// csf_reader.write(&csf, &mut vec)?;
    ///
    /// Ok(())
    /// ```
    fn write(&mut self, csf: &CsfStringtable, writer: &mut dyn Write) -> Result<()> {
        self.write_header(csf, writer)?;
        for label in csf.iter() {
            self.write_label(label, writer)?;
        }
        Ok(())
    }

    /// Writes [`CsfStringtable`] header info into a byte sink.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rust_alert::csf::{CsfStringtable, io::{CsfWrite, CsfReader}}; // CsfReader implements CsfWrite
    ///
    /// let mut vec = vec![];
    /// let mut csf_reader = CsfReader::default();
    /// let csf = CsfStringtable::default();
    ///
    /// csf_reader.write_header(&csf, &mut vec)?;
    ///
    /// Ok(())
    /// ```
    fn write_header(&mut self, csf: &CsfStringtable, writer: &mut dyn Write) -> Result<()>;

    /// Writes a [`CsfLabel`] into a byte sink.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rust_alert::csf::{CsfLabel, io::{CsfWrite, CsfReader}}; // CsfReader implements CsfWrite
    ///
    /// let mut vec = vec![];
    /// let mut csf_reader = CsfReader::default();
    /// let label = CsfLabel::new("A", "1");
    ///
    /// csf_reader.write_label(&label, &mut vec)?;
    ///
    /// Ok(())
    /// ```
    fn write_label(&mut self, label: &CsfLabel, writer: &mut dyn Write) -> Result<()>;

    /// Writes a [`CsfString`] into a byte sink.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rust_alert::csf::{CsfString, io::{CsfWrite, CsfReader}}; // CsfReader implements CsfWrite
    ///
    /// let mut vec = vec![];
    /// let mut csf_reader = CsfReader::default();
    /// let string = CsfString::new("A");
    ///
    /// csf_reader.write_string(&string, &mut vec)?;
    ///
    /// Ok(())
    /// ```
    fn write_string(&mut self, string: &CsfString, writer: &mut dyn Write) -> Result<()>;
}

/// Default implementation of [`CsfRead`] for binary CSF files.
/// See trait documentation for how to use it.
#[derive(Default)]
pub struct CsfReader {}

impl CsfReader {
    /// Creates a new empty [`CsfReader`], just like [`Default`].
    ///
    /// # Examples
    /// ```ignore
    /// use rust_alert::csf::io::CsfReader;
    ///
    /// let reader = CsfReader::new();
    /// ```
    pub fn new() -> Self {
        Default::default()
    }

    /// Reads and decodes (using bitwise negation) a UTF-16 string of given length.
    fn decode_utf16_string(reader: &mut dyn Read, len: usize) -> Result<String> {
        let mut buf = vec![0u8; len * 2];
        reader.read_exact(&mut buf)?;
        let buf: Vec<u16> = buf
            .chunks_exact(size_of::<u16>())
            .map(|x| !u16::from_le_bytes(x.try_into().unwrap_or_else(|_| unreachable!())))
            .collect();

        Ok(String::from_utf16(&buf)?)
    }

    /// Encodes (by bitwise negation) and writes a UTF-16 string.
    fn encode_utf16_string(string: &str) -> Vec<u8> {
        string
            .encode_utf16()
            .flat_map(|x| (!x).to_le_bytes())
            .collect::<Vec<_>>()
    }
}

impl CsfRead for CsfReader {
    fn read_header(&mut self, reader: &mut dyn Read) -> Result<(CsfStringtable, u32)> {
        // Read mandatory prefix.
        let mut buf = [0u8; size_of::<u32>()];
        reader.read_exact(&mut buf)?;
        if !buf.eq(CsfPrefixes::CSF_PREFIX) {
            return Err(Error::CsfMissingPrefix);
        };

        let mut csf = CsfStringtable::default();
        let mut buf = [0u8; size_of::<u32>()];

        reader.read_exact(&mut buf)?;
        csf.version = u32::from_le_bytes(buf).try_into()?;

        reader.read_exact(&mut buf)?;
        let num_labels = u32::from_le_bytes(buf);

        // We can ignore the total number of strings.
        reader.read_exact(&mut buf)?;

        reader.read_exact(&mut buf)?;
        csf.extra = u32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        csf.language = u32::from_le_bytes(buf).try_into()?;

        Ok((csf, num_labels))
    }

    fn read_label(&mut self, reader: &mut dyn Read) -> Result<CsfLabel> {
        let mut label = CsfLabel::default();
        let mut buf = [0u8; size_of::<u32>()];
        // Read mandatory prefix.
        reader.read_exact(&mut buf)?;
        if !buf.eq(CsfPrefixes::LBL_PREFIX) {
            return Err(Error::LblMissingPrefix);
        };
        // Read header values.
        reader.read_exact(&mut buf)?;
        let num_strings = u32::from_le_bytes(buf) as usize;
        reader.read_exact(&mut buf)?;
        let label_len = u32::from_le_bytes(buf) as usize;
        // Read label name.
        let mut buf = vec![0u8; label_len];
        reader.read_exact(&mut buf)?;
        label.name = String::from_utf8(buf)?;
        // Read list of strings.
        label.strings.reserve(num_strings);
        for _ in 0..num_strings {
            label.strings.push(self.read_string(reader)?);
        }

        Ok(label)
    }

    fn read_string(&mut self, reader: &mut dyn Read) -> Result<CsfString> {
        let mut string = CsfString::default();
        let mut buf = [0u8; size_of::<u32>()];
        // Read mandatory prefix.
        reader.read_exact(&mut buf)?;
        let has_extra = match buf.as_slice() {
            CsfPrefixes::STR_PREFIX => Ok(false),
            CsfPrefixes::STRW_PREFIX => Ok(true),
            _ => Err(Error::StrOrStrwMissingPrefix),
        }?;
        // Decode string.
        reader.read_exact(&mut buf)?;
        let len = u32::from_le_bytes(buf) as usize;
        string.value = Self::decode_utf16_string(reader, len)?;
        // Read extra data.
        if has_extra {
            reader.read_exact(&mut buf)?;
            let extra_len = u32::from_le_bytes(buf) as usize;
            let mut buf = vec![0u8; extra_len];
            reader.read_exact(&mut buf)?;
            string.extra_value = buf;
        }

        Ok(string)
    }
}

impl CsfWrite for CsfReader {
    fn write_header(&mut self, csf: &CsfStringtable, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(CsfPrefixes::CSF_PREFIX)?;
        writer.write_all(&TryInto::<u32>::try_into(csf.version)?.to_le_bytes())?;
        writer.write_all(&(csf.len() as u32).to_le_bytes())?;
        writer.write_all(&(csf.strings_len() as u32).to_le_bytes())?;
        writer.write_all(&csf.extra.to_le_bytes())?;
        writer.write_all(&TryInto::<u32>::try_into(csf.language)?.to_le_bytes())?;

        Ok(())
    }

    fn write_label(&mut self, label: &CsfLabel, writer: &mut dyn Write) -> Result<()> {
        // Write label info.
        writer.write_all(CsfPrefixes::LBL_PREFIX)?;
        writer.write_all(&(label.strings.len() as u32).to_le_bytes())?;
        writer.write_all(&(label.name.len() as u32).to_le_bytes())?;
        writer.write_all(label.name.as_bytes())?;
        // Write strings.
        for string in &label.strings {
            self.write_string(string, writer)?;
        }

        Ok(())
    }

    fn write_string(&mut self, string: &CsfString, writer: &mut dyn Write) -> Result<()> {
        let extra_len = string.extra_value.len() as u32;
        let has_extra = extra_len > 0;
        let prefix = if has_extra {
            CsfPrefixes::STRW_PREFIX
        } else {
            CsfPrefixes::STR_PREFIX
        };
        // Write string info.
        writer.write_all(prefix)?;
        let utf16 = Self::encode_utf16_string(&string.value);
        writer.write_all(&((utf16.len() / 2) as u32).to_le_bytes())?;
        // Write string data.
        writer.write_all(&utf16)?;
        if has_extra {
            writer.write_all(&extra_len.to_le_bytes())?;
            writer.write_all(&string.extra_value)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, io::Read};

    use crate::{
        csf::{
            io::{CsfRead, CsfReader, CsfWrite, Error},
            CsfLabel, CsfString, CsfStringtable,
        },
        unwrap_assert,
    };

    fn make_string(string: impl Into<String>, extra_string: impl Into<String>) -> Vec<u8> {
        let string: String = string.into();
        let wide: String = extra_string.into();
        let first = if !wide.is_empty() { 'W' } else { ' ' };
        let mut buf = vec![first as u8, b'R', b'T', b'S', string.len() as u8, 0, 0, 0];
        buf.extend(CsfReader::encode_utf16_string(&string));
        if !wide.is_empty() {
            buf.extend(vec![wide.len() as u8, 0, 0, 0]);
            buf.extend(wide.as_bytes());
        }
        buf
    }

    fn make_label(
        label: impl Into<String>,
        string: impl Into<String>,
        extra_string: impl Into<String>,
    ) -> Vec<u8> {
        let label: String = label.into();
        let mut buf = vec![b' ', b'L', b'B', b'L', 1, 0, 0, 0];
        buf.extend_from_slice(&(label.len() as u32).to_le_bytes());
        buf.extend_from_slice(label.as_bytes());
        buf.extend(make_string(string, extra_string));
        buf
    }

    fn make_header() -> Vec<u8> {
        vec![
            b' ', b'F', b'S', b'C', 3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]
    }

    fn make_stringtable(
        label: impl Into<String>,
        string: impl Into<String>,
        extra_string: impl Into<String>,
    ) -> Vec<u8> {
        let mut buf = make_header();
        buf.extend(make_label(label, string, extra_string));
        buf
    }

    /// Read a CsfString (Ok).
    #[test]
    fn read_string_ok() {
        let str = "String";
        let buf = make_string(str, "");
        let reader: &mut dyn Read = &mut buf.as_slice();

        let expected = CsfString::new(str);
        let actual = CsfReader::new().read_string(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    /// Read a CsfString (Err). Missing RTS prefix.
    #[test]
    fn read_string_err_rts() {
        let buf = vec![b' ', b'N', b'T', b'S'];
        let reader: &mut dyn Read = &mut buf.as_slice();

        let actual = CsfReader::new().read_string(reader);

        assert!(actual.is_err());
        matches!(actual.unwrap_err(), Error::StrOrStrwMissingPrefix);
    }

    /// Read a CsfString (Err). Sudden EOF.
    #[test]
    fn read_string_err_eof() {
        let mut buf = make_string("String", "");
        buf.pop();
        let reader: &mut dyn Read = &mut buf.as_slice();

        let actual = CsfReader::new().read_string(reader);

        assert!(actual.is_err());
        matches!(actual.unwrap_err(), Error::IO(..));
    }

    /// Read a CsfString (Err). Not a valid WW UTF-16 string.
    #[test]
    fn read_string_err_utf() {
        let buf = vec![b' ', b'R', b'T', b'S', 2u8, 0, 0, 0, 0xFF, 0x27, 0xFF, 0xFF];
        let reader: &mut dyn Read = &mut buf.as_slice();

        let actual = CsfReader::new().read_string(reader);

        assert!(actual.is_err());
        matches!(actual.unwrap_err(), Error::Utf16(..));
    }

    /// Read a wide CsfString (Ok).
    #[test]
    fn read_wide_string_ok() {
        let str = "String";
        let wstr = "Wide";
        let buf = make_string(str, wstr);
        let reader: &mut dyn Read = &mut buf.as_slice();

        let expected = CsfString {
            value: str.into(),
            extra_value: wstr.into(),
        };
        let actual = CsfReader::new().read_string(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    /// Read a wide CsfString (Err). Missing WRTS prefix.
    #[test]
    fn read_wide_string_err_wrts() {
        let buf = vec![b'N', b'R', b'T', b'S'];
        let reader: &mut dyn Read = &mut buf.as_slice();

        let actual = CsfReader::new().read_string(reader);

        assert!(actual.is_err());
        matches!(actual.unwrap_err(), Error::StrOrStrwMissingPrefix);
    }

    /// Read a wide CsfString (Err). Sudden extra value EOF.
    #[test]
    fn read_wide_string_err_eof() {
        let mut buf = make_string("String", "Extra");
        buf.pop();
        let reader: &mut dyn Read = &mut buf.as_slice();

        let actual = CsfReader::new().read_string(reader);

        assert!(actual.is_err());
        matches!(actual.unwrap_err(), Error::IO(..));
    }

    /// Read a CsfLabel (Ok).
    #[test]
    fn read_label_ok() {
        let label = "Label";
        let string = "String";
        let buf = make_label(label, string, "");
        let reader: &mut dyn Read = &mut buf.as_slice();

        let expected = CsfLabel {
            name: label.into(),
            strings: vec![CsfString::new(string)],
        };
        let actual = CsfReader::new().read_label(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    /// Read a CSF header (Ok).
    #[test]
    fn read_csf_header_ok() {
        let buf = make_header();
        let reader: &mut dyn Read = &mut buf.as_slice();

        let expected = CsfStringtable::default();
        let expected_len = 1;
        let actual = CsfReader::new().read_header(reader);

        assert!(actual.is_ok());
        let (csf, len) = actual.unwrap_or_else(|_| unreachable!());
        assert_eq!(csf, expected);
        assert_eq!(len, expected_len);
    }

    /// Read a CsfStringtable (Ok).
    #[test]
    fn read_stringtable_ok() {
        let label = "Label";
        let string = "String";
        let buf = make_stringtable(label, string, "");
        let reader: &mut dyn Read = &mut buf.as_slice();
        let mut labels: HashSet<CsfLabel> = Default::default();
        labels.insert(CsfLabel::new(label, string));

        let mut expected = CsfStringtable::default();
        expected.extend(labels);
        let actual = CsfReader::new().read(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    /// Write a CsfString (Ok).
    #[test]
    fn write_string_ok() {
        let expected = CsfString {
            value: "String".into(),
            extra_value: "".into(),
        };

        let mut buf: Vec<u8> = vec![];
        let res = CsfReader::new().write_string(&expected, &mut buf);
        assert!(res.is_ok());
        let reader: &mut dyn Read = &mut buf.as_slice();
        let actual = CsfReader::new().read_string(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

        /// Write a wide CsfString (Ok).
        #[test]
        fn write_wide_string_ok() {
            let expected = CsfString {
                value: "String".into(),
                extra_value: "Wide".into(),
            };
    
            let mut buf: Vec<u8> = vec![];
            let res = CsfReader::new().write_string(&expected, &mut buf);
            assert!(res.is_ok());
            let reader: &mut dyn Read = &mut buf.as_slice();
            let actual = CsfReader::new().read_string(reader);
    
            assert!(actual.is_ok());
            unwrap_assert!(actual, expected);
        }

    /// Write a CsfLabel (Ok).
    #[test]
    fn write_label_ok() {
        let expected = CsfLabel {
            name: "Label".into(),
            strings: vec![CsfString::new("String")],
        };

        let mut buf: Vec<u8> = vec![];
        let res = CsfReader::new().write_label(&expected, &mut buf);
        assert!(res.is_ok());
        let reader: &mut dyn Read = &mut buf.as_slice();
        let actual = CsfReader::new().read_label(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    /// Write a CSF header (Ok).
    #[test]
    fn write_header_ok() {
        let expected = CsfStringtable::default();

        let mut buf: Vec<u8> = vec![];
        let res = CsfReader::new().write_header(&expected, &mut buf);
        assert!(res.is_ok());
        let reader: &mut dyn Read = &mut buf.as_slice();
        let actual = CsfReader::new().read_header(reader);

        assert!(actual.is_ok());
        assert_eq!(actual.unwrap_or_else(|_| unreachable!()).0, expected);
    }

    /// Write a CsfStringtable (OK).
    #[test]
    fn write_stringtable_ok() {
        let mut expected = CsfStringtable::default();
        expected.create("Label", "String");
        expected.create("Label2", "String2");

        let mut buf: Vec<u8> = vec![];
        let res = CsfReader::new().write(&expected, &mut buf);
        assert!(res.is_ok());
        let reader: &mut dyn Read = &mut buf.as_slice();
        let actual = CsfReader::new().read(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }
}

#[cfg(test)]
mod examples {
    mod csf_reader {
        use crate as rust_alert;
        use crate::csf::io::Result;
        use std::{fs::File, io::Seek, path::PathBuf};

        #[test]
        fn new() {
            use rust_alert::csf::io::CsfReader;

            let _reader = CsfReader::new();
        }

        fn do_for_file<F>(path: &str, fun: F) -> std::io::Result<File>
        where
            F: Fn(PathBuf) -> std::io::Result<File>,
        {
            let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.pop();
            p.push(path);
            fun(p)
        }

        #[test]
        fn read() -> Result<()> {
            use rust_alert::csf::io::{CsfRead, CsfReader}; // CsfReader implements CsfRead

            let mut file = do_for_file("test_data/example.csf", std::fs::File::open)?; // NOTE: replacement for tests
            let mut csf_reader = CsfReader::default();
            let csf = csf_reader.read(&mut file)?;

            assert_eq!(csf.len(), 1);
            assert_eq!(csf.get_str("Label"), Some("String"));

            Ok(())
        }

        #[test]
        fn read_header() -> Result<()> {
            use rust_alert::csf::io::{CsfRead, CsfReader};

            let mut file = do_for_file("test_data/example.csf", std::fs::File::open)?; // NOTE: replacement for tests
            let mut csf_reader = CsfReader::default();
            let (_csf, num_labels) = csf_reader.read_header(&mut file)?;

            assert_eq!(num_labels, 1);

            Ok(())
        }

        #[test]
        fn read_label() -> Result<()> {
            use rust_alert::csf::io::{CsfRead, CsfReader}; // CsfReader implements CsfRead

            let mut file = do_for_file("test_data/example.csf", std::fs::File::open)?; // NOTE: replacement for std::fs::File::open()
            let mut csf_reader = CsfReader::default();
            file.seek(std::io::SeekFrom::Start(CsfReader::CSF_HEADER_SIZE as u64))?;
            let label = csf_reader.read_label(&mut file)?;

            assert_eq!(label.name, "Label");
            assert_eq!(label.get_first_str(), Some("String"));

            Ok(())
        }

        #[test]
        fn read_string() -> Result<()> {
            use rust_alert::csf::io::{CsfRead, CsfReader}; // CsfReader implements CsfRead

            let vec = vec![
                b' ', b'R', b'T', b'S', 3u8, 0, 0, 0, 0xAC, 0xFF, 0x8B, 0xFF, 0x8D, 0xFF,
            ];
            let mut csf_reader = CsfReader::default();
            let string = csf_reader.read_string(&mut vec.as_slice())?;

            assert_eq!(string.value, "Str");
            assert!(string.extra_value.is_empty());

            Ok(())
        }

        #[test]
        fn write_header() -> Result<()> {
            use rust_alert::csf::{
                io::{CsfReader, CsfWrite},
                CsfStringtable,
            };

            let mut vec = vec![];
            let mut csf_reader = CsfReader::default();
            let csf = CsfStringtable::default();

            csf_reader.write_header(&csf, &mut vec)?;

            Ok(())
        }

        #[test]
        fn write_label() -> Result<()> {
            use rust_alert::csf::{
                io::{CsfReader, CsfWrite},
                CsfLabel,
            };

            let mut vec = vec![];
            let mut csf_reader = CsfReader::default();
            let label = CsfLabel::new("A", "1");

            csf_reader.write_label(&label, &mut vec)?;

            Ok(())
        }

        #[test]
        fn write_string() -> Result<()> {
            use rust_alert::csf::{
                io::{CsfReader, CsfWrite},
                CsfString,
            };

            let mut vec = vec![];
            let mut csf_reader = CsfReader::default();
            let string = CsfString::new("A");

            csf_reader.write_string(&string, &mut vec)?;

            Ok(())
        }

        #[test]
        fn write() -> Result<()> {
            use rust_alert::csf::{
                io::{CsfReader, CsfWrite},
                CsfStringtable,
            };

            let mut vec = vec![];
            let mut csf_reader = CsfReader::default();
            let csf = CsfStringtable::default();

            csf_reader.write(&csf, &mut vec)?;

            Ok(())
        }
    }
}
