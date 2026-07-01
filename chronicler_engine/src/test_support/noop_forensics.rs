//! Canonical NoopForensics implementation for tests.
//!
//! Provides a no-op `LlmMessageRepository` implementation useful for tests
//! that don't need to persist LLM messages. Also provides helper functions
//! to construct `LlmCallRecorder` instances with noop forensics.

use std::sync::Arc;

use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_message_repository::{LlmMessage, LlmMessageRepository};
use crate::application::ports::llm_provider::LlmProvider;
use crate::error::EngineError;

/// No-op implementation of `LlmMessageRepository` for tests.
///
/// All methods succeed without persisting anything.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopForensics;

impl LlmMessageRepository for NoopForensics {
    fn save_llm_message(&self, _message: &LlmMessage) -> Result<(), EngineError> {
        Ok(())
    }

    fn list_latest_llm_messages(&self, _limit: usize) -> Result<Vec<LlmMessage>, EngineError> {
        Ok(vec![])
    }
}

/// Test helper: wrap an LlmProvider in LlmCallRecorder with noop forensics.
pub fn make_test_recorder(provider: Arc<dyn LlmProvider>) -> Arc<LlmCallRecorder> {
    Arc::new(LlmCallRecorder::new(provider, Arc::new(NoopForensics)))
}

/// Test helper: wrap an LlmProvider in LlmCallRecorder with real storage forensics.
pub fn make_test_recorder_with_storage(
    provider: Arc<dyn LlmProvider>,
    storage: Arc<crate::adapters::driven::storage::Storage>,
) -> Arc<LlmCallRecorder> {
    Arc::new(LlmCallRecorder::new(provider, storage))
}
