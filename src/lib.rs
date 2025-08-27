//! The OpenC2 Language Specification defines a language used to compose messages that instruct
//! and coordinate the command and control of cyber defenses between and within networks and systems.
//!
//! This crate provides types for OpenC2 commands and responses.
//!
//! # Crate Purpose
//! This crate helps actuator implementers and other cybersecurity vendors interact with OpenC2 messages.

pub mod actuator;
mod command;
mod data;
mod message;
mod response;
pub mod target;

#[doc(inline)]
pub use actuator::Actuator;

#[doc(inline)]
pub use command::{Action, Command};

#[doc(inline)]
pub use data::*;

#[doc(inline)]
pub use message::{Content, Message};

#[doc(inline)]
pub use target::Target;

#[doc(inline)]
pub use response::Response;
