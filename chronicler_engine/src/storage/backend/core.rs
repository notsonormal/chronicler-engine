//! [DOC: docs/system/storage.md]
//! SQLite connection pooling and transactions

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::{EngineError, internal_error};
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::game::Game;
use crate::model::llm_message::LlmMessage;
use crate::model::map::MapDef;
use crate::model::message::{Message, Swipe};
use crate::model::prompt_preset::PromptPreset;
use crate::model::settings::AppSettings;
use crate::model::state_snapshot::GameStateSnapshot;
use crate::model::world::{WorldCard, WorldManifest};
use crate::storage::db::DbPool;

pub struct Storage {
    game_id: AtomicU64,
    backend: Mutex<Backend>,
}

pub struct InMemoryData {
    pub snapshots: HashMap<u64, Vec<GameStateSnapshot>>,
    pub next_snapshot_id: u64,
    pub games: Vec<Game>,
    pub next_game_id: u64,
    pub messages: HashMap<u64, Vec<Message>>,
    pub next_message_id: u64,
    pub swipes: HashMap<u64, Vec<Swipe>>,
    pub presets: Vec<PromptPreset>,
    pub llm_messages: Vec<LlmMessage>,
    pub worlds: Vec<WorldSeed>,
    pub personas: Vec<PlayerCardWithKey>,
    pub characters: Vec<CharacterSeed>,
    pub settings: AppSettings,
}

pub struct WorldSeed {
    pub manifest: WorldManifest,
    pub world_card: WorldCard,
    pub map: MapDef,
}

pub struct PlayerCardWithKey {
    pub key: String,
    pub card: PlayerCard,
}

pub struct CharacterSeed {
    pub world_id: i64,
    pub card: NpcCard,
}

pub enum Backend {
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
    ListWorlds,
    GetWorld,
    SeedWorld,
    ListPersonas,
    GetPersona,
    SeedPersona,
    ListCharacters,
    GetCharacter,
    SeedCharacter,
    GetSettings,
    SaveSettings,
    SeedSettings,
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
                worlds: vec![],
                personas: vec![],
                characters: vec![],
                settings: AppSettings::default(),
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
                worlds: vec![],
                personas: vec![],
                characters: vec![],
                settings: AppSettings::default(),
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
        let base = match backend {
            Backend::Test { base, .. } => base,
            other => Box::new(other),
        };
        Self {
            game_id: self.game_id,
            backend: Mutex::new(Backend::Test { base, overrides }),
        }
    }

    pub(crate) fn game_id(&self) -> u64 {
        self.game_id.load(Ordering::SeqCst)
    }

    pub(crate) fn with_backend_mut<F, T>(&self, op: Operation, f: F) -> Result<T, EngineError>
    where
        F: FnOnce(&mut Backend, u64) -> Result<T, EngineError>,
    {
        let mut backend = match self.backend.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut current = &mut *backend;
        while let Backend::Test { overrides, base } = current {
            if let Some(override_) = overrides.lock().unwrap_or_else(|e| e.into_inner()).get(&op) {
                return Err(override_.to_error());
            }
            current = base.as_mut();
        }
        let game_id = self.game_id();
        f(current, game_id)
    }
}

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
                    tracing::error!("Failed to update games.updated_at for game {game_id}: {e}");
                }
            }
            drop(backend);
            self.game_id.store(game_id, Ordering::SeqCst);
        }
    }

    pub fn current_game_id(&self) -> u64 {
        self.game_id()
    }
}
