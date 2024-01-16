use solana_sdk::signature::Signature;

use crate::*;

pub fn process_set_enable_slot(args: &Args, set_enable_slot_args: &SetEnableSlotArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());
    let program = args.get_program_client();

    if set_enable_slot_args.airdrop_version.is_some() {
        let airdrop_version = set_enable_slot_args.airdrop_version.unwrap();

        let (distributor, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, airdrop_version);
        let distributor_state = program.account::<MerkleDistributor>(distributor).unwrap();
        if distributor_state.enable_slot == set_enable_slot_args.slot {
            println!("already set slot skip airdrop version {}", airdrop_version);
            return;
        }
        let set_admin_ix = Instruction {
            program_id: args.program_id,
            accounts: merkle_distributor::accounts::SetEnableSlot {
                distributor,
                admin: keypair.pubkey(),
            }
            .to_account_metas(None),
            data: merkle_distributor::instruction::SetEnableSlot {
                enable_slot: set_enable_slot_args.slot,
            }
            .data(),
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
            "Successfully set enable slot {} airdrop version {} ! signature: {signature:#?}",
            set_enable_slot_args.slot, airdrop_version
        );
        return;
    }

    let mut paths: Vec<_> = fs::read_dir(&set_enable_slot_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");

        let (distributor, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);

        loop {
            let distributor_state = program.account::<MerkleDistributor>(distributor).unwrap();
            if distributor_state.enable_slot == set_enable_slot_args.slot {
                println!(
                    "already set slot skip airdrop version {}",
                    merkle_tree.airdrop_version
                );
                break;
            }
            let set_admin_ix = Instruction {
                program_id: args.program_id,
                accounts: merkle_distributor::accounts::SetEnableSlot {
                    distributor,
                    admin: keypair.pubkey(),
                }
                .to_account_metas(None),
                data: merkle_distributor::instruction::SetEnableSlot {
                    enable_slot: set_enable_slot_args.slot,
                }
                .data(),
            };

            let tx = Transaction::new_signed_with_payer(
                &[set_admin_ix],
                Some(&keypair.pubkey()),
                &[&keypair],
                client.get_latest_blockhash().unwrap(),
            );

            match client.send_and_confirm_transaction_with_spinner(&tx) {
                Ok(signature) => {
                    println!(
                        "Successfully set enable slot {} airdrop version {} ! signature: {signature:#?}",
                        set_enable_slot_args.slot, merkle_tree.airdrop_version
                    );
                    break;
                }
                Err(err) => {
                    println!("airdrop version {} {}", merkle_tree.airdrop_version, err);
                }
            }
        }
    }
}
