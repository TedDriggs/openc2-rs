use std::{borrow::Cow, collections::HashMap};

use from_variants::FromVariants;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::skip_serializing_none;

use crate::{
    Check, Command, CommandId, DateTime, Error, IsEmpty, Notification, Response,
    error::ValidationError, response::StatusCode,
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
    #[serde(flatten, default, skip_serializing_if = "IsEmpty::is_empty")]
    pub additional: HashMap<String, V>,
}

impl<V> Headers<V> {
    pub fn is_empty(&self) -> bool {
        self.request_id.is_none()
            && self.created.is_none()
            && self.from.is_none()
            && self.to.is_empty()
            && self.additional.is_empty()
    }
}

impl<V> Default for Headers<V> {
    fn default() -> Self {
        Self {
            request_id: None,
            created: None,
            from: None,
            to: Default::default(),
            additional: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromVariants)]
#[non_exhaustive]
#[serde(bound = "V: Serialize + DeserializeOwned + Default")]
pub enum Body<V> {
    #[serde(rename = "openc2")]
    OpenC2(Content<V>),
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<H, B> {
    #[serde(default, skip_serializing_if = "IsEmpty::is_empty")]
    #[serde(bound(
        serialize = "H: Serialize + IsEmpty",
        deserialize = "H: Deserialize<'de> + Default"
    ))]
    pub headers: H,
    pub content_type: Cow<'static, str>,
    #[serde(bound(serialize = "B: Serialize", deserialize = "B: Deserialize<'de>"))]
    pub body: B,
    pub status_code: Option<StatusCode>,
}

impl<V> Message<Headers<V>, Body<V>> {
    /// The value for [`Message::content_type`] for v1 and v2 of the OpenC2 specification.
    pub const CONTENT_TYPE: &str = "application/openc2";

    pub fn command_id(&self) -> Option<&CommandId> {
        let Body::OpenC2(body) = &self.body;
        match body {
            // Per Spec:
            // A Consumer receiving a Command with command_id absent and request_id present in the
            // header of the Message MUST use the value of request_id as the command_id.
            Content::Request(cmd) => cmd.command_id.as_ref().or(self.headers.request_id.as_ref()),
            Content::Response(_) => None,
            Content::Notification(_) => None,
        }
    }
}

impl<V> From<Body<V>> for Message<Headers<V>, Body<V>> {
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

impl<V> From<Content<V>> for Message<Headers<V>, Body<V>> {
    fn from(value: Content<V>) -> Self {
        Body::from(value).into()
    }
}

impl<V> Check for Message<Headers<V>, Body<V>> {
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
                    acc.push(ValidationError::missing_required_field("request_id").at("headers"));
                }
            }
            Content::Response(_) => {
                if self.status_code.is_none() {
                    acc.push(ValidationError::missing_required_field("status_code").at("headers"));
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

    use serde_json::{from_value, json};

    #[test]
    fn deserialize() {
        let example: crate::json::Message = from_value(json!(
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
