//! The OpenC2 Language Specification defines a language used to compose messages that instruct
//! and coordinate the command and control of cyber defenses between and within networks and systems.
//!
//! This crate provides types for OpenC2 commands and responses.
//!
//! # Crate Purpose
//! This crate helps actuator implementers and other cybersecurity vendors interact with OpenC2 messages.

mod command;
mod data;
mod error;
mod message;
mod notification;
mod profile;
mod response;
pub mod target;
mod traits;

pub use error::{Error, ErrorAt};

#[doc(inline)]
pub use profile::Profile;

#[doc(inline)]
pub use command::{Action, Args, Command};

#[doc(inline)]
pub use data::*;

#[doc(inline)]
pub use message::{AsBody, AsContent, Body, Content, Headers, Message};

pub use notification::Notification;

#[doc(inline)]
pub use target::{Target, TargetType};

#[doc(inline)]
pub use response::{Response, Results, StatusCode};

pub use traits::{Check, IsEmpty};

/// Type aliases for JSON-based OpenC2 messages.
#[cfg(feature = "json")]
pub mod json {
    use serde_json::Value;

    pub type Args = super::Args<Value>;
    pub type Body = super::Body<Content>;
    pub type Content = super::Content<Value>;
    pub type Message = super::Message<super::Headers, Body>;
    pub type Command = super::Command<Value>;
    pub type Response = super::Response<Value>;
    pub type Extensions = super::Extensions<Value>;
    pub type Results = super::Results<Value>;
    pub type Target = super::Target<Value>;
}

/// Type aliases for CBOR-based OpenC2 messages.
#[cfg(feature = "cbor")]
pub mod cbor {
    use serde_cbor::Value;

    pub type Args = super::Args<Value>;
    pub type Body = super::Body<Content>;
    pub type Content = super::Content<Value>;
    pub type Message = super::Message<Headers, Body>;
    pub type Command = super::Command<Value>;
    pub type Response = super::Response<Value>;
    pub type Extensions = super::Extensions<Value>;
    pub type Results = super::Results<Value>;
    pub type Target = super::Target<Value>;
}
