use anchor_client::solana_client::rpc_filter::{Memcmp, RpcFilterType};
use merkle_distributor::state::claim_status::ClaimStatus;

use crate::*;

pub fn get_total_claim(args: &Args, total_claim_args: &TotalClaimAgrs) {
    let program = args.get_program_client();

    let mut total_node_claimed = 0u64;
    for i in 0..=total_claim_args.num_tree {
        let (distributor_pubkey, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, i);
        let distributor: MerkleDistributor = program.account(distributor_pubkey).unwrap();
        total_node_claimed += distributor.num_nodes_claimed;
    }

    println!("total_node_claimed {}", total_node_claimed);
}

pub fn view_claim_status(args: &Args) {
    let program = args.get_program_client();
    let claim_status_accounts: Vec<(Pubkey, ClaimStatus)> = program
        .accounts(vec![
            RpcFilterType::DataSize((ClaimStatus::LEN) as u64),
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                8 + 32 + 8 + 8 + 8,
                u8::from(true).to_le_bytes().to_vec(),
            )),
        ])
        .unwrap();

    println!("num account {}", claim_status_accounts.len());
}

pub fn process_close_claim_status(args: &Args) {
    let program = args.get_program_client();
    let claim_status_accounts: Vec<(Pubkey, ClaimStatus)> = program
        .accounts(vec![
            RpcFilterType::DataSize((ClaimStatus::LEN) as u64),
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                8 + 32 + 8 + 8 + 8,
                u8::from(true).to_le_bytes().to_vec(),
            )),
        ])
        .unwrap();

    println!("num account {}", claim_status_accounts.len());

    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");
    println!("num accounts {}", claim_status_accounts.len());

    let numer_of_close_ix_per_transaction = 11;
    let mut claim_status_accounts_iter = claim_status_accounts.iter();
    let mut current_status_account = claim_status_accounts_iter.next();

    let mut close_ixs = vec![];
    loop {
        if let Some(value) = current_status_account {
            close_ixs.push(Instruction {
                program_id: args.program_id,
                accounts: merkle_distributor::accounts::CloseClaimStatus {
                    admin: keypair.pubkey(),
                    claimant: value.1.claimant,
                    claim_status: value.0,
                }
                .to_account_metas(None),
                data: merkle_distributor::instruction::CloseClaimStatus {}.data(),
            });

            if close_ixs.len() >= numer_of_close_ix_per_transaction {
                let client =
                    RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
                let blockhash = client.get_latest_blockhash().unwrap();
                let tx = Transaction::new_signed_with_payer(
                    &close_ixs,
                    Some(&keypair.pubkey()),
                    &[&keypair],
                    blockhash,
                );
                match client.send_transaction(&tx) {
                    Ok(_) => {
                        println!("done close claim status {}", tx.get_signature());
                        close_ixs = vec![];
                    }
                    Err(e) => {
                        println!("Failed to close claim status account");
                    }
                }
            }
            current_status_account = claim_status_accounts_iter.next();
        } else {
            break;
        }
    }

    if close_ixs.len() > 0 {
        let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
        let blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &close_ixs,
            Some(&keypair.pubkey()),
            &[&keypair],
            blockhash,
        );
        match client.send_and_confirm_transaction_with_spinner(&tx) {
            Ok(_) => {
                println!("done close claim status {}", tx.get_signature());
            }
            Err(e) => {
                println!("Failed to close claim status account");
            }
        }
    }
}
