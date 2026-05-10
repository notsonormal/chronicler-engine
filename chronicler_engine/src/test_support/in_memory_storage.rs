use std::sync::Mutex;

use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::snapshot_storage::SnapshotStorage;

pub struct InMemorySnapshotStorage {
    snapshots: Mutex<Vec<GameStateSnapshot>>,
}

impl Default for InMemorySnapshotStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySnapshotStorage {
    pub fn new() -> Self {
        Self {
            snapshots: Mutex::new(Vec::new()),
        }
    }

    pub fn len(&self) -> usize {
        match self.snapshots.lock() {
            Ok(guard) => guard.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.snapshots.lock() {
            Ok(guard) => guard.is_empty(),
            Err(poisoned) => poisoned.into_inner().is_empty(),
        }
    }
}

impl SnapshotStorage for InMemorySnapshotStorage {
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<(), crate::error::EngineError> {
        let mut snaps = match self.snapshots.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        snaps.retain(|s| {
            !(s.message_id == snapshot.message_id && s.swipe_index == snapshot.swipe_index)
        });
        snaps.push(snapshot.clone());
        Ok(())
    }

    fn load_latest(
        &self,
        message_id: Option<&str>,
    ) -> Result<Option<GameStateSnapshot>, crate::error::EngineError> {
        let snaps = match self.snapshots.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let candidates: Vec<_> = if let Some(msg_id) = message_id {
            snaps.iter().filter(|s| s.message_id == msg_id).collect()
        } else {
            snaps.iter().collect()
        };
        Ok(candidates.into_iter().max_by_key(|s| s.created_at).cloned())
    }

    fn load_by_message(
        &self,
        message_id: &str,
        swipe_index: u32,
    ) -> Result<Option<GameStateSnapshot>, crate::error::EngineError> {
        let snaps = match self.snapshots.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        Ok(snaps
            .iter()
            .find(|s| s.message_id == message_id && s.swipe_index == swipe_index)
            .cloned())
    }

    fn commit(&self, snapshot_id: &str) -> Result<(), crate::error::EngineError> {
        let mut snaps = match self.snapshots.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(snap) = snaps.iter_mut().find(|s| s.id == snapshot_id) {
            snap.committed = true;
        }
        Ok(())
    }

    fn reset(&self) -> Result<(), crate::error::EngineError> {
        let mut snaps = match self.snapshots.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        snaps.clear();
        Ok(())
    }
}
