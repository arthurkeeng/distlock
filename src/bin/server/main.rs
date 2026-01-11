

pub mod route_handlers;
use std::sync::{Arc};

use axum::{Router, routing::{get, post}};
use distlock::{lock::manager::InMemoryLockManager, raft::{node::RaftNode, raft_client::RaftClient, raft_commands::{CommandResponse, LockCommand}}};

use route_handlers::{acquire_handler, health_check, release_handler, renew_handler, status_handler};
use tokio::sync::{mpsc, oneshot::{ Sender, channel}};

#[derive(Clone)]
pub struct AppState {
    pub lock_manager : Arc<InMemoryLockManager>
}
#[tokio::main]

async fn main(){
    tracing_subscriber::fmt::init();
    let (command_tx  , command_rx)= mpsc::channel::<(LockCommand , oneshot::Sender<CommandResponse>)>(100);
    // let (command_tx , command_rx) = m
    let lock_manager = Arc::new(InMemoryLockManager::new());

    let peers = vec![];
    let raft_client = RaftClient::new(command_tx);

    let raft_node = RaftNode::new(1, peers , command_rx);

    tokio::spawn(async move {
        raft_node.run().await
    });

    let state = AppState{
        lock_manager 
    };
    let app = Router::new()
    .route("/",get(health_check))
    .route("/acquire",post(acquire_handler))
    .route("/release",post(release_handler))
    .route("/renew",post(renew_handler))
    .route("/status/:lock_id",get(status_handler))
    .with_state(state);
    let addr = "0.0.0.0:3000";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Listening on {}",addr);
    axum::serve(listener , app).await.unwrap();
}
