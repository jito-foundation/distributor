use crate::*;

pub fn process_create_test_list(args: &Args, create_test_list_args: &CreateTestListArgs) {
    let pre_list = get_pre_list();
    let mut wtr = Writer::from_path(&create_test_list_args.csv_path).unwrap();
    wtr.write_record(&["pubkey", "amount"]).unwrap();

    for addr in pre_list.iter() {
        wtr.write_record(&[addr, "6000"]).unwrap();
    }
    wtr.flush().unwrap();

    let merkle_tree_args = &CreateMerkleTreeArgs {
        csv_path: create_test_list_args.csv_path.clone(),
        merkle_tree_path: create_test_list_args.merkle_tree_path.clone(),
        max_nodes_per_tree: 10000,
    };
    process_create_merkle_tree(merkle_tree_args);
}
