use std::time::Duration;

use axum::{Json, extract::{Path, State}, response::IntoResponse};
use chrono::format::parse;
use distlock::{api::{models::{AcquireRequest, AcquireResponse, ReleaseRequest, RenewRequest}, utils::{change_to_client_id, change_to_lock_id}}, 
lock::types::{ClientId, LeaseId, LockId, LockManager}};
use distlock::lock::types::{AcquireResult};
use crate::AppState;

pub async fn health_check() -> &'static str {
    "Ok"
}
pub async fn acquire_handler(
    State(state): State<AppState>,
    Json(payload): Json<AcquireRequest>,
) -> impl IntoResponse {
    let lock_id = change_to_lock_id(&payload.lock_id);
    let client_id = change_to_client_id(&payload.client_id);
    let ttl = Duration::from_secs(payload.time_to_live);

    let lock_manager = state.lock_manager;

    match lock_manager.try_acquire(&lock_id, &client_id, ttl){
        AcquireResult::Granted { lease_id, expires_at } => {
            return Json(AcquireResponse::Granted {lease_id : lease_id.0, expires_at : expires_at.to_rfc3339() })
        }
        AcquireResult::Queued { position, estimated_wait }=> return Json(AcquireResponse::Queued { position, estimated_wait : estimated_wait.as_secs()}),
        AcquireResult::Error(message) => return Json(AcquireResponse::Error { error_type: "AcquireFailure".to_string(), message })
    }
    
   
}
pub async fn release_handler(
    State(state): State<AppState>,
    Json(payload): Json<ReleaseRequest>,
) -> impl IntoResponse {
    "Release endpoint"
}
pub async fn renew_handler(
    State(state): State<AppState>,
    Json(payload): Json<RenewRequest>,
) -> impl IntoResponse {
    "Renew handler"
}
pub async fn status_handler(
    State(state): State<AppState>,
    Path(lock_id): Path<String>,
) -> impl IntoResponse {
    "Status handler"
}
