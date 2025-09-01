use crate::{
    Check, CommandId, DateTime, Duration, Error, Extensions, Profile, ResponseType, Target,
    error::ValidationError,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// An OpenC2 command communicates an action to be performed on a target.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Command<V> {
    /// The task or activity to be performed.
    pub action: Action,
    /// The object of the action. The action is performed on the target.
    pub target: Target<V>,
    #[serde(default, skip_serializing_if = "Args::is_empty")]
    pub args: Args<V>,
    /// The object which will perform the action on the target.
    pub profile: Option<Profile<V>>,
    pub command_id: Option<CommandId>,
}

impl<V> Command<V> {
    /// Create a new command without an actuator.
    pub fn new(action: Action, target: impl Into<Target<V>>) -> Self {
        Self {
            action,
            target: target.into(),
            args: Default::default(),
            profile: None,
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
pub struct Args<V> {
    pub start_time: Option<DateTime>,
    pub stop_time: Option<DateTime>,
    pub duration: Option<Duration>,
    pub response_requested: Option<ResponseType>,
    /// A human-readable note to annotate or provide information regarding the action.
    pub comment: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "Extensions::is_empty")]
    pub extensions: Extensions<V>,
}

impl<V> Args<V> {
    pub fn is_empty(&self) -> bool {
        self.start_time.is_none()
            && self.stop_time.is_none()
            && self.duration.is_none()
            && self.response_requested.is_none()
            && self.comment.is_none()
            && self.extensions.is_empty()
    }
}

impl<V> Check for Args<V> {
    fn check(&self) -> Result<(), Error> {
        let mut acc = Error::accumulator();
        if self.start_time.is_some() && self.stop_time.is_some() && self.duration.is_some() {
            acc.push(ValidationError::new(
                "duration",
                "Only two of start_time, stop_time, and duration may be specified at once",
            ));
        }

        acc.finish()
    }
}

impl<V> Default for Args<V> {
    fn default() -> Self {
        Self {
            start_time: None,
            stop_time: None,
            duration: None,
            response_requested: None,
            comment: None,
            extensions: Extensions::default(),
        }
    }
}
