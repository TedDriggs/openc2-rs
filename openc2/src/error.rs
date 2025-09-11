use std::{
    borrow::Cow,
    collections::VecDeque,
    fmt::{self, Display},
    iter,
};

use from_variants::FromVariants;

use crate::{Action, Response, StatusCode, TargetType};

/// Trait for prepending location information to errors.
pub trait ErrorAt: Sized {
    /// Add a new path segment to the front of an error's path.
    fn at<P: Into<PathSegment>>(self, segment: P) -> Self;
}

impl<T, E: ErrorAt> ErrorAt for Result<T, E> {
    fn at<P: Into<PathSegment>>(self, segment: P) -> Self {
        self.map_err(|e| e.at(segment))
    }
}

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

    pub fn not_implemented(message: impl Display) -> Self {
        NotImplementedError::new(message).into()
    }

    /// Returns an error indicating that the action-target pair is not implemented.
    pub fn not_implemented_pair(action: Action, target: &TargetType) -> Self {
        Self::not_implemented(format!(
            "unsupported action-target pair: {} - {}",
            action, target
        ))
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

impl ErrorAt for Error {
    fn at<P: Into<PathSegment>>(self, segment: P) -> Self {
        self.at(segment)
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

impl ErrorAt for Accumulator {
    fn at<P: Into<PathSegment>>(mut self, segment: P) -> Self {
        let segment = segment.into();
        self.errors = Some(
            self.errors
                .take()
                .expect("accumulator not yet dropped")
                .into_iter()
                .map(|err| err.at(segment.clone()))
                .collect(),
        );
        self
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
    path: Path,
    message: String,
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

impl ErrorAt for ValidationError {
    fn at<P: Into<PathSegment>>(self, segment: P) -> Self {
        self.at(segment)
    }
}

/// Error indicating that a consumer does not implement a requested feature.
#[derive(Debug, Clone, thiserror::Error)]
pub struct NotImplementedError {
    message: String,
    path: Option<Path>,
}

impl NotImplementedError {
    pub fn new(message: impl Display) -> Self {
        Self {
            message: message.to_string(),
            path: None,
        }
    }
}

impl ErrorAt for NotImplementedError {
    fn at<P: Into<PathSegment>>(mut self, segment: P) -> Self {
        let segment = segment.into();
        if let Some(path) = &mut self.path {
            path.push_front(segment);
        } else {
            self.path = Some(Path {
                segments: vec![segment].into(),
            });
        }
        self
    }
}

impl fmt::Display for NotImplementedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{} (at {path})", self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Self {
            kind: ErrorKind::Validation(err),
        }
    }
}

impl From<NotImplementedError> for Error {
    fn from(err: NotImplementedError) -> Self {
        Self {
            kind: ErrorKind::NotImplemented(err),
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
    #[error("{0}")]
    Validation(ValidationError),
    #[error("{0}")]
    NotImplemented(NotImplementedError),
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

impl<V> From<Error> for Response<V> {
    fn from(value: Error) -> Self {
        match value.kind {
            ErrorKind::Validation(e) => e.into(),
            ErrorKind::NotImplemented(e) => e.into(),
            ErrorKind::Custom(e) => Self::new(StatusCode::InternalError).with_status_text(e),
            #[cfg(feature = "json")]
            ErrorKind::Json(e) => Self::new(StatusCode::InternalError).with_status_text(e),
            #[cfg(feature = "cbor")]
            ErrorKind::Cbor(e) => Self::new(StatusCode::InternalError).with_status_text(e),
            ErrorKind::Multiple(errors) => errors
                .into_iter()
                .next()
                .expect("multi-error has errors")
                .into(),
        }
    }
}

impl<V> From<ValidationError> for Response<V> {
    fn from(value: ValidationError) -> Self {
        Self::new(StatusCode::BadRequest).with_status_text(value.to_string())
    }
}

impl<V> From<NotImplementedError> for Response<V> {
    fn from(value: NotImplementedError) -> Self {
        Self::new(StatusCode::NotImplemented).with_status_text(value.to_string())
    }
}
