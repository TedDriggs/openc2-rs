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
    address: Ipv4Addr,
    prefix_len: Option<u8>,
}

impl IpV4Net {
    pub fn new(address: Ipv4Addr, prefix_len: Option<u8>) -> Result<Self, ValidationError> {
        if let Some(pf) = prefix_len
            && pf > 32
        {
            return Err(ValidationError::new(
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
        if let Some(prefix_len) = self.prefix_len {
            write!(f, "{}/{prefix_len}", self.address)
        } else {
            self.address.fmt(f)
        }
    }
}

impl FromStr for IpV4Net {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr, prefix) = s
            .split_once('/')
            .map(|(s, t)| (s, Some(t)))
            .unwrap_or_else(|| (s, None));
        let address = addr
            .parse()
            .map_err(|e| ValidationError::new(format!("Invalid IPv4 address: {e}")))?;
        let prefix_len: Option<u8> = prefix
            .map(|prefix| {
                prefix
                    .parse()
                    .map_err(|e| ValidationError::new(format!("Invalid prefix length: {e}")))
            })
            .transpose()?;
        Self::new(address, prefix_len)
    }
}

impl From<Ipv4Addr> for IpV4Net {
    fn from(address: Ipv4Addr) -> Self {
        Self {
            address,
            prefix_len: None,
        }
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, SerializeDisplay, DeserializeFromStr,
)]
pub struct IpV6Net {
    address: Ipv6Addr,
    prefix_len: Option<u8>,
}

impl IpV6Net {
    pub fn new(address: Ipv6Addr, prefix_len: Option<u8>) -> Result<Self, ValidationError> {
        if let Some(prefix_len) = prefix_len
            && prefix_len > 128
        {
            return Err(ValidationError::new(
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
        if let Some(prefix_len) = self.prefix_len {
            write!(f, "{}/{}", self.address, prefix_len)
        } else {
            self.address.fmt(f)
        }
    }
}

impl FromStr for IpV6Net {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr, prefix) = s
            .split_once('/')
            .map(|(s, t)| (s, Some(t)))
            .unwrap_or_else(|| (s, None));
        let address = addr
            .parse()
            .map_err(|e| ValidationError::new(format!("Invalid IPv6 address: {e}")))?;
        let prefix_len: Option<u8> = prefix
            .map(|prefix| {
                prefix
                    .parse()
                    .map_err(|e| ValidationError::new(format!("Invalid prefix length: {e}")))
            })
            .transpose()?;
        Self::new(address, prefix_len)
    }
}

impl From<Ipv6Addr> for IpV6Net {
    fn from(address: Ipv6Addr) -> Self {
        Self {
            address,
            prefix_len: None,
        }
    }
}
