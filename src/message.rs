use std::borrow::Cow;

use from_variants::FromVariants;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{Command, DateTime, MessageType, Response, response::Status};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Message {
    pub content_type: Cow<'static, str>,
    #[serde(flatten)]
    pub content: Content,
    pub status_code: Option<Status>,
    pub request_id: Option<String>,
    pub created: Option<DateTime>,
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
}

impl Message {
    /// The value for [`Message::content_type`] for v1 of the OpenC2 specification.
    pub const CONTENT_TYPE_V1: &str = "application/openc2";
}

#[derive(Debug, Clone, Serialize, Deserialize, FromVariants)]
#[serde(tag = "msg_type", content = "content", rename_all = "snake_case")]
pub enum Content {
    Command(Command),
    Response(Response),
}

impl Content {
    pub fn message_type(&self) -> MessageType {
        match self {
            Content::Command(_) => MessageType::Command,
            Content::Response(_) => MessageType::Response,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Command, Content, MessageType, Target};

    use super::Message;
    use serde_json::{from_value, json};

    #[test]
    fn deserialize() {
        let example: Message = from_value(json!(
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
