use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anchor_lang::AccountDeserialize;
use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use http::Request;
use jito_merkle_tree::{tree_node::TreeNode, utils::get_claim_status_pda};
use merkle_distributor::state::{
    claim_status::ClaimStatus as MerkleDistributorClaimStatus,
    merkle_distributor::MerkleDistributor,
};
use serde_derive::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use tower::{
    buffer::BufferLayer, limit::RateLimitLayer, load_shed::LoadShedLayer, timeout::TimeoutLayer,
    ServiceBuilder,
};
use tower_http::{
    trace::{DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{info, instrument, warn, Span};

use crate::{error, error::ApiError, Result};

pub struct RouterState {
    pub distributor_pubkey: Pubkey, // Merkle Distributor PDA pubkey
    pub program_id: Pubkey,
    pub rpc_client: RpcClient,
    pub tree: HashMap<Pubkey, TreeNode>,
}

impl Debug for RouterState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouterState")
            .field("program_id", &self.program_id)
            .field("tree", &self.tree.len())
            .field("rpc_client", &self.rpc_client.url())
            .finish()
    }
}

#[instrument]
pub fn get_routes(state: Arc<RouterState>, enable_proof_endpoint: bool) -> Router {
    let middleware = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(error::handle_error)) // handle middleware errors explicitly!
        .layer(BufferLayer::new(100)) // buffer up to 100 requests in queue
        .layer(RateLimitLayer::new(1000, Duration::from_secs(10)))
        .layer(TimeoutLayer::new(Duration::from_secs(20)))
        .layer(LoadShedLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &Request<Body>, _span: &Span| {
                    info!("started {} {}", request.method(), request.uri().path())
                })
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing_core::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );

    let mut router = Router::new()
        .route("/", get(root))
        .route("/users", get(get_users))
        .route("/distributor", get(get_distributor))
        .route("/status/:user_pubkey", get(get_status))
        .route("/version", get(get_version));

    // don't enable until airdrop starts
    if enable_proof_endpoint {
        router = router.route("/proof/:user_pubkey", get(get_proof));
    }

    router.layer(middleware).with_state(state)
}

/// Retrieve the proof for a given user
#[instrument(ret)]
async fn get_proof(
    State(state): State<Arc<RouterState>>,
    Path(user_pubkey): Path<String>,
) -> Result<impl IntoResponse> {
    let merkle_tree = &state.tree;

    let user_pubkey: Pubkey = Pubkey::from_str(user_pubkey.as_str())?;
    let node = merkle_tree
        .get(&user_pubkey)
        .ok_or(ApiError::UserNotFound(user_pubkey.to_string()))?;

    let proof = Proof {
        amount_locked: node.amount_locked(),
        amount_unlocked: node.amount_unlocked(),
        proof: node
            .proof
            .to_owned()
            .ok_or(ApiError::ProofNotFound(user_pubkey.to_string()))?,
    };

    Ok(Json(proof))
}

/// Get the claim status account for the user, if it doesn't exist then Unclaimed
#[instrument(skip(state, node), ret, err)]
async fn get_claim_status(
    state: &RouterState,
    node: &TreeNode,
    user_pubkey: &Pubkey,
) -> Result<ClaimStatus> {
    let (claim_status_pda, _bump) =
        get_claim_status_pda(&state.program_id, user_pubkey, &state.distributor_pubkey);

    let mut accounts = state
        .rpc_client
        .get_multiple_accounts(&[claim_status_pda, state.distributor_pubkey])
        .await?;

    // Note: this method will return an error if the distributor isn't on-chain, even if the TreeNode exists.
    // This may happen in situations where one is previewing the airdrop amounts, but hasn't finalized the distributor on-chain.
    // TODO (LB): is that okay behavior?
    let distributor_account = accounts
        .pop()
        .ok_or_else(|| ApiError::InternalError)?
        .ok_or_else(|| ApiError::MerkleDistributorError("distributor not found".into()))?;
    let distributor = MerkleDistributor::try_deserialize(&mut distributor_account.data.as_slice())
        .map_err(|e| {
            warn!("error deserializing MerkleDistributor: {:?}", e);
            ApiError::MerkleDistributorError("Error parsing MerkleDistributor".into())
        })?;

    let claim_status_account = accounts.pop().ok_or_else(|| ApiError::InternalError)?;

    if distributor.clawed_back {
        match claim_status_account {
            Some(claim_status_account) => {
                match MerkleDistributorClaimStatus::try_deserialize(
                    &mut claim_status_account.data.as_slice(),
                ) {
                    // claimed some, but its expired now, so no more locked funds can be withdrawn
                    Ok(claim_status) => Ok(ClaimStatus {
                        status: Status::Expired,
                        total_unlocked_staker: node.total_unlocked_staker,
                        total_locked_staker: node.total_locked_staker,
                        total_unlocked_searcher: node.total_unlocked_searcher,
                        total_locked_searcher: node.total_locked_searcher,
                        total_unlocked_validator: node.total_unlocked_validator,
                        total_locked_validator: node.total_locked_validator,
                        amount_locked_withdrawable: 0, // expired, too bad
                        amount_locked_withdrawn: claim_status.locked_amount_withdrawn,
                    }),
                    // account parsing error, assume they didn't claim.
                    // let them know what could have been but it's expired
                    Err(e) => {
                        warn!("error reading ClaimStatus: {:?}", e);
                        Ok(ClaimStatus {
                            status: Status::Expired,
                            total_unlocked_staker: node.total_unlocked_staker,
                            total_locked_staker: node.total_locked_staker,
                            total_unlocked_searcher: node.total_unlocked_searcher,
                            total_locked_searcher: node.total_locked_searcher,
                            total_unlocked_validator: node.total_unlocked_validator,
                            total_locked_validator: node.total_locked_validator,
                            amount_locked_withdrawable: 0, // expired, too bad
                            amount_locked_withdrawn: 0, /* never withdrew any because account doesn't exist */
                        })
                    }
                }
            }
            // never claimed, let them know what could have been but it's expired now
            None => Ok(ClaimStatus {
                status: Status::Expired,
                total_unlocked_staker: node.total_unlocked_staker,
                total_locked_staker: node.total_locked_staker,
                total_unlocked_searcher: node.total_unlocked_searcher,
                total_locked_searcher: node.total_locked_searcher,
                total_unlocked_validator: node.total_unlocked_validator,
                total_locked_validator: node.total_locked_validator,
                amount_locked_withdrawable: 0,
                amount_locked_withdrawn: 0,
            }),
        }
    } else {
        match claim_status_account {
            Some(claim_status_account) => {
                match MerkleDistributorClaimStatus::try_deserialize(
                    &mut claim_status_account.data.as_slice(),
                ) {
                    // claimed, but might still have some locked tokens
                    Ok(claim_status) => Ok(ClaimStatus {
                        status: Status::Claimed,
                        total_unlocked_staker: node.total_unlocked_staker,
                        total_locked_staker: node.total_locked_staker,
                        total_unlocked_searcher: node.total_unlocked_searcher,
                        total_locked_searcher: node.total_locked_searcher,
                        total_unlocked_validator: node.total_unlocked_validator,
                        total_locked_validator: node.total_locked_validator,
                        amount_locked_withdrawable: claim_status
                            .amount_withdrawable(
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() as i64,
                                distributor.start_ts,
                                distributor.end_ts,
                            )
                            .unwrap(),
                        amount_locked_withdrawn: claim_status.locked_amount_withdrawn,
                    }),
                    // error parsing account, assume didn't claim. might have locked tokens too, so simulate the amount withdrawable
                    Err(e) => {
                        warn!("error reading ClaimStatus: {:?}", e);
                        Ok(ClaimStatus {
                            status: Status::Unclaimed,
                            total_unlocked_staker: node.total_unlocked_staker,
                            total_locked_staker: node.total_locked_staker,
                            total_unlocked_searcher: node.total_unlocked_searcher,
                            total_locked_searcher: node.total_locked_searcher,
                            total_unlocked_validator: node.total_unlocked_validator,
                            total_locked_validator: node.total_locked_validator,
                            amount_locked_withdrawable: MerkleDistributorClaimStatus {
                                locked_amount: node
                                    .total_locked_staker
                                    .checked_add(node.total_locked_searcher)
                                    .unwrap()
                                    .checked_add(node.total_locked_validator)
                                    .unwrap(),
                                // haven't claimed yet, so none withdrawn
                                locked_amount_withdrawn: 0,
                                // the rest of the fields don't matter
                                ..MerkleDistributorClaimStatus::default()
                            }
                            .amount_withdrawable(
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() as i64,
                                distributor.start_ts,
                                distributor.end_ts,
                            )
                            .unwrap(),
                            amount_locked_withdrawn: 0, /* never withdrew any because account doesn't exist */
                        })
                    }
                }
            }
            None => {
                // haven't claimed yet. might have some claimable tokens
                Ok(ClaimStatus {
                    status: Status::Unclaimed,
                    total_unlocked_staker: node.total_unlocked_staker,
                    total_locked_staker: node.total_locked_staker,
                    total_unlocked_searcher: node.total_unlocked_searcher,
                    total_locked_searcher: node.total_locked_searcher,
                    total_unlocked_validator: node.total_unlocked_validator,
                    total_locked_validator: node.total_locked_validator,
                    amount_locked_withdrawable: MerkleDistributorClaimStatus {
                        locked_amount: node
                            .total_locked_staker
                            .checked_add(node.total_locked_searcher)
                            .unwrap()
                            .checked_add(node.total_locked_validator)
                            .unwrap(),
                        // haven't claimed yet, so none withdrawn
                        locked_amount_withdrawn: 0,
                        // the rest of the fields don't matter
                        ..MerkleDistributorClaimStatus::default()
                    }
                    .amount_withdrawable(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                        distributor.start_ts,
                        distributor.end_ts,
                    )
                    .unwrap(),
                    amount_locked_withdrawn: 0, // never withdrew any because account doesn't exist
                })
            }
        }
    }
}

/// Fetch and deserialize merkle distributor information
pub async fn read_distributor(
    rpc_client: &RpcClient,
    distributor: &Pubkey,
) -> Result<MerkleDistributor> {
    return match rpc_client.get_account_data(distributor).await {
        Ok(data) => {
            return match MerkleDistributor::try_deserialize(&mut data.as_slice()) {
                Ok(distributor) => Ok(distributor),
                Err(e) => Err(ApiError::MerkleDistributorError(e.to_string())),
            };
        }
        Err(e) => {
            if e.kind.to_string().contains("AccountNotFound") {
                Err(ApiError::MerkleDistributorError(
                    "Merkle Distributor not found".to_string(),
                ))
            } else {
                Err(ApiError::RpcError(e))
            }
        }
    };
}

#[instrument(ret)]
async fn get_users(State(state): State<Arc<RouterState>>) -> Result<impl IntoResponse> {
    let merkle_tree = state.tree.to_owned();
    let users: Vec<String> = merkle_tree.into_keys().map(|key| key.to_string()).collect();

    Ok(Json(users))
}

#[derive(Serialize, Deserialize)]
struct Distributor {
    /// Public key of this distributor
    pub pubkey: Pubkey,
    /// Program ID this distributor belongs to
    pub program_id: Pubkey,

    /// MerkleDistributor fields

    /// Version of the airdrop
    pub version: u64,
    /// The 256-bit merkle root.
    pub root: [u8; 32],
    /// [Mint] of the token to be distributed.
    pub mint: Pubkey,
    /// Token Address of the vault
    pub token_vault: Pubkey,
    /// Maximum number of tokens that can ever be claimed from this [MerkleDistributor].
    pub max_total_claim: u64,
    /// Maximum number of nodes in [MerkleDistributor].
    pub max_num_nodes: u64,
    /// Total amount of tokens that have been claimed.
    pub total_amount_claimed: u64,
    /// Number of nodes that have been claimed.
    pub num_nodes_claimed: u64,
    /// Lockup time start (Unix Timestamp)
    pub start_ts: i64,
    /// Lockup time end (Unix Timestamp)
    pub end_ts: i64,
    /// Clawback start (Unix Timestamp)
    pub clawback_start_ts: i64,
    /// Clawback receiver
    pub clawback_receiver: Pubkey,
    /// Admin wallet
    pub admin: Pubkey,
    /// Whether or not the distributor has been clawed back
    pub clawed_back: bool,
}

async fn get_distributor(State(state): State<Arc<RouterState>>) -> Result<Json<Distributor>> {
    let d = read_distributor(&state.rpc_client, &state.distributor_pubkey).await?;
    Ok(Json(Distributor {
        pubkey: state.distributor_pubkey,
        program_id: state.program_id,
        version: d.version,
        root: d.root,
        mint: d.mint,
        token_vault: d.token_vault,
        max_total_claim: d.max_total_claim,
        max_num_nodes: d.max_num_nodes,
        total_amount_claimed: d.total_amount_claimed,
        num_nodes_claimed: d.num_nodes_claimed,
        start_ts: d.start_ts,
        end_ts: d.end_ts,
        clawback_start_ts: d.clawback_start_ts,
        clawback_receiver: d.clawback_receiver,
        admin: d.admin,
        clawed_back: d.clawed_back,
    }))
}

// TODO: This endpoint is likely worth caching as we make many outbound rpc requests to retrieve the same information
#[instrument(ret)]
async fn get_status(
    State(state): State<Arc<RouterState>>,
    Path(user_pubkey): Path<String>,
) -> Result<impl IntoResponse> {
    let merkle_tree = &state.tree;

    let user_pubkey: Pubkey = Pubkey::from_str(user_pubkey.as_str())?;
    let node = merkle_tree
        .get(&user_pubkey)
        .ok_or(ApiError::UserNotFound(user_pubkey.to_string()))?;

    Ok(Json(get_claim_status(&state, node, &user_pubkey).await?))
}

/// Gets the current airdrop version
#[instrument(ret)]
async fn get_version(State(state): State<Arc<RouterState>>) -> Result<impl IntoResponse> {
    Ok(Json(
        read_distributor(&state.rpc_client, &state.distributor_pubkey)
            .await?
            .version,
    ))
}

async fn root() -> impl IntoResponse {
    "Jito Airdrop API"
}

#[derive(Serialize, Deserialize, Debug)]
struct Proof {
    pub amount_locked: u64,
    pub amount_unlocked: u64,
    pub proof: Vec<[u8; 32]>,
}

#[derive(Serialize, Deserialize, Debug)]
enum Status {
    Unclaimed, // User has not yet claimed any tokens
    Claimed,   // User already claimed unlocked tokens
    Expired,   // claim period has expired
}

#[derive(Serialize, Deserialize, Debug)]
struct ClaimStatus {
    pub status: Status,
    pub total_unlocked_staker: u64,
    pub total_locked_staker: u64,
    pub total_unlocked_searcher: u64,
    pub total_locked_searcher: u64,
    pub total_unlocked_validator: u64,
    pub total_locked_validator: u64,
    pub amount_locked_withdrawable: u64,
    pub amount_locked_withdrawn: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Assert that serializing status to json response works as expected
    #[test]
    fn test_serialize_status() {
        let status = Status::Claimed;
        let claim_status = ClaimStatus {
            status,
            total_unlocked_searcher: 100,
            total_locked_searcher: 100,
            total_unlocked_staker: 0,
            total_locked_staker: 0,
            total_unlocked_validator: 0,
            total_locked_validator: 0,
            amount_locked_withdrawable: 100,
            amount_locked_withdrawn: 0,
        };

        let json = serde_json::to_string(&claim_status).unwrap();
        println!("json: {}", json);
    }
}
