//! [DOC: docs/diataxis/reference/storage.md]
//! Storage backend and core abstractions

use std::collections::HashMap;
use std::sync::Mutex;
#[cfg(feature = "testing")]
use std::sync::Arc;

use crate::adapters::driven::storage::db::DbPool;
use crate::adapters::driven::storage::in_memory_data::InMemoryData;
use crate::domain::model::game::GameId;
use crate::error::EngineError;
#[cfg(feature = "testing")]
use super::test_support::{TestFailureHandle, TestOverride};

pub struct Storage {
    backend: Mutex<BackendKind>,
}

/// [TRIVIAL_ENUM]
pub enum Backend {
    Sqlite { pool: DbPool },
    InMemory(Box<InMemoryData>),
}

/// [TRIVIAL_ENUM]
pub enum BackendKind {
    Direct(Backend),
    #[cfg(feature = "testing")]
    Test {
        // `Box<Backend>`, not `Box<BackendKind>`: pins the replace-not-nest
        // invariant, so at most one `Test` layer wraps one `Direct` backend.
        base: Box<Backend>,
        overrides: Arc<Mutex<HashMap<&'static str, TestOverride>>>,
    },
}

impl Storage {
    pub fn new_sqlite(pool: DbPool) -> Self {
        Self {
            backend: Mutex::new(BackendKind::Direct(Backend::Sqlite { pool })),
        }
    }

    pub fn new_in_memory() -> Self {
        Self {
            backend: Mutex::new(BackendKind::Direct(Backend::InMemory(Box::new(
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
            BackendKind::Direct(Backend::InMemory(Box::new(InMemoryData::empty()))),
        );
        let next = match current {
            BackendKind::Test { base, overrides } => {
                overrides
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(method, override_);
                BackendKind::Test { base, overrides }
            }
            BackendKind::Direct(other) => {
                let mut overrides = HashMap::new();
                overrides.insert(method, override_);
                BackendKind::Test {
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
            backend: Mutex::new(BackendKind::Test { base, overrides }),
        }
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
            BackendKind::Direct(inner) => f(inner),
            #[cfg(feature = "testing")]
            BackendKind::Test { overrides, base } => {
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
    /// The newest Game of a World is picked by `updated_at`, which
    /// `ActiveGame::set` relies on.
    pub fn touch_game_updated_at(&self, game_id: GameId) -> Result<(), EngineError> {
        self.with_backend_mut("touch_game_updated_at", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = chrono::Utc::now().to_rfc3339();
                conn.execute(
                    "UPDATE games SET updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![&now, game_id.0 as i64],
                )
                .map_err(|e| EngineError::Config(format!("Failed to touch game {game_id}: {e}")))?;
                Ok(())
            }
            Backend::InMemory(_) => Ok(()),
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
            Backend::InMemory(_) => "InMemory",
        }
    }
}
