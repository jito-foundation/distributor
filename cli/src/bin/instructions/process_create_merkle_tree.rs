use std::collections::HashMap;

use crate::*;

pub fn process_create_merkle_tree(merkle_tree_args: &CreateMerkleTreeArgs) {
    let mut csv_entries = CsvEntry::new_from_file(&merkle_tree_args.csv_path).unwrap();

    // exclude test address if have

    if merkle_tree_args.should_include_test_list {
        let test_list = get_test_list()
            .into_iter()
            .map(|x| (x, true))
            .collect::<HashMap<_, _>>();
        let mut new_entries = vec![];
        for entry in csv_entries.iter() {
            if test_list.contains_key(&entry.pubkey) {
                continue;
            }
            new_entries.push(entry.clone());
        }
        println!(
            "trim test wallets from {} to {}",
            csv_entries.len(),
            new_entries.len()
        );
        csv_entries = new_entries;
    }

    let max_nodes_per_tree = merkle_tree_args.max_nodes_per_tree as usize;

    let base_path = &merkle_tree_args.merkle_tree_path;
    let mut index = 0;
    while csv_entries.len() > 0 {
        let last_index = max_nodes_per_tree.min(csv_entries.len());
        let sub_tree = csv_entries[0..last_index].to_vec();
        csv_entries = csv_entries[last_index..csv_entries.len()].to_vec();

        // use index as version
        let merkle_tree = AirdropMerkleTree::new_from_entries(sub_tree, index).unwrap();

        let base_path_clone = base_path.clone();
        let path = base_path_clone
            .as_path()
            .join(format!("tree_{}.json", index));

        merkle_tree.write_to_file(&path);
        index += 1;
    }

    if merkle_tree_args.should_include_test_list {
        println!("create merkle tree for test claming index {}", index);
        let test_list = get_test_list()
            .into_iter()
            .map(|x| CsvEntry {
                pubkey: x,
                amount: 200,
            })
            .collect::<Vec<CsvEntry>>();

        let merkle_tree = AirdropMerkleTree::new_from_entries(test_list, index).unwrap();
        let base_path_clone = base_path.clone();
        let path = base_path_clone
            .as_path()
            .join(format!("tree_{}.json", index));

        merkle_tree.write_to_file(&path);
    }
}
