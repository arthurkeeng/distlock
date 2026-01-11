use std::time::Duration;

use distlock::lock::{manager::InMemoryLockManager, types::{AcquireResult, ClientId, LockId, LockManager}};



#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    // use crate::lock::{LockId , ClientId};
    // use distlock::lock::{}
    #[test]

fn test_basic_lock_acquisition(){
    let manager = InMemoryLockManager::new();

    let lock_id = LockId("Test_lock".to_string());

    let client1 = ClientId("client_1".to_string());

    let result = manager.try_acquire(&lock_id, &client1, Duration::from_secs(30));

    assert!(matches!(result , AcquireResult::Granted { .. }));

    let client2 = ClientId("client_2".to_string());

    let result = manager.try_acquire(&lock_id, &client2, Duration::from_secs(30));
    assert!(matches!(result , AcquireResult::Queued { .. }));
}

#[test]

fn test_lock_release (){
    let manager = InMemoryLockManager::new();

    let lock_id= LockId("test_lock".to_string());

    let client1 = ClientId("client_1".to_string());

    let result1 = manager.try_acquire(&lock_id, &client1, Duration::from_secs(30));

    let lease_id = match result1 {
        AcquireResult::Granted { lease_id,.. } => lease_id, 
        _ => panic!("Expected lock to be granted")
    };

    // let client2 = 
}
}

