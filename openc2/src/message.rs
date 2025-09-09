use std::borrow::Cow;

use from_variants::FromVariants;
use serde::{Deserialize, Serialize, Serializer, de::DeserializeOwned};
use serde_with::skip_serializing_none;

use crate::{
    Check, Command, CommandId, DateTime, Error, IsEmpty, Notification, Response,
    error::ValidationError, response::StatusCode,
};

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Headers {
    pub request_id: Option<CommandId>,
    pub created: Option<DateTime>,
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
}

impl IsEmpty for Headers {
    fn is_empty(&self) -> bool {
        self.request_id.is_none()
            && self.created.is_none()
            && self.from.is_none()
            && self.to.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromVariants)]
#[non_exhaustive]
pub enum Body<V> {
    #[serde(rename = "openc2")]
    OpenC2(V),
}

/// Trait for converting a type into a serializable body that conforms to the OpenC2 message body structure.
/// This allows for types such as `Message<Command>` to be serialized correctly as OpenC2 bodies.
pub trait AsBody {
    type Output: Serialize;

    /// Returns the body representation of the type. This method should borrow from the type to avoid unnecessary
    /// allocations.
    fn as_body(&self) -> Self::Output;
}

impl<'a, T: Serialize> AsBody for &'a Body<T> {
    type Output = Body<&'a T>;

    fn as_body(&self) -> Self::Output {
        match self {
            Body::OpenC2(v) => Body::OpenC2(v),
        }
    }
}

impl<T: AsContent> AsBody for T {
    type Output = Body<T::Output>;

    fn as_body(&self) -> Self::Output {
        Body::OpenC2(self.as_content())
    }
}

/// An OpenC2 message.
///
/// This type is generic over the headers and body. To ensure correctness for serialization and deserialization,
/// the body uses the [`AsBody`] trait at serialization time and [`TryFrom<Body<Content<V>>>`] at deserialization time.
///
/// Additionally, the headers must implement [`IsEmpty`], as the standard requires they be omitted during serialization.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<H, B> {
    #[serde(
        default,
        skip_serializing_if = "IsEmpty::is_empty",
        bound(
            serialize = "H: Serialize + IsEmpty",
            deserialize = "H: Deserialize<'de> + Default"
        )
    )]
    pub headers: H,
    pub content_type: Cow<'static, str>,
    #[serde(
        serialize_with = "serialize_body",
        deserialize_with = "deserialize_body",
        bound(
            serialize = "for<'b> &'b B: AsBody",
            deserialize = "B: Deserialize<'de> + TryFrom<Body<Content<serde_json::Value>>>, B::Error: std::fmt::Display"
        )
    )]
    pub body: B,
    pub status_code: Option<StatusCode>,
}

fn serialize_body<T, S: Serializer>(body: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    for<'a> &'a T: AsBody,
{
    body.as_body().serialize(serializer)
}

fn deserialize_body<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: TryFrom<Body<Content<serde_json::Value>>>,
    T::Error: std::fmt::Display,
{
    Body::<Content<serde_json::Value>>::deserialize(deserializer)?
        .try_into()
        .map_err(serde::de::Error::custom)
}

impl<H, B> Message<H, B> {
    /// The value for [`Message::content_type`] for v1 and v2 of the OpenC2 specification.
    pub const CONTENT_TYPE: &str = "application/openc2";
}

impl<V> Message<Headers, Body<Content<V>>> {
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

impl<H: Default, V> From<Body<Content<V>>> for Message<H, Body<Content<V>>> {
    fn from(value: Body<Content<V>>) -> Self {
        // Auto-promote status code from response body
        let status_code = if let Body::OpenC2(Content::Response(r)) = &value {
            Some(r.status)
        } else {
            None
        };

        Self {
            headers: H::default(),
            content_type: Cow::Borrowed(Self::CONTENT_TYPE),
            body: value,
            status_code,
        }
    }
}

impl<H: Default, V> From<Content<V>> for Message<H, Body<Content<V>>> {
    fn from(value: Content<V>) -> Self {
        Body::from(value).into()
    }
}

impl<V> Check for Message<Headers, Body<Content<V>>> {
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

pub trait AsContent {
    type Output: Serialize;

    fn as_content(&self) -> Self::Output;
}

mod content_as_content {
    use crate::{Command, Notification, Response};

    use super::{AsContent, Content};
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ContentAsContent<'a, V> {
        Request(&'a Command<V>),
        Response(&'a Response<V>),
        Notification(&'a Notification<V>),
    }

    impl<'a, V: Serialize> AsContent for &'a Content<V> {
        type Output = ContentAsContent<'a, V>;

        fn as_content(&self) -> Self::Output {
            match self {
                Content::Request(cmd) => ContentAsContent::Request(cmd),
                Content::Response(rsp) => ContentAsContent::Response(rsp),
                Content::Notification(n) => ContentAsContent::Notification(n),
            }
        }
    }
}

impl<V> TryFrom<Body<Content<V>>> for Content<V> {
    type Error = Error;

    fn try_from(value: Body<Content<V>>) -> Result<Self, Self::Error> {
        let Body::OpenC2(value) = value;
        Ok(value)
    }
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

    #[test]
    fn deserialize_through_body() {
        let message: crate::Message<crate::Headers, Content<serde_json::Value>> =
            from_value(json!(
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
            message.body,
            Content::Request(Command {
                action: crate::Action::Deny,
                target: Target::File(_),
                ..
            })
        ));
    }

    #[test]
    fn round_trip_command_through_body() {
        let message: crate::Message<crate::Headers, crate::Command<serde_json::Value>> =
            from_value(json!(
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
            message.body,
            Command {
                action: crate::Action::Deny,
                target: Target::File(_),
                ..
            }
        ));

        let value = serde_json::to_value(&message).unwrap();
        assert_eq!(value["body"]["openc2"]["request"]["action"], "deny");
    }
}
