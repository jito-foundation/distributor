use crate::*;

pub fn process_claim(args: &Args, claim_args: &ClaimArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");
    let claimant = keypair.pubkey();

    let merkle_tree = AirdropMerkleTree::new_from_file(&claim_args.merkle_tree_path)
        .expect("failed to load merkle tree from file");

    let (distributor, bump) =
        get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);
    println!("distributor pubkey {}", distributor);

    let (claim_status_pda, _bump) = get_claim_status_pda(&args.program_id, &claimant, &distributor);
    println!("claim pda: {claim_status_pda}, bump: {bump}");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

    match client.get_account(&claim_status_pda) {
        Ok(_) => {}
        Err(e) => {
            // TODO: match on the error kind
            if e.to_string().contains("AccountNotFound") {
                println!("PDA does not exist. creating.");
                process_new_claim(args, claim_args);
            } else {
                panic!("error getting PDA: {e}")
            }
        }
    }

    let claimant_ata = get_associated_token_address(&claimant, &args.mint);

    let claim_ix = Instruction {
        program_id: args.program_id,
        accounts: merkle_distributor::accounts::ClaimLocked {
            distributor,
            claim_status: claim_status_pda,
            from: get_associated_token_address(&distributor, &args.mint),
            to: claimant_ata,
            claimant,
            token_program: token::ID,
        }
        .to_account_metas(None),
        data: merkle_distributor::instruction::ClaimLocked {}.data(),
    };

    let blockhash = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[claim_ix],
        Some(&claimant.key()),
        &[&keypair],
        blockhash,
    );

    let signature = client
        .send_and_confirm_transaction_with_spinner(&tx)
        .unwrap();
    println!("successfully claimed tokens with signature {signature:#?}",);
}
