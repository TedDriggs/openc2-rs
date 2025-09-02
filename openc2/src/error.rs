use std::{
    borrow::Cow,
    collections::VecDeque,
    fmt::{self, Display},
    iter,
};

use from_variants::FromVariants;

#[derive(Debug, Clone, thiserror::Error)]
#[error("{kind}")]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn custom(message: impl Display) -> Self {
        Self {
            kind: ErrorKind::Custom(message.to_string()),
        }
    }

    pub fn validation(message: impl Display) -> Self {
        ValidationError::new(message.to_string()).into()
    }

    pub fn at(mut self, segment: impl Into<PathSegment>) -> Self {
        let segment = segment.into();
        match &mut self.kind {
            ErrorKind::Validation(ve) => {
                ve.path.push_front(segment);
            }
            ErrorKind::Multiple(errors) => {
                for err in errors {
                    *err = err.clone().at(segment.clone());
                }
            }
            _ => {}
        };

        self
    }

    pub fn accumulator() -> Accumulator {
        Accumulator::default()
    }
}

impl Error {
    pub fn as_validation(&self) -> Option<&ValidationError> {
        match &self.kind {
            ErrorKind::Validation(err) => Some(err),
            _ => None,
        }
    }
}

impl IntoIterator for Error {
    type Item = Error;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

impl<'a> IntoIterator for &'a Error {
    type Item = &'a Error;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self.kind {
            ErrorKind::Multiple(ref errors) => Iter::Multiple(errors.iter()),
            _ => Iter::Single(iter::once(self)),
        }
    }
}

pub enum Iter<'a> {
    Single(iter::Once<&'a Error>),
    Multiple(std::slice::Iter<'a, Error>),
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Error;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Single(iter) => iter.next(),
            Iter::Multiple(iter) => iter.next(),
        }
    }
}

pub enum IntoIter {
    Single(std::iter::Once<Error>),
    Multiple(std::vec::IntoIter<Error>),
}

impl Iterator for IntoIter {
    type Item = Error;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::Single(iter) => iter.next(),
            IntoIter::Multiple(iter) => iter.next(),
        }
    }
}

impl From<Error> for IntoIter {
    fn from(err: Error) -> Self {
        match err.kind {
            ErrorKind::Multiple(errors) => IntoIter::Multiple(errors.into_iter()),
            _ => IntoIter::Single(iter::once(err)),
        }
    }
}

#[derive(Debug)]
pub struct Accumulator {
    errors: Option<Vec<Error>>,
}

impl Accumulator {
    pub fn push(&mut self, error: impl Into<Error>) {
        self.errors
            .as_mut()
            .expect("Accumulator already finalized")
            .push(error.into());
    }

    pub fn handle<T>(&mut self, result: Result<T, impl Into<Error>>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(err) => {
                self.push(err.into());
                None
            }
        }
    }

    pub fn handle_in<T>(&mut self, op: impl Fn() -> Result<T, Error>) -> Option<T> {
        match op() {
            Ok(value) => Some(value),
            Err(err) => {
                self.push(err);
                None
            }
        }
    }

    pub fn checkpoint(&mut self) -> Result<(), Error> {
        let has_errors = !self
            .errors
            .as_ref()
            .expect("Accumulator already finalized")
            .is_empty();
        if !has_errors {
            return Ok(());
        }

        let mut errors = self.errors.take().expect("Accumulator already finalized");
        match errors.len() {
            0 => Ok(()),
            1 => Err(errors.drain(..).next().unwrap()),
            _ => Err(Error {
                kind: ErrorKind::Multiple(errors),
            }),
        }
    }

    pub fn finish(mut self) -> Result<(), Error> {
        self.checkpoint()?;
        self.errors = None;
        Ok(())
    }

    pub fn finish_with<T>(self, value: T) -> Result<T, Error> {
        self.finish().map(|_| value)
    }
}

impl Default for Accumulator {
    fn default() -> Self {
        Self {
            errors: Some(Vec::new()),
        }
    }
}

impl Drop for Accumulator {
    fn drop(&mut self) {
        if self.errors.is_some() {
            panic!("dropped Accumulator without finalizing");
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Path {
    segments: VecDeque<PathSegment>,
}

impl Path {
    pub fn push_front(&mut self, segment: impl Into<PathSegment>) {
        self.segments.push_front(segment.into());
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for segment in &self.segments {
            write!(f, "{segment:#}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, FromVariants)]
pub enum PathSegment {
    Key(Cow<'static, str>),
    Number(usize),
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            match self {
                PathSegment::Key(key) => write!(f, ".{}", key),
                PathSegment::Number(index) => write!(f, "[{}]", index),
            }
        } else {
            match self {
                PathSegment::Key(key) => write!(f, "{}", key),
                PathSegment::Number(index) => write!(f, "{}", index),
            }
        }
    }
}

impl From<&'static str> for PathSegment {
    fn from(value: &'static str) -> Self {
        PathSegment::Key(Cow::Borrowed(value))
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{path}: {message}")]
pub struct ValidationError {
    pub path: Path,
    pub message: String,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            path: Path::default(),
            message: message.into(),
        }
    }

    pub fn missing_required_field(field_name: impl Into<PathSegment>) -> Self {
        let field_name = field_name.into();
        Self {
            message: format!("missing required field '{field_name}'"),
            path: Path {
                segments: vec![field_name].into(),
            },
        }
    }

    pub fn at(mut self, segment: impl Into<PathSegment>) -> Self {
        self.path.push_front(segment);
        self
    }
}

impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Self {
            kind: ErrorKind::Validation(err),
        }
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self {
            kind: ErrorKind::Json(err.to_string()),
        }
    }
}

#[cfg(feature = "cbor")]
impl From<serde_cbor::Error> for Error {
    fn from(err: serde_cbor::Error) -> Self {
        Self {
            kind: ErrorKind::Cbor(err.to_string()),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, thiserror::Error)]
enum ErrorKind {
    #[error("validation error: {0}")]
    Validation(ValidationError),
    #[error("{0}")]
    Custom(String),
    #[cfg(feature = "json")]
    #[error("JSON error: {0}")]
    Json(String),
    #[cfg(feature = "cbor")]
    #[error("CBOR error: {0}")]
    Cbor(String),
    #[error("multiple errors")]
    Multiple(Vec<Error>),
}
