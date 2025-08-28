use std::borrow::Cow;

use from_variants::FromVariants;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::skip_serializing_none;

use crate::{Command, CommandId, DateTime, Extensions, Notification, Response, response::Status};

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(bound = "V: Serialize + DeserializeOwned + Default")]
pub struct Body<V> {
    pub openc2: Content<V>,
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
    use crate::{Command, Content, Target};

    use super::Message;
    use serde_json::{from_value, json};

    #[test]
    fn deserialize() {
        let example: Message<serde_json::Value> = from_value(json!(
            {
                "request_id": "123",
                "content_type": "application/openc2",
                "msg_type": "command",
                "content": {
                    "action": "deny",
                    "target": {
                        "file": {
                            "path": "/hello.pdf"
                        }
                    }
                }
            }
        ))
        .unwrap();

        assert!(matches!(
            example.body.openc2,
            Content::Request(Command {
                target: Target::File(_),
                ..
            })
        ));
    }
}
