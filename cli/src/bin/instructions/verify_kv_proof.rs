use jito_merkle_tree::airdrop_merkle_tree::UserProof;
use rand::Rng;

use crate::*;

pub fn verify_kv_proof(verify_kv_proof_args: &VerifyKvProofArgs) {
    let csv_entries = CsvEntry::new_from_file(&verify_kv_proof_args.csv_path).unwrap();

    let num_addresses = csv_entries.len();
    let mut rng = rand::thread_rng();
    for i in 0..verify_kv_proof_args.num_verify {
        let index = rng.gen_range(0, num_addresses);

        let pubkey = csv_entries[index].pubkey.clone();

        // request to kv
        let kv_proof: UserProof =
            reqwest::blocking::get(format!("{}/{}", verify_kv_proof_args.kv_api, pubkey))
                .unwrap()
                .json()
                .unwrap();
        // request to local
        let local_proof: UserProof =
            reqwest::blocking::get(format!("{}/{}", verify_kv_proof_args.local_api, pubkey))
                .unwrap()
                .json()
                .unwrap();

        assert_eq!(kv_proof.amount, local_proof.amount);
        assert_eq!(kv_proof.merkle_tree, local_proof.merkle_tree);
        assert_eq!(kv_proof.proof, local_proof.proof);

        println!("Done verify count {} index {} pubkey {}", i, index, pubkey);
    }
}
