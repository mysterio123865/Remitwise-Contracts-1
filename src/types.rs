use soroban_sdk::{contracttype, Address};

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

/// A single remittance transfer record stored in escrow.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    /// Unique sequential identifier for this transfer.
    pub id: u64,
    /// Address that funded and owns the transfer.
    pub from: Address,
    /// Address entitled to claim the funds.
    pub recipient: Address,
    /// Amount of the token held in escrow.
    pub amount: i128,
    /// Ledger timestamp after which the transfer can be cancelled.
    pub expiry: u64,
    /// Current lifecycle status of the transfer.
    pub status: Status,
}
