use std::sync::Mutex;

use crate::model::checkpoint::Checkpoint;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::snapshot_storage::SnapshotStorage;

pub struct InMemorySnapshotStorage {
    snapshots: Mutex<Vec<GameStateSnapshot>>,
    checkpoints: Mutex<Vec<Checkpoint>>,
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
            checkpoints: Mutex::new(Vec::new()),
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
        snaps.retain(|s| !(s.turn_id == snapshot.turn_id && s.swipe_index == snapshot.swipe_index));
        snaps.push(snapshot.clone());
        Ok(())
    }

    fn load_latest(
        &self,
        turn_id: Option<&str>,
    ) -> Result<Option<GameStateSnapshot>, crate::error::EngineError> {
        let snaps = match self.snapshots.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let candidates: Vec<_> = if let Some(msg_id) = turn_id {
            snaps.iter().filter(|s| s.turn_id == msg_id).collect()
        } else {
            snaps.iter().collect()
        };
        Ok(candidates.into_iter().max_by_key(|s| s.created_at).cloned())
    }

    fn load_by_turn(
        &self,
        turn_id: &str,
        swipe_index: u32,
    ) -> Result<Option<GameStateSnapshot>, crate::error::EngineError> {
        let snaps = match self.snapshots.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        Ok(snaps
            .iter()
            .find(|s| s.turn_id == turn_id && s.swipe_index == swipe_index)
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

    fn delete_turn_snapshots(&self, turn_id: &str) -> Result<(), crate::error::EngineError> {
        let mut snaps = match self.snapshots.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let pre_main = format!("pre-main:{turn_id}");
        let pre_event = format!("pre-event:{turn_id}");
        snaps.retain(|s| s.turn_id != turn_id && s.turn_id != pre_main && s.turn_id != pre_event);
        Ok(())
    }

    fn reset(&self) -> Result<(), crate::error::EngineError> {
        let mut snaps = match self.snapshots.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        snaps.clear();
        let mut cps = match self.checkpoints.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        cps.clear();
        Ok(())
    }

    fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), crate::error::EngineError> {
        let mut cps = match self.checkpoints.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        cps.retain(|c| c.id != checkpoint.id);
        cps.push(checkpoint.clone());
        Ok(())
    }

    fn load_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>, crate::error::EngineError> {
        let cps = match self.checkpoints.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        Ok(cps.iter().find(|c| c.id == id).cloned())
    }

    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, crate::error::EngineError> {
        let cps = match self.checkpoints.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut result: Vec<_> = cps.clone();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    fn delete_checkpoint(&self, id: &str) -> Result<(), crate::error::EngineError> {
        let mut cps = match self.checkpoints.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        cps.retain(|c| c.id != id);
        Ok(())
    }
}
