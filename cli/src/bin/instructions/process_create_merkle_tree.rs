use crate::*;

pub fn process_create_merkle_tree(merkle_tree_args: &CreateMerkleTreeArgs) {
    let mut csv_entries = CsvEntry::new_from_file(&merkle_tree_args.csv_path).unwrap();
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
}
