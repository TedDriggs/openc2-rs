//! PF-specific command arguments for OpenC2

use openc2::IsEmpty;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Specifies how to handle denied packets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DropProcess {
    /// Drop the packet and do not send a notification to the source of the packet.
    None,
    /// Drop the packet and send an ICMP host unreachable (or equivalent) to the source of the packet.
    Reject,
    /// Drop the traffic and send a false acknowledgment.
    FalseAck,
}

/// Specifies the direction for rule application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Both,
    Ingress,
    Egress,
}

/// PF-specific arguments.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Args {
    pub drop_process: Option<DropProcess>,
    pub persistent: Option<bool>,
    pub direction: Option<Direction>,
    pub insert_rule: Option<u32>,
    pub logged: Option<bool>,
    pub description: Option<String>,
    pub stateful: Option<bool>,
    pub priority: Option<u32>,
}

impl IsEmpty for Args {
    fn is_empty(&self) -> bool {
        self.drop_process.is_none()
            && self.persistent.is_none()
            && self.direction.is_none()
            && self.insert_rule.is_none()
            && self.logged.is_none()
            && self.description.is_none()
            && self.stateful.is_none()
            && self.priority.is_none()
    }
}
