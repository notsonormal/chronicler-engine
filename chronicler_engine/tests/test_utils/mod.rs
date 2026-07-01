//! [DOC: docs/reference/testing.md]

#![allow(dead_code)]

pub mod browser;
pub mod server;
pub mod settings_guard;
pub mod wait;

#[allow(unused_imports)]
pub use browser::*;
#[allow(unused_imports)]
pub use server::*;
#[allow(unused_imports)]
pub use wait::*;

use std::sync::Arc;
use chronicler_engine::application::llm_recorder::LlmCallRecorder;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::application::ports::llm_message_repository::LlmMessageRepository;
use chronicler_engine::error::EngineError;

pub const TEST_WORLD: &str = "test";
pub const TEST_PERSONA: &str = "test_player";
pub const CONFIG_PATH: &str = "tests/test_config.json";

/// Test helper: wrap an LlmProvider in LlmCallRecorder with noop forensics
pub fn make_test_recorder(provider: Arc<dyn LlmProvider>) -> Arc<LlmCallRecorder> {
    struct NoopForensics;
    impl LlmMessageRepository for NoopForensics {
        fn save_llm_message(
            &self,
            _: &chronicler_engine::application::ports::llm_message_repository::LlmMessage,
        ) -> Result<(), EngineError> {
            Ok(())
        }
        fn list_latest_llm_messages(
            &self,
            _: usize,
        ) -> Result<
            Vec<chronicler_engine::application::ports::llm_message_repository::LlmMessage>,
            EngineError,
        > {
            Ok(vec![])
        }
    }
    Arc::new(LlmCallRecorder::new(provider, Arc::new(NoopForensics)))
}

/// Test helper: wrap an LlmProvider in LlmCallRecorder with real storage forensics
pub fn make_test_recorder_with_storage(
    provider: Arc<dyn LlmProvider>,
    storage: Arc<chronicler_engine::adapters::driven::storage::Storage>,
) -> Arc<LlmCallRecorder> {
    Arc::new(LlmCallRecorder::new(provider, storage))
}
