//! Types for declaring entities that will execute actions on targets.

use from_variants::FromVariants;
use serde::{Deserialize, Serialize};

/// Information about the entity that will execute the action on the target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, FromVariants)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Actuator {
    Endpoint(Endpoint),
    NetworkRouter(NetworkRouter),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Endpoint(String);

impl Endpoint {
    pub fn new(name: impl Into<String>) -> Self {
        Endpoint(name.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct NetworkRouter {
    actuator_id: String,
}
