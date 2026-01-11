

use raft::{Config, RawNode, default_logger};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use tokio::sync::{RwLock, mpsc, oneshot};

use crate::{lock::{manager::InMemoryLockManager, types::{AcquireResult, ClientId, LeaseId, LockId, LockManager, ReleaseResult, RenewResult}}, raft::{raft_client::RaftClient, raft_commands::{CommandResponse, LockCommand}, storage::DistlockStorage}};

pub struct RaftNode {

    raft : RawNode<DistlockStorage>, 
    storage : DistlockStorage,

    state_machine : Arc<RwLock<InMemoryLockManager>>, 

    id : u64 , 

    peers : Vec<u64> , 
    command_rx: mpsc::Receiver<(LockCommand , oneshot::Sender<CommandResponse>)>,
    pending_maps :  Mutex<HashMap<u64 , oneshot::Sender<CommandResponse>>>
}

impl RaftNode {
    pub fn new(id : u64 , peers : Vec<u64> , command_rx :mpsc::Receiver<(LockCommand , oneshot::Sender<CommandResponse>)> ) -> Self {

        let storage = DistlockStorage::new();

        let config = Config{
            id , 
            election_tick : 10 , 
            heartbeat_tick : 3 , 
            max_size_per_msg : 1024 * 1024, 
            max_inflight_msgs : 256, 
            ..Default::default()
        };
        let raft = RawNode::new(&config, storage.clone() , &default_logger()).unwrap();
        let state_machine = Arc::new(RwLock::new(InMemoryLockManager::new()));

        let node =    Self { storage , raft, state_machine, id, peers , command_rx , pending_maps : Mutex::new(HashMap::new()) };

     
     node 

    }

    pub async fn run (mut self) {

        loop { 
           tokio::select! {
            Some((command , response_sender)) = self.command_rx.recv() =>{
                self.handle_command(command , response_sender).await
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                self.tick();
                self.process_raft_ready();
            }
           }
        }
    }

    pub async fn handle_command(&mut self , command : LockCommand , response_sender : oneshot::Sender<CommandResponse> ){
        if self.raft.status().id != self.id {
            let _ = response_sender.send(CommandResponse::Error{
                error_type : "Not the leader".to_string(), 
                message : "Failed to append log".to_string()
            });
            return 
        }
        let request_id = match &command{
            LockCommand::Acquire { request_id,..   } => request_id,
            LockCommand::Release { request_id, .. } => request_id,
            LockCommand::Renew { request_id,.. } => request_id,
        };

     self.propose(command.clone()).await;
        self.pending_maps.lock().unwrap().insert(*request_id, response_sender);
        let data = serde_json::to_vec(&command).unwrap();

        if let Err(e) = self.raft.propose(vec![], data){
            self.pending_maps.lock().unwrap().remove(request_id);
        }


    }

    pub async fn apply_entry (&self , entry : &raft::eraftpb::Entry){

        let command : LockCommand = serde_json::from_slice(entry.get_data()).unwrap();

        let request_id = match command{
            LockCommand::Acquire {  request_id,.. } => request_id , 
            LockCommand::Release { request_id, .. } => request_id, 
            LockCommand::Renew { request_id, .. } => request_id
        };

        let result = self.apply_command_to_state(command.clone()).await;

        if let Ok(mut pending) = self.pending_maps.lock(){
            
            if let Some(sender) = pending.remove(&request_id){
                _ = sender.send(result)
            }
        }

    }

 pub async fn apply_command_to_state(&self, command: LockCommand) -> CommandResponse {
    use std::time::Duration;
    
    let manager = self.state_machine.write().await;
    
    match command {
        LockCommand::Acquire { lock_id, client_id, ttl_seconds, request_id } => {
            let result = manager.try_acquire(
                &LockId(lock_id),
                &ClientId(client_id),
                Duration::from_secs(ttl_seconds),
            );
            
            match result {
                AcquireResult::Granted { lease_id, expires_at } => {
                    CommandResponse::AcquireGranted {
                        lease_id: lease_id.0,  // Assuming .0 is public
                        expires_at: expires_at.to_rfc3339(),
                    }
                }
                AcquireResult::Queued { position, .. } => {
                    CommandResponse::AcquireQueued { position }
                }
                AcquireResult::Error(message) => {
                    CommandResponse::Error {
                        error_type: "AcquireError".to_string(),
                        message,
                    }
                }
            }
        }
        
        LockCommand::Release { request_id, lock_id, client_id, lease_id } => {
            let result = manager.release(
                &LockId(lock_id),
                &ClientId(client_id),
                &LeaseId(lease_id),
            );
            
            match result {
                ReleaseResult::Success => CommandResponse::ReleaseSuccess,
                ReleaseResult::NotFound => CommandResponse::Error {
                    error_type: "NotFound".to_string(),
                    message: "Lock not found".to_string(),
                },
                ReleaseResult::NotHolder => CommandResponse::Error {
                    error_type: "NotHolder".to_string(),
                    message: "Not the lock holder".to_string(),
                },
                ReleaseResult::Error(message) => CommandResponse::Error {
                    error_type: "ReleaseError".to_string(),
                    message,
                },
            }
        }
        
        LockCommand::Renew { request_id, lock_id, client_id, ttl_seconds, lease_id } => {
            let result = manager.renew(
                &LockId(lock_id),
                &ClientId(client_id),
                &LeaseId(lease_id),
                Duration::from_secs(ttl_seconds),  // Fixed: was `ttl`
            );
            
            match result {
                RenewResult::Success { new_expiry } => CommandResponse::RenewSuccess {
                    new_expiry: new_expiry.to_rfc3339(),
                },
                RenewResult::Expired => CommandResponse::Error {
                    error_type: "Expired".to_string(),
                    message: "Lock already expired".to_string(),
                },
                RenewResult::NotFound => CommandResponse::Error {
                    error_type: "NotFound".to_string(),
                    message: "Lock not found".to_string(),
                },
                RenewResult::NotHolder => CommandResponse::Error {
                    error_type: "NotHolder".to_string(),
                    message: "Not the lock holder".to_string(),
                },
                RenewResult::Error(message) => CommandResponse::Error {
                    error_type: "RenewError".to_string(),
                    message,
                },
            }
        }
    }
}
    fn tick(&mut self){
        self.raft.tick();
    }

    
    pub async fn propose(&mut self , command : LockCommand ) -> Result<u64 , String> {
        let data = serde_json::to_vec(&command)
            .map_err(|e| format!("Serialization Error {}" , e))?;

        self.raft.propose(vec![] , data).unwrap();

        Ok(0)
    }
    
}
