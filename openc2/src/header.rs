use std::{
    borrow::{Borrow, Cow},
    fmt, ops,
    str::FromStr,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::{IsEmpty, Value};

pub const FROM: HeaderName = HeaderName::from_static("from");
pub const TO: HeaderName = HeaderName::from_static("to");
pub const CREATED: HeaderName = HeaderName::from_static("created");
pub const REQUEST_ID: HeaderName = HeaderName::from_static("request_id");

#[derive(Clone, PartialEq, Eq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct HeaderName {
    inner: Cow<'static, str>,
}

impl HeaderName {
    /// Returns a `HeaderName` from a static string.
    ///
    /// # Panics
    /// This function could panic if the provided string is not a valid header name.
    pub const fn from_static(s: &'static str) -> Self {
        HeaderName {
            inner: Cow::Borrowed(s),
        }
    }
}

impl fmt::Debug for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HeaderName({})", self.inner)
    }
}

impl fmt::Display for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl FromStr for HeaderName {
    type Err = ParseHeaderNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(HeaderName {
            inner: Cow::Owned(s.to_string()),
        })
    }
}

impl PartialEq<String> for HeaderName {
    fn eq(&self, other: &String) -> bool {
        &self.inner == other
    }
}

impl PartialEq<str> for HeaderName {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl Borrow<str> for HeaderName {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

mod into_header_name {
    use indexmap::IndexMap;
    use serde::Serialize;

    use crate::Value;

    use super::HeaderName;

    pub trait IntoHeaderName: Sealed {}

    impl<T: Sealed> IntoHeaderName for T {}

    pub trait Sealed: Sized {
        fn insert<V>(self, map: &mut IndexMap<HeaderName, V>, value: V);

        fn try_insert_typed<V: Value, TV: Serialize>(
            self,
            map: &mut IndexMap<HeaderName, V>,
            value: TV,
        ) -> Result<(), V::Error> {
            self.insert(map, V::from_typed(&value)?);
            Ok(())
        }
    }

    impl Sealed for HeaderName {
        fn insert<V>(self, map: &mut IndexMap<HeaderName, V>, value: V) {
            map.insert(self, value);
        }
    }

    impl Sealed for &HeaderName {
        fn insert<V>(self, map: &mut IndexMap<HeaderName, V>, value: V) {
            map.insert(self.clone(), value);
        }
    }

    impl Sealed for String {
        fn insert<V>(self, map: &mut IndexMap<HeaderName, V>, value: V) {
            map.insert(self.parse().expect("valid header name"), value);
        }
    }

    impl Sealed for &str {
        fn insert<V>(self, map: &mut IndexMap<HeaderName, V>, value: V) {
            map.insert(self.parse().expect("valid header name"), value);
        }
    }
}

pub use into_header_name::IntoHeaderName;

mod as_header_name {
    use indexmap::IndexMap;

    pub trait AsHeaderName: Sealed {}

    impl<T: Sealed> AsHeaderName for T {}

    pub trait Sealed {
        fn get_from_map<'map, V>(
            &self,
            map: &'map IndexMap<super::HeaderName, V>,
        ) -> Option<&'map V>;
    }

    impl Sealed for super::HeaderName {
        fn get_from_map<'map, V>(
            &self,
            map: &'map IndexMap<super::HeaderName, V>,
        ) -> Option<&'map V> {
            map.get(self)
        }
    }

    impl Sealed for &super::HeaderName {
        fn get_from_map<'map, V>(
            &self,
            map: &'map IndexMap<super::HeaderName, V>,
        ) -> Option<&'map V> {
            map.get(*self)
        }
    }

    impl Sealed for &str {
        fn get_from_map<'map, V>(
            &self,
            map: &'map IndexMap<super::HeaderName, V>,
        ) -> Option<&'map V> {
            map.get(*self)
        }
    }

    impl Sealed for String {
        fn get_from_map<'map, V>(
            &self,
            map: &'map IndexMap<super::HeaderName, V>,
        ) -> Option<&'map V> {
            map.get(self.as_str())
        }
    }
}

pub use as_header_name::AsHeaderName;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ParseHeaderNameError {}

impl fmt::Display for ParseHeaderNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "malformed header name")
    }
}

impl std::error::Error for ParseHeaderNameError {}

/// A collection of OpenC2 message headers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Headers<V> {
    #[serde(flatten)]
    values: IndexMap<HeaderName, V>,
}

impl<V> Headers<V> {
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, name: impl AsHeaderName) -> Option<&V> {
        name.get_from_map(&self.values)
    }

    pub fn contains(&self, name: impl AsHeaderName) -> bool {
        name.get_from_map(&self.values).is_some()
    }

    pub fn insert(&mut self, name: impl IntoHeaderName, value: V) {
        name.insert(&mut self.values, value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&HeaderName, &V)> {
        self.values.iter()
    }
}

impl<V: Value> Headers<V> {
    /// Tries to get and deserialize a header value, returning `None` if the header name is not present,
    /// and `Some(Err(_))` if deserialization fails.
    pub fn try_get<'de, T: Deserialize<'de>>(
        &'de self,
        name: impl AsHeaderName,
    ) -> Option<Result<T, V::Error>> {
        name.get_from_map(&self.values).map(|v| V::to_typed(v))
    }

    /// Tries to insert a header value, returning an error if serialization fails.
    pub fn try_insert_value(
        &mut self,
        name: impl IntoHeaderName,
        value: impl Serialize,
    ) -> Result<(), V::Error> {
        name.try_insert_typed(&mut self.values, value)
    }
}

impl<V> IsEmpty for Headers<V> {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<K: AsHeaderName, V> ops::Index<K> for Headers<V> {
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(index).expect("no entry found for header name")
    }
}

impl<'a, V> IntoIterator for &'a Headers<V> {
    type Item = (&'a HeaderName, &'a V);
    type IntoIter = indexmap::map::Iter<'a, HeaderName, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl<V> IntoIterator for Headers<V> {
    type Item = (HeaderName, V);
    type IntoIter = indexmap::map::IntoIter<HeaderName, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<V> FromIterator<(HeaderName, V)> for Headers<V> {
    fn from_iter<T: IntoIterator<Item = (HeaderName, V)>>(iter: T) -> Self {
        Headers {
            values: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_static_str() {
        let mut headers = Headers::default();
        headers.insert("demo", "application/json");
        assert_eq!(headers.len(), 1);
        assert!(headers.contains("demo"));
    }

    #[test]
    fn insert_const() {
        let mut headers = Headers::default();
        headers.insert(TO, "application/json");
        assert_eq!(headers.len(), 1);
        assert!(headers.contains(TO));
    }
}
