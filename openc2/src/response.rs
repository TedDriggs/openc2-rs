use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{ActionTargets, Extensions, Nsid, Version};

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
    #[serde(default)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Results<V> {
    #[serde(default, skip_serializing_if = "IndexSet::is_empty")]
    pub versions: IndexSet<Version>,
    #[serde(default, skip_serializing_if = "IndexSet::is_empty")]
    pub profiles: IndexSet<Nsid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pairs: Option<ActionTargets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<u64>,
    #[serde(flatten, default, skip_serializing_if = "Extensions::is_empty")]
    pub extensions: Extensions<V>,
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
        assert!(scan.contains(&TargetType::IpV4Net));
        assert!(scan.contains(&TargetType::File));
        assert!(scan.len() == 2);

        let query = &pairs[&Action::Query];
        assert!(query.contains(&TargetType::ProfileDefined(
            "crwd/hostgroup".parse().unwrap()
        )));
        assert!(query.len() == 1);
    }
}
