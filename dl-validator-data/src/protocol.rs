/// A trait that can be implemented for different blockchain protocols which allow downloading
/// validator data that can be feed into a simulation of validator assignment.
pub trait Protocol {
    /// Download validator data from a service like a RPC node.
    fn download_validator_data(&self) -> anyhow::Result<Vec<ValidatorData>>;
}

/// The information per validator required to run a simulation.
pub struct ValidatorData {
    pub account_id: String,
    pub stake: u128,
    /// In case downloaded validator contains no information whether a node is malicious this is
    /// `None`.
    pub is_malicious: Option<bool>,
}
