#[path = "../helpers/pipeline_helpers.rs"]
mod pipeline_helpers;

#[path = "../helpers/fixtures.rs"]
mod fixtures;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;

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

pub fn failing_service() -> GameService {
    GameService::with_mock_quantifier(
        Arc::new(MockBackend::failing()),
        Arc::new(MockBackend::default()),
    )
}

pub fn working_service() -> GameService {
    GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default())
}

mod application_service;
mod game_service;
mod lifecycle;
mod llm_client;
mod model;
mod storage;

#[path = "flow/retry_event.rs"]
mod flow_retry_event;
#[path = "flow/retry_main.rs"]
mod flow_retry_main;
#[path = "flow/sequence.rs"]
mod flow_sequence;
#[path = "pipeline/actions.rs"]
mod pipeline_actions;
#[path = "pipeline/retry.rs"]
mod pipeline_retry;
#[path = "pipeline/pipeline.rs"]
mod pipeline_tests;
