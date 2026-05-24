use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::model::game::Game;
use crate::model::message::Message;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::SnapshotStorage;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

// ─── Snapshot Repository ───────────────────────────────────────────────────

pub struct InMemorySnapshotRepository {
    game_id: AtomicU64,
    snapshots: Mutex<HashMap<u64, Vec<GameStateSnapshot>>>,
    games: Mutex<Vec<Game>>,
    next_id: Mutex<u64>,
}

impl Default for InMemorySnapshotRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySnapshotRepository {
    pub fn new() -> Self {
        Self::with_game_id(1)
    }

    pub fn with_game_id(game_id: u64) -> Self {
        Self {
            game_id: AtomicU64::new(game_id),
            snapshots: Mutex::new(HashMap::new()),
            games: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }

    fn do_set_game_id(&self, game_id: u64) {
        let current = self.do_current_game_id();
        if current != game_id {
            let mut games = lock(&self.games);
            if let Some(game) = games.iter_mut().find(|g| g.id == game_id) {
                game.updated_at = chrono::Utc::now();
            }
            self.game_id.store(game_id, Ordering::SeqCst);
        }
    }

    fn do_current_game_id(&self) -> u64 {
        self.game_id.load(Ordering::SeqCst)
    }

    pub fn len(&self) -> usize {
        let gid = self.do_current_game_id();
        lock(&self.snapshots)
            .get(&gid)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl SnapshotStorage for InMemorySnapshotRepository {
    fn set_game_id(&self, game_id: u64) {
        self.do_set_game_id(game_id);
    }

    fn current_game_id(&self) -> u64 {
        self.do_current_game_id()
    }

    fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, crate::error::EngineError> {
        let mut snaps = lock(&self.snapshots);
        let mut next_id = lock(&self.next_id);
        let id = *next_id;
        *next_id += 1;
        let mut snap = snapshot.clone();
        snap.db_id = Some(id);
        let gid = self.do_current_game_id();
        snaps.entry(gid).or_default().push(snap);
        Ok(id)
    }

    fn load_latest(&self) -> Result<Option<GameStateSnapshot>, crate::error::EngineError> {
        let snaps = lock(&self.snapshots);
        let gid = self.do_current_game_id();
        let result = snaps.get(&gid).and_then(|vec| {
            vec.iter()
                .max_by(|a, b| {
                    a.created_at
                        .cmp(&b.created_at)
                        .then_with(|| a.db_id.cmp(&b.db_id))
                })
                .cloned()
        });
        Ok(result)
    }

    fn load_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, crate::error::EngineError> {
        let snaps = lock(&self.snapshots);
        let gid = self.do_current_game_id();
        let result = snaps
            .get(&gid)
            .and_then(|vec| vec.iter().find(|s| s.db_id == Some(id)).cloned());
        Ok(result)
    }

    fn list_games(&self) -> Result<Vec<Game>, crate::error::EngineError> {
        let games = lock(&self.games);
        Ok(games.clone())
    }

    fn create_game(&self, world_name: &str, name: &str) -> Result<u64, crate::error::EngineError> {
        let mut games = lock(&self.games);
        let mut next_id = lock(&self.next_id);
        let id = *next_id;
        *next_id += 1;
        let now = chrono::Utc::now();
        games.push(Game {
            id,
            world_name: world_name.to_string(),
            name: name.to_string(),
            created_at: now,
            updated_at: now,
        });
        Ok(id)
    }

    fn delete_game(&self, id: u64) -> Result<(), crate::error::EngineError> {
        let mut games = lock(&self.games);
        let mut snaps = lock(&self.snapshots);
        games.retain(|g| g.id != id);
        snaps.remove(&id);
        Ok(())
    }

    fn get_game(&self, id: u64) -> Result<Option<Game>, crate::error::EngineError> {
        let games = lock(&self.games);
        Ok(games.iter().find(|g| g.id == id).cloned())
    }
}

// ─── Message Repository ────────────────────────────────────────────────────

pub struct InMemoryMessageRepository {
    game_id: AtomicU64,
    messages: Mutex<HashMap<u64, Vec<Message>>>,
    next_message_id: Mutex<u64>,
}

impl Default for InMemoryMessageRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryMessageRepository {
    pub fn new() -> Self {
        Self::with_game_id(1)
    }

    pub fn with_game_id(game_id: u64) -> Self {
        Self {
            game_id: AtomicU64::new(game_id),
            messages: Mutex::new(HashMap::new()),
            next_message_id: Mutex::new(0),
        }
    }

    fn do_set_game_id(&self, game_id: u64) {
        self.game_id.store(game_id, Ordering::SeqCst);
    }

    fn do_current_game_id(&self) -> u64 {
        self.game_id.load(Ordering::SeqCst)
    }
}

impl MessageStorage for InMemoryMessageRepository {
    fn set_game_id(&self, game_id: u64) {
        self.do_set_game_id(game_id);
    }

    fn current_game_id(&self) -> u64 {
        self.do_current_game_id()
    }

    fn insert_message(&self, msg: &Message) -> Result<u64, crate::error::EngineError> {
        let mut next_id = lock(&self.next_message_id);
        *next_id += 1;
        let id = *next_id;
        let gid = self.do_current_game_id();
        let mut msgs = lock(&self.messages);
        let entry = msgs.entry(gid).or_default();
        let mut msg = msg.clone();
        msg.id = id;
        entry.push(msg);
        Ok(id)
    }

    fn update_message(&self, id: u64, text: &str) -> Result<(), crate::error::EngineError> {
        let mut msgs = lock(&self.messages);
        let gid = self.do_current_game_id();
        if let Some(vec) = msgs.get_mut(&gid) {
            if let Some(m) = vec.iter_mut().find(|m| m.id == id) {
                m.text = text.to_string();
                if let Some(swipe) = m.swipes.get_mut(m.active_swipe_index) {
                    swipe.text = text.to_string();
                }
            }
        }
        Ok(())
    }

    fn delete_message(&self, id: u64) -> Result<(), crate::error::EngineError> {
        let mut msgs = lock(&self.messages);
        let gid = self.do_current_game_id();
        if let Some(vec) = msgs.get_mut(&gid) {
            vec.retain(|m| m.id != id);
        }
        Ok(())
    }

    fn load_messages(&self) -> Result<Vec<Message>, crate::error::EngineError> {
        let msgs = lock(&self.messages);
        let gid = self.do_current_game_id();
        Ok(msgs
            .get(&gid)
            .map(|vec| vec.iter().filter(|m| !m.is_deleted).cloned().collect())
            .unwrap_or_default())
    }

    fn soft_delete_message(&self, id: u64) -> Result<(), crate::error::EngineError> {
        let mut msgs = lock(&self.messages);
        let gid = self.do_current_game_id();
        if let Some(vec) = msgs.get_mut(&gid) {
            if let Some(m) = vec.iter_mut().find(|m| m.id == id) {
                m.is_deleted = true;
            }
        }
        Ok(())
    }

    fn restore_soft_deleted(&self, ids: &[u64]) -> Result<(), crate::error::EngineError> {
        let mut msgs = lock(&self.messages);
        let gid = self.do_current_game_id();
        if let Some(vec) = msgs.get_mut(&gid) {
            for m in vec.iter_mut().filter(|m| ids.contains(&m.id)) {
                m.is_deleted = false;
            }
        }
        Ok(())
    }

    fn purge_soft_deleted(&self, ids: &[u64]) -> Result<(), crate::error::EngineError> {
        let mut msgs = lock(&self.messages);
        let gid = self.do_current_game_id();
        if let Some(vec) = msgs.get_mut(&gid) {
            vec.retain(|m| !ids.contains(&m.id));
        }
        Ok(())
    }

    fn insert_swipe(
        &self,
        message_id: u64,
        swipe: &crate::model::message::Swipe,
        index: usize,
    ) -> Result<(), crate::error::EngineError> {
        let mut msgs = lock(&self.messages);
        let gid = self.do_current_game_id();
        if let Some(vec) = msgs.get_mut(&gid) {
            if let Some(m) = vec.iter_mut().find(|m| m.id == message_id) {
                if index < m.swipes.len() {
                    m.swipes.insert(index, swipe.clone());
                } else {
                    m.swipes.push(swipe.clone());
                }
            }
        }
        Ok(())
    }

    fn update_active_swipe(
        &self,
        message_id: u64,
        index: usize,
    ) -> Result<(), crate::error::EngineError> {
        let mut msgs = lock(&self.messages);
        let gid = self.do_current_game_id();
        if let Some(vec) = msgs.get_mut(&gid) {
            if let Some(m) = vec.iter_mut().find(|m| m.id == message_id) {
                m.set_active_swipe(index);
            }
        }
        Ok(())
    }

    fn shift_swipe_indices(
        &self,
        _message_id: u64,
        _offset: usize,
    ) -> Result<(), crate::error::EngineError> {
        // No-op for in-memory: insert_swipe handles index insertion directly.
        Ok(())
    }

    fn migrate_swipes(
        &self,
        message_id: u64,
        pending_swipes: &[crate::model::message::Swipe],
        new_active_index: usize,
        to_delete: &[u64],
    ) -> Result<(), crate::error::EngineError> {
        let mut msgs = lock(&self.messages);
        let gid = self.do_current_game_id();

        if let Some(vec) = msgs.get_mut(&gid) {
            if let Some(m) = vec.iter_mut().find(|m| m.id == message_id) {
                for (idx, swipe) in pending_swipes.iter().enumerate().rev() {
                    if idx < m.swipes.len() {
                        m.swipes.insert(idx, swipe.clone());
                    } else {
                        m.swipes.push(swipe.clone());
                    }
                }
                m.set_active_swipe(new_active_index);
            }
            vec.retain(|m| !to_delete.contains(&m.id));
        }
        Ok(())
    }
}
