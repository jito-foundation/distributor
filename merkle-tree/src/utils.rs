use solana_program::pubkey::Pubkey;

use crate::{merkle_tree::MerkleTree, tree_node::TreeNode};

pub fn get_proof(merkle_tree: &MerkleTree, index: usize) -> Vec<[u8; 32]> {
    let mut proof = Vec::new();
    let path = merkle_tree.find_path(index).expect("path to index");
    for branch in path.get_proof_entries() {
        if let Some(hash) = branch.get_left_sibling() {
            proof.push(hash.to_bytes());
        } else if let Some(hash) = branch.get_right_sibling() {
            proof.push(hash.to_bytes());
        } else {
            panic!("expected some hash at each level of the tree");
        }
    }
    proof
}

/// Given a set of tree nodes, get the max total claim amount. Panics on overflow
pub fn get_max_total_claim(nodes: &[TreeNode]) -> u64 {
    nodes
        .iter()
        .try_fold(0, |acc: u64, n| acc.checked_add(n.total_amount()))
        .unwrap()
}

pub fn get_merkle_distributor_pda(
    program_id: &Pubkey,
    mint: &Pubkey,
    version: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"MerkleDistributor".as_ref(),
            mint.as_ref(),
            version.to_le_bytes().as_ref(),
        ],
        program_id,
    )
}

pub fn get_claim_status_pda(
    program_id: &Pubkey,
    claimant: &Pubkey,
    distributor: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"ClaimStatus".as_ref(),
            claimant.to_bytes().as_ref(),
            distributor.to_bytes().as_ref(),
        ],
        program_id,
    )
}

#[derive(Debug)]
pub struct MerkleValidationError {
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper function to create a tree node
    fn create_node(
        claimant: Pubkey,
        total_unlocked_staker: u64,
        total_locked_staker: u64,
        total_unlocked_searcher: u64,
        total_locked_searcher: u64,
        total_unlocked_validator: u64,
        total_locked_validator: u64,
    ) -> TreeNode {
        TreeNode {
            claimant,
            proof: None,
            total_unlocked_staker,
            total_locked_staker,
            total_unlocked_searcher,
            total_locked_searcher,
            total_unlocked_validator,
            total_locked_validator,
        }
    }

    #[test]
    fn test_get_max_total_claim_no_overflow() {
        let nodes = vec![
            create_node(Pubkey::new_unique(), 100, 200, 0, 0, 0, 0),
            create_node(Pubkey::new_unique(), 300, 400, 0, 0, 0, 0),
        ];

        let total = get_max_total_claim(&nodes);
        assert_eq!(total, 1000); // 100 + 200 + 300 + 400
    }

    #[test]
    #[should_panic(expected = "Option::unwrap()` on a `None` value")]
    fn test_get_max_total_claim_overflow() {
        let large_number = u64::MAX / 2;
        let nodes = vec![
            create_node(Pubkey::new_unique(), large_number, large_number, 0, 0, 0, 0),
            create_node(Pubkey::new_unique(), large_number, large_number, 0, 0, 0, 0),
        ];

        let _ = get_max_total_claim(&nodes);
    }
}
