use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    borrow::{Borrow, Cow},
    fmt,
    str::FromStr,
};

use crate::error::ValidationError;

#[derive(
    Debug, Clone, SerializeDisplay, DeserializeFromStr, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct Nsid(Cow<'static, str>);

impl Nsid {
    pub const fn new_static(text: &'static str) -> Self {
        if text.len() > 16 {
            panic!("NSID must be 16 characters or fewer");
        }
        Self(Cow::Borrowed(text))
    }
}

impl TryFrom<String> for Nsid {
    type Error = ValidationError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > 16 {
            return Err(ValidationError::new(
                "nsid",
                "NSID must be 16 characters or fewer",
            ));
        }
        Ok(Self(Cow::Owned(value)))
    }
}

impl TryFrom<&'static str> for Nsid {
    type Error = ValidationError;
    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        if value.len() > 16 {
            return Err(ValidationError::new(
                "nsid",
                "NSID must be 16 characters or fewer",
            ));
        }
        Ok(Self(Cow::Borrowed(value)))
    }
}

impl FromStr for Nsid {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 16 {
            return Err(ValidationError::new(
                "nsid",
                "NSID must be 16 characters or fewer",
            ));
        }
        Ok(Self(Cow::Owned(s.to_string())))
    }
}

impl fmt::Display for Nsid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Borrow<str> for Nsid {
    fn borrow(&self) -> &str {
        &self.0
    }
}
