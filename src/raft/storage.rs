use std::{sync::{Arc , RwLock}, time::Duration};

use raft::{storage::{MemStorage, Storage}};
use crate::lock::types::{LockId , ClientId , LeaseId};
use crate::{lock::{manager::InMemoryLockManager, types::LockManager}, raft::raft_commands::LockCommand};



#[derive(Clone)]
pub struct DistlockStorage{
    inner : Arc<RwLock<MemStorage>>,
}

impl DistlockStorage{
    pub fn new() -> Self {
        Self { inner
            : Arc::new(RwLock::new(MemStorage::new())) ,
        }
    }



}

impl Storage for DistlockStorage{
    fn initial_state(&self) -> raft::Result<raft::RaftState> {
        let storage = self.inner.read().unwrap();
        storage.initial_state()
    }

    fn entries(
            &self,
            low: u64,
            high: u64,
            max_size: impl Into<Option<u64>>,
            context: raft::GetEntriesContext,
        ) -> raft::Result<Vec<raft::prelude::Entry>> {
        let storage = self.inner.read().unwrap();
        storage.entries(low ,high , max_size,context)
    }

    fn term(&self, idx: u64) -> raft::Result<u64> {
        let storage = self.inner.read().unwrap();
        storage.term(idx)
    }
    fn first_index(&self) -> raft::Result<u64> {
        let storage = self.inner.read().unwrap();
        storage.first_index()
    }
    fn last_index(&self) -> raft::Result<u64> {
        let storage = self.inner.read().unwrap();
        storage.last_index()
    }
    fn snapshot(&self, request_index: u64, to: u64) -> raft::Result<raft::prelude::Snapshot> {
        let storage = self.inner.read().unwrap();
        storage.snapshot(request_index , to)
    }
}