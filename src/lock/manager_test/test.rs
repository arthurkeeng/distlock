

#[cfg(test)]

mod test{
    use super::*;
    use std::time::Duration;
    use crate::lock::{manager::InMemoryLockManager, types::{AcquireResult, ClientId, LeaseId, LockId, LockManager, ReleaseResult}};

    #[test]

    fn test_basic_lock_acquire_and_release(){

        let manager = InMemoryLockManager::new();

        let lock_id = LockId("test_lock".to_string());
        let client1 = ClientId("client_1".to_string());

        let result = manager.try_acquire(&lock_id, &client1, Duration::from_secs(30));
        assert!(matches!(result , AcquireResult::Granted { ..}));

        assert_eq!(manager.current_holder(&lock_id) , Some(client1.clone()));

        assert_eq!(manager.queue_length(&lock_id), 0);

        let lease_id = match result {
            AcquireResult::Granted { lease_id,.. } => lease_id, 
            _ => panic!("Expected granted")
        };

        let release_result = manager.release(&lock_id, &client1, &lease_id);

        assert!(matches!(release_result , ReleaseResult::Success));

        assert_eq!(manager.current_holder(&lock_id) , None);
    }

}