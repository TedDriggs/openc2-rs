//! Types for declaring the object of an action.

use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, FromVariants)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    File(File),
    IpAddress(IpAddr),
    Device(Device),
    #[doc(hidden)]
    NonExhaustive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct File {
    pub name: String,
    pub hashes: (),
    pub device: Device,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Device {
    pub hostname: String,
}

impl Device {
    pub fn with_hostname<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            hostname: value.into(),
        }
    }
}
