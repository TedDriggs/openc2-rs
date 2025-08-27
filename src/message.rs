use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{DateTime, MessageType, response::Status};

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Message<C> {
    pub content_type: Cow<'static, str>,
    pub msg_type: MessageType,
    pub content: C,
    pub status_code: Option<Status>,
    pub request_id: Option<String>,
    pub created: Option<DateTime>,
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
}
