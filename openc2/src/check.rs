use crate::Error;

/// Check an OpenC2 structure for validity that can't be enforced by the type system.
pub trait Check {
    /// Returns all the validation errors in the structure, or `Ok` if there are none.
    ///
    /// Use `Error::accumulator()` to accumulate multiple errors rather than returning
    /// immediately.
    fn check(&self) -> Result<(), Error>;
}

impl<T: Check> Check for Option<T> {
    fn check(&self) -> Result<(), Error> {
        if let Some(inner) = self {
            inner.check()
        } else {
            Ok(())
        }
    }
}
