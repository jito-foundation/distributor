use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

use solana_sdk::{
    commitment_config::CommitmentLevel, compute_budget::ComputeBudgetInstruction,
    signature::Signature,
};

use crate::*;

pub fn parse_send_addresse(path: &PathBuf) -> Result<HashMap<String, String>> {
    let mut file = File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let hashmap = serde_json::from_str(&data)?;
    Ok(hashmap)
}

fn send_for_batch_address(
    args: &Args,
    batch_address: Vec<String>,
) -> Result<(Signature, Vec<String>)> {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap())
        .expect("Failed reading keypair file");
    let source_vault = get_associated_token_address(&keypair.pubkey(), &args.mint);
    let mut mass_ixs = vec![ComputeBudgetInstruction::set_compute_unit_limit(1_000_000)];

    let mut qualified_address = vec![];
    for address in batch_address.iter() {
        match Pubkey::from_str(address) {
            Ok(user) => {
                qualified_address.push(address.clone());
                let user_ata =
                    spl_associated_token_account::get_associated_token_address(&user, &args.mint);

                if client.get_account_data(&user_ata).is_err() {
                    mass_ixs.push(
                        spl_associated_token_account::instruction::create_associated_token_account(
                            &keypair.pubkey(),
                            &user,
                            &args.mint,
                            &spl_token::ID,
                        ),
                    );
                }

                mass_ixs.push(
                    spl_token::instruction::transfer(
                        &spl_token::id(),
                        &source_vault,
                        &user_ata,
                        &keypair.pubkey(),
                        &[],
                        10000000000000,
                    )
                    .unwrap(),
                )
            }
            Err(_) => {
                println!("{} is not pubkey", address);
            }
        }
    }

    let tx = Transaction::new_signed_with_payer(
        &mass_ixs,
        Some(&keypair.pubkey()),
        &[&keypair],
        client.get_latest_blockhash().unwrap(),
    );

    let signature = client.send_transaction(&tx)?;
    Ok((signature, qualified_address))
}

pub fn process_mass_send(args: &Args, mass_send_args: &MassSendArgs) {
    let addresses = parse_new_record(&mass_send_args.csv_path).unwrap();

    let number_of_addresses_per_transaction = mass_send_args.max_address_per_tx as usize;

    let mut sent_addresses = parse_send_addresse(&mass_send_args.des_path).unwrap();

    println!("{:?}", sent_addresses);

    let mut index = 0;
    let mut batch_addresses = vec![];

    let mut wrap_batch_addresses = vec![];

    while index < addresses.len() {
        if let Some(address) = sent_addresses.get(&addresses[index]) {
            // this addresses has been sent, skip
            index += 1;
            continue;
        }
        batch_addresses.push(addresses[index].clone());
        if batch_addresses.len() >= number_of_addresses_per_transaction {
            wrap_batch_addresses.push(batch_addresses.clone());
            batch_addresses = vec![];
        }
        index += 1;
    }

    if batch_addresses.len() > 0 {
        wrap_batch_addresses.push(batch_addresses.clone());
    }
    println!(
        "num record {} {}",
        addresses.len(),
        wrap_batch_addresses.len()
    );

    for batch_address in wrap_batch_addresses.iter() {
        match send_for_batch_address(args, batch_address.clone()) {
            Ok((signature, qualified_address)) => {
                println!("signature {}", signature);
                for address in qualified_address.iter() {
                    sent_addresses.insert(address.clone(), signature.to_string());
                }

                // write it again to cache
                let serialized = serde_json::to_string_pretty(&sent_addresses).unwrap();
                let mut file: File = File::create(&mass_send_args.des_path).unwrap();
                file.write_all(serialized.as_bytes()).unwrap();
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}

pub fn process_resend(args: &Args, resend_args: &ResendSendArgs) {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    let mut sent_addresses = parse_send_addresse(&resend_args.des_path).unwrap();

    let mut signature_to_address: HashMap<String, Vec<String>> = HashMap::new();

    for (key, value) in sent_addresses.iter() {
        if let Some(array_addr) = signature_to_address.get_mut(value) {
            array_addr.push(key.clone());
            // signature_to_address.insert(value, array_addr.clone());
        } else {
            signature_to_address.insert(value.clone(), vec![key.clone()]);
        }
    }

    let number_of_addresses_per_transaction = resend_args.max_address_per_tx as usize;

    for (signature, addresses) in signature_to_address.iter() {
        match client.get_signature_status_with_commitment_and_history(
            &Signature::from_str(signature).unwrap(),
            CommitmentConfig {
                commitment: CommitmentLevel::Finalized,
            },
            true,
        ) {
            Ok(value) => {
                let mut should_resend = false;
                if value.is_none() {
                    println!("{} is not existed resend", signature);
                    should_resend = true;
                } else {
                    match value.unwrap() {
                        Ok(_) => {}
                        Err(err) => {
                            println!("{} is error {}", signature, err);
                            should_resend = true;
                        }
                    }
                }

                if should_resend {
                    let should_send_addresses =
                        if addresses.len() > number_of_addresses_per_transaction {
                            // break it to double, or ignore
                            addresses[0..number_of_addresses_per_transaction].to_vec()
                        } else {
                            addresses.clone()
                        };
                    match send_for_batch_address(args, should_send_addresses) {
                        Ok((signature, qualified_address)) => {
                            println!("signature {}", signature);
                            for address in qualified_address.iter() {
                                sent_addresses.insert(address.clone(), signature.to_string());
                            }

                            let serialized =
                                serde_json::to_string_pretty(&sent_addresses.clone()).unwrap();
                            let mut file: File = File::create(&resend_args.des_path).unwrap();
                            file.write_all(serialized.as_bytes()).unwrap();
                        }
                        Err(err) => {
                            println!("{}", err);
                        }
                    }
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}
