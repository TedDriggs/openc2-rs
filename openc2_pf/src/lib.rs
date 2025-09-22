//! OpenC2 Actuator Profile for Packet Filtering (PF)
//! Implements types and logic for the PF actuator profile as defined in the OASIS specification.

mod args;
pub mod target;

pub use args::*;
use openc2::Nsid;
pub use target::*;

pub static NS: &Nsid = &Nsid::PF;
