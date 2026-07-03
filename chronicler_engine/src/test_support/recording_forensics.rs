//! Recording spy for `LlmMessageRepository`.
//!
//! Companion to `noop_forensics::NoopForensics`. Where `NoopForensics` discards
//! everything, `RecordingForensics` captures calls so tests can assert on what
//! the orchestrator (`LlmCallRecorder`) actually persisted.
//!
//! Use this when a test needs to verify:
//! - how many times `save_llm_message` was called,
//! - the contents of the last saved `LlmMessage`,
//! - or that a forensics write propagated an error.
//!
//! For tests that just need a sink with no behavior assertions, prefer
//! `NoopForensics`.

use parking_lot::Mutex;

use crate::application::ports::llm_message_repository::{LlmMessage, LlmMessageRepository};
use crate::error::EngineError;

/// Spy implementation of `LlmMessageRepository` for tests.
///
/// Counts `save_llm_message` calls, captures the last saved message, and can
/// be configured to return an error on the next write.
#[derive(Debug, Default)]
pub struct RecordingForensics {
    inner: Mutex<RecordingForensicsState>,
}

#[derive(Debug, Default)]
struct RecordingForensicsState {
    save_calls: usize,
    last_message: Option<LlmMessage>,
    /// When set, the next `save_llm_message` call returns this error.
    next_save_error: Option<EngineError>,
    /// Messages returned by `list_latest_llm_messages` regardless of what was saved.
    list_response: Vec<LlmMessage>,
}

impl RecordingForensics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the spy to return `err` on the next `save_llm_message` call.
    pub fn with_next_save_error(mut self, err: EngineError) -> Self {
        self.inner.get_mut().next_save_error = Some(err);
        self
    }

    /// Configure the spy to return `messages` from `list_latest_llm_messages`.
    pub fn with_list_response(mut self, messages: Vec<LlmMessage>) -> Self {
        self.inner.get_mut().list_response = messages;
        self
    }

    /// Number of times `save_llm_message` was called, including attempts that
    /// returned a configured error. Increments on entry, before the configured
    /// error (if any) is taken.
    pub fn save_call_count(&self) -> usize {
        self.inner.lock().save_calls
    }

    /// Snapshot of the most recently saved `LlmMessage`, if any.
    pub fn last_saved_message(&self) -> Option<LlmMessage> {
        self.inner.lock().last_message.clone()
    }
}

impl LlmMessageRepository for RecordingForensics {
    fn save_llm_message(&self, message: &LlmMessage) -> Result<(), EngineError> {
        let mut state = self.inner.lock();
        // Count every attempt on entry, before the configured error is taken.
        // Callers asserting on save_call_count see attempts, not just successes.
        state.save_calls += 1;
        if let Some(err) = state.next_save_error.take() {
            return Err(err);
        }
        state.last_message = Some(message.clone());
        Ok(())
    }

    fn list_latest_llm_messages(&self, _limit: usize) -> Result<Vec<LlmMessage>, EngineError> {
        Ok(self.inner.lock().list_response.clone())
    }
}
