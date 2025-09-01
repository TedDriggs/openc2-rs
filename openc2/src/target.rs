//! Types for declaring the object of an action.

use from_variants::FromVariants;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay, skip_serializing_none};
use std::{fmt, str::FromStr};

use crate::{
    CommandId, Error, Feature, Hashes, IpV4Net, IpV6Net, Nsid, Payload, error::ValidationError,
    primitive::Choice,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromVariants)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Target<V> {
    Artifact(Artifact),
    Command(CommandId),
    File(File),
    #[serde(rename = "ipv4_net")]
    IpV4Net(IpV4Net),
    #[serde(rename = "ipv6_net")]
    IpV6Net(IpV6Net),
    Device(Device),
    Features(IndexSet<Feature>),
    #[serde(untagged)]
    Extension(Choice<Nsid, Choice<String, V>>),
}

impl<V> Target<V> {
    pub fn kind(&self) -> TargetType {
        self.into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    Artifact,
    Command,
    File,
    #[serde(rename = "ipv4_net")]
    IpV4Net,
    #[serde(rename = "ipv6_net")]
    IpV6Net,
    Device,
    Features,
    #[serde(untagged)]
    Extension(ProfileTargetType),
}

impl<V> From<&Target<V>> for TargetType {
    fn from(value: &Target<V>) -> Self {
        match value {
            Target::Artifact(_) => TargetType::Artifact,
            Target::Command(_) => TargetType::Command,
            Target::File(_) => TargetType::File,
            Target::IpV4Net(_) => TargetType::IpV4Net,
            Target::IpV6Net(_) => TargetType::IpV6Net,
            Target::Device(_) => TargetType::Device,
            Target::Features(_) => TargetType::Features,
            Target::Extension(ext) => TargetType::Extension(ProfileTargetType {
                profile: ext.key.clone(),
                name: ext.value.key.clone(),
            }),
        }
    }
}

/// A target type defined by a profile.
#[derive(Clone, SerializeDisplay, DeserializeFromStr, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileTargetType {
    pub profile: Nsid,
    pub name: String,
}

impl fmt::Debug for ProfileTargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for ProfileTargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.profile, self.name)
    }
}

impl FromStr for ProfileTargetType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (profile, name) = s.split_once('/').ok_or_else(|| {
            ValidationError::new(
                "profile target",
                "Profile target must be in the format 'profile/name'",
            )
        })?;
        Ok(Self {
            profile: Nsid::try_from(profile.to_string())?,
            name: name.to_string(),
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Artifact {
    pub media_type: Option<String>,
    pub payload: Option<Payload>,
    pub hashes: Option<Hashes>,
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

#[cfg(all(test, feature = "json"))]
mod tests {
    use crate::primitive::Choice;

    use super::Target;
    use serde_json::{Value, from_value, json};

    #[test]
    fn ip_target() {
        let example: Target<Value> = from_value(json!(
            {
                "ipv4_net": "1.2.3.4/32"
            }
        ))
        .unwrap();

        assert_eq!(example, Target::IpV4Net("1.2.3.4/32".parse().unwrap()));
    }

    #[test]
    fn extension_target() {
        let example: Target<Value> = from_value(json!(
            {
                "slpf": {
                    "rule_number": 31
                }
            }
        ))
        .unwrap();

        assert!(matches!(example, Target::Extension(Choice { .. })));
    }
}
