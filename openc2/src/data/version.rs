use std::{fmt, str::FromStr};

use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::error::ValidationError;

/// OpenC2 version in "Major.Minor" format
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SerializeDisplay, DeserializeFromStr,
)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for Version {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (major_str, minor_str) = s
            .split_once('.')
            .ok_or_else(|| ValidationError::new("invalid version format"))?;
        let major = major_str
            .parse()
            .map_err(|e| ValidationError::new(format!("invalid version: {e}")).at("major"))?;
        let minor = minor_str
            .parse()
            .map_err(|e| ValidationError::new(format!("invalid version: {e}")).at("minor"))?;
        Ok(Version { major, minor })
    }
}
