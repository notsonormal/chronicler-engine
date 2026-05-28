use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::{EngineError, internal_error};
use crate::model::game::Game;
use crate::model::llm_message::LlmMessage;
use crate::model::message::{Message, Swipe};
use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::model::state_snapshot::GameStateSnapshot;
use crate::storage::db::DbPool;
use crate::storage::models::game::DbGame;
use crate::storage::models::game_state_snapshot::DbGameStateSnapshot;
use crate::storage::models::llm_message::DbLlmMessage;
use crate::storage::models::message::DbMessage;
use crate::storage::models::prompt_preset::DbPromptPreset;

pub struct Storage {
    game_id: AtomicU64,
    backend: Mutex<Backend>,
}

struct InMemoryData {
    snapshots: HashMap<u64, Vec<GameStateSnapshot>>,
    next_snapshot_id: u64,
    games: Vec<Game>,
    next_game_id: u64,
    messages: HashMap<u64, Vec<Message>>,
    next_message_id: u64,
    swipes: HashMap<u64, Vec<Swipe>>,
    presets: Vec<PromptPreset>,
    llm_messages: Vec<LlmMessage>,
}

enum Backend {
    Sqlite {
        pool: DbPool,
    },
    InMemory(Box<InMemoryData>),
    Test {
        base: Box<Backend>,
        overrides: Arc<Mutex<HashMap<Operation, TestOverride>>>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Operation {
    SaveSnapshot,
    LoadLatestSnapshot,
    LoadSnapshotById,
    ListGames,
    CreateGame,
    DeleteGame,
    GetGame,
    InsertMessage,
    DeleteMessage,
    LoadMessageRows,
    GetActiveSwipeIndex,
    UpdateActiveSwipe,
    SoftDeleteMessage,
    RestoreSoftDeleted,
    PurgeSoftDeleted,
    InsertSwipe,
    UpdateSwipeText,
    ShiftSwipeIndices,
    LoadSwipesForMessages,
    ListPresets,
    GetPreset,
    SavePreset,
    DeletePreset,
    SaveLlmMessage,
    ListLatestLlmMessages,
}

pub struct TestOverride {
    kind: ErrorKind,
    message: String,
}

#[derive(Clone, Copy)]
pub enum ErrorKind {
    Config,
    Internal,
}

pub struct TestFailureHandle {
    overrides: Arc<Mutex<HashMap<Operation, TestOverride>>>,
}

impl TestFailureHandle {
    pub fn set(&self, op: Operation, override_: TestOverride) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(op, override_);
    }

    pub fn clear(&self, op: Operation) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&op);
    }

    pub fn clear_all(&self) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}

impl TestOverride {
    pub fn config(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Config,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            message: message.into(),
        }
    }

    fn to_error(&self) -> EngineError {
        match self.kind {
            ErrorKind::Config => EngineError::Config(self.message.clone()),
            ErrorKind::Internal => EngineError::Internal(internal_error(self.message.clone())),
        }
    }
}

impl Storage {
    pub fn new_sqlite(pool: DbPool, game_id: u64) -> Self {
        Self {
            game_id: AtomicU64::new(game_id),
            backend: Mutex::new(Backend::Sqlite { pool }),
        }
    }

    pub fn new_in_memory() -> Self {
        let now = chrono::Utc::now();
        Self {
            game_id: AtomicU64::new(1),
            backend: Mutex::new(Backend::InMemory(Box::new(InMemoryData {
                snapshots: HashMap::new(),
                next_snapshot_id: 1,
                games: vec![Game {
                    id: 1,
                    world_name: "default".to_string(),
                    name: "default".to_string(),
                    created_at: now,
                    updated_at: now,
                }],
                next_game_id: 2,
                messages: HashMap::new(),
                next_message_id: 0,
                swipes: HashMap::new(),
                presets: Vec::new(),
                llm_messages: Vec::new(),
            }))),
        }
    }

    pub fn with_failure(self, op: Operation, override_: TestOverride) -> Self {
        let mut overrides = HashMap::new();
        overrides.insert(op, override_);
        self.with_overrides(Arc::new(Mutex::new(overrides)))
    }

    pub fn with_shared_overrides(
        self,
        overrides: Arc<Mutex<HashMap<Operation, TestOverride>>>,
    ) -> Self {
        self.with_overrides(overrides)
    }

    pub fn with_test_failures(self) -> (Self, TestFailureHandle) {
        let overrides = Arc::new(Mutex::new(HashMap::new()));
        let storage = self.with_overrides(Arc::clone(&overrides));
        (storage, TestFailureHandle { overrides })
    }

    pub fn add_failure(&self, op: Operation, override_: TestOverride) {
        use std::mem;
        let mut backend = match self.backend.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let current = mem::replace(
            &mut *backend,
            Backend::InMemory(Box::new(InMemoryData {
                snapshots: HashMap::new(),
                next_snapshot_id: 1,
                games: vec![],
                next_game_id: 1,
                messages: HashMap::new(),
                next_message_id: 0,
                swipes: HashMap::new(),
                presets: vec![],
                llm_messages: vec![],
            })),
        );
        let new_backend = match current {
            Backend::Test { base, overrides } => {
                overrides
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(op, override_);
                Backend::Test { base, overrides }
            }
            other => {
                let mut overrides = HashMap::new();
                overrides.insert(op, override_);
                Backend::Test {
                    base: Box::new(other),
                    overrides: Arc::new(Mutex::new(overrides)),
                }
            }
        };
        *backend = new_backend;
    }

    fn with_overrides(self, overrides: Arc<Mutex<HashMap<Operation, TestOverride>>>) -> Self {
        let backend = match self.backend.into_inner() {
            Ok(b) => b,
            Err(poisoned) => poisoned.into_inner(),
        };
        Self {
            game_id: self.game_id,
            backend: Mutex::new(Backend::Test {
                base: Box::new(backend),
                overrides,
            }),
        }
    }

    fn game_id(&self) -> u64 {
        self.game_id.load(Ordering::SeqCst)
    }

    fn with_backend_mut<F, T>(&self, op: Operation, f: F) -> Result<T, EngineError>
    where
        F: FnOnce(&mut Backend, u64) -> Result<T, EngineError>,
    {
        let mut backend = match self.backend.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Backend::Test { overrides, .. } = &*backend {
            if let Some(override_) = overrides.lock().unwrap_or_else(|e| e.into_inner()).get(&op) {
                return Err(override_.to_error());
            }
        }
        let game_id = self.game_id();
        let effective = match &mut *backend {
            Backend::Test { base, .. } => &mut **base,
            other => other,
        };
        f(effective, game_id)
    }
}

// [DOC: docs/architecture/system.md]
impl Storage {
    pub fn set_game_id(&self, game_id: u64) {
        let current = self.game_id();
        if current != game_id {
            let backend = match self.backend.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if let Backend::Sqlite { pool } = &*backend {
                let conn = pool.conn();
                let now = chrono::Utc::now().to_rfc3339();
                if let Err(e) = conn.execute(
                    "UPDATE games SET updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![&now, game_id as i64],
                ) {
                    log::error!("Failed to update games.updated_at for game {game_id}: {e}");
                }
            }
            drop(backend);
            self.game_id.store(game_id, Ordering::SeqCst);
        }
    }

    pub fn current_game_id(&self) -> u64 {
        self.game_id()
    }

    // ─── Game methods ─────────────────────────────────────────────────────────

    pub fn list_games(&self) -> Result<Vec<Game>, EngineError> {
        self.with_backend_mut(Operation::ListGames, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, world_name, name, created_at, updated_at
                         FROM games
                         ORDER BY updated_at DESC",
                    )
                    .map_err(|e| {
                        EngineError::Config(format!("Failed to prepare list games: {e}"))
                    })?;

                let db_games: Vec<DbGame> = stmt
                    .query_map([], |row| {
                        Ok(DbGame {
                            id: row.get(0)?,
                            world_name: row.get(1)?,
                            name: row.get(2)?,
                            created_at: row.get(3)?,
                            updated_at: row.get(4)?,
                        })
                    })
                    .map_err(|e| EngineError::Config(format!("Failed to list games: {e}")))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| EngineError::Config(format!("Failed to read game row: {e}")))?;

                db_games.iter().map(db_game_to_game).collect()
            }
            Backend::InMemory(data) => {
                let mut games = data.games.clone();
                games.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                Ok(games)
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn create_game(&self, world_name: &str, name: &str) -> Result<u64, EngineError> {
        self.with_backend_mut(Operation::CreateGame, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = chrono::Utc::now().to_rfc3339();
                conn.execute(
                    "INSERT INTO games (world_name, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
                    rusqlite::params![world_name, name, &now],
                )
                .map_err(|e| EngineError::Config(format!("Failed to create game: {e}")))?;
                Ok(conn.last_insert_rowid() as u64)
            }
            Backend::InMemory(data) => {
                let id = data.next_game_id;
                data.next_game_id += 1;
                let now = chrono::Utc::now();
                data.games.push(Game {
                    id,
                    world_name: world_name.to_string(),
                    name: name.to_string(),
                    created_at: now,
                    updated_at: now,
                });
                Ok(id)
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn delete_game(&self, id: u64) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::DeleteGame, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "DELETE FROM games WHERE id = ?1",
                    rusqlite::params![id as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to delete game: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                data.games.retain(|g| g.id != id);
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn get_game(&self, id: u64) -> Result<Option<Game>, EngineError> {
        self.with_backend_mut(Operation::GetGame, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, world_name, name, created_at, updated_at
                         FROM games
                         WHERE id = ?1
                         LIMIT 1",
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to prepare get game: {e}")))?;

                let db_result = stmt.query_row(rusqlite::params![id as i64], |row| {
                    Ok(DbGame {
                        id: row.get(0)?,
                        world_name: row.get(1)?,
                        name: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                });

                match db_result {
                    Ok(db) => Ok(Some(db_game_to_game(&db)?)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(EngineError::Config(format!("Failed to get game: {e}"))),
                }
            }
            Backend::InMemory(data) => Ok(data.games.iter().find(|g| g.id == id).cloned()),
            Backend::Test { .. } => unreachable!(),
        })
    }

    // ─── Snapshot methods ─────────────────────────────────────────────────────

    pub fn save_snapshot(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError> {
        self.with_backend_mut(Operation::SaveSnapshot, |backend, game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let db_snap = crate::storage::mappers::state_snapshot::snapshot_to_db(
                    snapshot,
                    game_id as i64,
                )?;

                conn.execute(
                    "INSERT INTO game_state_snapshots
                     (game_id, movement, narrative, scene, npc_encounter_log, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        db_snap.game_id,
                        db_snap.movement_json,
                        db_snap.narrative_json,
                        db_snap.scene_json,
                        db_snap.npc_encounter_log_json,
                        db_snap.created_at,
                    ],
                )
                .map_err(|e| EngineError::Config(format!("Failed to save snapshot: {e}")))?;

                Ok(conn.last_insert_rowid() as u64)
            }
            Backend::InMemory(data) => {
                let id = data.next_snapshot_id;
                data.next_snapshot_id += 1;
                let mut snap = snapshot.clone();
                snap.db_id = Some(id);
                data.snapshots.entry(game_id).or_default().push(snap);
                Ok(id)
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn load_latest_snapshot(&self) -> Result<Option<GameStateSnapshot>, EngineError> {
        self.with_backend_mut(
            Operation::LoadLatestSnapshot,
            |backend, game_id| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    let mut stmt = conn
                        .prepare(
                            "SELECT id, movement, narrative, scene, npc_encounter_log, created_at
                         FROM game_state_snapshots
                         WHERE game_id = ?1
                         ORDER BY created_at DESC, id DESC
                         LIMIT 1",
                        )
                        .map_err(|e| {
                            EngineError::Config(format!("Failed to prepare query: {e}"))
                        })?;

                    let db_result = stmt.query_row(rusqlite::params![game_id as i64], |row| {
                        Ok(DbGameStateSnapshot {
                            id: row.get(0)?,
                            game_id: game_id as i64,
                            movement_json: row.get(1)?,
                            narrative_json: row.get(2)?,
                            scene_json: row.get(3)?,
                            npc_encounter_log_json: row.get(4)?,
                            created_at: row.get(5)?,
                        })
                    });

                    match db_result {
                        Ok(db_snap) => Ok(Some(GameStateSnapshot::try_from(&db_snap)?)),
                        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                        Err(e) => Err(EngineError::Config(format!(
                            "Failed to load latest snapshot: {e}"
                        ))),
                    }
                }
                Backend::InMemory(data) => {
                    let result = data.snapshots.get(&game_id).and_then(|vec| {
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
                Backend::Test { .. } => unreachable!(),
            },
        )
    }

    pub fn load_snapshot_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, EngineError> {
        self.with_backend_mut(
            Operation::LoadSnapshotById,
            |backend, game_id| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    let mut stmt = conn
                        .prepare(
                            "SELECT id, movement, narrative, scene, npc_encounter_log, created_at
                         FROM game_state_snapshots
                         WHERE id = ?1 AND game_id = ?2",
                        )
                        .map_err(|e| {
                            EngineError::Config(format!("Failed to prepare query: {e}"))
                        })?;

                    let db_result = stmt.query_row(rusqlite::params![id, game_id as i64], |row| {
                        Ok(DbGameStateSnapshot {
                            id: row.get(0)?,
                            game_id: game_id as i64,
                            movement_json: row.get(1)?,
                            narrative_json: row.get(2)?,
                            scene_json: row.get(3)?,
                            npc_encounter_log_json: row.get(4)?,
                            created_at: row.get(5)?,
                        })
                    });

                    match db_result {
                        Ok(db_snap) => Ok(Some(GameStateSnapshot::try_from(&db_snap)?)),
                        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                        Err(e) => Err(EngineError::Config(format!(
                            "Failed to load snapshot by id: {e}"
                        ))),
                    }
                }
                Backend::InMemory(data) => {
                    let result = data
                        .snapshots
                        .get(&game_id)
                        .and_then(|vec| vec.iter().find(|s| s.db_id == Some(id)).cloned());
                    Ok(result)
                }
                Backend::Test { .. } => unreachable!(),
            },
        )
    }

    // ─── Message methods ──────────────────────────────────────────────────────

    pub fn insert_message(&self, msg: &Message) -> Result<u64, EngineError> {
        self.with_backend_mut(Operation::InsertMessage, |backend, game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let db_msg =
                    crate::storage::mappers::message::model_message_to_db(msg, game_id as i64)?;
                conn.execute(
                    "INSERT INTO messages (game_id, sender, log_type, timestamp, active_swipe_index, is_deleted)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        db_msg.game_id,
                        db_msg.sender.as_deref(),
                        db_msg.log_type_json,
                        db_msg.timestamp,
                        db_msg.active_swipe_index,
                        db_msg.is_deleted,
                    ],
                )
                .map_err(|e| EngineError::Config(format!("Failed to insert message: {e}")))?;
                let id = conn.last_insert_rowid() as u64;
                Ok(id)
            }
            Backend::InMemory(data) => {
                data.next_message_id += 1;
                let id = data.next_message_id;
                let mut msg = msg.clone();
                msg.id = id;
                msg.swipes.clear();
                data.messages.entry(game_id).or_default().push(msg);
                Ok(id)
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn delete_message(&self, id: u64) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::DeleteMessage, |backend, game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "DELETE FROM messages WHERE id = ?1 AND game_id = ?2",
                    rusqlite::params![id as i64, game_id as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to delete message: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                if let Some(vec) = data.messages.get_mut(&game_id) {
                    vec.retain(|m| m.id != id);
                }
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn load_message_rows(&self) -> Result<Vec<Message>, EngineError> {
        self.with_backend_mut(
            Operation::LoadMessageRows,
            |backend, game_id| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    let mut stmt = conn
                        .prepare(
                            "SELECT id, sender, log_type, timestamp, active_swipe_index
                         FROM messages
                         WHERE game_id = ?1 AND is_deleted = 0
                         ORDER BY id ASC",
                        )
                        .map_err(|e| {
                            EngineError::Config(format!("Failed to prepare message query: {e}"))
                        })?;

                    let msg_rows = stmt
                        .query_map(rusqlite::params![game_id as i64], |row| {
                            Ok(DbMessage {
                                id: row.get(0)?,
                                game_id: game_id as i64,
                                sender: row.get(1)?,
                                log_type_json: row.get(2)?,
                                timestamp: row.get(3)?,
                                active_swipe_index: row.get(4)?,
                                is_deleted: 0,
                            })
                        })
                        .map_err(|e| {
                            EngineError::Config(format!("Failed to query messages: {e}"))
                        })?;

                    let mut messages: Vec<Message> = Vec::new();
                    for row in msg_rows {
                        let db_msg = row.map_err(|e| {
                            EngineError::Config(format!("Failed to read message row: {e}"))
                        })?;
                        messages.push(crate::storage::mappers::message::db_message_to_model(
                            &db_msg,
                            &[],
                        )?);
                    }

                    Ok(messages)
                }
                Backend::InMemory(data) => Ok(data
                    .messages
                    .get(&game_id)
                    .map(|vec| vec.iter().filter(|m| !m.is_deleted).cloned().collect())
                    .unwrap_or_default()),
                Backend::Test { .. } => unreachable!(),
            },
        )
    }

    pub fn get_active_swipe_index(&self, id: u64) -> Result<usize, EngineError> {
        self.with_backend_mut(Operation::GetActiveSwipeIndex, |backend, game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let idx: i64 = conn
                    .query_row(
                        "SELECT active_swipe_index FROM messages WHERE id = ?1 AND game_id = ?2",
                        rusqlite::params![id as i64, game_id as i64],
                        |row| row.get(0),
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to get active swipe index: {e}")))?;
                Ok(idx as usize)
            }
            Backend::InMemory(data) => {
                if let Some(vec) = data.messages.get(&game_id) {
                    if let Some(m) = vec.iter().find(|m| m.id == id) {
                        return Ok(m.active_swipe_index);
                    }
                }
                Err(EngineError::Config(format!("Message {id} not found")))
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn update_active_swipe(&self, message_id: u64, index: usize) -> Result<(), EngineError> {
        self.with_backend_mut(
            Operation::UpdateActiveSwipe,
            |backend, game_id| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    conn.execute(
                    "UPDATE messages SET active_swipe_index = ?1 WHERE id = ?2 AND game_id = ?3",
                    rusqlite::params![index as i64, message_id as i64, game_id as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to update active swipe: {e}")))?;
                    Ok(())
                }
                Backend::InMemory(data) => {
                    if let Some(vec) = data.messages.get_mut(&game_id) {
                        if let Some(m) = vec.iter_mut().find(|m| m.id == message_id) {
                            m.active_swipe_index = index;
                        }
                    }
                    Ok(())
                }
                Backend::Test { .. } => unreachable!(),
            },
        )
    }

    pub fn soft_delete_message(&self, id: u64) -> Result<(), EngineError> {
        self.with_backend_mut(
            Operation::SoftDeleteMessage,
            |backend, game_id| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    conn.execute(
                        "UPDATE messages SET is_deleted = 1 WHERE id = ?1 AND game_id = ?2",
                        rusqlite::params![id as i64, game_id as i64],
                    )
                    .map_err(|e| {
                        EngineError::Config(format!("Failed to soft delete message: {e}"))
                    })?;
                    Ok(())
                }
                Backend::InMemory(data) => {
                    if let Some(vec) = data.messages.get_mut(&game_id) {
                        if let Some(m) = vec.iter_mut().find(|m| m.id == id) {
                            m.is_deleted = true;
                        }
                    }
                    Ok(())
                }
                Backend::Test { .. } => unreachable!(),
            },
        )
    }

    pub fn restore_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError> {
        self.with_backend_mut(
            Operation::RestoreSoftDeleted,
            |backend, game_id| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    for id in ids {
                        conn.execute(
                            "UPDATE messages SET is_deleted = 0 WHERE id = ?1 AND game_id = ?2",
                            rusqlite::params![*id as i64, game_id as i64],
                        )
                        .map_err(|e| {
                            EngineError::Config(format!("Failed to restore message: {e}"))
                        })?;
                    }
                    Ok(())
                }
                Backend::InMemory(data) => {
                    if let Some(vec) = data.messages.get_mut(&game_id) {
                        for m in vec.iter_mut().filter(|m| ids.contains(&m.id)) {
                            m.is_deleted = false;
                        }
                    }
                    Ok(())
                }
                Backend::Test { .. } => unreachable!(),
            },
        )
    }

    pub fn purge_soft_deleted(&self, ids: &[u64]) -> Result<(), EngineError> {
        self.with_backend_mut(
            Operation::PurgeSoftDeleted,
            |backend, game_id| match backend {
                Backend::Sqlite { pool } => {
                    let conn = pool.conn();
                    for id in ids {
                        conn.execute(
                            "DELETE FROM messages WHERE id = ?1 AND game_id = ?2",
                            rusqlite::params![*id as i64, game_id as i64],
                        )
                        .map_err(|e| {
                            EngineError::Config(format!("Failed to purge message: {e}"))
                        })?;
                    }
                    Ok(())
                }
                Backend::InMemory(data) => {
                    if let Some(vec) = data.messages.get_mut(&game_id) {
                        vec.retain(|m| !ids.contains(&m.id));
                    }
                    Ok(())
                }
                Backend::Test { .. } => unreachable!(),
            },
        )
    }

    // ─── Message swipe methods ────────────────────────────────────────────────

    pub fn insert_swipe(
        &self,
        message_id: u64,
        swipe: &Swipe,
        index: usize,
    ) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::InsertSwipe, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "INSERT INTO message_swipes (message_id, swipe_index, text, snapshot_id, location_header, event_header)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        message_id as i64,
                        index as i64,
                        &swipe.text,
                        swipe.snapshot_id.map(|id| id as i64),
                        swipe.location_header.as_deref(),
                        swipe.event_header.as_deref(),
                    ],
                )
                .map_err(|e| EngineError::Config(format!("Failed to insert swipe: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                let entry = data.swipes.entry(message_id).or_default();
                if index < entry.len() {
                    entry.insert(index, swipe.clone());
                } else {
                    entry.push(swipe.clone());
                }
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn update_swipe_text(
        &self,
        message_id: u64,
        swipe_index: usize,
        text: &str,
    ) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::UpdateSwipeText, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "UPDATE message_swipes SET text = ?1 WHERE message_id = ?2 AND swipe_index = ?3",
                    rusqlite::params![text, message_id as i64, swipe_index as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to update swipe text: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                if let Some(swipes) = data.swipes.get_mut(&message_id) {
                    if let Some(swipe) = swipes.get_mut(swipe_index) {
                        swipe.text = text.to_string();
                    }
                }
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn shift_swipe_indices(&self, message_id: u64, offset: usize) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::ShiftSwipeIndices, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute(
                    "UPDATE message_swipes SET swipe_index = swipe_index + ?1 WHERE message_id = ?2",
                    rusqlite::params![offset as i64, message_id as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to shift swipe indices: {e}")))?;
                Ok(())
            }
            Backend::InMemory(_) => {
                // No-op for in-memory: insert_swipe handles index insertion directly.
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn load_swipes_for_messages(
        &self,
        message_ids: &[u64],
    ) -> Result<HashMap<u64, Vec<Swipe>>, EngineError> {
        self.with_backend_mut(Operation::LoadSwipesForMessages, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                if message_ids.is_empty() {
                    return Ok(HashMap::new());
                }

                let conn = pool.conn();
                let placeholders = message_ids
                    .iter()
                    .map(|_| "?")
                    .collect::<Vec<_>>()
                    .join(",");
                let sql = format!(
                    "SELECT message_id, swipe_index, text, snapshot_id, location_header, event_header
                     FROM message_swipes
                     WHERE message_id IN ({placeholders})
                     ORDER BY message_id, swipe_index"
                );

                let mut stmt = conn
                    .prepare(&sql)
                    .map_err(|e| EngineError::Config(format!("Failed to prepare swipe query: {e}")))?;

                let rows = stmt
                    .query_map(
                        rusqlite::params_from_iter(message_ids.iter().map(|id| *id as i64)),
                        |row| {
                            Ok((
                                row.get::<_, i64>(0)? as u64,
                                Swipe {
                                    text: row.get(2)?,
                                    snapshot_id: row.get::<_, Option<i64>>(3)?.map(|id| id as u64),
                                    location_header: row.get(4)?,
                                    event_header: row.get(5)?,
                                },
                            ))
                        },
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to query swipes: {e}")))?;

                let mut result: HashMap<u64, Vec<Swipe>> = HashMap::new();
                for row in rows {
                    let (message_id, swipe) =
                        row.map_err(|e| EngineError::Config(format!("Failed to read swipe row: {e}")))?;
                    result.entry(message_id).or_default().push(swipe);
                }

                Ok(result)
            }
            Backend::InMemory(data) => {
                let mut result = HashMap::new();
                for &id in message_ids {
                    if let Some(s) = data.swipes.get(&id) {
                        result.insert(id, s.clone());
                    }
                }
                Ok(result)
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    // ─── Prompt preset methods ────────────────────────────────────────────────

    pub fn list_presets(&self, preset_type: PresetType) -> Result<Vec<PromptPreset>, EngineError> {
        self.with_backend_mut(Operation::ListPresets, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at
                         FROM prompt_presets
                         WHERE preset_type = ?1
                         ORDER BY updated_at DESC",
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

                let rows = stmt
                    .query_map([preset_type.as_str()], db_row_to_preset)
                    .map_err(|e| EngineError::Config(format!("Failed to query presets: {e}")))?;

                let mut presets = Vec::new();
                for row in rows {
                    let db =
                        row.map_err(|e| EngineError::Config(format!("Failed to read preset row: {e}")))?;
                    presets.push(from_db(db));
                }
                Ok(presets)
            }
            Backend::InMemory(data) => {
                Ok(data.presets
                    .iter()
                    .filter(|p| p.preset_type == preset_type)
                    .cloned()
                    .collect())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn get_preset(&self, id: &str) -> Result<Option<PromptPreset>, EngineError> {
        self.with_backend_mut(Operation::GetPreset, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at
                         FROM prompt_presets
                         WHERE id = ?1",
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

                let mut rows = stmt
                    .query_map([id], db_row_to_preset)
                    .map_err(|e| EngineError::Config(format!("Failed to query preset: {e}")))?;

                match rows.next() {
                    Some(row) => {
                        let db = row
                            .map_err(|e| EngineError::Config(format!("Failed to read preset row: {e}")))?;
                        Ok(Some(from_db(db)))
                    }
                    None => Ok(None),
                }
            }
            Backend::InMemory(data) => {
                Ok(data.presets.iter().find(|p| p.id == id).cloned())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn save_preset(&self, preset: &PromptPreset) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::SavePreset, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = chrono::Utc::now().to_rfc3339();
                let is_default = if preset.is_default { 1 } else { 0 };

                conn.execute(
                    "INSERT INTO prompt_presets (id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                     ON CONFLICT(id) DO UPDATE SET
                         name = excluded.name,
                         preset_type = excluded.preset_type,
                         role = excluded.role,
                         instructions = excluded.instructions,
                         writing_style = excluded.writing_style,
                         output_format = excluded.output_format,
                         is_default = excluded.is_default,
                         updated_at = excluded.updated_at",
                    rusqlite::params![
                        preset.id,
                        preset.name,
                        preset.preset_type.as_str(),
                        preset.role,
                        preset.instructions,
                        preset.writing_style,
                        preset.output_format,
                        is_default,
                        now,
                        now,
                    ],
                )
                .map_err(|e| EngineError::Config(format!("Failed to save preset: {e}")))?;

                Ok(())
            }
            Backend::InMemory(data) => {
                if let Some(idx) = data.presets.iter().position(|p| p.id == preset.id) {
                    data.presets[idx] = preset.clone();
                } else {
                    data.presets.push(preset.clone());
                }
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn delete_preset(&self, id: &str) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::DeletePreset, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute("DELETE FROM prompt_presets WHERE id = ?1", [id])
                    .map_err(|e| EngineError::Config(format!("Failed to delete preset: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                data.presets.retain(|p| p.id != id);
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    // ─── LLM message methods ──────────────────────────────────────────────────

    pub fn save_llm_message(&self, message: &LlmMessage) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::SaveLlmMessage, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let db_msg = DbLlmMessage::from(message);
                conn.execute(
                    "INSERT INTO llm_messages
                     (agent_name, backend_name, model_name, system_prompt, user_prompt,
                      raw_request_json, raw_response_json, parsed_response, error_message, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    rusqlite::params![
                        db_msg.agent_name,
                        db_msg.backend_name,
                        db_msg.model_name,
                        db_msg.system_prompt,
                        db_msg.user_prompt,
                        db_msg.raw_request_json,
                        db_msg.raw_response_json,
                        db_msg.parsed_response,
                        db_msg.error_message,
                        db_msg.created_at,
                    ],
                )
                .map_err(|e| EngineError::Config(format!("Failed to save LLM message: {e}")))?;

                conn.execute(
                    "DELETE FROM llm_messages
                     WHERE id NOT IN (
                         SELECT id FROM llm_messages ORDER BY created_at DESC LIMIT 50
                     )",
                    [],
                )
                .map_err(|e| EngineError::Config(format!("Failed to prune LLM messages: {e}")))?;

                Ok(())
            }
            Backend::InMemory(data) => {
                data.llm_messages.push(message.clone());
                if data.llm_messages.len() > 50 {
                    data.llm_messages.remove(0);
                }
                Ok(())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }

    pub fn list_latest_llm_messages(&self, limit: usize) -> Result<Vec<LlmMessage>, EngineError> {
        self.with_backend_mut(Operation::ListLatestLlmMessages, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, agent_name, backend_name, model_name, system_prompt, user_prompt,
                                raw_request_json, raw_response_json, parsed_response, error_message, created_at
                         FROM llm_messages
                         ORDER BY created_at DESC
                         LIMIT ?1",
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

                let rows = stmt
                    .query_map([limit as i64], |row| {
                        Ok(DbLlmMessage {
                            id: row.get(0)?,
                            agent_name: row.get(1)?,
                            backend_name: row.get(2)?,
                            model_name: row.get(3)?,
                            system_prompt: row.get(4)?,
                            user_prompt: row.get(5)?,
                            raw_request_json: row.get(6)?,
                            raw_response_json: row.get(7)?,
                            parsed_response: row.get(8)?,
                            error_message: row.get(9)?,
                            created_at: row.get(10)?,
                        })
                    })
                    .map_err(|e| EngineError::Config(format!("Failed to query LLM messages: {e}")))?;

                let mut messages = Vec::new();
                for row in rows {
                    let db_msg = row
                        .map_err(|e| EngineError::Config(format!("Failed to read LLM message row: {e}")))?;
                    messages.push(LlmMessage::try_from(&db_msg)?);
                }
                messages.reverse();
                Ok(messages)
            }
            Backend::InMemory(data) => {
                let start = data.llm_messages.len().saturating_sub(limit);
                Ok(data.llm_messages[start..].to_vec())
            }
            Backend::Test { .. } => unreachable!(),
        })
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn parse_datetime(
    rfc3339: &str,
    field: &str,
) -> Result<chrono::DateTime<chrono::Utc>, EngineError> {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map_err(|e| EngineError::Config(format!("Invalid {field}: {e}")))
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

fn db_game_to_game(db: &DbGame) -> Result<Game, EngineError> {
    Ok(Game {
        id: db.id as u64,
        world_name: db.world_name.clone(),
        name: db.name.clone(),
        created_at: parse_datetime(&db.created_at, "created_at")?,
        updated_at: parse_datetime(&db.updated_at, "updated_at")?,
    })
}

fn db_row_to_preset(row: &rusqlite::Row) -> rusqlite::Result<DbPromptPreset> {
    Ok(DbPromptPreset {
        id: row.get(0)?,
        name: row.get(1)?,
        preset_type: row.get(2)?,
        role: row.get(3)?,
        instructions: row.get(4)?,
        writing_style: row.get(5)?,
        output_format: row.get(6)?,
        is_default: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn from_db(db: DbPromptPreset) -> PromptPreset {
    let preset_type = match db.preset_type.as_str() {
        "quantifier" => PresetType::Quantifier,
        _ => PresetType::System,
    };
    PromptPreset {
        id: db.id,
        name: db.name,
        role: db.role,
        instructions: db.instructions,
        writing_style: db.writing_style,
        output_format: db.output_format,
        is_default: db.is_default != 0,
        preset_type,
    }
}
