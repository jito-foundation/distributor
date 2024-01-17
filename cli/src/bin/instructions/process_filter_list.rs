use crate::*;

pub fn process_filter_list(filter_list_args: &FilterListArgs) {
    let community_list = CsvEntry::new_from_file(&filter_list_args.csv_path).unwrap();
    let test_list: Vec<String> = get_pre_list();
    let mut full_list = vec![];
    for node in community_list.iter() {
        let addr = Pubkey::from_str(&node.pubkey);
        if addr.is_err() {
            println!("{} is not pubkey", node.pubkey);
            continue;
        }
        full_list.push((addr.unwrap(), 200));
    }

    for node in test_list.iter() {
        let addr = Pubkey::from_str(&node);
        if addr.is_err() {
            println!("{} is not pubkey", node);
            continue;
        }
        full_list.push((addr.unwrap(), filter_list_args.amount));
    }

    // remove duplicate
    full_list.sort_unstable();
    full_list.dedup_by(|a, b| a.0 == b.0);

    let mut wtr = Writer::from_path(&filter_list_args.destination_path).unwrap();
    wtr.write_record(&["pubkey", "amount"]).unwrap();
    for address in full_list.iter() {
        wtr.write_record(&[address.0.to_string(), address.1.to_string()])
            .unwrap();
    }

    wtr.flush().unwrap();
}
