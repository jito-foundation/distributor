use crate::*;

pub fn process_extend_list(extend_list_args: &ExtendListArgs) {
    let community_list = CsvEntry::new_from_file(&extend_list_args.csv_path).unwrap();
    let test_list: Vec<String> = get_pre_list();
    let mut pre_list = vec![];
    for node in community_list.iter() {
        let addr = Pubkey::from_str(&node.pubkey);
        if addr.is_err() {
            println!("{} is not pubkey", node.pubkey);
            continue;
        }
        pre_list.push((addr.unwrap(), node.amount));
    }

    for node in test_list.iter() {
        let addr = Pubkey::from_str(&node);
        if addr.is_err() {
            println!("{} is not pubkey", node);
            continue;
        }
        pre_list.push((addr.unwrap(), extend_list_args.amount));
    }

    // remove duplicate
    pre_list.sort_unstable();
    pre_list.dedup();

    // // add my key
    // let pre_list: Vec<String> = get_pre_list();
    let mut full_list = vec![];
    for _i in 0..(extend_list_args.num_records - full_list.len() as u64) {
        full_list.push((Pubkey::new_unique(), extend_list_args.amount));
    }
    // // merge with pre_list
    let num_node = extend_list_args
        .num_records
        .checked_div(pre_list.len() as u64)
        .unwrap() as usize;
    for (i, address) in pre_list.iter().enumerate() {
        full_list.insert(num_node * i, address.clone());
    }

    let mut wtr = Writer::from_path(&extend_list_args.destination_path).unwrap();
    wtr.write_record(&["pubkey", "amount"]).unwrap();
    for address in full_list.iter() {
        wtr.write_record(&[address.0.to_string(), address.1.to_string()])
            .unwrap();
    }

    wtr.flush().unwrap();
}
