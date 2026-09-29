//! `ApiKeyEnvGuard` — serializes tests that mutate API-key environment variables.

use std::sync::{Mutex, MutexGuard};

static API_KEY_ENV_LOCK: Mutex<()> = Mutex::new(());

/// Hold for the duration of a test that sets or removes an API-key env var.
/// `cargo test --lib` runs the lib suite in one process on parallel threads, so
/// an unguarded `set_var` races any test asserting the variable is unset.
pub struct ApiKeyEnvGuard {
    _lock: MutexGuard<'static, ()>,
}

impl Default for ApiKeyEnvGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiKeyEnvGuard {
    pub fn new() -> Self {
        Self {
            _lock: API_KEY_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner()),
        }
    }
}
