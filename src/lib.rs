//! The OpenC2 Language Specification defines a language used to compose messages that instruct
//! and coordinate the command and control of cyber defenses between and within networks and systems.
//!
//! This crate provides types for OpenC2 commands and responses.
//!
//! # Crate Purpose
//! This crate helps actuator implementers and other cybersecurity vendors interact with OpenC2 messages.

#[macro_use]
extern crate from_variants;

extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod actuator;
pub mod target;
mod command;
mod response;

#[doc(inline)]
pub use actuator::Actuator;

#[doc(inline)]
pub use command::{Action, Command};

#[doc(inline)]
pub use target::Target;

#[doc(inline)]
pub use response::Response;