use crate::Nsid;

/// An OpenC2 actuator profile.
pub trait Profile {
    /// Returns the profile's namespace identifier.
    fn ns() -> &'static Nsid;
}
