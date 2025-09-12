use std::borrow::Borrow;

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize, de::Error as _};
use serde_with::skip_serializing_none;
use url::Url;

use crate::{Action, TargetType};

mod ipnet;
mod nsid;
pub mod primitive;
mod value;
mod version;

pub use ipnet::{Ipv4Net, Ipv6Net};
pub use nsid::Nsid;
pub use value::Value;
pub use version::Version;

pub type ActionTargets = IndexMap<Action, IndexSet<TargetType<'static>>>;

pub type CommandId = String;

/// Epoch milliseconds
pub type DateTime = u64;

pub type DomainName = String;

pub type Duration = u64;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Extensions<V>(IndexMap<Nsid, V>);

impl<V> Extensions<V> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, key: &impl Borrow<str>) -> bool {
        self.0.contains_key(key.borrow())
    }

    pub fn get_raw(&self, key: &impl Borrow<str>) -> Option<&V> {
        self.0.get(key.borrow())
    }
}

impl<V: Value + Clone> Extensions<V> {
    /// Gets an extension's value by key, or returns `None` if the key doesn't exist.
    pub fn get<'de, T: Deserialize<'de>>(
        &'de self,
        key: &impl Borrow<str>,
    ) -> Option<Result<T, V::Error>> {
        self.get_raw(key).map(|v| v.to_typed())
    }

    /// Get's an extension's value by key, returning an error if the key doesn't exist or
    /// doesn't deserialize into the provided type.
    pub fn require<'de, T: Deserialize<'de>>(
        &'de self,
        key: &impl Borrow<str>,
    ) -> Result<T, V::Error> {
        self.get::<T>(key)
            .transpose()?
            .ok_or_else(|| V::Error::custom(format!("extension {} is required", key.borrow())))
    }
}

impl<V> Default for Extensions<V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'de, V: Deserialize<'de>> Deserialize<'de> for Extensions<V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = IndexMap::<Nsid, V>::deserialize(deserializer)?;
        Ok(Self(map))
    }
}

impl<V> IntoIterator for Extensions<V> {
    type Item = (Nsid, V);
    type IntoIter = indexmap::map::IntoIter<Nsid, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, V> IntoIterator for &'a Extensions<V> {
    type Item = (&'a Nsid, &'a V);
    type IntoIter = indexmap::map::Iter<'a, Nsid, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<V> FromIterator<(Nsid, V)> for Extensions<V> {
    fn from_iter<T: IntoIterator<Item = (Nsid, V)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Feature {
    Versions,
    Profiles,
    Pairs,
    RateLimit,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Hashes {
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    #[serde(rename = "bin")]
    Binary(Vec<u8>),
    Url(Url),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    None,
    Ack,
    Status,
    Complete,
}

impl ResponseType {
    pub fn requires_request_id(self) -> bool {
        !matches!(self, ResponseType::None)
    }
}
