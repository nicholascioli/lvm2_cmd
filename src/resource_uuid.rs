use std::{error::Error, fmt::Display, ops::Deref};

use regex::Regex;
use serde::{Deserialize, Deserializer};

lazy_static::lazy_static! {
    static ref UUID_REGEX: Regex = Regex::new("^[a-zA-Z0-9]{6}-([a-zA-Z0-9]{4}-){5}[a-zA-Z0-9]{6}$").expect("could not compile name enforcement regex!");
}

/// Represents a valid resource UUID for LVM2
///
/// A valid resource name is represented by the following pattern:
/// ^[a-zA-Z0-9]{6}-([a-zA-Z0-9]{4}-){5}[a-zA-Z0-9]{6}$
#[derive(Clone, Debug)]
pub struct ResourceUUID(String);

impl Display for ResourceUUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ResourceUUID {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&str> for ResourceUUID {
    type Error = InvalidResourceUUIDError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if !UUID_REGEX.is_match(value.as_ref()) {
            return Err(InvalidResourceUUIDError(value.into()));
        }

        Ok(Self(value.into()))
    }
}

impl TryFrom<String> for ResourceUUID {
    type Error = InvalidResourceUUIDError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !UUID_REGEX.is_match(value.as_ref()) {
            return Err(InvalidResourceUUIDError(value));
        }

        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for ResourceUUID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ResourceUUID::try_from(s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct InvalidResourceUUIDError(String);
impl Error for InvalidResourceUUIDError {}

impl Display for InvalidResourceUUIDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UUID must be valid for LVM2 (^[a-zA-Z0-9]{{6}}-([a-zA-Z0-9]{{4}}-){{5}}[a-zA-Z0-9]{{6}}$): {}",
            self.0
        )
    }
}
