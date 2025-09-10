use crate::{
    Body, Check, CommandId, Content, DateTime, Duration, Error, Extensions, IsEmpty, Nsid,
    ResponseType, Target,
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
    pub profile: Option<Nsid>,
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

mod command_as_content {
    use serde::Serialize;

    use crate::AsContent;

    use super::Command;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum CommandAsContent<'a, V> {
        Request(&'a Command<V>),
    }

    impl<'a, V: Serialize> AsContent for &'a Command<V> {
        type Output = CommandAsContent<'a, V>;

        fn as_content(&self) -> Self::Output {
            CommandAsContent::Request(self)
        }
    }
}

impl<V> TryFrom<Body<Content<V>>> for Command<V> {
    type Error = Error;

    fn try_from(value: Body<Content<V>>) -> Result<Self, Self::Error> {
        let Body::OpenC2(value) = value;
        match value {
            Content::Request(req) => Ok(req),
            _ => Err(Error::validation("body is not a command")),
        }
    }
}

/// The task or activity to be performed.
#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    strum::EnumString,
    strum::Display,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Action {
    /// Systematic examination of some aspect of the entity or its environment.
    Scan,
    /// Find an object physically, logically, functionally, or by organization.
    Locate,
    /// Initiate a request for information.
    Query,
    /// Prevent a certain event or action from completion, such as preventing a flow from reaching a destination or preventing access.
    Deny,
    /// Isolate a file, process, or entity so that it cannot modify or access assets or processes.
    Contain,
    /// Permit access to or execution of a Target.
    Allow,
    Start,
    Stop,
    Restart,
    Cancel,
    Set,
    Update,
    Redirect,
    Create,
    Delete,
    Detonate,
    Restore,
    Copy,
    Investigate,
    Remediate,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Period {
    pub start_time: Option<DateTime>,
    pub stop_time: Option<DateTime>,
    pub duration: Option<Duration>,
}

impl Period {
    /// Returns an error if any fields are set.
    pub fn require_empty(&self) -> Result<(), Error> {
        let mut errors = Error::accumulator();
        if self.duration.is_some() {
            errors.push(Error::not_implemented("duration is not supported").at("duration"));
        }

        if self.start_time.is_some() {
            errors.push(Error::not_implemented("start_time is not supported").at("start_time"));
        }

        if self.stop_time.is_some() {
            errors.push(Error::not_implemented("stop_time is not supported").at("stop_time"));
        }

        errors.finish()
    }
}

impl IsEmpty for Period {
    fn is_empty(&self) -> bool {
        self.start_time.is_none() && self.stop_time.is_none() && self.duration.is_none()
    }
}

impl Check for Period {
    fn check(&self) -> Result<(), Error> {
        let mut acc = Error::accumulator();
        if self.start_time.is_some() && self.stop_time.is_some() && self.duration.is_some() {
            acc.push(
                Error::validation(
                    "Only two of start_time, stop_time, and duration may be specified at once",
                )
                .at("duration"),
            );
        }

        acc.finish()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Args<V> {
    #[serde(flatten)]
    pub period: Period,
    pub response_requested: Option<ResponseType>,
    /// A human-readable note to annotate or provide information regarding the action.
    pub comment: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "Extensions::is_empty")]
    pub extensions: Extensions<V>,
}

impl<V> Args<V> {
    pub fn is_empty(&self) -> bool {
        self.period.is_empty()
            && self.response_requested.is_none()
            && self.comment.is_none()
            && self.extensions.is_empty()
    }
}

impl<V> Check for Args<V> {
    fn check(&self) -> Result<(), Error> {
        let mut acc = Error::accumulator();
        acc.handle(self.period.check());

        acc.finish()
    }
}

impl<V> Default for Args<V> {
    fn default() -> Self {
        Self {
            period: Period::default(),
            response_requested: None,
            comment: None,
            extensions: Extensions::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Action;

    #[test]
    fn action_display() {
        assert_eq!(Action::Scan.to_string(), "scan");
    }

    #[test]
    fn action_from_str() {
        assert_eq!("scan".parse::<Action>().unwrap(), Action::Scan);
    }
}
