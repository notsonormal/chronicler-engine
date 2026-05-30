//! [DOC: docs/reference/testing.md]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

#[path = "components/actions.rs"]
mod actions;
#[path = "components/connections.rs"]
mod connections;
#[path = "components/css.rs"]
mod css;
#[path = "components/debug.rs"]
mod debug;
#[path = "components/fragment.rs"]
mod fragment;
#[path = "components/misc.rs"]
mod misc;
#[path = "components/prompt_presets.rs"]
mod prompt_presets;
#[path = "components/settings.rs"]
mod settings;
#[path = "components/state_patch_tests.rs"]
mod state_patch;
#[path = "components/text_check.rs"]
mod text_check;
#[path = "components/world.rs"]
mod world;

static SETTINGS_TEST_LOCK: Mutex<()> = Mutex::new(());
static SETTINGS_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TempSettingsGuard {
    _lock: MutexGuard<'static, ()>,
    temp_path: std::path::PathBuf,
}

impl Default for TempSettingsGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl TempSettingsGuard {
    pub fn new() -> Self {
        let lock = SETTINGS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let counter = SETTINGS_TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_path = std::env::temp_dir().join(format!(
            "chronicler_test_settings_{}_{}.json",
            std::process::id(),
            counter
        ));
        unsafe { std::env::set_var("CHRONICLER_SETTINGS_PATH", &temp_path) };
        Self {
            _lock: lock,
            temp_path,
        }
    }
}

impl Drop for TempSettingsGuard {
    fn drop(&mut self) {
        unsafe { std::env::remove_var("CHRONICLER_SETTINGS_PATH") };
        let _ = std::fs::remove_file(&self.temp_path);
    }
}
