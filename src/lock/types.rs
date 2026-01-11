use std::{time::Duration};
use chrono::{DateTime, Utc };


#[derive(Clone)]
pub struct Lock {
    pub id: String , 
    pub holder : Option<LockHolder> , 
    pub created_at : DateTime<Utc>, 
    pub wait_queue : Vec<LockRequest>, 
    // stats : LockStats
}
#[derive(Clone)]

pub struct LockHolder {
    pub client_id : ClientId, 
    pub lease_id : LeaseId,
    pub acquired_at : DateTime<Utc> , 
    pub expires_at : DateTime<Utc> , 
    pub renewal_count : u32
}

#[derive(Clone)]
pub struct LockRequest{
   pub client_id : String , 
    pub requested_at : DateTime<Utc> , 
    pub timeout : Duration
}
#[derive(Clone)]
pub struct LockState{
    pub holder : Option<LockHolder> , 
    pub wait_queue : Vec<WaitRequest>, 
    pub created_at : DateTime<Utc>
}
#[derive(Clone)]

pub struct WaitRequest{
    pub client_id : ClientId , 
    pub requested_at : DateTime<Utc>
}

#[derive(Debug , Clone , PartialEq)]

pub struct ClientId(pub String);
#[derive(Debug , Clone , PartialEq, Eq , Hash)]
pub struct LockId (pub String);

#[derive(Debug , Clone , PartialEq)]
pub struct LeaseId (pub String);


#[derive(Debug , Clone)]
pub enum AcquireResult{
    Granted {
         lease_id : LeaseId , 
         expires_at : DateTime<Utc>
    } , 
    Queued {
         position : usize , 
         estimated_wait : Duration
    }
    , Error(String)
}

#[derive(Debug )]
pub enum ReleaseResult{
    Success , NotHolder , NotFound , Error(String)
}

#[derive(Debug)]

pub enum RenewResult{
     Success {
        new_expiry: DateTime<Utc>,
    },
    NotHolder,
    NotFound,
    Expired,
    Error(String),
}

pub trait LockManager : Send + Sync  {
    fn try_acquire (&self , lock_id : &LockId , client_id : &ClientId , ttl : Duration ) -> AcquireResult ;
    fn release (&self , lock_id : &LockId , client_id : &ClientId , lease_id : &LeaseId ) -> ReleaseResult ;
    fn renew (&self , lock_id : &LockId , client_id : &ClientId , lease_id : &LeaseId ,ttl: Duration ) ->RenewResult ;
    fn status(&self , lock_id : &LockId ) -> Option<LockState>;
    fn current_holder(&self , lock_id : &LockId) -> Option<ClientId>;
    fn queue_length(&self , lock_id : &LockId) -> usize;

}