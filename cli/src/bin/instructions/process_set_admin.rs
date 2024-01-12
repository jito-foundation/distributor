use crate::*;

pub fn process_set_admin(args: &Args, set_admin_args: &SetAdminArgs) {
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

    let (distributor, _bump) =
        get_merkle_distributor_pda(&args.program_id, &args.mint, set_admin_args.airdrop_version);

    let set_admin_ix = Instruction {
        program_id: args.program_id,
        accounts: merkle_distributor::accounts::SetAdmin {
            distributor,
            admin: keypair.pubkey(),
            new_admin: set_admin_args.new_admin,
        }
        .to_account_metas(None),
        data: merkle_distributor::instruction::SetAdmin {}.data(),
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

    println!("Successfully set admin! signature: {signature:#?}");
}
