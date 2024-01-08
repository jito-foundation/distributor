mod error;
mod router;

use std::{fmt::Debug, net::SocketAddr, path::PathBuf, str::FromStr, sync::Arc};

use clap::Parser;
use jito_merkle_tree::{airdrop_merkle_tree::AirdropMerkleTree, utils::get_merkle_distributor_pda};
use router::RouterState;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use tracing::{info, instrument};

use crate::{error::ApiError, router::read_distributor};

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

    /// Airdrop version
    #[clap(long, env)]
    airdrop_version: u64,
    // #[clap(long, env)]
    // enable_proof_endpoint: bool,
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

    let (merkle_distributor, _bump) =
        get_merkle_distributor_pda(&args.program_id, &args.mint, args.airdrop_version);

    let distributor = read_distributor(&rpc_client, &merkle_distributor).await?;

    info!("distributor: {:?}", distributor);

    let state = Arc::new(RouterState {
        tree: AirdropMerkleTree::new_from_file(&args.merkle_tree_path)?.convert_to_hashmap(),
        program_id: args.program_id,
        distributor_pubkey: merkle_distributor,
        rpc_client,
    });

    let app = router::get_routes(state);

    axum::Server::bind(&args.bind_addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
