use std::str::FromStr;

use serde::{Deserialize, Serialize};
use solana_program::{hash::hashv, pubkey::Pubkey};
use solana_sdk::hash::Hash;

use crate::csv_entry::CsvEntry;

/// Represents the claim information for an account.
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    /// Pubkey of the claimant; will be responsible for signing the claim
    pub claimant: Pubkey,
    /// Amount that claimant can claim
    pub amount: u64,
    /// Claimant's proof of inclusion in the Merkle Tree
    pub proof: Option<Vec<[u8; 32]>>,
}

impl TreeNode {
    pub fn hash(&self) -> Hash {
        hashv(&[
            &self.claimant.to_bytes(),
            &self.amount.to_le_bytes(),
            &0u64.to_le_bytes(),
        ])
    }

    /// Return amount for this claimant
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

/// Converts a ui amount to a token amount (with decimals)
fn ui_amount_to_token_amount(amount: u64, decimals: u32) -> u64 {
    amount * 10u64.checked_pow(decimals).unwrap()
}

impl TreeNode {
    pub fn from_csv(entry: CsvEntry, decimals: u32) -> Self {
        let node = Self {
            claimant: Pubkey::from_str(entry.pubkey.as_str()).unwrap(),
            amount: ui_amount_to_token_amount(entry.amount, decimals),
            proof: None,
        };
        node
    }
}
