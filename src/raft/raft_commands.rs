use async_raft::AppData;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize , Clone , Debug)]
pub enum LockCommand{
    Acquire{
        lock_id : String, 
        client_id : String , 
        ttl_seconds : u64 , 
        request_id : u64
    },
    Release{
        request_id : u64,
        lock_id : String, 
        client_id : String , 
        lease_id : String
    },
    Renew{
        request_id : u64,
        lock_id : String, 
        client_id : String , 
        ttl_seconds : u64 , 
        lease_id : String
    },
}

#[derive(Debug, Clone , Serialize , Deserialize)]
pub enum CommandResponse {
    AcquireGranted {
        lease_id : String , 
        expires_at : String 
    },
    AcquireQueued{
        position :usize
    }, 
    // Error(String),
    Error{
        error_type : String , 
        message : String
    },
    ReleaseSuccess, 
    RenewSuccess { new_expiry : String}
}
impl AppData for LockCommand{}