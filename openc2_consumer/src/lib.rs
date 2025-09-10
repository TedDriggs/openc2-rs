use async_trait::async_trait;
use openc2::{
    Error, Headers, Message,
    json::{Command, Response},
};

mod registry;

pub use registry::{Registration, Registry};

/// Consumer trait for handling OpenC2 messages.
#[async_trait]
pub trait Consume {
    /// Handle an incoming OpenC2 command and produce a response.
    async fn consume(&self, msg: Message<Headers, Command>) -> Result<Response, Error>;
}
