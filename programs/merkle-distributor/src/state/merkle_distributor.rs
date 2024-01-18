use anchor_lang::{
    account,
    prelude::{Pubkey, *},
};

/// State for the account which distributes tokens.
#[account]
#[derive(Default, Debug)]
pub struct MerkleDistributor {
    /// Bump seed.
    pub bump: u8,
    /// Version of the airdrop
    pub version: u64,
    /// The 256-bit merkle root.
    pub root: [u8; 32],
    /// [Mint] of the token to be distributed.
    pub mint: Pubkey,
    /// Token Address of the vault
    pub token_vault: Pubkey,
    /// Maximum number of tokens that can ever be claimed from this [MerkleDistributor].
    pub max_total_claim: u64,
    /// Maximum number of nodes in [MerkleDistributor].
    pub max_num_nodes: u64,
    /// Total amount of tokens that have been claimed.
    pub total_amount_claimed: u64,
    /// Number of nodes that have been claimed.
    pub num_nodes_claimed: u64,
    /// Lockup time start (Unix Timestamp)
    pub start_ts: i64,
    /// Lockup time end (Unix Timestamp)
    pub end_ts: i64,
    /// Clawback start (Unix Timestamp)
    pub clawback_start_ts: i64,
    /// Clawback receiver
    pub clawback_receiver: Pubkey,
    /// Admin wallet
    pub admin: Pubkey,
    /// Whether or not the distributor has been clawed back
    pub clawed_back: bool,
    /// this merkle tree is enable from this slot
    pub enable_slot: u64,
    /// indicate that whether admin can close this pool, for testing purpose
    pub closable: bool,
    /// Buffer 0
    pub buffer_0: [u8; 32],
    /// Buffer 1
    pub buffer_1: [u8; 32],
    /// Buffer 2
    pub buffer_2: [u8; 32],
}

impl MerkleDistributor {
    pub const LEN: usize = 8 + std::mem::size_of::<MerkleDistributor>();
}
