use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTargets {}

pub type CommandId = String;

pub type DateTime = ();

pub type Duration = u64;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Extensions<V>(IndexMap<String, V>);

impl<V> Extensions<V> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn get_raw(&self, key: &str) -> Option<&V> {
        self.0.get(key)
    }
}

#[cfg(feature = "json")]
impl Extensions<serde_json::Value> {
    pub fn get<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Option<Result<T, serde_json::Error>> {
        self.0.get(key).map(|v| serde_json::from_value(v.clone()))
    }
}

#[cfg(feature = "cbor")]
impl Extensions<serde_cbor::Value> {
    pub fn get<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Option<Result<T, serde_cbor::Error>> {
        self.0
            .get(key)
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
        let map = IndexMap::<String, V>::deserialize(deserializer)?;
        Ok(Self(map))
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Hashes {
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Command,
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    None,
    Ack,
    Status,
    Complete,
}
