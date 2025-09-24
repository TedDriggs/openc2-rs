use openc2::{Nsid, Profile, Value, target::ProfileTargetType};
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

impl<V> TryFrom<openc2::Target<V>> for Target
where
    V: openc2::Value,
    openc2::Error: From<V::Error>,
{
    type Error = openc2::Error;

    fn try_from(value: openc2::Target<V>) -> Result<Self, Self::Error> {
        match value {
            openc2::Target::ProfileDefined(d) if &d.key == NS => Ok(d.value.get()?),
            _ => Err(openc2::Error::custom(
                "target is not defined by the ER-profile",
            )),
        }
    }
}

/// Convert to a generic OpenC2 target.
///
/// # Panics
/// This panics if serialization to the value type fails. This should not happen.
impl<V: Value> From<Target> for openc2::Target<V> {
    fn from(value: Target) -> Self {
        openc2::Target::profile_defined(
            NS,
            value.kind().as_str(),
            match value {
                Target::Account(a) => V::from_typed(&a).unwrap(),
                Target::Service(s) => V::from_typed(&s).unwrap(),
                Target::RegistryEntry(r) => V::from_typed(&r).unwrap(),
            },
        )
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

impl TargetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetType::Account => "account",
            TargetType::Service => "service",
            TargetType::RegistryEntry => "registry_entry",
        }
    }
}

impl Profile for TargetType {
    fn ns() -> &'static Nsid {
        NS
    }
}

impl From<TargetType> for ProfileTargetType<'static> {
    fn from(value: TargetType) -> Self {
        ProfileTargetType::new(NS, value.as_str())
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    /// The unique identifier of the account.
    pub uid: Option<String>,
    /// The chosen display name of the account.
    pub account_name: Option<String>,
    /// The path to the account's home directory.
    pub directory: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Service {
    pub name: Option<String>,
    pub display_name: Option<String>,
}
