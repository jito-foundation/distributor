#![cfg_attr(not(test), no_std)]

#[cfg(feature = "sp-hash")]
fn hash_bytes(data: &[&[u8]]) -> [u8; 32] {
    solana_program::keccak::hashv(data).0
}

#[cfg(not(feature = "sp-hash"))]
fn hash_bytes(data: &[&[u8]]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut keccak = Keccak::v256();
    for d in data {
        keccak.update(d);
    }
    let mut out = [0u8; 32];
    keccak.finalize(&mut out);
    out
}

/// Verify a Merkle proof for a given leaf and root.
pub fn verify_proof(leaf: [u8; 32], root: [u8; 32], proof: &[[u8; 32]]) -> bool {
    let mut computed_hash = leaf;
    for proof_element in proof.iter() {
        if computed_hash <= *proof_element {
            computed_hash = hash_bytes(&[&[1u8], &computed_hash, proof_element]);
        } else {
            computed_hash = hash_bytes(&[&[1u8], proof_element, &computed_hash]);
        }
    }
    computed_hash == root
}

pub use verify_proof as verify;

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_leaf(data: &[u8]) -> [u8; 32] {
        hash_bytes(&[&[0u8], data])
    }

    fn hash_node(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
        let (l, r) = if a <= b { (a, b) } else { (b, a) };
        hash_bytes(&[&[1u8], &l, &r])
    }

    #[test]
    fn verify_two_leaves() {
        let a = hash_leaf(b"a");
        let b = hash_leaf(b"b");
        let root = hash_node(a, b);
        assert!(verify_proof(a, root, &[b]));
    }

    #[test]
    fn verify_multi_step() {
        let a = hash_leaf(b"a");
        let b = hash_leaf(b"b");
        let c = hash_leaf(b"c");
        let d = hash_leaf(b"d");

        let node_ab = hash_node(a, b);
        let node_cd = hash_node(c, d);
        let root = hash_node(node_ab, node_cd);

        let proof_for_b = [a, node_cd];
        assert!(verify_proof(b, root, &proof_for_b));
    }
}
