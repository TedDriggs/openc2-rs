//! Types for declaring the object of an action.

use from_variants::FromVariants;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::net::IpAddr;

use crate::Hashes;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, FromVariants)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Target {
    File(File),
    IpAddress(IpAddr),
    Device(Device),
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct File {
    pub name: Option<String>,
    pub hashes: Option<Hashes>,
    pub path: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Device {
    pub hostname: Option<String>,
    pub idn_hostname: Option<String>,
    pub device_id: Option<String>,
}

impl Device {
    pub fn with_hostname(value: impl Into<String>) -> Self {
        Self {
            hostname: Some(value.into()),
            idn_hostname: None,
            device_id: None,
        }
    }
}
