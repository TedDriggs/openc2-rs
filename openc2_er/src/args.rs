use openc2::{DomainName, Ipv4Net, Ipv6Net, target::Device};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Args {
    pub account_status: Option<AccountStatus>,
    pub device_containment: Option<DeviceContainment>,
    pub permitted_addresses: Option<PermittedAddresses>,
    pub scan_depth: Option<ScanDepth>,
    pub periodic_scan: Option<PeriodicScan>,
    pub downstream_device: Option<DownstreamDevice>,
}

impl Args {
    pub fn is_empty(&self) -> bool {
        self.account_status.is_none()
            && self.device_containment.is_none()
            && self.permitted_addresses.is_none()
            && self.scan_depth.is_none()
            && self.periodic_scan.is_none()
            && self.downstream_device.is_none()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Enabled,
    Disabled,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, strum::EnumString, strum::Display,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DeviceContainment {
    /// Isolate the endpoint from communicating with other networked entities,
    /// typically through relegation to a private VLAN segment and/or port isolation.
    /// MAY be combined with the 'permitted_addresses' Argument to allow communication
    /// with select IP or domain name addresses.
    NetworkIsolation,
    /// Restrict the execution of applications to only those that are signed by a trusted party.
    AppRestriction,
    /// Disable the network interface controller(s) on the endpoint.
    DisableNic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownstreamDevice {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<Device>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub device_groups: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

impl DownstreamDevice {
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty() && self.device_groups.is_empty() && self.tenant_id.is_none()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PeriodicScan {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermittedAddresses {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ipv4_net: Vec<Ipv4Net>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ipv6_net: Vec<Ipv6Net>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domain_names: Vec<DomainName>,
}

impl PermittedAddresses {
    pub fn is_empty(&self) -> bool {
        self.ipv4_net.is_empty() && self.ipv6_net.is_empty() && self.domain_names.is_empty()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScanDepth {
    Shallow,
    Deep,
}
