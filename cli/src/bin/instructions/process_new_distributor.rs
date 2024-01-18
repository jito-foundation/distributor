use crate::*;

pub fn process_new_distributor(args: &Args, new_distributor_args: &NewDistributorArgs) {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    // println!("{}", &args.keypair_path);
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");
    println!("creating new distributor with args: {new_distributor_args:#?}");

    let mut paths: Vec<_> = fs::read_dir(&new_distributor_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());
    let program = args.get_program_client();

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");

        if new_distributor_args.airdrop_version.is_some() {
            let airdrop_version = new_distributor_args.airdrop_version.unwrap();
            if airdrop_version != merkle_tree.airdrop_version {
                continue;
            }
        }
        let (distributor_pubkey, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);

        if let Some(account) = client
            .get_account_with_commitment(&distributor_pubkey, CommitmentConfig::confirmed())
            .unwrap()
            .value
        {
            println!(
                "merkle distributor {} account exists, checking parameters...",
                merkle_tree.airdrop_version
            );
            check_distributor_onchain_matches(
                &account,
                &merkle_tree,
                new_distributor_args,
                keypair.pubkey(),
                &args,
            ).expect("merkle root on-chain does not match provided arguments! Confirm admin and clawback parameters to avoid loss of funds!");
            continue;
        }

        let mut ixs = vec![];
        let token_vault = spl_associated_token_account::get_associated_token_address(
            &distributor_pubkey,
            &args.mint,
        );
        if client.get_account_data(&token_vault).is_err() {
            ixs.push(
                spl_associated_token_account::instruction::create_associated_token_account(
                    &keypair.pubkey(),
                    &distributor_pubkey,
                    &args.mint,
                    &spl_token::ID,
                ),
            );
        }
        let clawback_receiver = get_or_create_ata(&program, args.mint, keypair.pubkey()).unwrap();

        ixs.push(Instruction {
            program_id: args.program_id,
            accounts: merkle_distributor::accounts::NewDistributor {
                clawback_receiver,
                mint: args.mint,
                token_vault,
                distributor: distributor_pubkey,
                system_program: solana_program::system_program::id(),
                associated_token_program: spl_associated_token_account::ID,
                token_program: token::ID,
                admin: keypair.pubkey(),
            }
            .to_account_metas(None),
            data: merkle_distributor::instruction::NewDistributor {
                version: merkle_tree.airdrop_version,
                root: merkle_tree.merkle_root,
                max_total_claim: merkle_tree.max_total_claim,
                max_num_nodes: merkle_tree.max_num_nodes,
                start_vesting_ts: new_distributor_args.start_vesting_ts,
                end_vesting_ts: new_distributor_args.end_vesting_ts,
                clawback_start_ts: new_distributor_args.clawback_start_ts,
                enable_slot: new_distributor_args.enable_slot,
                closable: new_distributor_args.closable,
            }
            .data(),
        });

        let blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&keypair.pubkey()),
            &[&keypair],
            blockhash,
        );

        // See comments on new_distributor instruction inside the program to ensure this transaction
        // didn't get frontrun.
        // If this fails, make sure to run it again.

        if new_distributor_args.skip_verify {
            match client.send_transaction(&tx) {
                Ok(_) => {
                    println!(
                        "done create merkle distributor version {} {:?}",
                        merkle_tree.airdrop_version,
                        tx.get_signature(),
                    );
                }
                Err(e) => {
                    println!("Failed to create MerkleDistributor: {:?}", e);
                }
            }
        } else {
            match client.send_and_confirm_transaction_with_spinner(&tx) {
                Ok(_) => {
                    println!(
                        "done create merkle distributor version {} {:?}",
                        merkle_tree.airdrop_version,
                        tx.get_signature(),
                    );
                }
                Err(e) => {
                    println!("Failed to create MerkleDistributor: {:?}", e);
                }
            }

            // double check someone didn't frontrun this transaction with a malicious merkle root
            if let Some(account) = client
                .get_account_with_commitment(&distributor_pubkey, CommitmentConfig::processed())
                .unwrap()
                .value
            {
                check_distributor_onchain_matches(
                  &account,
                  &merkle_tree,
                  new_distributor_args,
                  keypair.pubkey(),
                  args,
              ).expect("merkle root on-chain does not match provided arguments! Confirm admin and clawback parameters to avoid loss of funds!");
            }
        }

        if new_distributor_args.airdrop_version.is_some() {
            let airdrop_version = new_distributor_args.airdrop_version.unwrap();
            if airdrop_version == merkle_tree.airdrop_version {
                break;
            }
        }
    }
}
