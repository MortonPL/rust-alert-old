use std::io::{Read, Write};
use std::mem::size_of;

use crate::core::csf::{CsfLabel, CsfString, CsfStringtable, Error};

type Result<T> = std::result::Result<T, Error>;

pub struct CsfReader {}
pub struct CsfWriter {}

struct CsfPrefixes {}
impl CsfPrefixes {
    const CSF_PREFIX: &str = " FSC";
    const LBL_PREFIX: &str = " LBL";
    const STR_PREFIX: &str = " RTS";
    const WSTR_PREFIX: &str = "WRTS";
}

impl CsfReader {
    /// Create a CSF file struct from input.
    pub fn read_file(reader: &mut dyn Read) -> Result<CsfStringtable> {
        // Read mandatory prefix.
        let mut buf = [0u8; size_of::<u32>()];
        reader.read_exact(&mut buf)?;
        if !std::str::from_utf8(&buf)
            .unwrap()
            .eq(CsfPrefixes::CSF_PREFIX)
        {
            return Err(Error::CsfMissingPrefix);
        };

        let (mut csf, num_labels) = Self::read_csf_header(reader)?;

        csf.labels.reserve(num_labels as usize);
        for _ in 0..num_labels {
            csf.add_label(Self::read_label(reader)?);
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
        if !std::str::from_utf8(&buf)
            .unwrap()
            .eq(CsfPrefixes::LBL_PREFIX)
        {
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
        let is_wide = match std::str::from_utf8(&buf) {
            Ok(CsfPrefixes::STR_PREFIX) => Ok(false),
            Ok(CsfPrefixes::WSTR_PREFIX) => Ok(true),
            _ => Err(Error::RtsOrWrtsMissingPrefix),
        }?;

        reader.read_exact(&mut buf)?;
        let len = u32::from_le_bytes(buf) as usize;
        string.value = Self::decode_utf16_string(reader, len)?;

        // Read extra data.
        if is_wide {
            reader.read_exact(&mut buf)?;
            let extra_len = u32::from_le_bytes(buf) as usize;

            let mut buf = vec![0u8; extra_len];
            reader.read_exact(&mut buf)?;
            string.extra_value = String::from_utf8(buf)?;
        }

        Ok(string)
    }

    /// Read and decode (by bitwise negation) a UTF-16 string.
    fn decode_utf16_string(reader: &mut dyn Read, len: usize) -> Result<String> {
        let mut buf = vec![0u8; len * 2];
        reader.read_exact(&mut buf)?;
        let buf: Vec<u16> = buf
            .chunks(size_of::<u16>())
            .map(|x| !u16::from_le_bytes(x.try_into().unwrap()))
            .collect();

        Ok(String::from_utf16(&buf)?)
    }
}

impl CsfWriter {
    /// Write a CSF file struct to output.
    pub fn write_file(csf: &CsfStringtable, writer: &mut dyn Write) -> Result<()> {
        Self::write_csf_header(csf, writer)?;

        for label in csf.labels.values() {
            CsfWriter::write_label(label, writer)?;
        }

        Ok(())
    }

    /// Write a CSF file header for a provided stringtable.
    pub fn write_csf_header(csf: &CsfStringtable, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(CsfPrefixes::CSF_PREFIX.as_bytes())?;
        writer.write_all(&TryInto::<u32>::try_into(csf.version)?.to_le_bytes())?;
        writer.write_all(&(csf.get_label_count() as u32).to_le_bytes())?;
        writer.write_all(&(csf.get_string_count() as u32).to_le_bytes())?;
        writer.write_all(&csf.extra.to_le_bytes())?;
        writer.write_all(&TryInto::<u32>::try_into(csf.language)?.to_le_bytes())?;

        Ok(())
    }

    /// Write a CSF label struct to output.
    pub fn write_label(label: &CsfLabel, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(CsfPrefixes::LBL_PREFIX.as_bytes())?;
        writer.write_all(&(label.strings.len() as u32).to_le_bytes())?;
        writer.write_all(&(label.name.len() as u32).to_le_bytes())?;
        writer.write_all(label.name.as_bytes())?;

        for string in &label.strings {
            CsfWriter::write_string(string, writer)?;
        }

        Ok(())
    }

    /// Write a CSF string struct to output.
    pub fn write_string(string: &CsfString, writer: &mut dyn Write) -> Result<()> {
        let extra_len = string.extra_value.len() as u32;
        let is_wide = extra_len > 0;
        let prefix = if is_wide {
            CsfPrefixes::WSTR_PREFIX
        } else {
            CsfPrefixes::STR_PREFIX
        };
        writer.write_all(prefix.as_bytes())?;
        writer.write_all(&(string.value.len() as u32).to_le_bytes())?;

        writer.write_all(&Self::encode_utf16_string(&string.value)?)?;

        if is_wide {
            writer.write_all(&extra_len.to_le_bytes())?;
            writer.write_all(string.extra_value.as_bytes())?;
        }

        Ok(())
    }

    fn encode_utf16_string(string: &str) -> Result<Vec<u8>> {
        Ok(string
            .encode_utf16()
            .flat_map(|x| (!x).to_le_bytes())
            .collect::<Vec<_>>())
    }
}
