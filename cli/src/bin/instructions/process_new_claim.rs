use crate::*;

pub fn process_new_claim(args: &Args, claim_args: &ClaimArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");
    let claimant = keypair.pubkey();
    println!("Claiming tokens for user {}...", claimant);

    let merkle_tree = AirdropMerkleTree::new_from_file(&claim_args.merkle_tree_path)
        .expect("failed to load merkle tree from file");

    let (distributor, _bump) =
        get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);

    // Get user's node in claim
    let node = merkle_tree.get_node(&claimant);

    let (claim_status_pda, _bump) = get_claim_status_pda(&args.program_id, &claimant, &distributor);

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

    let claimant_ata = get_associated_token_address(&claimant, &args.mint);

    let mut ixs = vec![];

    match client.get_account(&claimant_ata) {
        Ok(_) => {}
        Err(e) => {
            // TODO: directly pattern match on error kind
            if e.to_string().contains("AccountNotFound") {
                println!("PDA does not exist. creating.");
                let ix =
                    create_associated_token_account(&claimant, &claimant, &args.mint, &token::ID);
                ixs.push(ix);
            } else {
                panic!("Error fetching PDA: {e}")
            }
        }
    }

    let new_claim_ix = Instruction {
        program_id: args.program_id,
        accounts: merkle_distributor::accounts::NewClaim {
            distributor,
            claim_status: claim_status_pda,
            from: get_associated_token_address(&distributor, &args.mint),
            to: claimant_ata,
            claimant,
            token_program: token::ID,
            system_program: solana_program::system_program::ID,
        }
        .to_account_metas(None),
        data: merkle_distributor::instruction::NewClaim {
            amount_unlocked: node.amount(),
            amount_locked: 0,
            proof: node.proof.expect("proof not found"),
        }
        .data(),
    };

    ixs.push(new_claim_ix);

    let blockhash = client.get_latest_blockhash().unwrap();
    let tx =
        Transaction::new_signed_with_payer(&ixs, Some(&claimant.key()), &[&keypair], blockhash);

    let signature = client
        .send_and_confirm_transaction_with_spinner(&tx)
        .unwrap();
    println!("successfully created new claim with signature {signature:#?}");
}
