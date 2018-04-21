/// A message sent from an entity as the result of a command. Response
/// messages provide acknowledgement, status, results from a query or other information as requested from
/// the issuer of the command.
///
/// Response messages are solicited and correspond to a command. The recipient of the OpenC2 Response
/// is typically the entity that issued the command.
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    /// A hidden field which forces callers to create responses using
    /// the public methods rather than struct literals.
    #[serde(default, skip_serializing)]
    __extensible: (),
}
