use crate::*;

pub fn process_set_clawback_receiver(
    args: &Args,
    set_clawback_receiver_args: &ClawbackReceiverArgs,
) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());
    let program = args.get_program_client();

    let mut paths: Vec<_> = fs::read_dir(&set_clawback_receiver_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    let new_clawback_account = spl_associated_token_account::get_associated_token_address(
        &set_clawback_receiver_args.receiver,
        &args.mint,
    );

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");

        let (distributor, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);

        loop {
            let distributor_state = program.account::<MerkleDistributor>(distributor).unwrap();
            if distributor_state.clawback_receiver == new_clawback_account {
                println!(
                    "already the same skip airdrop version {}",
                    merkle_tree.airdrop_version
                );
                break;
            }
            let set_clawback_ix = Instruction {
                program_id: args.program_id,
                accounts: merkle_distributor::accounts::SetClawbackReceiver {
                    distributor,
                    admin: keypair.pubkey(),
                    new_clawback_account,
                }
                .to_account_metas(None),
                data: merkle_distributor::instruction::SetClawbackReceiver {}.data(),
            };

            let tx = Transaction::new_signed_with_payer(
                &[set_clawback_ix],
                Some(&keypair.pubkey()),
                &[&keypair],
                client.get_latest_blockhash().unwrap(),
            );

            match client.send_transaction(&tx) {
                Ok(signature) => {
                    println!(
                        "Successfully set clawback receiver {} airdrop version {} ! signature: {signature:#?}",
                        new_clawback_account, merkle_tree.airdrop_version
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
