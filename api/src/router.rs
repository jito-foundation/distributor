use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use http::Request;
use jito_merkle_tree::{airdrop_merkle_tree::UserProof, tree_node::TreeNode};
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
    pub program_id: Pubkey,
    pub distributors: Distributors,
    pub tree: HashMap<Pubkey, (Pubkey, TreeNode)>,
}

impl Debug for RouterState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouterState")
            .field("program_id", &self.program_id)
            .field("tree", &self.tree.len())
            .finish()
    }
}

#[instrument]
pub fn get_routes(state: Arc<RouterState>) -> Router {
    let middleware = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(error::handle_error))
        .layer(BufferLayer::new(10000))
        .layer(RateLimitLayer::new(10000, Duration::from_secs(1)))
        .layer(TimeoutLayer::new(Duration::from_secs(20)))
        .layer(LoadShedLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &Request<Body>, _span: &Span| {
                    info!("started {} {}", request.method(), request.uri().path())
                })
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing_core::Level::ERROR)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );

    let router = Router::new()
        .route("/", get(root))
        .route("/distributors", get(get_distributors))
        .route("/user/:user_pubkey", get(get_user_info));

    router.layer(middleware).with_state(state)
}

/// Retrieve the proof for a given user
#[instrument(ret)]
async fn get_user_info(
    State(state): State<Arc<RouterState>>,
    Path(user_pubkey): Path<String>,
) -> Result<impl IntoResponse> {
    let merkle_tree = &state.tree;

    let user_pubkey: Pubkey = Pubkey::from_str(user_pubkey.as_str())?;
    let node = merkle_tree
        .get(&user_pubkey)
        .ok_or(ApiError::UserNotFound(user_pubkey.to_string()))?;

    let proof = UserProof {
        merkle_tree: node.0.to_string(),
        amount: node.1.amount(),
        proof: node
            .1
            .proof
            .to_owned()
            .ok_or(ApiError::ProofNotFound(user_pubkey.to_string()))?,
    };

    Ok(Json(proof))
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SingleDistributor {
    pub distributor_pubkey: String,
    // pub merkle_root: [u8; 32],
    pub airdrop_version: u64,
    pub max_num_nodes: u64,
    pub max_total_claim: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Distributors {
    pub max_num_nodes: u64,
    pub max_total_claim: u64,
    pub trees: Vec<SingleDistributor>,
}

async fn get_distributors(State(state): State<Arc<RouterState>>) -> Result<impl IntoResponse> {
    Ok(Json(state.distributors.clone()))
}

async fn root() -> impl IntoResponse {
    "Jupiter Airdrop API"
}

// #[cfg(test)]
// mod router_test {
//     use std::{collections::HashMap, time, time::Instant};

//     use futures::future::join_all;
//     use hyper::{Body, Client, Method, Request};
//     use hyper_tls::HttpsConnector;
//     use solana_sdk::pubkey::Pubkey;

//     use super::*;

//     #[tokio::test]
//     async fn test_parallel_request() {
//         let url = format!("http://localhost:7001/user/1111sU1sYbe2QWn1rNjUFziHmj6R8Xdt7tj2Q2RFaM",);

//         let https = HttpsConnector::new();
//         let client = Client::builder().build::<_, hyper::Body>(https);

//         let now = time::Instant::now();
//         let mut handles = vec![];
//         for _i in 0..10_000 {
//             handles.push(async {
//                 let result = client.get(url.parse().unwrap()).await;
//                 match result {
//                     Ok(response) => response.status().as_u16() == 200,
//                     Err(_) => false,
//                 }
//             });
//         }
//         let outputs = join_all(handles).await;

//         let mut successes = 0u64;
//         let mut failures = 0u64;
//         for is_ok in outputs {
//             if is_ok {
//                 successes += 1;
//             } else {
//                 failures += 1;
//             }
//         }
//         println!("handle all request took {:.2?}", now.elapsed());
//         println!("success: {}, failures: {}", successes, failures);
//         assert_eq!(0, failures);
//     }
// }
