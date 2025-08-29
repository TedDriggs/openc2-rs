use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::error::ValidationError;

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, SerializeDisplay, DeserializeFromStr,
)]
pub struct IpV4Net {
    pub address: Ipv4Addr,
    pub prefix_len: u8,
}

impl IpV4Net {
    pub fn new(address: Ipv4Addr, prefix_len: u8) -> Result<Self, ValidationError> {
        if prefix_len > 32 {
            return Err(ValidationError::new(
                "ipv4_net",
                "Prefix length must be between 0 and 32",
            ));
        }
        Ok(Self {
            address,
            prefix_len,
        })
    }
}

impl fmt::Debug for IpV4Net {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for IpV4Net {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.prefix_len)
    }
}

impl FromStr for IpV4Net {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr, prefix) = s.split_once('/').ok_or_else(|| {
            ValidationError::new("ipv4_net", "IPv4 network must be in the format 'x.x.x.x/y'")
        })?;
        let address = addr
            .parse()
            .map_err(|e| ValidationError::new("ipv4_net", format!("Invalid IPv4 address: {e}")))?;
        let prefix_len: u8 = prefix
            .parse()
            .map_err(|e| ValidationError::new("ipv4_net", format!("Invalid prefix length: {e}")))?;
        Self::new(address, prefix_len)
    }
}

impl From<Ipv4Addr> for IpV4Net {
    fn from(address: Ipv4Addr) -> Self {
        Self {
            address,
            prefix_len: 32,
        }
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, SerializeDisplay, DeserializeFromStr,
)]
pub struct IpV6Net {
    pub address: Ipv6Addr,
    pub prefix_len: u64,
}

impl IpV6Net {
    pub fn new(address: Ipv6Addr, prefix_len: u64) -> Result<Self, ValidationError> {
        if prefix_len > 128 {
            return Err(ValidationError::new(
                "ipv6_net",
                "Prefix length must be between 0 and 128",
            ));
        }
        Ok(Self {
            address,
            prefix_len,
        })
    }
}

impl fmt::Debug for IpV6Net {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for IpV6Net {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.prefix_len)
    }
}

impl FromStr for IpV6Net {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr, prefix) = s.split_once('/').ok_or_else(|| {
            ValidationError::new(
                "ipv6_net",
                "IPv6 network must be in the format 'x:x:x:x:y:y:y:y/z'",
            )
        })?;
        let address = addr
            .parse()
            .map_err(|e| ValidationError::new("ipv6_net", format!("Invalid IPv6 address: {e}")))?;
        let prefix_len: u64 = prefix
            .parse()
            .map_err(|e| ValidationError::new("ipv6_net", format!("Invalid prefix length: {e}")))?;
        Self::new(address, prefix_len)
    }
}

impl From<Ipv6Addr> for IpV6Net {
    fn from(address: Ipv6Addr) -> Self {
        Self {
            address,
            prefix_len: 128,
        }
    }
}
