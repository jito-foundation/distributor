use crate::*;

pub fn process_close_distributor(args: &Args, close_distributor_args: &CloseDistributorArgs) {
    let mut paths: Vec<_> = fs::read_dir(&close_distributor_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");

        if close_distributor_args.airdrop_version.is_some() {
            let airdrop_version = close_distributor_args.airdrop_version.unwrap();
            if airdrop_version != merkle_tree.airdrop_version {
                continue;
            }
        }

        let (distributor, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);
        let program = args.get_program_client();
        let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
            .expect("Failed reading keypair file");
        // verify distributor is existed
        let merkle_distributor_state = program.account::<MerkleDistributor>(distributor);
        if merkle_distributor_state.is_err() {
            println!("skip version {}", merkle_tree.airdrop_version);
            continue;
        }
        let merkle_distributor_state = merkle_distributor_state.unwrap();

        let destination_token_account =
            get_or_create_ata(&program, args.mint, keypair.pubkey()).unwrap();

        let close_distributor_ix = Instruction {
            program_id: args.program_id,
            accounts: merkle_distributor::accounts::CloseDistributor {
                distributor,
                token_vault: merkle_distributor_state.token_vault,
                admin: keypair.pubkey(),
                destination_token_account,
                token_program: spl_token::ID,
            }
            .to_account_metas(None),
            data: merkle_distributor::instruction::CloseDistributor {}.data(),
        };
        let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
        let blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[close_distributor_ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            blockhash,
        );
        match client.send_and_confirm_transaction_with_spinner(&tx) {
            Ok(_) => {
                println!(
                    "done close merkle distributor version {} {:?}",
                    merkle_tree.airdrop_version,
                    tx.get_signature(),
                );
            }
            Err(e) => {
                println!(
                    "Failed to close MerkleDistributor version {}: {:?}",
                    merkle_tree.airdrop_version, e
                );
            }
        }

        if close_distributor_args.airdrop_version.is_some() {
            let airdrop_version = close_distributor_args.airdrop_version.unwrap();
            if airdrop_version == merkle_tree.airdrop_version {
                break;
            }
        }
    }
}
