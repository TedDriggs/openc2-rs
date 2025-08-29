#[cfg(feature = "json")]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeMap};

/// A map containing a single key-value pair.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Choice<K, V> {
    pub key: K,
    pub value: V,
}

impl<K, V> Choice<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

#[cfg(feature = "json")]
impl<K> Choice<K, serde_json::Value> {
    pub fn get<V: DeserializeOwned>(&self) -> serde_json::Result<V> {
        serde_json::from_value(self.value.clone())
    }
}

#[cfg(feature = "cbor")]
impl<K> Choice<K, serde_cbor::Value> {
    pub fn get<V: DeserializeOwned>(&self) -> serde_cbor::Result<V> {
        serde_cbor::value::from_value(self.value.clone())
    }
}

impl<K: Serialize, V: Serialize> Serialize for Choice<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.key, &self.value)?;
        map.end()
    }
}

impl<'de, K: Deserialize<'de>, V: Deserialize<'de>> Deserialize<'de> for Choice<K, V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ChoiceVisitor<K, V>(std::marker::PhantomData<(K, V)>);

        impl<'de, K: Deserialize<'de>, V: Deserialize<'de>> serde::de::Visitor<'de>
            for ChoiceVisitor<K, V>
        {
            type Value = Choice<K, V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map with a single key-value pair")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                if let Some((key, value)) = access.next_entry()? {
                    if access.next_key::<K>()?.is_some() {
                        return Err(serde::de::Error::custom("expected a single key-value pair"));
                    }
                    Ok(Choice { key, value })
                } else {
                    Err(serde::de::Error::custom("expected a single key-value pair"))
                }
            }
        }

        deserializer.deserialize_map(ChoiceVisitor(std::marker::PhantomData))
    }
}
