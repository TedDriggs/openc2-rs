use std::fmt::Debug;

use serde::{Serialize, de::DeserializeOwned};

/// An abstraction over different value types, such as JSON or CBOR.
pub trait Value: Sized {
    /// The error type returned when converting to or from the value type.
    type Error: Debug;

    /// Serialize a value to the value type.
    fn from_typed<V: Serialize>(value: &V) -> Result<Self, Self::Error>;

    /// Deserialize a value from the value type.
    fn to_typed<T: DeserializeOwned>(self) -> Result<T, Self::Error>;
}

#[cfg(feature = "json")]
impl Value for serde_json::Value {
    type Error = serde_json::Error;

    fn from_typed<V: Serialize>(value: &V) -> Result<Self, Self::Error> {
        serde_json::to_value(value)
    }

    fn to_typed<T: DeserializeOwned>(self) -> Result<T, Self::Error> {
        serde_json::from_value(self)
    }
}

#[cfg(feature = "cbor")]
impl Value for serde_cbor::Value {
    type Error = serde_cbor::Error;

    fn from_typed<V: Serialize>(value: &V) -> Result<Self, Self::Error> {
        serde_cbor::value::to_value(value)
    }

    fn to_typed<T: DeserializeOwned>(self) -> Result<T, Self::Error> {
        serde_cbor::value::from_value(self)
    }
}
