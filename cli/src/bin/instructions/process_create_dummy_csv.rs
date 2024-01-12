use crate::*;

pub fn process_create_dummy_csv(args: &CreateDummyCsv) {
    let mut wtr = Writer::from_path(&args.csv_path).unwrap();

    wtr.write_record(&["pubkey", "amount"]).unwrap();

    // add my key
    let pre_list: Vec<String> = get_pre_list();
    let mut full_list = vec![];
    for _i in 0..(args.num_records - pre_list.len() as u64) {
        full_list.push(Pubkey::new_unique().to_string());
    }
    // merge with pre_list
    let num_node = args.num_records.checked_div(pre_list.len() as u64).unwrap() as usize;
    for (i, address) in pre_list.iter().enumerate() {
        full_list.insert(num_node * i, address.clone());
    }

    for address in full_list.iter() {
        wtr.write_record(&[address, &args.amount.to_string()])
            .unwrap();
    }

    wtr.flush().unwrap();
}
