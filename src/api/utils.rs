use crate::lock::types::{ClientId, LeaseId, LockId};



pub fn change_to_lease_id(lease_id : &String) ->LeaseId{
    LeaseId(lease_id.to_string())
}
pub fn change_to_lock_id(lock_id : &String) ->LockId{
    LockId(lock_id.to_string())
}
pub fn change_to_client_id(client_id : &String) ->ClientId{
    ClientId(client_id.to_string())
}
