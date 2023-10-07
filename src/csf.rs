use std::collections::HashMap;
use std::io::{Read, Write};
use std::mem::size_of;

type AnyError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl Error {
    fn new(msg: String) -> Self {
        Error { msg }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, Default)]
pub enum CsfVersionEnum {
    Nox = 2,
    #[default]
    Cnc = 3,
}

impl TryFrom<u32> for CsfVersionEnum {
    type Error = Error;
    fn try_from(value: u32) -> Result<Self, Error> {
        match value {
            x if x == CsfVersionEnum::Nox as u32 => Ok(CsfVersionEnum::Nox),
            x if x == CsfVersionEnum::Cnc as u32 => Ok(CsfVersionEnum::Cnc),
            x => Err(Error::new(format!("Unknown version number {x}!"))),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, Default)]
pub enum CsfLanguageEnum {
    #[default]
    ENUS = 0,
    ENUK = 1,
    DE = 2,
    FR = 3,
    ES = 4,
    IT = 5,
    JA = 6,
    XX = 7,
    KO = 8,
    ZHCN = 9,
}

impl TryFrom<u32> for CsfLanguageEnum {
    type Error = Error;
    fn try_from(value: u32) -> Result<Self, Error> {
        match value {
            x if x == CsfLanguageEnum::ENUS as u32 => Ok(CsfLanguageEnum::ENUS),
            x if x == CsfLanguageEnum::ENUK as u32 => Ok(CsfLanguageEnum::ENUK),
            x if x == CsfLanguageEnum::DE as u32 => Ok(CsfLanguageEnum::DE),
            x if x == CsfLanguageEnum::FR as u32 => Ok(CsfLanguageEnum::FR),
            x if x == CsfLanguageEnum::ES as u32 => Ok(CsfLanguageEnum::ES),
            x if x == CsfLanguageEnum::IT as u32 => Ok(CsfLanguageEnum::IT),
            x if x == CsfLanguageEnum::JA as u32 => Ok(CsfLanguageEnum::JA),
            x if x == CsfLanguageEnum::XX as u32 => Ok(CsfLanguageEnum::XX),
            x if x == CsfLanguageEnum::KO as u32 => Ok(CsfLanguageEnum::KO),
            x if x == CsfLanguageEnum::ZHCN as u32 => Ok(CsfLanguageEnum::ZHCN),
            x => Err(Error::new(format!("Unknown language number {x}!"))),
        }
    }
}

/// A CSF file contains a header and a list of CSF labels.
/// Labels are stored as a dictionary for easy manipulation.
#[derive(Clone, Debug, Default)]
pub struct Csf {
    pub version: CsfVersionEnum,
    pub num_labels: u32,
    pub num_strings: u32,
    pub extra: u32,
    pub language: CsfLanguageEnum,
    pub labels: HashMap<String, CsfLabel>,
}

impl Csf {
    const PREFIX: &str = " FSC";

    /// Create a CSF file struct from input.
    pub fn read(reader: &mut impl Read) -> Result<Self, AnyError> {
        let mut csf = Self::default();
        let mut buf = [0u8; size_of::<u32>()];

        reader.read_exact(&mut buf)?;
        if !std::str::from_utf8(&buf).unwrap().eq(Self::PREFIX) {
            return Err(Error::new("CSF prefix missing!".to_string()).into());
        };

        reader.read_exact(&mut buf)?;
        csf.version = u32::from_le_bytes(buf).try_into()?;
        reader.read_exact(&mut buf)?;
        csf.num_labels = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        csf.num_strings = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        csf.extra = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        csf.language = u32::from_le_bytes(buf).try_into()?;

        for _ in 0..csf.num_labels {
            let label = CsfLabel::read(reader)?;
            csf.labels.insert(label.label.clone(), label);
        }

        Ok(csf)
    }

    // Write a CSF file struct to output.
    pub fn write(&self, writer: &mut impl Write) -> Result<(), AnyError> {
        writer.write_all(Self::PREFIX.as_bytes())?;
        writer.write_all(&(self.version as u32).to_le_bytes())?;
        writer.write_all(&self.num_labels.to_le_bytes())?;
        writer.write_all(&self.num_strings.to_le_bytes())?;
        writer.write_all(&self.extra.to_le_bytes())?;
        writer.write_all(&(self.language as u32).to_le_bytes())?;

        for label in self.labels.values() {
            label.write(writer)?;
        }

        Ok(())
    }

    pub fn add_label(&mut self, label: CsfLabel) {
        let num_strings = label.strings.len() as u32;
        if self.labels.insert(label.label.clone(), label).is_none() {
            self.num_labels += 1;
            self.num_strings += num_strings;
        }
    }

    pub fn remove_label(&mut self, string: &String) {
        if let Some(label) = self.labels.remove(string) {
            self.num_labels -= 1;
            self.num_strings -= label.strings.len() as u32;
        }
    }
}

/// A CSF label contains a name and a collection of CSF strings.
/// Every label in vanilla game files contains only one string.
#[derive(Clone, Debug, Default)]
pub struct CsfLabel {
    pub label: String,
    pub strings: Vec<CsfString>,
}

impl CsfLabel {
    const PREFIX: &str = " LBL";

    pub fn new(label: String, string: String) -> Self {
        CsfLabel {
            label,
            strings: vec![CsfString {
                string,
                ..Default::default()
            }],
        }
    }

    /// Create a CSF label struct from input.
    pub fn read(reader: &mut impl Read) -> Result<Self, AnyError> {
        let mut label = Self::default();
        let mut buf = [0u8; size_of::<u32>()];

        reader.read_exact(&mut buf)?;
        if !std::str::from_utf8(&buf).unwrap().eq(Self::PREFIX) {
            return Err(Error::new("LBL prefix missing!".to_string()).into());
        };

        reader.read_exact(&mut buf)?;
        let num_strings = u32::from_le_bytes(buf) as usize;
        reader.read_exact(&mut buf)?;
        let label_len = u32::from_le_bytes(buf) as usize;

        let mut buf = vec![0u8; label_len];
        reader.read_exact(&mut buf)?;
        label.label = String::from_utf8(buf)?;

        for _ in 0..num_strings {
            let string = CsfString::read(reader)?;
            label.strings.push(string);
        }

        Ok(label)
    }

    // Write a CSF label struct to output.
    pub fn write(&self, writer: &mut impl Write) -> Result<(), AnyError> {
        writer.write_all(Self::PREFIX.as_bytes())?;
        writer.write_all(&(self.strings.len() as u32).to_le_bytes())?;
        writer.write_all(&(self.label.len() as u32).to_le_bytes())?;
        writer.write_all(self.label.as_bytes())?;

        for string in &self.strings {
            string.write(writer)?;
        }

        Ok(())
    }
}

/// A CSF string contains a LE UTF-16 string. There are two types of CSF strings:
/// normal (prefix RTS) and wide (prefix WRTS) which can contain an extra ASCII string.
/// All vanilla game strings are normal.
/// To obtain actual UTF-16 strings, bytes have to be negated bitwise.
#[derive(Clone, Debug, Default)]
pub struct CsfString {
    pub string: String,
    pub extra_string: String,
}

impl CsfString {
    const PREFIX: &str = " RTS";
    const PREFIX_WIDE: &str = "WRTS";

    /// Create a CSF string struct from input.
    pub fn read(reader: &mut impl Read) -> Result<Self, AnyError> {
        let mut string = CsfString::default();
        let mut buf = [0u8; size_of::<u32>()];

        reader.read_exact(&mut buf)?;
        let is_wide = match std::str::from_utf8(&buf).unwrap() {
            CsfString::PREFIX => Ok(false),
            CsfString::PREFIX_WIDE => Ok(true),
            _ => Err(Error::new("RTS/WRTS prefix missing!".into())),
        }?;

        reader.read_exact(&mut buf)?;
        let len = u32::from_le_bytes(buf) as usize;

        let mut buf = vec![0u8; len * 2];
        reader.read_exact(&mut buf)?;
        let buf: Vec<u16> = buf
            .chunks(size_of::<u16>())
            .map(|x| !u16::from_le_bytes(x.try_into().unwrap()))
            .collect();
        string.string = String::from_utf16(&buf)?;

        if is_wide {
            let mut buf = [0u8; size_of::<u32>()];
            reader.read_exact(&mut buf)?;
            let extra_len = u32::from_le_bytes(buf) as usize;

            let mut buf = vec![0u8; extra_len];
            reader.read_exact(&mut buf)?;
            string.extra_string = String::from_utf8(buf)?;
        }

        Ok(string)
    }

    // Write a CSF string struct to output.
    pub fn write(&self, writer: &mut impl Write) -> Result<(), AnyError> {
        let extra_len = self.extra_string.len() as u32;
        let is_wide = extra_len > 0;
        let prefix = if is_wide {
            Self::PREFIX_WIDE
        } else {
            Self::PREFIX
        };
        writer.write_all(prefix.as_bytes())?;
        writer.write_all(&(self.string.len() as u32).to_le_bytes())?;
        unsafe {
            writer.write_all(
                self.string
                    .encode_utf16()
                    .map(|x| !x)
                    .collect::<Vec<u16>>()
                    .align_to::<u8>()
                    .1,
            )?;
        }
        if is_wide {
            writer.write_all(&extra_len.to_le_bytes())?;
            writer.write_all(self.extra_string.as_bytes())?;
        }

        Ok(())
    }
}

impl From<String> for CsfString {
    fn from(s: String) -> Self {
        CsfString {
            string: s,
            ..Default::default()
        }
    }
}
