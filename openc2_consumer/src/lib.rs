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

#[async_trait]
impl<T: Consume + Sync + Send> Consume for Box<T> {
    async fn consume(&self, msg: Message<Headers, Command>) -> Result<Response, Error> {
        (**self).consume(msg).await
    }
}

#[async_trait]
impl<T: Consume + Sync + Send> Consume for std::sync::Arc<T> {
    async fn consume(&self, msg: Message<Headers, Command>) -> Result<Response, Error> {
        (**self).consume(msg).await
    }
}

#[async_trait]
impl<'a, T: Consume + Sync + Send + Clone> Consume for std::borrow::Cow<'a, T> {
    async fn consume(&self, msg: Message<Headers, Command>) -> Result<Response, Error> {
        (**self).consume(msg).await
    }
}

#[async_trait]
impl<T: Consume + Sync + Send> Consume for Option<T> {
    async fn consume(&self, msg: Message<Headers, Command>) -> Result<Response, Error> {
        match self {
            Some(consumer) => consumer.consume(msg).await,
            None => Err(Error::custom("no consumer available")),
        }
    }
}

#[async_trait]
impl<T: Consume + Sync + Send> Consume for Result<T, Error> {
    async fn consume(&self, msg: Message<Headers, Command>) -> Result<Response, Error> {
        match self {
            Ok(consumer) => consumer.consume(msg).await,
            Err(e) => Err(e.clone()),
        }
    }
}
