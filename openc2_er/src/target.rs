use openc2::{Nsid, Profile, target::ProfileTargetType};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::NS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Account(Account),
    Service(Service),
    RegistryEntry(RegistryEntry),
}

impl Target {
    pub fn kind(&self) -> TargetType {
        match self {
            Target::Account(_) => TargetType::Account,
            Target::Service(_) => TargetType::Service,
            Target::RegistryEntry(_) => TargetType::RegistryEntry,
        }
    }
}

impl TryFrom<openc2::json::Target> for Target {
    type Error = openc2::Error;

    fn try_from(value: openc2::json::Target) -> Result<Self, Self::Error> {
        match value {
            openc2::Target::Extension(d) if &d.key == NS => Ok(d.value.get()?),
            _ => Err(openc2::Error::custom(
                "target is not defined by the ER-profile",
            )),
        }
    }
}

impl Profile for Target {
    fn ns() -> &'static openc2::Nsid {
        NS
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    strum::EnumString,
    strum::Display,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    Account,
    Service,
    RegistryEntry,
}

impl Profile for TargetType {
    fn ns() -> &'static Nsid {
        NS
    }
}

impl From<TargetType> for ProfileTargetType {
    fn from(value: TargetType) -> Self {
        ProfileTargetType {
            profile: NS.clone(),
            name: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryEntry {
    /// Specifies the full registry key including the hive.
    pub key: String,
    /// The registry value type as defined in the [Winnt.h header].
    ///
    /// [Winnt.h header]: https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    /// The unique identifier of the account.
    pub uid: Option<String>,
    /// The chosen display name of the account.
    pub account_name: Option<String>,
    /// The path to the account's home directory.
    pub directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Service {
    pub name: Option<String>,
    pub display_name: Option<String>,
}
