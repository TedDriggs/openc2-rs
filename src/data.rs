use std::borrow::Borrow;

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use crate::{Action, TargetType};

mod ipnet;
mod nsid;
pub mod primitive;
mod version;

pub use ipnet::{IpV4Net, IpV6Net};
pub use nsid::Nsid;
pub use version::Version;

pub type ActionTargets = IndexMap<Action, IndexSet<TargetType>>;

pub type CommandId = String;

/// Epoch milliseconds
pub type DateTime = u64;

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

#[cfg(feature = "json")]
impl Extensions<serde_json::Value> {
    pub fn get<T: serde::de::DeserializeOwned>(
        &self,
        key: &impl Borrow<str>,
    ) -> Option<Result<T, serde_json::Error>> {
        self.get_raw(key).map(|v| serde_json::from_value(v.clone()))
    }
}

#[cfg(feature = "cbor")]
impl Extensions<serde_cbor::Value> {
    pub fn get<T: serde::de::DeserializeOwned>(
        &self,
        key: &impl Borrow<str>,
    ) -> Option<Result<T, serde_cbor::Error>> {
        self.get_raw(key)
            .map(|v| serde_cbor::value::from_value(v.clone()))
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
