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
    pub const SLPF: Self = Nsid(Cow::Borrowed("slpf"));
    pub const SFPF: Self = Nsid(Cow::Borrowed("sfpf"));
    pub const ER: Self = Nsid(Cow::Borrowed("er"));

    /// Returns a new `Nsid` for a given string.
    ///
    /// # Panics
    /// This function will panic if the provided string is not a valid NSID.
    pub const fn new_const(value: &'static str) -> Self {
        if Nsid::check(value).is_err() {
            panic!("invalid NSID");
        }

        Self(Cow::Borrowed(value))
    }

    const fn check(s: &str) -> Result<(), &'static str> {
        if s.len() > 16 {
            return Err("NSID must be 16 characters or fewer");
        }
        Ok(())
    }
}

impl TryFrom<String> for Nsid {
    type Error = ValidationError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Nsid::check(&value).map_err(ValidationError::new)?;
        Ok(Self(Cow::Owned(value)))
    }
}

impl TryFrom<&'static str> for Nsid {
    type Error = ValidationError;
    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        Nsid::check(&value).map_err(ValidationError::new)?;
        Ok(Self(Cow::Borrowed(value)))
    }
}

impl FromStr for Nsid {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Nsid::check(&s).map_err(ValidationError::new)?;

        // For known profiles, reuse the const to avoid heap allocations
        match s {
            "slpf" => Ok(Self::SLPF),
            "sfpf" => Ok(Self::SFPF),
            "er" => Ok(Self::ER),
            _ => Ok(Self(Cow::Owned(s.to_string()))),
        }
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

impl PartialEq<Cow<'_, Nsid>> for Nsid {
    fn eq(&self, other: &Cow<'_, Nsid>) -> bool {
        self.0 == other.as_ref().0
    }
}

impl PartialEq<Nsid> for Cow<'_, Nsid> {
    fn eq(&self, other: &Nsid) -> bool {
        self.as_ref().0 == other.0
    }
}

impl From<Nsid> for Cow<'_, Nsid> {
    fn from(value: Nsid) -> Self {
        Cow::Owned(value)
    }
}

impl<'a> From<&'a Nsid> for Cow<'a, Nsid> {
    fn from(value: &'a Nsid) -> Self {
        Cow::Borrowed(value)
    }
}
