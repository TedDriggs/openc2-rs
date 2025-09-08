pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

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

impl<K, V, S> IsEmpty for std::collections::HashMap<K, V, S> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
