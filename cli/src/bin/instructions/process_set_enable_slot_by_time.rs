use anyhow::Ok;

use crate::*;

pub fn process_set_enable_slot_by_time(
    args: &Args,
    set_enable_slot_by_time_args: &SetEnableSlotByTimeArgs,
) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

    let mut paths: Vec<_> = fs::read_dir(&set_enable_slot_by_time_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    let enable_time = set_enable_slot_by_time_args.timestamp;

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
    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");

        let (distributor, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);

        let set_admin_ix = Instruction {
            program_id: args.program_id,
            accounts: merkle_distributor::accounts::SetEnableSlot {
                distributor,
                admin: keypair.pubkey(),
            }
            .to_account_metas(None),
            data: merkle_distributor::instruction::SetEnableSlot { enable_slot: slot }.data(),
        };

        let tx = Transaction::new_signed_with_payer(
            &[set_admin_ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            client.get_latest_blockhash().unwrap(),
        );

        let signature = client
            .send_and_confirm_transaction_with_spinner(&tx)
            .unwrap();

        println!(
            "Successfully enable slot {slot} timestamp {} airdrop version {}! signature: {signature:#?}",
            enable_time,
            merkle_tree.airdrop_version
        );
    }
}

fn get_average_slot_time(client: &RpcClient) -> Result<u64> {
    let samples = client.get_recent_performance_samples(Some(60))?;
    let num_samples = samples.len() as u64;
    if num_samples == 0 {
        println!("num sample is zero, use default time");
        return Ok(DEFAULT_MS_PER_SLOT);
    }

    let mut total_time = 0;
    for sample in samples.iter() {
        total_time = total_time + sample.sample_period_secs as u64 * 1000 / sample.num_slots;
    }

    let average_time = total_time / num_samples;
    // sanity check
    if average_time < DEFAULT_MS_PER_SLOT / 2 && average_time > DEFAULT_MS_PER_SLOT * 2 {
        println!("average_time is passed sanity check {}", average_time);
        return Ok(DEFAULT_MS_PER_SLOT);
    }
    Ok(average_time)
}
