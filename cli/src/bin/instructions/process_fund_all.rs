use crate::*;

pub fn process_fund_all(args: &Args, fund_all_args: &FundAllArgs) {
    let program = args.get_program_client();
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");
    let mut paths: Vec<_> = fs::read_dir(&fund_all_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    let source_vault = get_associated_token_address(&keypair.pubkey(), &args.mint);

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");
        let (distributor_pubkey, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);

        let token_vault = get_associated_token_address(&distributor_pubkey, &args.mint);

        let token_vault_state: TokenAccount = program.account(token_vault).unwrap();
        if token_vault_state.amount >= merkle_tree.max_total_claim {
            println!(
                "already fund airdrop version {}!",
                merkle_tree.airdrop_version
            );
            continue;
        }

        let tx = Transaction::new_signed_with_payer(
            &[spl_token::instruction::transfer(
                &spl_token::id(),
                &source_vault,
                &token_vault,
                &keypair.pubkey(),
                &[],
                merkle_tree.max_total_claim,
            )
            .unwrap()],
            Some(&keypair.pubkey()),
            &[&keypair],
            client.get_latest_blockhash().unwrap(),
        );

        let signature = client.send_transaction(&tx).unwrap();

        println!(
            "Successfully transfer {} to merkle tree with airdrop version {}! signature: {signature:#?}",
            merkle_tree.max_total_claim,
            merkle_tree.airdrop_version
        );
    }
}
