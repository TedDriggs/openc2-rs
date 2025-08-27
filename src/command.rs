use crate::{Actuator, CommandId, DateTime, Duration, Extensions, ResponseType, Target};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// An OpenC2 command communicates an action to be performed on a target.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Command {
    /// The task or activity to be performed.
    pub action: Action,
    /// The object of the action. The action is performed on the target.
    pub target: Target,
    pub args: Option<Args>,
    /// The object which will perform the action on the target.
    pub actuator: Option<Actuator>,
    pub command_id: Option<CommandId>,
}

impl Command {
    /// Create a new command without an actuator.
    pub fn new(action: Action, target: impl Into<Target>) -> Self {
        Self {
            action,
            target: target.into(),
            args: None,
            actuator: None,
            command_id: None,
        }
    }

    /// Create a new command including an actuator.
    pub fn with_actuator<T, A>(action: Action, target: T, actuator: A) -> Self
    where
        T: Into<Target>,
        A: Into<Actuator>,
    {
        Self {
            action,
            target: target.into(),
            args: None,
            actuator: Some(actuator.into()),
            command_id: None,
        }
    }
}

/// The task or activity to be performed.
///
/// To support future extension of the OpenC2 language, this enum has a hidden variant
/// that prevents exhaustive matching.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Action {
    /// The ‘scan’ action is the systematic examination of some aspect of the entity or
    /// its environment in order to obtain information.
    Scan,
    /// The ‘locate’ action is used to find an object either physically, logically,
    /// functionally, or by organization.
    Locate,
    /// The ‘query’ action initiates a single request for information.
    Query,
    /// The ‘report’ action tasks an entity to provide information to a designated
    /// recipient of the information.
    Report,
    /// The ‘notify’ action is used to set an entity's alerting preferences.
    Notify,
    Deny,
    Contain,
    Allow,
    Start,
    Stop,
    Restart,
    Pause,
    Resume,
    Cancel,
    Set,
    Update,
    Move,
    Redirect,
    Create,
    Delete,
    Snapshot,
    Detonate,
    Restore,
    Save,
    Throttle,
    Delay,
    Substitute,
    Copy,
    Sync,
    Investigate,
    Mitigate,
    Remediate,
}

impl Action {
    /// Whether this action is used to gather information needed to determine the current state or enhance
    /// cyber situational awareness.
    pub fn controls_information(&self) -> bool {
        matches!(
            *self,
            Action::Scan | Action::Locate | Action::Query | Action::Report | Action::Notify
        )
    }

    /// Whether this action is used to control traffic flow and file permissions (e.g., allow/deny).
    pub fn controls_permissions(&self) -> bool {
        matches!(*self, Action::Deny | Action::Contain | Action::Allow)
    }

    /// Whether this action is used to control the state or the activity of a system, a process, a connection, a
    /// host, or a device. The actions are used to execute tasks, adjust configurations, set and update
    /// parameters, and modify attributes.
    pub fn controls_activity(&self) -> bool {
        matches!(
            *self,
            Action::Start
                | Action::Stop
                | Action::Restart
                | Action::Pause
                | Action::Resume
                | Action::Cancel
                | Action::Set
                | Action::Update
                | Action::Move
                | Action::Redirect
                | Action::Create
                | Action::Delete
                | Action::Snapshot
                | Action::Detonate
                | Action::Restore
                | Action::Save
                | Action::Throttle
                | Action::Delay
                | Action::Substitute
                | Action::Copy
                | Action::Sync
        )
    }

    /// Whether this action is an effect-based action.
    ///
    /// Effects-based actions are at a higher level of abstraction for purposes of communicating a
    /// desired impact rather than a command to execute specific tasks. This level of abstraction enables
    /// coordinated actions between enclaves, while permitting a local enclave to optimize its workflow
    /// for its specific environment. Effects-based action assumes that the recipient enclave has a
    /// decision-making capability because effects-based actions typically do not have a one-to-one
    /// mapping to the other actions.
    pub fn is_effect(&self) -> bool {
        matches!(
            *self,
            Action::Investigate | Action::Mitigate | Action::Remediate
        )
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Args {
    pub start_time: Option<DateTime>,
    pub stop_time: Option<DateTime>,
    pub duration: Option<Duration>,
    pub response_requested: Option<ResponseType>,
    #[serde(flatten, default, skip_serializing_if = "Extensions::is_empty")]
    pub extensions: Extensions,
}

#[cfg(test)]
mod tests {
    use super::{Action, Command};
    use crate::{actuator, target};

    #[test]
    fn rsa_demo() {
        let cmd = Command::with_actuator(
            Action::Delete,
            target::File {
                name: Some("Hello".into()),
                hashes: None,
                path: None,
            },
            actuator::Endpoint::new("host"),
        );
        assert!(cmd.action.controls_activity());
        assert!(!cmd.action.is_effect());
    }
}
