use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::skip_serializing_none;

use crate::{ActionTargets, Body, Content, Error, Extensions, IsEmpty, Nsid, Value, Version};

/// A message sent from an entity as the result of a command. Response
/// messages provide acknowledgement, status, results from a query or other information as requested from
/// the issuer of the command.
///
/// Response messages are solicited and correspond to a command. The recipient of the OpenC2 Response
/// is typically the entity that issued the command.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Response<V> {
    /// The status of the response to the command.
    pub status: StatusCode,
    /// A description providing additional information about the status of the response.
    pub status_text: Option<String>,
    #[serde(default, skip_serializing_if = "IsEmpty::is_empty")]
    pub results: Option<Results<V>>,
}

impl<V> Response<V> {
    /// Create a new Response with the given status code.
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            status_text: None,
            results: None,
        }
    }

    pub fn with_status_text(mut self, text: impl Into<String>) -> Self {
        self.status_text = Some(text.into());
        self
    }

    pub fn with_results(mut self, results: Results<V>) -> Self {
        self.results = Some(results);
        self
    }
}

mod response_as_content {
    use serde::Serialize;

    use crate::AsContent;

    use super::Response;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ResponseAsContent<'a, V> {
        Response(&'a Response<V>),
    }

    impl<'a, V: Serialize> AsContent for &'a Response<V> {
        type Output = ResponseAsContent<'a, V>;

        fn as_content(&self) -> Self::Output {
            ResponseAsContent::Response(self)
        }
    }
}

impl<V> AsRef<StatusCode> for Response<V> {
    fn as_ref(&self) -> &StatusCode {
        &self.status
    }
}

impl<V> From<Result<Response<V>, Error>> for Response<V> {
    fn from(value: Result<Response<V>, Error>) -> Self {
        value.unwrap_or_else(Self::from)
    }
}

impl<V> TryFrom<Body<Content<V>>> for Response<V> {
    type Error = Error;

    fn try_from(value: Body<Content<V>>) -> Result<Self, Self::Error> {
        let Body::OpenC2(value) = value;
        match value {
            Content::Response(resp) => Ok(resp),
            _ => Err(Error::validation("body is not a response")),
        }
    }
}

#[derive(
    Debug, Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[repr(u16)]
pub enum StatusCode {
    Processing = 102,
    Ok = 200,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    InternalError = 500,
    NotImplemented = 501,
    ServiceUnavailable = 503,
}

impl StatusCode {
    /// Check if status is within 100-199.
    pub fn is_informational(&self) -> bool {
        matches!(self, StatusCode::Processing)
    }

    /// Check if status is within 200-299.
    pub fn is_success(&self) -> bool {
        matches!(self, StatusCode::Ok)
    }

    /// Check if status is within 400-599.
    pub fn is_error(&self) -> bool {
        self.is_producer_error() || self.is_consumer_error()
    }

    /// Check if status is within 400-499.
    pub fn is_producer_error(&self) -> bool {
        matches!(
            self,
            StatusCode::BadRequest
                | StatusCode::Unauthorized
                | StatusCode::Forbidden
                | StatusCode::NotFound
        )
    }

    /// Check if status is within 500-599.
    pub fn is_consumer_error(&self) -> bool {
        matches!(
            self,
            StatusCode::InternalError | StatusCode::NotImplemented | StatusCode::ServiceUnavailable
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Results<V> {
    #[serde(default, skip_serializing_if = "IsEmpty::is_empty")]
    pub versions: IndexSet<Version>,
    #[serde(default, skip_serializing_if = "IsEmpty::is_empty")]
    pub profiles: IndexSet<Nsid>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub pairs: Option<ActionTargets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<u64>,
    #[serde(flatten, default, skip_serializing_if = "Extensions::is_empty")]
    pub extensions: Extensions<V>,
}

impl<V: Value> Results<V> {
    /// Returns a new [`Results`] with the given extensions converted to `V`.
    pub fn with_extensions<E: Serialize>(
        mut self,
        extensions: impl IntoIterator<Item = (Nsid, E)>,
    ) -> Result<Self, V::Error> {
        self.extensions = extensions
            .into_iter()
            .map(|(k, v)| Ok((k, V::from_typed(&v)?)))
            .collect::<Result<_, _>>()?;
        Ok(self)
    }
}

impl<V> Default for Results<V> {
    fn default() -> Self {
        Self {
            versions: Default::default(),
            profiles: Default::default(),
            pairs: Default::default(),
            rate_limit: Default::default(),
            extensions: Default::default(),
        }
    }
}

impl<V> IsEmpty for Results<V> {
    fn is_empty(&self) -> bool {
        self.versions.is_empty()
            && self.profiles.is_empty()
            && self.pairs.is_empty()
            && self.rate_limit.is_none()
            && self.extensions.is_empty()
    }
}

impl<V> From<Results<V>> for Response<V> {
    fn from(value: Results<V>) -> Self {
        Self::new(StatusCode::Ok).with_results(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFeatures {
    pub pairs: ActionTargets,
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use serde_json::{Value, from_value, json};

    use crate::{Action, TargetType};

    use super::Results;

    #[test]
    fn pairs() {
        let example: Results<Value> = from_value(json!(
            {
                "pairs": {
                    "scan": ["ipv4_net", "file"],
                    "locate": ["device"],
                    "query": ["crwd/hostgroup"]
                }
            }
        ))
        .unwrap();

        let pairs = example.pairs.unwrap();

        let scan = &pairs[&Action::Scan];
        assert!(scan.contains(&TargetType::Ipv4Net));
        assert!(scan.contains(&TargetType::File));
        assert!(scan.len() == 2);

        let query = &pairs[&Action::Query];
        assert!(query.contains(&TargetType::ProfileDefined(
            "crwd/hostgroup".parse().unwrap()
        )));
        assert!(query.len() == 1);
    }
}
