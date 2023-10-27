use anchor_lang::{event, prelude::*};

/// Emitted when a new claim is created.
#[event]
pub struct NewClaimEvent {
    /// User that claimed.
    pub claimant: Pubkey,
    /// Timestamp.
    pub timestamp: i64,
}

/// Emitted when tokens are claimed.
#[event]
pub struct ClaimedEvent {
    /// User that claimed.
    pub claimant: Pubkey,
    /// Amount of tokens to distribute.
    pub amount: u64,
}
