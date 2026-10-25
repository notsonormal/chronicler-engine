//! [DOC: docs/diataxis/reference/storage.md]
//! Storage backend and core abstractions

use std::collections::HashMap;
use std::sync::Mutex;
#[cfg(feature = "testing")]
use std::sync::Arc;

use crate::adapters::driven::storage::changes::ChangeCounters;
use crate::adapters::driven::storage::db::DbPool;
use crate::adapters::driven::storage::transaction::TxWrite;
use crate::domain::model::game::GameId;
use crate::error::{DatabaseContext, EngineError};
#[cfg(feature = "testing")]
use super::test_support::{TestFailureHandle, TestOverride};

pub struct Storage {
    backend: Mutex<BackendKind>,
    changes: ChangeCounters,
}

/// [TRIVIAL_ENUM]
#[derive(Clone)]
pub enum Backend {
    Sqlite { pool: DbPool },
}

/// One variant per `Storage` method that the failure layer can fail, keyed by a
/// type like `TxWrite`, so a failure injection stays compiler-checked.
/// [TRIVIAL_ENUM]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StorageCall {
    CountGamesUsingPreset,
    CreateWorld,
    DeletePreset,
    DeleteWorld,
    DocksGet,
    GetCharacter,
    GetGame,
    GetPersona,
    GetPreset,
    GetSettings,
    GetWorld,
    GetWorldById,
    ListCharacters,
    ListGames,
    ListLatestLlmMessages,
    ListPersonas,
    ListPresets,
    ListWorlds,
    NewestGameOfWorld,
    SaveLlmMessage,
    SavePreset,
    SaveSettings,
    SeedCharacter,
    SeedGameRow,
    SeedPersona,
    SeedSettings,
    SeedWorld,
    StoriesActiveSwipe,
    StoriesEntries,
    StoriesHasText,
    StoriesMessages,
    StoriesSwipeState,
    StoriesTail,
    StoriesWindow,
    TouchGameUpdatedAt,
    Transaction,
    UpdateGameDisplayName,
    UpdateSettings,
    UpdateWorld,
}

/// [TRIVIAL_ENUM]
pub enum BackendKind {
    Direct(Backend),
    #[cfg(feature = "testing")]
    Test {
        // `Box<Backend>`, not `Box<BackendKind>`: pins the replace-not-nest
        // invariant, so at most one `Test` layer wraps one `Direct` backend.
        base: Box<Backend>,
        overrides: Arc<Mutex<HashMap<StorageCall, TestOverride>>>,
        // Checked inside the unit, unlike `overrides`, which keys on the storage
        // method.
        writes: Arc<Mutex<HashMap<TxWrite, TestOverride>>>,
    },
}

pub(crate) enum WriteOverrides {
    /// No records fail: a `Direct` backend, or a build without the test layer.
    Direct,
    /// The `Test` layer's map of record-write failures.
    #[cfg(feature = "testing")]
    Test(Arc<Mutex<HashMap<TxWrite, TestOverride>>>),
}

impl WriteOverrides {
    /// Refuse `write` when the test layer planted a failure on it.
    #[cfg_attr(not(feature = "testing"), allow(unused_variables))]
    pub(crate) fn fail_if_injected(&self, write: TxWrite) -> Result<(), EngineError> {
        match self {
            Self::Direct => Ok(()),
            #[cfg(feature = "testing")]
            Self::Test(writes) => {
                match writes.lock().unwrap_or_else(|e| e.into_inner()).get(&write) {
                    Some(override_) => Err(override_.to_error()),
                    None => Ok(()),
                }
            }
        }
    }
}

impl Storage {
    pub fn new_sqlite(pool: DbPool) -> Self {
        Self {
            backend: Mutex::new(BackendKind::Direct(Backend::Sqlite { pool })),
            changes: ChangeCounters::new(),
        }
    }

    /// The numbers that say which kinds of data changed. Storage raises them itself; the
    /// busy registry raises the busy state.
    pub fn changes(&self) -> &ChangeCounters {
        &self.changes
    }

    /// A SQLite `:memory:` database, migrated like a file database.
    #[cfg(any(test, feature = "testing"))]
    #[allow(clippy::expect_used)]
    pub fn new_in_memory() -> Self {
        // arch-lint: allow(no-unwrap-expect) reason="a fresh :memory: database cannot fail to open or migrate"
        let pool = DbPool::new(":memory:").expect("open a SQLite :memory: database");
        Self::new_sqlite(pool)
    }

    #[cfg(feature = "testing")]
    pub fn with_failure(self, call: StorageCall, override_: TestOverride) -> Self {
        self.add_failure(call, override_);
        self
    }

    #[cfg(feature = "testing")]
    pub fn with_test_failures(self) -> (Self, TestFailureHandle) {
        let overrides = Arc::new(Mutex::new(HashMap::new()));
        let writes = Arc::new(Mutex::new(HashMap::new()));
        let storage = self.with_overrides(Arc::clone(&overrides), Arc::clone(&writes));
        (storage, TestFailureHandle { overrides, writes })
    }

    #[cfg(feature = "testing")]
    pub fn add_failure(&self, call: StorageCall, override_: TestOverride) {
        let mut backend = match self.backend.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let direct = match &*backend {
            BackendKind::Test { overrides, .. } => {
                overrides
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(call, override_);
                return;
            }
            BackendKind::Direct(direct) => direct.clone(),
        };
        let mut overrides = HashMap::new();
        overrides.insert(call, override_);
        *backend = BackendKind::Test {
            base: Box::new(direct),
            overrides: Arc::new(Mutex::new(overrides)),
            writes: Arc::new(Mutex::new(HashMap::new())),
        };
    }

    #[cfg(feature = "testing")]
    fn with_overrides(
        self,
        overrides: Arc<Mutex<HashMap<StorageCall, TestOverride>>>,
        writes: Arc<Mutex<HashMap<TxWrite, TestOverride>>>,
    ) -> Self {
        let changes = self.changes;
        let backend = match self.backend.into_inner() {
            Ok(b) => b,
            Err(poisoned) => poisoned.into_inner(),
        };
        let base = match backend {
            BackendKind::Direct(other) => Box::new(other),
            BackendKind::Test { base, .. } => {
                debug_assert!(
                    false,
                    "with_overrides called on Storage already in Test mode; existing overrides silently dropped"
                );
                base
            }
        };
        Self {
            backend: Mutex::new(BackendKind::Test {
                base,
                overrides,
                writes,
            }),
            changes,
        }
    }

    pub(crate) fn with_backend_mut<F, T>(&self, call: StorageCall, f: F) -> Result<T, EngineError>
    where
        F: FnOnce(&mut Backend) -> Result<T, EngineError>,
    {
        self.with_backend_writes(call, |backend, _writes| f(backend))
    }

    #[allow(unused_variables)]
    pub(crate) fn with_backend_writes<F, T>(
        &self,
        call: StorageCall,
        f: F,
    ) -> Result<T, EngineError>
    where
        F: FnOnce(&mut Backend, WriteOverrides) -> Result<T, EngineError>,
    {
        let mut backend = match self.backend.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let result = match &mut *backend {
            BackendKind::Direct(inner) => f(inner, WriteOverrides::Direct),
            #[cfg(feature = "testing")]
            BackendKind::Test {
                overrides,
                base,
                writes,
            } => {
                if let Some(override_) = overrides
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(&call)
                {
                    return Err(override_.to_error());
                }
                f(base.as_mut(), WriteOverrides::Test(Arc::clone(writes)))
            }
        };
        // Raised while the backend lock is held: a load that reads the committed data
        // waits for the lock, so it never sees a raised number with the old data.
        if result.is_ok() {
            self.changes
                .raise(None, ChangeCounters::kinds_of_call(call));
        }
        result
    }
}

impl Storage {
    /// The `updated_at` of a Game orders the Games of a World that hold no
    /// Message.
    pub fn touch_game_updated_at(&self, game_id: GameId) -> Result<(), EngineError> {
        self.with_backend_mut(StorageCall::TouchGameUpdatedAt, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = chrono::Utc::now().to_rfc3339();
                conn.execute(
                    "UPDATE games SET updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![&now, game_id.0 as i64],
                )
                .db_context("Failed to touch the game")?;
                Ok(())
            }
        })
    }
}

#[cfg(test)]
impl Storage {
    pub(crate) fn backend_layer_info(&self) -> (&'static str, &'static str) {
        let backend = self.backend.lock().unwrap_or_else(|e| e.into_inner());
        match &*backend {
            BackendKind::Direct(b) => ("Direct", b.variant_name()),
            BackendKind::Test { base, .. } => ("Test", base.as_ref().variant_name()),
        }
    }
}

#[cfg(test)]
impl Backend {
    fn variant_name(&self) -> &'static str {
        match self {
            Backend::Sqlite { .. } => "Sqlite",
        }
    }
}
