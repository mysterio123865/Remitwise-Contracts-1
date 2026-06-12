use soroban_sdk::contracttype;

/// Lifecycle status of a remittance transfer held in escrow.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Funds are locked in escrow awaiting the recipient's claim.
    Pending = 0,
    /// The recipient has successfully claimed the funds.
    Claimed = 1,
    /// The sender cancelled the transfer (or it expired) and reclaimed funds.
    Cancelled = 2,
}
