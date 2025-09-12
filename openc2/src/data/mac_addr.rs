use std::{fmt, str::FromStr};

pub use macaddr::{MacAddr6, MacAddr8};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::error::ValidationError;

/// A MAC address, either in *EUI-48* or *EUI-64* format.
#[derive(
    Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, SerializeDisplay, DeserializeFromStr,
)]
pub enum MacAddr {
    V6(MacAddr6),
    V8(MacAddr8),
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacAddr::V6(addr) => addr.fmt(f),
            MacAddr::V8(addr) => addr.fmt(f),
        }
    }
}

impl FromStr for MacAddr {
    type Err = crate::error::ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macaddr::MacAddr::from_str(s)
            .map_err(|e| ValidationError::new(e.to_string()))
            .map(MacAddr::from)
    }
}

impl From<macaddr::MacAddr> for MacAddr {
    fn from(addr: macaddr::MacAddr) -> Self {
        match addr {
            macaddr::MacAddr::V6(a) => MacAddr::V6(a),
            macaddr::MacAddr::V8(a) => MacAddr::V8(a),
        }
    }
}

impl From<MacAddr> for macaddr::MacAddr {
    fn from(addr: MacAddr) -> Self {
        match addr {
            MacAddr::V6(a) => macaddr::MacAddr::V6(a),
            MacAddr::V8(a) => macaddr::MacAddr::V8(a),
        }
    }
}

impl PartialEq<MacAddr> for macaddr::MacAddr {
    fn eq(&self, other: &MacAddr) -> bool {
        match (self, other) {
            (macaddr::MacAddr::V6(a), MacAddr::V6(b)) => a == b,
            (macaddr::MacAddr::V8(a), MacAddr::V8(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MacAddr, MacAddr6};

    #[test]
    fn macaddr_parse_case_insensitive() {
        let addr: MacAddr = "AA:BB:CC:DD:EE:FF".parse().unwrap();
        assert_eq!(addr, "aa:bb:cc:dd:ee:ff".parse().unwrap());
    }

    #[test]
    fn macaddr6_parse() {
        let addr: MacAddr6 = "01:23:45:67:89:ab".parse().unwrap();
        assert_eq!(addr.to_string(), "01:23:45:67:89:AB");
    }
}
