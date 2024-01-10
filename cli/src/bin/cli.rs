extern crate jito_merkle_tree;
extern crate merkle_distributor;
use std::{
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};

use anchor_client::{
    solana_sdk::signer::keypair::read_keypair_file, Client as AnchorClient, Cluster, Program,
};
use anchor_lang::{
    prelude::{Clock, Pubkey},
    solana_program::sysvar,
    AccountDeserialize, InstructionData, Key, ToAccountMetas,
};
use anchor_spl::token::{self, TokenAccount};
use anyhow::Result;
use bincode::deserialize;
use clap::{Parser, Subcommand};
use csv::Writer;
use jito_merkle_tree::{
    airdrop_merkle_tree::AirdropMerkleTree,
    csv_entry::CsvEntry,
    utils::{get_claim_status_pda, get_merkle_distributor_pda},
};
use merkle_distributor::state::merkle_distributor::MerkleDistributor;
use solana_program::{clock::DEFAULT_MS_PER_SLOT, instruction::Instruction};
use solana_rpc_client::rpc_client::{RpcClient, SerializableTransaction};
use solana_sdk::{
    account::Account,
    commitment_config::CommitmentConfig,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,

    /// SPL Mint address
    #[clap(long, env, default_value_t = Pubkey::default())]
    pub mint: Pubkey,

    /// RPC url
    #[clap(long, env, default_value = "http://localhost:8899")]
    pub rpc_url: String,

    /// Program id
    #[clap(long, env, default_value_t = merkle_distributor::id())]
    pub program_id: Pubkey,

    /// Payer keypair
    #[clap(long, env, default_value = "~/.config/solana/id.json")]
    pub keypair_path: String,
}

impl Args {
    fn get_program_client(&self) -> Program<Rc<Keypair>> {
        let payer =
            read_keypair_file(self.keypair_path.clone()).expect("Wallet keypair file not found");
        let client = AnchorClient::new_with_options(
            Cluster::Custom(self.rpc_url.clone(), self.rpc_url.clone()),
            Rc::new(Keypair::from_bytes(&payer.to_bytes()).unwrap()),
            CommitmentConfig::finalized(),
        );
        let program: anchor_client::Program<Rc<Keypair>> =
            client.program(merkle_distributor::id()).unwrap();
        program
    }
}

// Subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Claim unlocked tokens
    Claim(ClaimArgs),
    /// Create a new instance of a merkle distributor
    NewDistributor(NewDistributorArgs),
    CloseDistributor(CloseDistributorArgs),
    /// Clawback tokens from merkle distributor
    #[clap(hide = true)]
    Clawback(ClawbackArgs),
    /// Create a Merkle tree, given a CSV of recipients
    CreateMerkleTree(CreateMerkleTreeArgs),
    SetAdmin(SetAdminArgs),

    SetEnableSlot(SetEnableSlotArgs),
    SetEnableSlotByTime(SetEnableSlotByTimeArgs),

    CreateTestList(CreateTestListArgs),
    CreateDummyCsv(CreateDummyCsv),

    FundAll(FundAllArgs),
    Verify(VerifyArgs),
}

#[derive(Parser, Debug)]
pub struct CloseDistributorArgs {
    #[clap(long, env)]
    pub airdrop_version: u64,
}
// NewClaim and Claim subcommand args
#[derive(Parser, Debug)]
pub struct ClaimArgs {
    /// Merkle distributor path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct FundAllArgs {
    /// Merkle distributor path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct VerifyArgs {
    /// Merkle distributor path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,

    /// When to make the clawback period start. Must be at least a day after the end_vesting_ts
    #[clap(long, env)]
    pub clawback_start_ts: i64,

    #[clap(long, env)]
    pub enable_slot: u64,
}

// NewDistributor subcommand args
#[derive(Parser, Debug)]
pub struct NewDistributorArgs {
    /// Lockup timestamp start
    #[clap(long, env)]
    pub start_vesting_ts: i64,

    /// Lockup timestamp end (unix timestamp)
    #[clap(long, env)]
    pub end_vesting_ts: i64,

    /// Merkle distributor path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,

    /// When to make the clawback period start. Must be at least a day after the end_vesting_ts
    #[clap(long, env)]
    pub clawback_start_ts: i64,

    #[clap(long, env)]
    pub enable_slot: u64,

    #[clap(long, env)]
    pub airdrop_version: Option<u64>,
}

#[derive(Parser, Debug)]
pub struct ClawbackArgs {
    #[clap(long, env)]
    pub clawback_keypair_path: PathBuf,
    #[clap(long, env)]
    pub airdrop_version: u64,
}

#[derive(Parser, Debug)]
pub struct CreateMerkleTreeArgs {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: PathBuf,

    /// Merkle tree out path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,

    /// max nodes per tree
    #[clap(long, env)]
    pub max_nodes_per_tree: u64,
}

#[derive(Parser, Debug)]
pub struct SetAdminArgs {
    #[clap(long, env)]
    pub new_admin: Pubkey,
    #[clap(long, env)]
    pub airdrop_version: u64,
}

#[derive(Parser, Debug)]
pub struct SetEnableSlotArgs {
    /// Merkle tree out path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
    #[clap(long, env)]
    pub slot: u64,
}

#[derive(Parser, Debug)]
pub struct SetEnableSlotByTimeArgs {
    /// Merkle tree out path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
    #[clap(long, env)]
    pub timestamp: u64,
}

#[derive(Parser, Debug)]
pub struct CreateTestListArgs {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: PathBuf,

    /// Merkle tree out path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct CreateDummyCsv {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: String,
    #[clap(long, env)]
    pub num_records: u64,
    #[clap(long, env)]
    pub amount: u64,
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::NewDistributor(new_distributor_args) => {
            process_new_distributor(&args, new_distributor_args);
        }
        Commands::CloseDistributor(close_distributor_args) => {
            process_close_distributor(&args, close_distributor_args);
        }
        Commands::Claim(claim_args) => {
            process_claim(&args, claim_args);
        }
        Commands::Clawback(clawback_args) => process_clawback(&args, clawback_args),
        Commands::CreateMerkleTree(merkle_tree_args) => {
            process_create_merkle_tree(merkle_tree_args);
        }
        Commands::SetAdmin(set_admin_args) => {
            process_set_admin(&args, set_admin_args);
        }
        Commands::SetEnableSlot(set_enable_slot_args) => {
            process_set_enable_slot(&args, set_enable_slot_args);
        }
        Commands::SetEnableSlotByTime(set_enable_slot_by_time_args) => {
            process_set_enable_slot_by_time(&args, set_enable_slot_by_time_args);
        }
        Commands::CreateDummyCsv(test_args) => {
            process_create_dummy_csv(test_args);
        }
        Commands::CreateTestList(create_test_list_args) => {
            process_create_test_list(&args, create_test_list_args);
        }
        Commands::FundAll(fund_all_args) => {
            process_fund_all(&args, fund_all_args);
        }
        Commands::Verify(verfiy_args) => {
            process_verify(&args, verfiy_args);
        }
    }
}

fn process_new_claim(args: &Args, claim_args: &ClaimArgs) {
    let keypair = read_keypair_file(&args.keypair_path).expect("Failed reading keypair file");
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

fn process_close_distributor(args: &Args, close_distributor_args: &CloseDistributorArgs) {
    let (distributor, _bump) = get_merkle_distributor_pda(
        &args.program_id,
        &args.mint,
        close_distributor_args.airdrop_version,
    );
    let program = args.get_program_client();
    let keypair = read_keypair_file(&args.keypair_path).expect("Failed reading keypair file");
    // verify distributor is existed
    let merkle_distributor_state = program.account::<MerkleDistributor>(distributor).unwrap();
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
                close_distributor_args.airdrop_version,
                tx.get_signature(),
            );
        }
        Err(e) => {
            println!(
                "Failed to close MerkleDistributor version {}: {:?}",
                close_distributor_args.airdrop_version, e
            );
        }
    }
}

fn process_claim(args: &Args, claim_args: &ClaimArgs) {
    let keypair = read_keypair_file(&args.keypair_path).expect("Failed reading keypair file");
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

fn check_distributor_onchain_matches(
    account: &Account,
    merkle_tree: &AirdropMerkleTree,
    new_distributor_args: &NewDistributorArgs,
    pubkey: Pubkey,
    args: &Args,
) -> Result<(), &'static str> {
    if let Ok(distributor) = MerkleDistributor::try_deserialize(&mut account.data.as_slice()) {
        if distributor.root != merkle_tree.merkle_root {
            return Err("root mismatch");
        }
        if distributor.max_total_claim != merkle_tree.max_total_claim {
            return Err("max_total_claim mismatch");
        }
        if distributor.max_num_nodes != merkle_tree.max_num_nodes {
            return Err("max_num_nodes mismatch");
        }

        if distributor.start_ts != new_distributor_args.start_vesting_ts {
            return Err("start_ts mismatch");
        }
        if distributor.end_ts != new_distributor_args.end_vesting_ts {
            return Err("end_ts mismatch");
        }
        if distributor.clawback_start_ts != new_distributor_args.clawback_start_ts {
            return Err("clawback_start_ts mismatch");
        }

        if distributor.enable_slot != new_distributor_args.enable_slot {
            return Err("enable_slot mismatch");
        }

        // TODO fix code
        let program = args.get_program_client();
        let clawback_receiver_token_account: TokenAccount =
            program.account(distributor.clawback_receiver).unwrap();

        if clawback_receiver_token_account.owner != distributor.admin {
            return Err("clawback_receiver mismatch");
        }
        if distributor.admin != pubkey {
            return Err("admin mismatch");
        }
    }
    Ok(())
}

fn process_new_distributor(args: &Args, new_distributor_args: &NewDistributorArgs) {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    // println!("{}", &args.keypair_path);
    let keypair = read_keypair_file(&args.keypair_path).expect("Failed reading keypair file");
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
        let token_vault = get_or_create_ata(&program, args.mint, distributor_pubkey).unwrap();
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

        let clawback_receiver = get_or_create_ata(&program, args.mint, keypair.pubkey()).unwrap();

        let new_distributor_ix = Instruction {
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
            }
            .data(),
        };

        let blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[new_distributor_ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            blockhash,
        );

        // See comments on new_distributor instruction inside the program to ensure this transaction
        // didn't get frontrun.
        // If this fails, make sure to run it again.
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
        }

        if new_distributor_args.airdrop_version.is_some() {
            let airdrop_version = new_distributor_args.airdrop_version.unwrap();
            if airdrop_version == merkle_tree.airdrop_version {
                break;
            }
        }
    }
}

fn process_clawback(args: &Args, clawback_args: &ClawbackArgs) {
    let payer_keypair = read_keypair_file(&args.keypair_path).expect("Failed reading keypair file");
    let clawback_keypair = read_keypair_file(&clawback_args.clawback_keypair_path)
        .expect("Failed reading keypair file");

    let clawback_ata = get_associated_token_address(&clawback_keypair.pubkey(), &args.mint);

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

    let (distributor, _bump) =
        get_merkle_distributor_pda(&args.program_id, &args.mint, clawback_args.airdrop_version);

    let from = get_associated_token_address(&distributor, &args.mint);
    println!("from: {from}");

    let clawback_ix = Instruction {
        program_id: args.program_id,
        accounts: merkle_distributor::accounts::Clawback {
            distributor,
            from,
            to: clawback_ata,
            claimant: clawback_keypair.pubkey(),
            system_program: solana_program::system_program::ID,
            token_program: token::ID,
        }
        .to_account_metas(None),
        data: merkle_distributor::instruction::Clawback {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[clawback_ix],
        Some(&payer_keypair.pubkey()),
        &[&payer_keypair, &clawback_keypair],
        client.get_latest_blockhash().unwrap(),
    );

    let signature = client
        .send_and_confirm_transaction_with_spinner(&tx)
        .unwrap();

    println!("Successfully clawed back funds! signature: {signature:#?}");
}

fn process_create_merkle_tree(merkle_tree_args: &CreateMerkleTreeArgs) {
    let mut csv_entries = CsvEntry::new_from_file(&merkle_tree_args.csv_path).unwrap();
    let max_nodes_per_tree = merkle_tree_args.max_nodes_per_tree as usize;

    let base_path = &merkle_tree_args.merkle_tree_path;
    let mut index = 0;
    while csv_entries.len() > 0 {
        let last_index = max_nodes_per_tree.min(csv_entries.len());
        let sub_tree = csv_entries[0..last_index].to_vec();
        csv_entries = csv_entries[last_index..csv_entries.len()].to_vec();

        // use index as version
        let merkle_tree = AirdropMerkleTree::new_from_entries(sub_tree, index).unwrap();

        let base_path_clone = base_path.clone();
        let path = base_path_clone
            .as_path()
            .join(format!("tree_{}.json", index));

        merkle_tree.write_to_file(&path);
        index += 1;
    }
}

fn process_set_admin(args: &Args, set_admin_args: &SetAdminArgs) {
    let keypair = read_keypair_file(&args.keypair_path).expect("Failed reading keypair file");

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

fn process_set_enable_slot(args: &Args, set_enable_slot_args: &SetEnableSlotArgs) {
    let keypair = read_keypair_file(&args.keypair_path).expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

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
            set_enable_slot_args.slot, merkle_tree.airdrop_version
        );
    }
}

fn process_set_enable_slot_by_time(
    args: &Args,
    set_enable_slot_by_time_args: &SetEnableSlotByTimeArgs,
) {
    let keypair = read_keypair_file(&args.keypair_path).expect("Failed reading keypair file");

    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::confirmed());

    let mut paths: Vec<_> = fs::read_dir(&set_enable_slot_by_time_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    let enable_time = set_enable_slot_by_time_args.timestamp;

    let clock_account = client.get_account(&sysvar::clock::id()).unwrap();
    let clock = deserialize::<Clock>(&clock_account.data).unwrap();
    let current_time = u64::try_from(clock.unix_timestamp).unwrap();
    let current_slot = clock.slot;
    let default_slot_time = DEFAULT_MS_PER_SLOT;

    let slot = if enable_time > current_time {
        current_slot + (enable_time - current_time) * 1000 / default_slot_time
    } else {
        current_slot - (current_time - enable_time) * 1000 / default_slot_time
    };

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");

        let (distributor, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);

        let set_admin_ix = Instruction {
            program_id: args.program_id,
            accounts: merkle_distributor::accounts::SetEnableSlot {
                distributor,
                admin: keypair.pubkey(),
            }
            .to_account_metas(None),
            data: merkle_distributor::instruction::SetEnableSlot { enable_slot: slot }.data(),
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
            "Successfully enable slot {slot} timestamp {} airdrop version {}! signature: {signature:#?}",
            enable_time,
            merkle_tree.airdrop_version
        );
    }
}

fn get_pre_list() -> Vec<String> {
    let list = vec![
        "DHLXnJdACTY83yKwnUkeoDjqi4QBbsYGa1v8tJL76ViX",
        "BULRqL3U2jPgwvz6HYCyBVq9BMtK94Y1Nz98KQop23aD",
        "7w32LzRsJrQiE7S3ZSdkz9TSFGey1XNsonPmdm9xDUch",
        "55pPhcCcp8gEKvKWr1JUkAcdwMeemmNhTHmkWNR9sJib",
        "62ucxc2gd5TBCwzToEEWVV4M5drVK7Fi7aYozniqWtac",
        "5unTfT2kssBuNvHPY6LbJfJpLqEcdMxGYLWHwShaeTLi",
        "9zg3seAh4Er1Nz8GAuiciH437apxtzgUWBT8frhudevR",
        "AjefJWRfjRCVNSQ1pHnTW8F7szLV7xFZftiB3yM5vnTa",
        "8SEFruHjgNrnV8ak2Ff11wg9em8Nh72RWTwk359bRyzE",
        "7jBypy9HX1dyLHPnmRnRubibNUaBPrShnERGnoE7rc3C",
        "XWpxVfYTeKmmp18DPxqPvWFL7P1C2vbdegDPAbXkV1n",
        "AuTFdqo4GsxpDgtag87pDaHE259cE94Z82kdpFozVBhC",
        "6h43GsVT3TjtLa5nRpsXp15GDpAY4smWCYHgcq58dSPM",
        "2mAax9cNqDXDg9eDJDby1tBh9Q8N3TS7qLhX9rMp8EVc",
        "JBeYA7dmBGCNgaEdtqdoUnESwKJho5YvgXVNLgo4n3MM",
        "HeTpE5BnNinzNv92MzVAGyVT5LjAwTWuk5qQcPURmi2L",
        "Bidku3jkJUxiTzBJZroEfwPcUWueNUst9LrMbZQLhrtG",
        "HUQytvb7WCCqbHnpQrVgXhmXSw4XfWMnmqCiKz6T1vsU",
        "4zvTjdpyr3SAgLeSpCnq4KaHvX2j5SbkwxYydzbfqhRQ",
        "EVfUfs9XNwJmfNvoazDbZVb6ecnGCxgQrJzsCQHoQ4q7",
        "GMtwcuktJfrRcnyGktWW4Vab8cfjPcBy3xbuZgRegw6E",
    ];
    let list: Vec<String> = list.iter().map(|x| x.to_string()).collect();
    list
}

fn process_create_dummy_csv(args: &CreateDummyCsv) {
    let mut wtr = Writer::from_path(&args.csv_path).unwrap();

    wtr.write_record(&["pubkey", "amount"]).unwrap();

    // add my key
    let pre_list: Vec<String> = get_pre_list();
    let mut full_list = vec![];
    for _i in 0..(args.num_records - pre_list.len() as u64) {
        full_list.push(Pubkey::new_unique().to_string());
    }
    // merge with pre_list
    let num_node = args.num_records.checked_div(pre_list.len() as u64).unwrap() as usize;
    for (i, address) in pre_list.iter().enumerate() {
        full_list.insert(num_node * i, address.clone());
    }

    for address in full_list.iter() {
        wtr.write_record(&[address, &args.amount.to_string()])
            .unwrap();
    }

    wtr.flush().unwrap();
}

fn process_create_test_list(args: &Args, create_test_list_args: &CreateTestListArgs) {
    let pre_list = get_pre_list();
    let mut wtr = Writer::from_path(&create_test_list_args.csv_path).unwrap();
    wtr.write_record(&["pubkey", "amount"]).unwrap();

    for addr in pre_list.iter() {
        wtr.write_record(&[addr, "6000"]).unwrap();
    }
    wtr.flush().unwrap();

    let merkle_tree_args = &CreateMerkleTreeArgs {
        csv_path: create_test_list_args.csv_path.clone(),
        merkle_tree_path: create_test_list_args.merkle_tree_path.clone(),
        max_nodes_per_tree: 10000,
    };
    process_create_merkle_tree(merkle_tree_args);
}

fn process_fund_all(args: &Args, fund_all_args: &FundAllArgs) {
    let client = RpcClient::new_with_commitment(&args.rpc_url, CommitmentConfig::finalized());
    let keypair = read_keypair_file(&args.keypair_path).expect("Failed reading keypair file");
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

        let signature = client
            .send_and_confirm_transaction_with_spinner(&tx)
            .unwrap();

        println!(
            "Successfully transfer {} to merkle tree with airdrop version {}! signature: {signature:#?}",
            merkle_tree.max_total_claim,
            merkle_tree.airdrop_version
        );
    }
}

fn process_verify(args: &Args, verfify_args: &VerifyArgs) {
    let mut paths: Vec<_> = fs::read_dir(&verfify_args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    // TODO fix code
    let program = args.get_program_client();

    for file in paths {
        let single_tree_path = file.path();

        let merkle_tree =
            AirdropMerkleTree::new_from_file(&single_tree_path).expect("failed to read");
        let (distributor_pubkey, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, merkle_tree.airdrop_version);
        let token_vault = get_associated_token_address(&distributor_pubkey, &args.mint);

        let token_vault_account: TokenAccount = program.account(token_vault).unwrap();
        assert_eq!(token_vault_account.amount, merkle_tree.max_total_claim);

        let merke_tree_state: MerkleDistributor = program.account(distributor_pubkey).unwrap();
        assert_eq!(merke_tree_state.root, merkle_tree.merkle_root);

        assert_eq!(
            merke_tree_state.clawback_start_ts,
            verfify_args.clawback_start_ts
        );
        assert_eq!(merke_tree_state.enable_slot, verfify_args.enable_slot);
        println!(
            "done verify merkle tree airdrop version {}",
            merkle_tree.airdrop_version
        );
    }
}

fn get_or_create_ata<C: Deref<Target = impl Signer> + Clone>(
    program_client: &anchor_client::Program<C>,
    token_mint: Pubkey,
    user: Pubkey,
) -> Result<Pubkey> {
    let user_token_account =
        spl_associated_token_account::get_associated_token_address(&user, &token_mint);
    let rpc_client = program_client.rpc();
    if rpc_client.get_account_data(&user_token_account).is_err() {
        println!("Create ATA for TOKEN {} \n", &token_mint);

        let builder = program_client.request().instruction(
            spl_associated_token_account::create_associated_token_account(
                &program_client.payer(),
                &user,
                &token_mint,
            ),
        );

        let signature = builder.send()?;
        println!("{}", signature);
    }
    Ok(user_token_account)
}
