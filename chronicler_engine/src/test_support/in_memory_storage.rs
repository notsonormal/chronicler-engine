use std::sync::{Mutex, MutexGuard};

use crate::model::checkpoint::Checkpoint;
use crate::model::message::Message;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::SnapshotStorage;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

pub struct InMemoryGameStorage {
    _game_id: u64,
    snapshots: Mutex<Vec<GameStateSnapshot>>,
    checkpoints: Mutex<Vec<Checkpoint>>,
    messages: Mutex<Vec<Message>>,
    next_id: Mutex<u64>,
    next_message_id: Mutex<u64>,
}

impl Default for InMemoryGameStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryGameStorage {
    pub fn new() -> Self {
        Self::with_game_id(1)
    }

    pub fn with_game_id(game_id: u64) -> Self {
        Self {
            _game_id: game_id,
            snapshots: Mutex::new(Vec::new()),
            checkpoints: Mutex::new(Vec::new()),
            messages: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
            next_message_id: Mutex::new(0),
        }
    }

    pub fn len(&self) -> usize {
        lock(&self.snapshots).len()
    }

    pub fn is_empty(&self) -> bool {
        lock(&self.snapshots).is_empty()
    }
}

impl SnapshotStorage for InMemoryGameStorage {
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, crate::error::EngineError> {
        let mut snaps = lock(&self.snapshots);
        let mut next_id = lock(&self.next_id);
        let id = *next_id;
        *next_id += 1;
        let mut snap = snapshot.clone();
        snap.db_id = Some(id);
        snaps.push(snap);
        Ok(id)
    }

    fn load_latest(&self) -> Result<Option<GameStateSnapshot>, crate::error::EngineError> {
        let snaps = lock(&self.snapshots);
        let result = snaps
            .iter()
            .max_by(|a, b| {
                a.created_at
                    .cmp(&b.created_at)
                    .then_with(|| a.db_id.cmp(&b.db_id))
            })
            .cloned();
        Ok(result)
    }

    fn load_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, crate::error::EngineError> {
        let snaps = lock(&self.snapshots);
        let result = snaps.iter().find(|s| s.db_id == Some(id)).cloned();
        Ok(result)
    }

    fn commit(&self, snapshot_id: u64) -> Result<(), crate::error::EngineError> {
        let mut snaps = lock(&self.snapshots);
        if let Some(snap) = snaps.iter_mut().find(|s| s.db_id == Some(snapshot_id)) {
            snap.committed = true;
        }
        Ok(())
    }

    fn reset(&self) -> Result<(), crate::error::EngineError> {
        lock(&self.snapshots).clear();
        lock(&self.checkpoints).clear();
        lock(&self.messages).clear();
        *lock(&self.next_id) = 1;
        *lock(&self.next_message_id) = 0;
        Ok(())
    }

    fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), crate::error::EngineError> {
        let mut cps = lock(&self.checkpoints);
        cps.retain(|c| c.id != checkpoint.id);
        cps.push(checkpoint.clone());
        Ok(())
    }

    fn load_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>, crate::error::EngineError> {
        let cps = lock(&self.checkpoints);
        Ok(cps.iter().find(|c| c.id == id).cloned())
    }

    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, crate::error::EngineError> {
        let mut result: Vec<_> = lock(&self.checkpoints).clone();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    fn delete_checkpoint(&self, id: &str) -> Result<(), crate::error::EngineError> {
        lock(&self.checkpoints).retain(|c| c.id != id);
        Ok(())
    }
}

impl MessageStorage for InMemoryGameStorage {
    fn insert_message(&self, msg: &mut Message) -> Result<(), crate::error::EngineError> {
        let mut next_id = lock(&self.next_message_id);
        *next_id += 1;
        msg.id = *next_id;
        lock(&self.messages).push(msg.clone());
        Ok(())
    }

    fn update_message(&self, id: u64, text: &str) -> Result<(), crate::error::EngineError> {
        let mut msgs = lock(&self.messages);
        if let Some(m) = msgs.iter_mut().find(|m| m.id == id) {
            m.text = text.to_string();
        }
        Ok(())
    }

    fn delete_message(&self, id: u64) -> Result<(), crate::error::EngineError> {
        lock(&self.messages).retain(|m| m.id != id);
        Ok(())
    }

    fn load_messages(&self) -> Result<Vec<Message>, crate::error::EngineError> {
        Ok(lock(&self.messages).clone())
    }
}
