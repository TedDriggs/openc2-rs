//! PF-specific targets for OpenC2

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use from_variants::FromVariants;
use openc2::{
    Ipv4Net, Ipv6Net, Nsid, Port, Profile, Value,
    target::{L4Protocol, ProfileTargetType},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::NS;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    RuleNumber(RuleId),
    AdvConnection(AdvancedConnection),
}

impl Target {
    pub fn kind(&self) -> TargetType {
        match self {
            Target::RuleNumber(_) => TargetType::RuleNumber,
            Target::AdvConnection(_) => TargetType::AdvConnection,
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
                "target is not defined by the PF-profile",
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
                Target::RuleNumber(a) => V::from_typed(&a).unwrap(),
                Target::AdvConnection(s) => V::from_typed(&s).unwrap(),
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
    RuleNumber,
    AdvConnection,
}

impl TargetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetType::RuleNumber => "rule_number",
            TargetType::AdvConnection => "adv_connection",
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

/// Immutable identifier assigned when a packet filtering rule is created.
pub type RuleId = u32;

/// Advanced connection type to support application layer firewalls.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdvancedConnection {
    pub src_addr: AdvAddr,
    pub src_port: Option<Port>,
    pub dst_addr: AdvAddr,
    pub dst_port: Option<Port>,
    pub protocol: Option<L4Protocol>,
    pub network: Option<String>,
    pub application: Option<String>,
}

/// Address type for advanced connections.
#[derive(
    Debug, Clone, PartialEq, Deserialize, Serialize, Eq, PartialOrd, Ord, Hash, FromVariants,
)]
pub enum AdvAddr {
    /// CIDR notation
    #[serde(rename = "v4addr")]
    V4Addr(Ipv4Net),
    /// CIDR notation
    #[serde(rename = "v6addr")]
    V6Addr(Ipv6Net),
    /// Network tag
    #[serde(rename = "net_tag")]
    #[from_variants(skip)]
    NetTag(String),
}

impl From<IpAddr> for AdvAddr {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(v4) => AdvAddr::V4Addr(Ipv4Net::from(v4)),
            IpAddr::V6(v6) => AdvAddr::V6Addr(Ipv6Net::from(v6)),
        }
    }
}

impl From<Ipv4Addr> for AdvAddr {
    fn from(value: Ipv4Addr) -> Self {
        AdvAddr::V4Addr(Ipv4Net::from(value))
    }
}

impl From<Ipv6Addr> for AdvAddr {
    fn from(value: Ipv6Addr) -> Self {
        AdvAddr::V6Addr(Ipv6Net::from(value))
    }
}

impl PartialEq<Ipv4Net> for AdvAddr {
    fn eq(&self, other: &Ipv4Net) -> bool {
        matches!(self, AdvAddr::V4Addr(v4) if v4 == other)
    }
}

impl PartialEq<Ipv6Net> for AdvAddr {
    fn eq(&self, other: &Ipv6Net) -> bool {
        matches!(self, AdvAddr::V6Addr(v6) if v6 == other)
    }
}
