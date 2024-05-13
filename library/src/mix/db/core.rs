//! MIX database structures and manipulation.

use std::collections::HashMap;

use crate::{core::GameEnum, utils::BuildNothingHasher};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Unknown LMD version found in the LMD: {0}")]
    UnknownLMDVersion(u32),
}

type Result<T> = std::result::Result<T, Error>;

/// LMD format version (XCC addition, not in the vanilla game). Doesn't seem to do anything.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum LMDVersionEnum {
    TD = 0,
    RA = 1,
    TS = 2,
    RA2 = 5,
    #[default]
    YR = 6,
}

impl TryFrom<u32> for LMDVersionEnum {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            x if x == LMDVersionEnum::TD as u32 => Ok(LMDVersionEnum::TD),
            x if x == LMDVersionEnum::RA as u32 => Ok(LMDVersionEnum::RA),
            x if x == LMDVersionEnum::TS as u32 => Ok(LMDVersionEnum::TS),
            x if x == LMDVersionEnum::RA2 as u32 => Ok(LMDVersionEnum::RA2),
            x if x == LMDVersionEnum::YR as u32 => Ok(LMDVersionEnum::YR),
            x => Err(Error::UnknownLMDVersion(x)),
        }
    }
}

impl TryFrom<LMDVersionEnum> for u32 {
    type Error = Error;

    fn try_from(value: LMDVersionEnum) -> Result<Self> {
        Ok(value as u32)
    }
}

impl From<LMDVersionEnum> for GameEnum {
    fn from(value: LMDVersionEnum) -> Self {
        match value {
            LMDVersionEnum::TD => GameEnum::TD,
            LMDVersionEnum::RA => GameEnum::RA,
            LMDVersionEnum::TS => GameEnum::TS,
            LMDVersionEnum::RA2 => GameEnum::RA2,
            LMDVersionEnum::YR => GameEnum::YR,
        }
    }
}

impl From<GameEnum> for LMDVersionEnum {
    fn from(value: GameEnum) -> Self {
        match value {
            GameEnum::TD => LMDVersionEnum::TD,
            GameEnum::RA => LMDVersionEnum::RA,
            GameEnum::TS => LMDVersionEnum::TS,
            GameEnum::FS => LMDVersionEnum::TS,
            GameEnum::RA2 => LMDVersionEnum::RA2,
            GameEnum::YR => LMDVersionEnum::YR,
        }
    }
}

impl std::fmt::Display for LMDVersionEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

/// A MIX database is a file mapping unique file IDs into their original names.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MixDatabase {
    pub names: HashMap<i32, String, BuildNothingHasher>,
}

/// A local MIX database is a file within a MIX. XCC addition.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LocalMixDatabase {
    pub db: MixDatabase,
    pub version: LMDVersionEnum,
}

/// A global MIX database is a separate file containing several databases. XCC addition.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GlobalMixDatabase {
    pub dbs: Vec<MixDatabase>,
}

impl GlobalMixDatabase {
    pub fn get_name(&self, id: i32) -> Option<&String> {
        self.dbs.iter().find_map(|x| x.names.get(&id))
    }

    pub fn get_name_mut(&mut self, id: i32) -> Option<&mut String> {
        self.dbs.iter_mut().find_map(|x| x.names.get_mut(&id))
    }

    pub fn get_name_or_id(&self, id: i32) -> String {
        self.dbs
            .iter()
            .find_map(|x| x.names.get(&id))
            .map_or_else(|| format!("{:0>8X}", id), |x| x.to_string())
    }
}
