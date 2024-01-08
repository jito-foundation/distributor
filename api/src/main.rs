mod error;
mod router;

use std::{
    collections::HashMap, fmt::Debug, fs, net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc,
};

use clap::Parser;
use jito_merkle_tree::{airdrop_merkle_tree::AirdropMerkleTree, utils::get_merkle_distributor_pda};
use router::RouterState;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use tracing::{info, instrument};

use crate::{
    error::ApiError,
    router::{Distributors, SingleDistributor},
};

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Bind address for the server
    #[clap(long, env, default_value_t = SocketAddr::from_str("0.0.0.0:7001").unwrap())]
    bind_addr: SocketAddr,

    /// Path of merkle tree
    #[clap(long, env)]
    merkle_tree_path: PathBuf,

    /// RPC url
    #[clap(long, env)]
    rpc_url: String,

    /// Mint address of token in question
    #[clap(long, env)]
    mint: Pubkey,

    /// Program ID
    #[clap(long, env)]
    program_id: Pubkey,
}

#[tokio::main]
#[instrument]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    tracing_subscriber::fmt().init();

    info!("args: {:?}", args);

    info!("starting server at {}", args.bind_addr);

    let rpc_client = RpcClient::new(args.rpc_url.clone());
    info!("started rpc client at {}", args.rpc_url);

    let mut paths: Vec<_> = fs::read_dir(&args.merkle_tree_path)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    let mut tree = HashMap::new();
    let mut max_num_nodes = 0u64;
    let mut max_total_claim = 0u64;
    let mut distributors = vec![];
    for file in paths {
        let single_tree_path = file.path();
        let single_tree = AirdropMerkleTree::new_from_file(&single_tree_path)?;

        let (distributor_pubkey, _bump) =
            get_merkle_distributor_pda(&args.program_id, &args.mint, single_tree.airdrop_version);

        max_total_claim = max_total_claim
            .checked_add(single_tree.max_total_claim)
            .unwrap();
        max_num_nodes = max_num_nodes
            .checked_add(single_tree.max_num_nodes)
            .unwrap();
        distributors.push(SingleDistributor {
            distributor_pubkey: distributor_pubkey.to_string(),
            // merkle_root: single_tree.merkle_root.clone(),
            airdrop_version: single_tree.airdrop_version,
            max_num_nodes: single_tree.max_num_nodes,
            max_total_claim: single_tree.max_total_claim,
        });
        for node in single_tree.tree_nodes.iter() {
            tree.insert(node.claimant, (distributor_pubkey, node.clone()));
        }
    }

    distributors.sort_unstable_by(|a, b| a.airdrop_version.cmp(&b.airdrop_version));

    let state = Arc::new(RouterState {
        distributors: Distributors {
            max_num_nodes,
            max_total_claim,
            trees: distributors,
        },
        tree,
        program_id: args.program_id,
        rpc_client,
    });

    let app = router::get_routes(state);

    axum::Server::bind(&args.bind_addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
