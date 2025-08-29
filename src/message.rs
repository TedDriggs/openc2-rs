use std::borrow::Cow;

use from_variants::FromVariants;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::skip_serializing_none;

use crate::{
    Check, Command, CommandId, DateTime, Error, Extensions, Notification, Response,
    error::ValidationError, response::Status,
};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Headers<V> {
    pub request_id: Option<CommandId>,
    pub created: Option<DateTime>,
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(flatten, default, skip_serializing_if = "Extensions::is_empty")]
    pub extensions: Extensions<V>,
}

impl<V> Headers<V> {
    pub fn is_empty(&self) -> bool {
        self.request_id.is_none()
            && self.created.is_none()
            && self.from.is_none()
            && self.to.is_empty()
            && self.extensions.is_empty()
    }
}

impl<V> Default for Headers<V> {
    fn default() -> Self {
        Self {
            request_id: None,
            created: None,
            from: None,
            to: Default::default(),
            extensions: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromVariants)]
#[non_exhaustive]
#[serde(
    rename = "snake_case",
    bound = "V: Serialize + DeserializeOwned + Default"
)]
pub enum Body<V> {
    OpenC2(Content<V>),
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(bound = "V: Serialize + DeserializeOwned + Default")]
pub struct Message<V> {
    #[serde(default, skip_serializing_if = "Headers::is_empty")]
    pub headers: Headers<V>,
    pub content_type: Cow<'static, str>,
    pub body: Body<V>,
    pub status_code: Option<Status>,
}

impl<V> Message<V> {
    /// The value for [`Message::content_type`] for v1 and v2 of the OpenC2 specification.
    pub const CONTENT_TYPE: &str = "application/openc2";

    pub fn command_id(&self) -> Option<&CommandId> {
        let Body::OpenC2(body) = &self.body;
        match body {
            // Per Spec:
            // A Consumer receiving a Command with command_id absent and request_id present in the
            // header of the Message MUST use the value of request_id as the command_id.
            Content::Request(cmd) => cmd
                .command_id
                .as_ref()
                .or_else(|| self.headers.request_id.as_ref()),
            Content::Response(_) => None,
            Content::Notification(_) => None,
        }
    }
}

impl<V> From<Body<V>> for Message<V> {
    fn from(value: Body<V>) -> Self {
        // Auto-promote status code from response body
        let status_code = if let Body::OpenC2(Content::Response(r)) = &value {
            Some(r.status)
        } else {
            None
        };

        Self {
            headers: Headers::default(),
            content_type: Cow::Borrowed(Self::CONTENT_TYPE),
            body: value,
            status_code,
        }
    }
}

impl<V> From<Content<V>> for Message<V> {
    fn from(value: Content<V>) -> Self {
        Body::from(value).into()
    }
}

impl<V> Check for Message<V> {
    fn check(&self) -> Result<(), Error> {
        let mut acc = Error::accumulator();

        let Body::OpenC2(body) = &self.body;
        match body {
            Content::Request(cmd) => {
                acc.handle(cmd.args.check());
                if let Some(rsp) = cmd.args.response_requested
                    && rsp.requires_request_id()
                    && self.headers.request_id.is_none()
                {
                    acc.push(ValidationError::missing_required_field(
                        "headers.request_id",
                    ));
                }
            }
            Content::Response(_) => {
                if self.status_code.is_none() {
                    acc.push(ValidationError::missing_required_field(
                        "headers.status_code",
                    ));
                }
            }
            Content::Notification(_) => {}
        }

        acc.finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromVariants)]
#[serde(
    rename_all = "snake_case",
    bound = "V: Serialize + DeserializeOwned + Default"
)]
pub enum Content<V> {
    Request(Command<V>),
    Response(Response<V>),
    Notification(Notification<V>),
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use crate::{Body, Command, Content, Target};

    use super::Message;
    use serde_json::{from_value, json};

    #[test]
    fn deserialize() {
        let example: Message<serde_json::Value> = from_value(json!(
            {
                "headers": {
                    "request_id": "123",
                },
                "body": {
                    "openc2": {
                        "request": {
                            "action": "deny",
                            "target": {
                                "file": {
                                    "path": "/hello.pdf"
                                }
                            }
                        }
                    }
                },
                "content_type": "application/openc2",
            }
        ))
        .unwrap();

        assert!(matches!(
            example.body,
            Body::OpenC2(Content::Request(Command {
                target: Target::File(_),
                ..
            }))
        ));
    }
}
