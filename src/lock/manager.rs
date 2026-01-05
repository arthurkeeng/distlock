
use std::{collections::HashMap,sync::RwLock};
use std::{time::Duration};

use chrono::{Utc , Duration as ChronoDuration};

use crate::lock::types::{AcquireResult, ClientId, LeaseId, LockHolder, LockId, LockManager, LockState, ReleaseResult, RenewResult, WaitRequest};


pub struct InMemoryLockManager{
    locks : std::sync::RwLock<HashMap<LockId , LockState>>,
     default_ttl : ChronoDuration
}


impl InMemoryLockManager{
    pub fn new() -> Self{
        InMemoryLockManager { locks: RwLock::new(HashMap::new()) ,
            default_ttl : ChronoDuration::seconds(30)
        }
    }

    pub fn with_default_ttl(ttl : ChronoDuration) -> Self{
        InMemoryLockManager { locks: RwLock::new(HashMap::new()), default_ttl: ttl }
    }
}

impl LockManager for InMemoryLockManager{
    fn try_acquire (&self , lock_id : &LockId , client_id : &ClientId , ttl : std::time::Duration ) -> AcquireResult {
        let mut locks = self.locks.write().unwrap();

        // let count = locks.keys().count();
        let chrono_ttl = ChronoDuration::from_std(ttl).expect("TTL too large for chrono");

        let lock_state = locks.entry(lock_id.clone()).or_insert(LockState { holder: None, wait_queue: Vec::new(), created_at: Utc::now() });

        if lock_state.holder.as_ref().map_or(false, |h| h.expires_at < Utc::now()){
            lock_state.holder = None;

        }
      
        if lock_state.holder.is_none(){

            let lease_id = LeaseId(
                 uuid::Uuid::new_v4().to_string()
            );
            let expires_at = Utc::now() + chrono_ttl; 

            lock_state.holder = Some(LockHolder { client_id: client_id.clone(), lease_id: lease_id.clone() , acquired_at: Utc::now(), expires_at, renewal_count: 0 });

            AcquireResult::Granted { lease_id, expires_at }
        }
        else {
            let position = lock_state.wait_queue.len();
            lock_state.wait_queue.push(WaitRequest {
                client_id : client_id.clone() , 
                requested_at : Utc::now()
            });
            let estimated_wait = if let Some(holder) = &lock_state.holder{
                let remaining = holder.expires_at - Utc::now();
                std::time::Duration::from_secs(remaining.num_seconds().max(0) as u64)
            }else {
                std::time::Duration::from_secs(0)
            };
            AcquireResult::Queued { position, estimated_wait }

        }

    }

     fn release (&self , lock_id : &LockId , client_id : &ClientId , lease_id : &LeaseId ) -> ReleaseResult {
        // verify identy 
        let mut locks = self.locks.write().unwrap();
       let lock_state = match locks.get_mut(&lock_id){
         Some(state) => state , 
         None => return ReleaseResult::NotFound
       };
       let current_holder = match &lock_state.holder{
        Some(holder) =>holder, 
        None => return ReleaseResult::NotHolder
       };
       if current_holder.client_id != *client_id || current_holder.lease_id != *lease_id{
        return  ReleaseResult::NotHolder;
       }
       lock_state.holder = None;

    

       if let true  = !lock_state.wait_queue.is_empty(){

           let next_waiter = lock_state.wait_queue.remove(0);
            let new_lease_id = LeaseId(uuid::Uuid::new_v4().to_string());
           
            
            let expires_at = Utc::now() + self.default_ttl;

           lock_state.holder =Some( LockHolder{
            client_id : next_waiter.client_id,
            lease_id : new_lease_id , 
            acquired_at:Utc::now() , expires_at , renewal_count : 0
       });

    }
    ReleaseResult::Success

     }
 fn renew(&self, lock_id: &LockId, client_id: &ClientId, lease_id: &LeaseId, ttl: Duration) -> RenewResult {
    let mut locks = self.locks.write().unwrap();

    let lock_state = match locks.get_mut(lock_id) {
        Some(state) => state,
        None => return RenewResult::NotFound,
    };

    match lock_state.holder.as_mut() {
        Some(holder) => {
            // Verify ownership
            if holder.client_id != *client_id || holder.lease_id != *lease_id {
                return RenewResult::NotHolder;
            }
            
            // Check if already expired
            if holder.expires_at < Utc::now() {
                return RenewResult::Expired;
            }
            
            // Perform renewal
            let new_expiry = Utc::now() + ChronoDuration::from_std(ttl).expect("TTL too large for Chrono");
            holder.expires_at = new_expiry;
            holder.renewal_count += 1;
            
            RenewResult::Success { new_expiry }
        }
        None => {
            RenewResult::NotHolder
        }
    }
}
fn status(&self , lock_id : &LockId ) -> Option<LockState> {
    let locks = self.locks.read().unwrap();
    let lock_state = locks.get(lock_id).cloned();
    lock_state

        
    }
}