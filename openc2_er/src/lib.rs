mod args;
pub mod target;

pub use args::*;
use openc2::Nsid;
pub use target::{Target, TargetType};

pub const NS: &Nsid = &Nsid::ER;
