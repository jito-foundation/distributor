use crate::*;

pub fn process_get_slot(args: &Args, slot_by_time_args: &SlotByTimeArgsArgs) {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());
    let enable_time = slot_by_time_args.timestamp;

    let clock_account = client.get_account(&sysvar::clock::id()).unwrap();
    let clock = deserialize::<Clock>(&clock_account.data).unwrap();
    let current_time = u64::try_from(clock.unix_timestamp).unwrap();
    let current_slot = clock.slot;
    let average_slot_time = get_average_slot_time(&client).unwrap();

    println!("average slot time {}", average_slot_time);

    let slot = if enable_time > current_time {
        current_slot + (enable_time - current_time) * 1000 / average_slot_time
    } else {
        current_slot - (current_time - enable_time) * 1000 / average_slot_time
    };

    println!("slot activate {}", slot);
}
