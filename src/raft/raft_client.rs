use std::sync::atomic::AtomicU64;

use tokio::sync::{mpsc::{self, Sender}, oneshot};

use crate::raft::raft_commands::{CommandResponse, LockCommand};


// #[derive(Clone)]
pub struct RaftClient{
    pub command_tx : Sender<(LockCommand , oneshot::Sender<CommandResponse>)>,
    next_request_id : AtomicU64
}

impl RaftClient{
   pub fn new(command_tx : mpsc::Sender<(LockCommand , oneshot::Sender<CommandResponse>)>) -> Self{
    Self { command_tx  , next_request_id : AtomicU64::new(1)}
   }

   pub fn generate_new_index(&self) -> u64 {
    self.next_request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)

   }

    pub async fn propose(&self , command : LockCommand ) -> Result<CommandResponse , String>{

        let (response_tx , response_rx)= oneshot::channel();

        self.command_tx.send((command , response_tx)).await
            .map_err(|e| format!("Failed to send command : {}" , e))?;

        response_rx.await
            .map_err(|e|format!("Failed to receive response : {}" , e))

    
    }

    pub async fn propose_acquire(&self , lock_id : String , client_id : String , ttl_seconds: u64 , ) -> Result<CommandResponse , String>{
        let request_id = self.generate_new_index();

        let command = LockCommand::Acquire { lock_id
            , client_id, ttl_seconds, request_id };

        self.propose(command).await

    }
    pub async fn propose_renew(&self ,lease_id : String ,  lock_id : String , client_id : String , ttl_seconds: u64 , ) -> Result<CommandResponse , String>{
        let request_id = self.generate_new_index();

        let command = LockCommand::Renew { request_id, lock_id, client_id, ttl_seconds, lease_id };

        self.propose(command).await

    }
    pub async fn propose_release(&self ,lease_id : String ,  lock_id : String , client_id : String  ) -> Result<CommandResponse , String>{
        let request_id = self.generate_new_index();

        let command = LockCommand::Release { request_id, lock_id, client_id, lease_id };

        self.propose(command).await

    }

}