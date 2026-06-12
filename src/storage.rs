use soroban_sdk::contracttype;

/// Keys used to address values in contract storage.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataKey {
    /// Administrator address (instance storage).
    Admin,
    /// Token contract address used for transfers (instance storage).
    Token,
    /// Monotonic counter for issued transfer ids (instance storage).
    Counter,
    /// A single transfer record keyed by its id (persistent storage).
    Transfer(u64),
}
