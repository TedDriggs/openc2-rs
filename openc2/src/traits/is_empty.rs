pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

/// Implement `IsEmpty` for optional fields that themselves impl `IsEmpty`.
/// To make a generic `Option<T>` impl `IsEmpty`, add an `IsEmpty` impl to `T` that
/// always returns false.
///
/// This is done because without specialization it's impossible to have separate impls
/// for `Option<T>` and `Option<T: IsEmpty>`, and we want to allow for `Option<T: IsEmpty>` fields
/// to avoid allocations.
impl<T: IsEmpty> IsEmpty for Option<T> {
    fn is_empty(&self) -> bool {
        match self {
            Some(v) => v.is_empty(),
            None => true,
        }
    }
}

impl<T: IsEmpty> IsEmpty for Box<T> {
    fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmpty for &[T] {
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> IsEmpty for std::collections::HashSet<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmpty for std::collections::BTreeSet<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<K, V, S> IsEmpty for std::collections::HashMap<K, V, S> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<K, V> IsEmpty for std::collections::BTreeMap<K, V> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
