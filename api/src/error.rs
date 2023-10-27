use std::convert::Infallible;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    BoxError, Json,
};
use jito_merkle_tree::error::MerkleTreeError;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use solana_program::pubkey::ParsePubkeyError;
use solana_rpc_client_api::client_error::Error as RpcError;
use thiserror::Error;
use tracing::log::error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Merkle Tree Validation Error: {0}")]
    MerkleTreeError(#[from] MerkleTreeError),

    #[error("User {0} not found")]
    UserNotFound(String),

    #[error("Rpc Error")]
    RpcError(#[from] RpcError),

    #[error("Proof not found for user{0}")]
    ProofNotFound(String),

    #[error("Parse Pubkey Error")]
    ParsePubkeyError(#[from] ParsePubkeyError),

    #[error("Vesting Error")]
    VestingError(String),

    #[error("Merkle Distributor Error")]
    MerkleDistributorError(String),

    #[error("Internal Error")]
    InternalError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub error: String,
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::MerkleTreeError(_) => {
                error!("Merkle Tree Error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            ApiError::UserNotFound(s) => {
                error!("User {s} not found");
                (StatusCode::NOT_FOUND, "User not found")
            }
            ApiError::ProofNotFound(u) => {
                error!("Proof not found for user {u}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Proof not found")
            }

            ApiError::ParsePubkeyError(e) => {
                error!("Parse pubkey error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Pubkey parse error")
            }
            ApiError::RpcError(e) => {
                error!("Rpc error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Rpc error")
            }
            ApiError::VestingError(e) => {
                error!("Vesting error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Vesting error")
            }

            ApiError::MerkleDistributorError(e) => {
                error!("Merkle Distributor error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            ApiError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        };
        (
            status,
            Json(Error {
                error: error_message.to_string(),
            }),
        )
            .into_response()
    }
}

pub async fn handle_error(error: BoxError) -> Result<impl IntoResponse, Infallible> {
    if error.is::<tower::timeout::error::Elapsed>() {
        return Ok((
            StatusCode::REQUEST_TIMEOUT,
            Json(json!({
                "code" : 408,
                "error" : "Request Timeout",
            })),
        ));
    };
    if error.is::<tower::load_shed::error::Overloaded>() {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "code" : 503,
                "error" : "Service Unavailable",
            })),
        ));
    }

    Ok((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "code" : 500,
            "error" : "Internal Server Error",
        })),
    ))
}
