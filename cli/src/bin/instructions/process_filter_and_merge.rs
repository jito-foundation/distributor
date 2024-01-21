use crate::{fs::File, *};

pub fn parse_new_record(path: &PathBuf) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut entries = Vec::new();
    for result in rdr.deserialize() {
        let record: String = result.unwrap();
        entries.push(record);
    }

    Ok(entries)
}

pub fn process_filter_and_merge(filter_list_args: &FilterAndMergeListArgs) {
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

    // push sub node
    let new_records = parse_new_record(&filter_list_args.sub_path).unwrap();
    println!("sub list {}", new_records.len());
    for node in new_records.iter() {
        let addr = Pubkey::from_str(&node);
        if addr.is_err() {
            println!("{} is not pubkey", node);
            continue;
        }
        full_list.push((addr.unwrap(), filter_list_args.amount));
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
