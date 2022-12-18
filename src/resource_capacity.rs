use std::{error::Error, fmt::Display, ops::Deref};

use serde::{Deserialize, Deserializer};
use serde_aux::prelude::deserialize_number_from_string;

/// Represents a valid resource capacity for LVM2
///
/// A valid resource capacity must be a multiple of 512
#[derive(Clone, Debug)]
pub struct ResourceCapacity(usize);

impl ResourceCapacity {
    /// Create a [ResourceCapacity], rounding up to the nearest valid multiple
    pub fn from_nearest(capacity: usize) -> Self {
        ResourceCapacity(nearest_size_multiple(capacity))
    }
}

impl Display for ResourceCapacity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for ResourceCapacity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<usize> for ResourceCapacity {
    type Error = InvalidResourceCapacityError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value != nearest_size_multiple(value) {
            return Err(InvalidResourceCapacityError(value));
        }

        Ok(Self(value))
    }
}

impl TryFrom<u64> for ResourceCapacity {
    type Error = InvalidResourceCapacityError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let as_usize = value as usize;
        if as_usize != nearest_size_multiple(as_usize) {
            return Err(InvalidResourceCapacityError(as_usize));
        }

        Ok(Self(as_usize))
    }
}

impl<'de> Deserialize<'de> for ResourceCapacity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let cap: usize = deserialize_number_from_string(deserializer)?;
        ResourceCapacity::try_from(cap).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct InvalidResourceCapacityError(usize);
impl Error for InvalidResourceCapacityError {}

impl Display for InvalidResourceCapacityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "capacity must be valid for LVM2 (multiple of 512): {} != {} (nearest 512 multiple)",
            self.0,
            nearest_size_multiple(self.0),
        )
    }
}

/// Finds the next nearest valid size multiple for an LVM construct
fn nearest_size_multiple(x: usize) -> usize {
    ((x - 1) | 511) + 1
}
