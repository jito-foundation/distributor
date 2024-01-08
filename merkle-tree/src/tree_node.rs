use std::str::FromStr;

use serde::{Deserialize, Serialize};
use solana_program::{hash::hashv, pubkey::Pubkey};
use solana_sdk::hash::Hash;

use crate::csv_entry::CsvEntry;
pub const MINT_DECIMALS: u32 = 9;

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
fn ui_amount_to_token_amount(amount: u64) -> u64 {
    amount * 10u64.checked_pow(MINT_DECIMALS).unwrap()
}

impl From<CsvEntry> for TreeNode {
    fn from(entry: CsvEntry) -> Self {
        let node = Self {
            claimant: Pubkey::from_str(entry.pubkey.as_str()).unwrap(),
            amount: ui_amount_to_token_amount(entry.amount),
            proof: None,
        };
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_tree_node() {
        let tree_node = TreeNode {
            claimant: Pubkey::default(),
            amount: 0,
            proof: None,
        };
        let serialized = serde_json::to_string(&tree_node).unwrap();
        let deserialized: TreeNode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tree_node, deserialized);
    }

    #[test]
    fn test_ui_amount_to_token_amount() {
        let ui_amount = 5;
        let token_amount = ui_amount_to_token_amount(ui_amount);
        assert_eq!(token_amount, 5_000_000_000);
    }
}
