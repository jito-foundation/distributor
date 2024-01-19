use std::{collections::HashMap, fs::File, io::Write};

use serde::{Deserialize, Serialize};
use zip::write::FileOptions;

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvProof {
    pub merkle_tree: String,
    pub amount: u64,
    /// Claimant's proof of inclusion in the Merkle Tree
    pub proof: Vec<[u8; 32]>,
}

pub fn process_generate_kv_proof(args: &Args, generate_kv_proof_args: &GenerateKvProofArgs) {
    let mut paths: Vec<_> = fs::read_dir(&generate_kv_proof_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    let mut proofs = HashMap::new();
    let mut file_index = 0;

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");

        let (distributor_pubkey, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);

        for node in merkle_tree.tree_nodes.iter() {
            let user_pk = Pubkey::from(node.claimant);
            proofs.insert(
                user_pk.to_string(),
                KvProof {
                    merkle_tree: distributor_pubkey.to_string(),
                    amount: node.amount,
                    proof: node.proof.clone().unwrap(),
                },
            );

            if proofs.len() as u64 >= generate_kv_proof_args.max_entries_per_file {
                // write to file
                write_to_file(generate_kv_proof_args, file_index, &proofs);
                file_index += 1;
                proofs = HashMap::new();
            }
        }

        println!("done {}", merkle_tree.airdrop_version);
    }
    if proofs.len() > 0 {
        write_to_file(generate_kv_proof_args, file_index, &proofs);
    }
}

fn write_to_file(
    generate_kv_proof_args: &GenerateKvProofArgs,
    file_index: u64,
    proofs: &HashMap<String, KvProof>,
) {
    let path = generate_kv_proof_args
        .kv_path
        .as_path()
        .join(format!("{}.zip", file_index));

    println!("zip to file {}", file_index);
    let serialized = serde_json::to_string_pretty(proofs).unwrap();
    let file: File = File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file(format!("{}.json", file_index), options)
        .unwrap();
    zip.write_all(serialized.as_bytes()).unwrap();
    // Apply the changes you've made.
    // Dropping the `ZipWriter` will have the same effect, but may silently fail
    zip.finish().unwrap();
}
