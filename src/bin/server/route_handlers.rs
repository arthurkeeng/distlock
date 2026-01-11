use std::time::Duration;

use axum::{Json, extract::{Path, State}, response::IntoResponse};
use chrono::format::parse;
use distlock::{api::{models::{AcquireRequest, AcquireResponse, ReleaseRequest, ReleaseResponse, RenewRequest, RenewResponse, StatusResponse}, utils::{change_to_client_id, change_to_lease_id, change_to_lock_id}}, 
lock::{self, types::{ClientId, LeaseId, LockId, LockManager, ReleaseResult, RenewResult}}};
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
             Json(AcquireResponse::Granted {lease_id : lease_id.0, expires_at : expires_at.to_rfc3339() })
        }
        AcquireResult::Queued { position, estimated_wait }=> Json(AcquireResponse::Queued { position, estimated_wait : estimated_wait.as_secs()}),
        AcquireResult::Error(message) => Json(AcquireResponse::Error { error_type: "AcquireFailure".to_string(), message })
    }
    
   
}
pub async fn release_handler(
    State(state): State<AppState>,
    Json(payload): Json<ReleaseRequest>,
) -> impl IntoResponse {
    let lock_manager = state.lock_manager;

    let lock_id = change_to_lock_id(&payload.lock_id);
    let client_id = change_to_client_id(&payload.client_id);
    let lease_id = change_to_lease_id(&payload.lease_id);

    match lock_manager.release(&lock_id, &client_id, &lease_id){
        ReleaseResult::Success => {
            Json(ReleaseResponse::Success)
        }
        ReleaseResult::NotFound => {
            Json(ReleaseResponse::Error { error_type: "NotFound".to_string(), message: "This lock does not exist".to_string() })
        }
        ReleaseResult::NotHolder => {
            Json(ReleaseResponse::Error { error_type: "NotHolder".to_string(), message: "This lock does not exist".to_string() })
        }
        ReleaseResult::Error(message) =>{
            Json(ReleaseResponse::Error { error_type: "ServerError".to_string(), message })
        }
    }
}
pub async fn renew_handler(
    State(state): State<AppState>,
    Json(payload): Json<RenewRequest>,
) -> impl IntoResponse {
    let lock_manager = state.lock_manager;
    let lock_id = change_to_lock_id(&payload.lock_id);
    let lease_id = change_to_lease_id(&payload.lease_id);
    let client_id = change_to_client_id(&payload.client_id);
    let ttl = Duration::from_secs(payload.time_to_live);

    match lock_manager.renew(&lock_id, &client_id, &lease_id, ttl){
        RenewResult::Success { new_expiry } => {
            Json(RenewResponse::Success{new_expiry : new_expiry.to_rfc3339()})
        }
        RenewResult::Expired => {
            Json(RenewResponse::Error { error_type: "Expired".to_string(), message: "Lease expired".to_string() })
        }
        RenewResult::NotFound => {
            Json(RenewResponse::Error { error_type: "NotFound".to_string(), message: "Lock not found".to_string() })
        }
        RenewResult::NotHolder => {
            Json(RenewResponse::Error { error_type: "NotHolder".to_string(), message: "Lock not Holder".to_string() })
        }
        RenewResult::Error(message) => {
            Json(RenewResponse::Error { error_type: "ServerError".to_string(), message })
        }
    }
}
pub async fn status_handler(
    State(state): State<AppState>,
    Path(lock_id): Path<String>,
) -> impl IntoResponse {
    let lock_manager = state.lock_manager;

    match lock_manager.status(&LockId(lock_id)){
        Some(state) => {
            if let Some(holder) = state.holder{
                return Json( StatusResponse::InUse { client_id: holder.client_id.0, expires_at: holder.expires_at.to_rfc3339(), lease_id: holder.lease_id.0, queue_length: state.wait_queue.len(), created_at: state.created_at.to_rfc3339() }
)            }
            return Json(StatusResponse::Free)
        },
        None => {
            return Json(StatusResponse::NotFound)
        }
    }
}
