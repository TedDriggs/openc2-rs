//! Types for declaring the object of an action.

use from_variants::FromVariants;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay, skip_serializing_none};
use std::{borrow::Cow, fmt, str::FromStr};
pub use url::Url;

use crate::{
    CommandId, DomainName, EmailAddr, Feature, Hashes, Ipv4Net, Ipv6Net, IsEmpty, MacAddr, Nsid,
    Payload, Port, error::ValidationError, primitive::Choice,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromVariants)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Target<V> {
    Artifact(Artifact),
    Command(CommandId),
    Device(Device),
    DomainName(DomainName),
    EmailAddr(EmailAddr),
    Features(Features),
    File(File),
    Ipv4Net(Ipv4Net),
    Ipv6Net(Ipv6Net),
    Ipv4Connection(Ipv4Connection),
    Ipv6Connection(Ipv6Connection),
    MacAddr(MacAddr),
    Process(Process),
    Uri(Url),
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

impl<V> From<Vec<Feature>> for Target<V> {
    fn from(value: Vec<Feature>) -> Self {
        Self::Features(value.into_iter().collect())
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, strum::Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TargetType<'a> {
    Artifact,
    Command,
    File,
    Device,
    DomainName,
    EmailAddr,
    Features,
    Ipv4Net,
    Ipv6Net,
    Ipv4Connection,
    Ipv6Connection,
    MacAddr,
    Process,
    Uri,
    #[serde(untagged)]
    #[strum(to_string = "{0}")]
    ProfileDefined(ProfileTargetType<'a>),
}

impl<'a, V> From<&'a Target<V>> for TargetType<'a> {
    fn from(value: &'a Target<V>) -> Self {
        match value {
            Target::Artifact(_) => TargetType::Artifact,
            Target::Command(_) => TargetType::Command,
            Target::File(_) => TargetType::File,
            Target::Ipv4Net(_) => TargetType::Ipv4Net,
            Target::Ipv6Net(_) => TargetType::Ipv6Net,
            Target::Device(_) => TargetType::Device,
            Target::DomainName(_) => TargetType::DomainName,
            Target::EmailAddr(_) => TargetType::EmailAddr,
            Target::Features(_) => TargetType::Features,
            Target::Ipv4Connection(_) => TargetType::Ipv4Connection,
            Target::Ipv6Connection(_) => TargetType::Ipv6Connection,
            Target::MacAddr(_) => TargetType::MacAddr,
            Target::Process(_) => TargetType::Process,
            Target::Uri(_) => TargetType::Uri,
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

impl IsEmpty for File {
    fn is_empty(&self) -> bool {
        self.name.is_none() && self.hashes.is_empty() && self.path.is_none()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Device {
    pub hostname: Option<String>,
    pub idn_hostname: Option<String>,
    pub device_id: Option<String>,
}

impl Device {
    pub fn with_device_id(value: impl Into<String>) -> Self {
        Self {
            hostname: None,
            idn_hostname: None,
            device_id: Some(value.into()),
        }
    }

    pub fn with_hostname(value: impl Into<String>) -> Self {
        Self {
            hostname: Some(value.into()),
            idn_hostname: None,
            device_id: None,
        }
    }
}

/// The set of features queried in a `query` action.
pub type Features = IndexSet<Feature>;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Ipv4Connection {
    pub src_addr: Option<Ipv4Net>,
    pub src_port: Option<Port>,
    pub dst_addr: Option<Ipv4Net>,
    pub dst_port: Option<Port>,
    pub protocol: Option<L4Protocol>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Ipv6Connection {
    pub src_addr: Option<Ipv6Net>,
    pub src_port: Option<Port>,
    pub dst_addr: Option<Ipv6Net>,
    pub dst_port: Option<Port>,
    pub protocol: Option<L4Protocol>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum L4Protocol {
    Tcp,
    Udp,
    Icmp,
    Other(u8),
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Process {
    pub pid: Option<u32>,
    pub name: Option<String>,
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub executable: Option<File>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub parent: Option<Box<Process>>,
    pub command_line: Option<String>,
}

impl IsEmpty for Process {
    fn is_empty(&self) -> bool {
        self.pid.is_none()
            && self.name.is_none()
            && self.cwd.is_none()
            && self.executable.is_empty()
            && self.parent.is_empty()
            && self.command_line.is_none()
    }
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use crate::{Nsid, TargetType, primitive::Choice, target::ProfileTargetType};

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

    #[test]
    fn target_type_display() {
        assert_eq!(TargetType::Ipv4Net.to_string(), "ipv4_net");
    }

    #[test]
    fn target_type_display_extended() {
        assert_eq!(
            TargetType::ProfileDefined(ProfileTargetType::new(Nsid::ER, "account")).to_string(),
            "er/account"
        );
    }
}
