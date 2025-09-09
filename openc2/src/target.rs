//! Types for declaring the object of an action.

use from_variants::FromVariants;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay, skip_serializing_none};
use std::{borrow::Cow, fmt, str::FromStr};

use crate::{
    CommandId, Feature, Hashes, Ipv4Net, Ipv6Net, Nsid, Payload, error::ValidationError,
    primitive::Choice,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromVariants)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Target<V> {
    Artifact(Artifact),
    Command(CommandId),
    Device(Device),
    Features(IndexSet<Feature>),
    File(File),
    Ipv4Net(Ipv4Net),
    Ipv6Net(Ipv6Net),
    #[serde(untagged)]
    ProfileDefined(Choice<Cow<'static, Nsid>, Choice<Cow<'static, str>, V>>),
}

impl<V> Target<V> {
    pub fn kind<'a>(&'a self) -> TargetType<'a> {
        self.into()
    }

    pub fn profile_defined(
        profile: impl Into<Cow<'static, Nsid>>,
        type_name: impl Into<Cow<'static, str>>,
        value: V,
    ) -> Self {
        Self::ProfileDefined(Choice::new(
            profile.into(),
            Choice::new(type_name.into(), value),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TargetType<'a> {
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
    ProfileDefined(ProfileTargetType<'a>),
}

impl<'a, V> From<&'a Target<V>> for TargetType<'a> {
    fn from(value: &'a Target<V>) -> Self {
        match value {
            Target::Artifact(_) => TargetType::Artifact,
            Target::Command(_) => TargetType::Command,
            Target::File(_) => TargetType::File,
            Target::Ipv4Net(_) => TargetType::IpV4Net,
            Target::Ipv6Net(_) => TargetType::IpV6Net,
            Target::Device(_) => TargetType::Device,
            Target::Features(_) => TargetType::Features,
            Target::ProfileDefined(ext) => TargetType::ProfileDefined(ProfileTargetType::new(
                ext.key.as_ref(),
                ext.value.key.as_ref(),
            )),
        }
    }
}

/// A target type defined by a profile.
#[derive(Clone, SerializeDisplay, DeserializeFromStr, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileTargetType<'a> {
    pub profile: Cow<'a, Nsid>,
    pub name: Cow<'a, str>,
}

impl<'a> ProfileTargetType<'a> {
    pub fn new(profile: impl Into<Cow<'a, Nsid>>, name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            profile: profile.into(),
            name: name.into(),
        }
    }
}

impl fmt::Debug for ProfileTargetType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for ProfileTargetType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.profile, self.name)
    }
}

impl FromStr for ProfileTargetType<'_> {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (profile, name) = s.split_once('/').ok_or_else(|| {
            ValidationError::new("Profile target must be in the format 'profile/name'")
        })?;
        Ok(Self::new(
            Nsid::try_from(profile.to_string()).map_err(|e| e.at("profile"))?,
            name.to_string(),
        ))
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

        assert_eq!(example, Target::Ipv4Net("1.2.3.4/32".parse().unwrap()));
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

        assert!(matches!(example, Target::ProfileDefined(Choice { .. })));
    }
}
