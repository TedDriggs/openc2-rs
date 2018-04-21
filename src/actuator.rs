//! Types for declaring entities that will execute actions on targets.

/// Information about the entity that will execute the action on the target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, FromVariants)]
#[serde(rename_all = "snake_case")]
pub enum Actuator {
    Endpoint(Endpoint),
    NetworkRouter(NetworkRouter),
    #[doc(hidden)]
    NonExhaustive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Endpoint(String);

impl Endpoint {
    pub fn new<S>(name: S) -> Self where S: Into<String> {
        Endpoint(name.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct NetworkRouter {
    actuator_id: String,
}
