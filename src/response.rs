use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{ActionTargets, Extensions};

/// A message sent from an entity as the result of a command. Response
/// messages provide acknowledgement, status, results from a query or other information as requested from
/// the issuer of the command.
///
/// Response messages are solicited and correspond to a command. The recipient of the OpenC2 Response
/// is typically the entity that issued the command.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Response {
    /// The status of the response to the command.
    pub status: Status,
    /// A description providing additional information about the status of the response.
    pub status_text: Option<String>,
    pub results: Option<Results>,
}

pub type Status = u16;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Results {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<()>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pairs: Option<ActionTargets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<u64>,
    #[serde(flatten, default, skip_serializing_if = "Extensions::is_empty")]
    pub extensions: Extensions,
}
