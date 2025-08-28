//! Types for declaring entities that will execute actions on targets.

use std::borrow::Cow;

use serde::{Deserialize, Serialize, Serializer, de::DeserializeOwned, ser::SerializeMap};

/// Information about the entity that will execute the action on the target.
#[derive(Debug, Clone, PartialEq, Hash)]
#[non_exhaustive]
pub struct Profile<V>(Cow<'static, str>, V);

impl<V> Profile<V> {
    pub fn new(ns: impl Into<Cow<'static, str>>, value: V) -> Self {
        Profile(ns.into(), value)
    }

    pub fn ns(&self) -> &str {
        todo!()
    }
}

#[cfg(feature = "json")]
impl Profile<serde_json::Value> {
    pub fn get<U: DeserializeOwned>(&self, ns: &str) -> Option<serde_json::Result<U>> {
        if self.0 == ns {
            Some(serde_json::from_value(self.1.clone()))
        } else {
            None
        }
    }
}

#[cfg(feature = "cbor")]
impl Profile<serde_cbor::Value> {
    pub fn get<U: DeserializeOwned>(&self, ns: &str) -> Option<serde_cbor::Result<U>> {
        if self.0 == ns {
            Some(serde_cbor::value::from_value(self.1.clone()))
        } else {
            None
        }
    }
}

impl<V: Serialize> Serialize for Profile<V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.0, &self.1)?;
        map.end()
    }
}

impl<'de, V: Deserialize<'de>> Deserialize<'de> for Profile<V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ActuatorVisitor<V>(std::marker::PhantomData<V>);

        impl<'de, V: Deserialize<'de>> serde::de::Visitor<'de> for ActuatorVisitor<V> {
            type Value = Profile<V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map with a single key-value pair")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                if let Some((key, value)) = access.next_entry()? {
                    if access.next_entry::<String, V>()?.is_some() {
                        return Err(serde::de::Error::custom("expected a single key-value pair"));
                    }
                    Ok(Profile(key, value))
                } else {
                    Err(serde::de::Error::custom("expected a single key-value pair"))
                }
            }
        }

        deserializer.deserialize_map(ActuatorVisitor(std::marker::PhantomData))
    }
}
