use std::borrow::Cow;

use from_variants::FromVariants;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::skip_serializing_none;

use crate::{Command, DateTime, MessageType, Response, response::Status};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(bound = "V: Serialize + DeserializeOwned + Default")]
pub struct Message<V> {
    pub content_type: Cow<'static, str>,
    #[serde(flatten)]
    pub content: Content<V>,
    pub status_code: Option<Status>,
    pub request_id: Option<String>,
    pub created: Option<DateTime>,
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
}

impl<V> Message<V> {
    /// The value for [`Message::content_type`] for v1 of the OpenC2 specification.
    pub const CONTENT_TYPE_V1: &str = "application/openc2";
}

#[derive(Debug, Clone, Serialize, Deserialize, FromVariants)]
#[serde(
    tag = "msg_type",
    content = "content",
    rename_all = "snake_case",
    bound = "V: Serialize + DeserializeOwned + Default"
)]
pub enum Content<V> {
    Command(Command<V>),
    Response(Response<V>),
}

impl<V> Content<V> {
    pub fn message_type(&self) -> MessageType {
        match self {
            Content::Command(_) => MessageType::Command,
            Content::Response(_) => MessageType::Response,
        }
    }
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use crate::{Command, Content, MessageType, Target};

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

        assert_eq!(example.content.message_type(), MessageType::Command);
        assert!(matches!(
            example.content,
            Content::Command(Command {
                target: Target::File(_),
                ..
            })
        ));
    }
}
