use std::{error::Error, fmt::Display, ops::Deref};

use regex::Regex;
use serde::{Deserialize, Deserializer};

lazy_static::lazy_static! {
    static ref NAME_REGEX: Regex = Regex::new("^[a-zA-Z0-9+_.\\-]+$").expect("could not compile name enforcement regex!");
}

/// Represents a valid resource name for LVM2
///
/// A valid resource name is represented by the following pattern:
/// ^[a-zA-Z0-9+_.\\-]+$
#[derive(Clone, Debug)]
pub struct ResourceName(String);

impl ResourceName {
    pub fn empty() -> Self {
        ResourceName(String::new())
    }
}

impl Display for ResourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ResourceName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&str> for ResourceName {
    type Error = InvalidResourceNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if !NAME_REGEX.is_match(value.as_ref()) {
            return Err(InvalidResourceNameError(value.into()));
        }

        Ok(Self(value.into()))
    }
}

impl TryFrom<String> for ResourceName {
    type Error = InvalidResourceNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !NAME_REGEX.is_match(value.as_ref()) {
            return Err(InvalidResourceNameError(value));
        }

        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for ResourceName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ResourceName::try_from(s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct InvalidResourceNameError(String);
impl Error for InvalidResourceNameError {}

impl Display for InvalidResourceNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "name must be valid for LVM2 ([a-zA-Z0-9_.+-]): {}",
            self.0
        )
    }
}
