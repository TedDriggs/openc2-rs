use std::borrow::Cow;

use futures::stream::BoxStream;
use openc2::{
    Error, Message,
    json::{Command, Headers, Response},
};

mod registry;
pub mod util;

pub use registry::{BoxConsumer, Registration, Registry, ToRegistration};

use crate::util::stream_just;

/// Consumer trait for handling OpenC2 messages.
pub trait Consume {
    /// Handle an incoming OpenC2 command and produce a response stream.
    fn consume<'a>(
        &'a self,
        msg: Message<Headers, Command>,
    ) -> BoxStream<'a, Result<Response, Error>>;
}

impl<T: Consume + Sync + Send> Consume for Box<T> {
    fn consume<'a>(
        &'a self,
        msg: Message<Headers, Command>,
    ) -> BoxStream<'a, Result<Response, Error>> {
        (**self).consume(msg)
    }
}

impl<T: Consume + Sync + Send> Consume for std::sync::Arc<T> {
    fn consume<'a>(
        &'a self,
        msg: Message<Headers, Command>,
    ) -> BoxStream<'a, Result<Response, Error>> {
        (**self).consume(msg)
    }
}

impl<'b, T> Consume for Cow<'b, T>
where
    T: Consume + ToOwned + ?Sized,
    T::Owned: Consume + Sync + Send,
{
    fn consume<'a>(
        &'a self,
        msg: Message<Headers, Command>,
    ) -> BoxStream<'a, Result<Response, Error>> {
        match self {
            Cow::Borrowed(t) => t.consume(msg),
            Cow::Owned(t) => t.consume(msg),
        }
    }
}

impl<T: Consume + Sync + Send> Consume for Option<T> {
    fn consume<'a>(
        &'a self,
        msg: Message<Headers, Command>,
    ) -> BoxStream<'a, Result<Response, Error>> {
        match self {
            Some(consumer) => consumer.consume(msg),
            None => stream_just(Err(Error::custom("no consumer available"))),
        }
    }
}

impl<T: Consume + Sync + Send> Consume for Result<T, Error> {
    fn consume<'a>(
        &'a self,
        msg: Message<Headers, Command>,
    ) -> BoxStream<'a, Result<Response, Error>> {
        match self {
            Ok(consumer) => consumer.consume(msg),
            Err(e) => stream_just(Err(e.clone())),
        }
    }
}
