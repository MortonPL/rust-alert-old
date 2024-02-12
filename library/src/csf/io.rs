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
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("CSF prefix missing")]
    CsfMissingPrefix,
    #[error("LBL prefix missing")]
    LblMissingPrefix,
    #[error("RTS/WRTS prefix missing!")]
    RtsOrWrtsMissingPrefix,
    #[error("{0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("{0}")]
    Utf16(#[from] FromUtf16Error),
    #[error("{0}")]
    CSF(#[from] crate::csf::Error),
}

#[doc(hidden)]
type Result<T> = std::result::Result<T, Error>;

struct CsfPrefixes {}
impl CsfPrefixes {
    const CSF_PREFIX: &[u8] = b" FSC";
    const LBL_PREFIX: &[u8] = b" LBL";
    const STR_PREFIX: &[u8] = b" RTS";
    const WSTR_PREFIX: &[u8] = b"WRTS";
}

/// Provides static methods for reading CSF files.
pub struct CsfReader {}

impl CsfReader {
    /// Create a CSF file struct from input.
    pub fn read_file(reader: &mut dyn Read) -> Result<CsfStringtable> {
        // Read mandatory prefix.
        let mut buf = [0u8; size_of::<u32>()];
        reader.read_exact(&mut buf)?;
        if !buf.eq(CsfPrefixes::CSF_PREFIX) {
            return Err(Error::CsfMissingPrefix);
        };
        // Read file header.
        let (mut csf, num_labels) = Self::read_csf_header(reader)?;
        // Read all labels.
        csf.reserve(num_labels as usize);
        for _ in 0..num_labels {
            csf.insert(Self::read_label(reader)?);
        }

        Ok(csf)
    }

    /// Read a CSF file header and construct an empty CsfStringtable.
    /// Returns an empty stringtable and number of labels to be read.
    pub fn read_csf_header(reader: &mut dyn Read) -> Result<(CsfStringtable, u32)> {
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

    /// Create a CSF label struct from input.
    pub fn read_label(reader: &mut dyn Read) -> Result<CsfLabel> {
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
            label.strings.push(Self::read_string(reader)?);
        }

        Ok(label)
    }

    /// Create a CSF string struct from input.
    pub fn read_string(reader: &mut dyn Read) -> Result<CsfString> {
        let mut string = CsfString::default();
        let mut buf = [0u8; size_of::<u32>()];
        // Read mandatory prefix.
        reader.read_exact(&mut buf)?;
        let has_extra = match buf.as_slice() {
            CsfPrefixes::STR_PREFIX => Ok(false),
            CsfPrefixes::WSTR_PREFIX => Ok(true),
            _ => Err(Error::RtsOrWrtsMissingPrefix),
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

    /// Read and decode (by bitwise negation) a UTF-16 string.
    fn decode_utf16_string(reader: &mut dyn Read, len: usize) -> Result<String> {
        let mut buf = vec![0u8; len * 2];
        reader.read_exact(&mut buf)?;
        let buf: Vec<u16> = buf
            .chunks_exact(size_of::<u16>())
            .map(|x| !u16::from_le_bytes(x.try_into().unwrap_or_else(|_| unreachable!())))
            .collect();

        Ok(String::from_utf16(&buf)?)
    }
}

/// Provides static methods for writing CSF files.
pub struct CsfWriter {}

impl CsfWriter {
    /// Write a CSF file struct to output.
    pub fn write_file(csf: &CsfStringtable, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(CsfPrefixes::CSF_PREFIX)?;
        Self::write_csf_header(csf, writer)?;
        for label in csf.iter() {
            CsfWriter::write_label(label, writer)?;
        }

        Ok(())
    }

    /// Write a CSF file header for a provided stringtable.
    pub fn write_csf_header(csf: &CsfStringtable, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(&TryInto::<u32>::try_into(csf.version)?.to_le_bytes())?;
        writer.write_all(&(csf.len() as u32).to_le_bytes())?;
        writer.write_all(&(csf.strings_len() as u32).to_le_bytes())?;
        writer.write_all(&csf.extra.to_le_bytes())?;
        writer.write_all(&TryInto::<u32>::try_into(csf.language)?.to_le_bytes())?;

        Ok(())
    }

    /// Write a CSF label struct to output.
    pub fn write_label(label: &CsfLabel, writer: &mut dyn Write) -> Result<()> {
        // Write label info.
        writer.write_all(CsfPrefixes::LBL_PREFIX)?;
        writer.write_all(&(label.strings.len() as u32).to_le_bytes())?;
        writer.write_all(&(label.name.len() as u32).to_le_bytes())?;
        writer.write_all(label.name.as_bytes())?;
        // Write strings.
        for string in &label.strings {
            CsfWriter::write_string(string, writer)?;
        }

        Ok(())
    }

    /// Write a CSF string struct to output.
    pub fn write_string(string: &CsfString, writer: &mut dyn Write) -> Result<()> {
        let extra_len = string.extra_value.len() as u32;
        let has_extra = extra_len > 0;
        let prefix = if has_extra {
            CsfPrefixes::WSTR_PREFIX
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

    /// Encode (by bitwise negation) and write a UTF-16 string.
    fn encode_utf16_string(string: &str) -> Vec<u8> {
        string
            .encode_utf16()
            .flat_map(|x| (!x).to_le_bytes())
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, io::Read};

    use crate::{
        csf::{
            io::{CsfReader, CsfWriter},
            CsfLabel, CsfLanguageEnum, CsfString, CsfStringtable, CsfVersionEnum,
        },
        unwrap_assert,
    };

    fn make_string(string: impl Into<String>, extra_string: impl Into<String>) -> Vec<u8> {
        let string: String = string.into();
        let wide: String = extra_string.into();
        let first = if !wide.is_empty() { 'W' } else { ' ' };
        let mut buf = vec![first as u8, b'R', b'T', b'S', string.len() as u8, 0, 0, 0];
        buf.extend(CsfWriter::encode_utf16_string(&string));
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
        vec![3u8, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    }

    fn make_stringtable(
        label: impl Into<String>,
        string: impl Into<String>,
        extra_string: impl Into<String>,
    ) -> Vec<u8> {
        let mut buf = vec![b' ', b'F', b'S', b'C'];
        buf.extend(make_header());
        buf.extend(make_label(label, string, extra_string));
        buf
    }

    #[test]
    /// Read a CsfString (Ok).
    fn read_string_ok() {
        let str = "String";
        let buf = make_string(str, "");
        let reader: &mut dyn Read = &mut buf.as_slice();

        let expected = CsfString::new(str);
        let actual = CsfReader::read_string(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    #[test]
    /// Read a wide CsfString (Ok).
    fn read_wide_string_ok() {
        let str = "String";
        let wstr = "Wide";
        let buf = make_string(str, wstr);
        let reader: &mut dyn Read = &mut buf.as_slice();

        let expected = CsfString {
            value: str.into(),
            extra_value: wstr.into(),
        };
        let actual = CsfReader::read_string(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    #[test]
    /// Read a CsfLabel (Ok).
    fn read_label_ok() {
        let label = "Label";
        let string = "String";
        let buf = make_label(label, string, "");
        let reader: &mut dyn Read = &mut buf.as_slice();

        let expected = CsfLabel {
            name: label.into(),
            strings: vec![CsfString::new(string)],
        };
        let actual = CsfReader::read_label(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    #[test]
    /// Read a CSF header (Ok).
    fn read_csf_header_ok() {
        let buf = make_header();
        let reader: &mut dyn Read = &mut buf.as_slice();

        let expected = CsfStringtable::default();
        let expected_len = 1;
        let actual = CsfReader::read_csf_header(reader);

        assert!(actual.is_ok());
        let (csf, len) = actual.unwrap_or_else(|_| unreachable!());
        assert_eq!(csf, expected);
        assert_eq!(len, expected_len);
    }

    #[test]
    /// Read a CsfStringtable (Ok).
    fn read_stringtable_ok() {
        let label = "Label";
        let string = "String";
        let buf = make_stringtable(label, string, "");
        let reader: &mut dyn Read = &mut buf.as_slice();
        let mut labels: HashSet<CsfLabel> = Default::default();
        labels.insert(CsfLabel::new(label, string));

        let mut expected = CsfStringtable::default();
        expected.extend(labels);
        let actual = CsfReader::read_file(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    #[test]
    /// Write a CsfString (Ok).
    fn write_string_ok() {
        let expected = CsfString {
            value: "String".into(),
            extra_value: "".into(),
        };

        let mut buf: Vec<u8> = vec![];
        let res = CsfWriter::write_string(&expected, &mut buf);
        assert!(res.is_ok());
        let reader: &mut dyn Read = &mut buf.as_slice();
        let actual = CsfReader::read_string(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    #[test]
    /// Write a CsfLabel (Ok).
    fn write_label_ok() {
        let expected = CsfLabel {
            name: "Label".into(),
            strings: vec![CsfString::new("String")],
        };

        let mut buf: Vec<u8> = vec![];
        let res = CsfWriter::write_label(&expected, &mut buf);
        assert!(res.is_ok());
        let reader: &mut dyn Read = &mut buf.as_slice();
        let actual = CsfReader::read_label(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }

    #[test]
    /// Write a CSF header (Ok).
    fn write_header_ok() {
        let expected = CsfStringtable::default();

        let mut buf: Vec<u8> = vec![];
        let res = CsfWriter::write_csf_header(&expected, &mut buf);
        assert!(res.is_ok());
        let reader: &mut dyn Read = &mut buf.as_slice();
        let actual = CsfReader::read_csf_header(reader);

        assert!(actual.is_ok());
        assert_eq!(actual.unwrap_or_else(|_| unreachable!()).0, expected);
    }

    #[test]
    /// Write a CsfStringtable (OK).
    fn write_stringtable_ok() {
        let mut expected = CsfStringtable::default();
        expected.create("Label", "String");
        expected.create("Label2", "String2");

        let mut buf: Vec<u8> = vec![];
        let res = CsfWriter::write_file(&expected, &mut buf);
        assert!(res.is_ok());
        let reader: &mut dyn Read = &mut buf.as_slice();
        let actual = CsfReader::read_file(reader);

        assert!(actual.is_ok());
        unwrap_assert!(actual, expected);
    }
}
