//! [DOC: docs/system/storage.md]
//! Storage backend trait and core abstractions

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
#[cfg(feature = "testing")]
use std::sync::Arc;

use crate::error::EngineError;
use crate::domain::model::character::{NpcCard, PlayerCard};
use crate::domain::model::game::Game;
use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::map::MapDef;
use crate::domain::model::message::{Message, Swipe};
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state_snapshot::GameStateSnapshot;
use crate::domain::model::world::WorldCard;
use crate::storage::db::DbPool;
#[cfg(feature = "testing")]
use super::test_support::{TestFailureHandle, TestOverride};

pub struct Storage {
    game_id: AtomicU64,
    backend: Mutex<LayeredBackend>,
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
    pub worlds: Vec<InMemoryWorld>,
    pub personas: Vec<PlayerCardWithKey>,
    pub characters: Vec<CharacterSeed>,
    pub settings: AppSettings,
}

pub struct InMemoryWorld {
    pub world_id: i64,
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
    Sqlite { pool: DbPool },
    InMemory(Box<InMemoryData>),
}

/// Non-recursive by design: `Test.base` is `Box<Backend>`, not `Box<LayeredBackend>` —
/// enforces "at most one Test layer" (replace-not-nest invariant).
pub enum LayeredBackend {
    Direct(Backend),
    #[cfg(feature = "testing")]
    Test {
        base: Box<Backend>,
        overrides: Arc<Mutex<HashMap<&'static str, TestOverride>>>,
    },
}

impl InMemoryData {
    /// Empty placeholder used by constructors that need a throwaway backend
    /// for `mem::replace` borrow-checker workaround.
    pub(crate) fn empty() -> Self {
        Self {
            snapshots: HashMap::new(),
            next_snapshot_id: 1,
            games: Vec::new(),
            next_game_id: 1,
            messages: HashMap::new(),
            next_message_id: 0,
            swipes: HashMap::new(),
            presets: Vec::new(),
            llm_messages: Vec::new(),
            worlds: Vec::new(),
            personas: Vec::new(),
            characters: Vec::new(),
            settings: AppSettings::default(),
        }
    }
}

impl Storage {
    pub fn new_sqlite(pool: DbPool, game_id: u64) -> Self {
        Self {
            game_id: AtomicU64::new(game_id),
            backend: Mutex::new(LayeredBackend::Direct(Backend::Sqlite { pool })),
        }
    }

    pub fn new_in_memory() -> Self {
        Self {
            game_id: AtomicU64::new(0), // No default game - calling code should create one explicitly
            backend: Mutex::new(LayeredBackend::Direct(Backend::InMemory(Box::new(
                InMemoryData::empty(),
            )))),
        }
    }

    #[cfg(feature = "testing")]
    pub fn with_failure(self, method: &'static str, override_: TestOverride) -> Self {
        self.add_failure(method, override_);
        self
    }

    #[cfg(feature = "testing")]
    pub fn with_test_failures(self) -> (Self, TestFailureHandle) {
        let overrides = Arc::new(Mutex::new(HashMap::new()));
        let storage = self.with_overrides(Arc::clone(&overrides));
        (storage, TestFailureHandle { overrides })
    }

    #[cfg(feature = "testing")]
    pub fn add_failure(&self, method: &'static str, override_: TestOverride) {
        use std::mem;
        let mut backend = match self.backend.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let current = mem::replace(
            &mut *backend,
            LayeredBackend::Direct(Backend::InMemory(Box::new(InMemoryData::empty()))),
        );
        let next = match current {
            LayeredBackend::Test { base, overrides } => {
                overrides
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(method, override_);
                LayeredBackend::Test { base, overrides }
            }
            LayeredBackend::Direct(other) => {
                let mut overrides = HashMap::new();
                overrides.insert(method, override_);
                LayeredBackend::Test {
                    base: Box::new(other),
                    overrides: Arc::new(Mutex::new(overrides)),
                }
            }
        };
        *backend = next;
    }

    #[cfg(feature = "testing")]
    fn with_overrides(self, overrides: Arc<Mutex<HashMap<&'static str, TestOverride>>>) -> Self {
        let backend = match self.backend.into_inner() {
            Ok(b) => b,
            Err(poisoned) => poisoned.into_inner(),
        };
        let base = match backend {
            LayeredBackend::Direct(other) => Box::new(other),
            LayeredBackend::Test { base, .. } => {
                debug_assert!(
                    false,
                    "with_overrides called on Storage already in Test mode; existing overrides silently dropped"
                );
                base
            }
        };
        Self {
            game_id: self.game_id,
            backend: Mutex::new(LayeredBackend::Test { base, overrides }),
        }
    }

    pub(crate) fn game_id(&self) -> u64 {
        self.game_id.load(Ordering::SeqCst)
    }

    #[allow(unused_variables)]
    pub(crate) fn with_backend_mut<F, T>(
        &self,
        method: &'static str,
        f: F,
    ) -> Result<T, EngineError>
    where
        F: FnOnce(&mut Backend) -> Result<T, EngineError>,
    {
        let mut backend = match self.backend.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        match &mut *backend {
            LayeredBackend::Direct(inner) => f(inner),
            #[cfg(feature = "testing")]
            LayeredBackend::Test { overrides, base } => {
                if let Some(override_) = overrides
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(method)
                {
                    return Err(override_.to_error());
                }
                f(base.as_mut())
            }
        }
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
            let target = match &*backend {
                LayeredBackend::Direct(b) => b,
                #[cfg(feature = "testing")]
                LayeredBackend::Test { base, .. } => base.as_ref(),
            };
            if let Backend::Sqlite { pool } = target {
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

#[cfg(test)]
impl Storage {
    /// Pins non-recursive replace-not-nest invariant:
    /// `with_failure`/`add_failure` must produce at most one `Test` layer with a `Direct` base.
    pub(crate) fn backend_layer_info(&self) -> (&'static str, &'static str) {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        match &*backend {
            LayeredBackend::Direct(b) => ("Direct", backend_variant_name(b)),
            LayeredBackend::Test { base, .. } => ("Test", backend_variant_name(base.as_ref())),
        }
    }
}

#[cfg(test)]
fn backend_variant_name(b: &Backend) -> &'static str {
    match b {
        Backend::Sqlite { .. } => "Sqlite",
        Backend::InMemory(_) => "InMemory",
    }
}
