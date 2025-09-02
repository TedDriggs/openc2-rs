mod args;
pub mod target;

pub use args::*;
use openc2::Nsid;

pub const NS: &'static Nsid = &Nsid::new_static("er");
