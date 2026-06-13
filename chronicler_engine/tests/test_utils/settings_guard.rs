use std::sync::{Mutex, MutexGuard};

static SETTINGS_TEST_LOCK: Mutex<()> = Mutex::new(());

pub struct SettingsTestGuard {
    _lock: MutexGuard<'static, ()>,
}

impl Default for SettingsTestGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsTestGuard {
    pub fn new() -> Self {
        let lock = SETTINGS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        Self { _lock: lock }
    }
}
