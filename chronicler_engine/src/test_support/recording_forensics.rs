//! Recording spy for `LlmMessageRepository`

use parking_lot::Mutex;

use crate::application::ports::llm_message_repository::{LlmMessage, LlmMessageRepository};
use crate::error::EngineError;

#[derive(Debug, Default)]
pub struct RecordingForensics {
    inner: Mutex<RecordingForensicsState>,
}

#[derive(Debug, Default)]
struct RecordingForensicsState {
    save_calls: usize,
    last_message: Option<LlmMessage>,
    next_save_error: Option<EngineError>,
    list_response: Vec<LlmMessage>,
}

impl RecordingForensics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_next_save_error(mut self, err: EngineError) -> Self {
        self.inner.get_mut().next_save_error = Some(err);
        self
    }

    pub fn with_list_response(mut self, messages: Vec<LlmMessage>) -> Self {
        self.inner.get_mut().list_response = messages;
        self
    }

    /// Counts every `save_llm_message` *attempt*, including those that return
    /// the configured `next_save_error`. Increments before the error is taken.
    pub fn save_call_count(&self) -> usize {
        self.inner.lock().save_calls
    }

    pub fn last_saved_message(&self) -> Option<LlmMessage> {
        self.inner.lock().last_message.clone()
    }
}

impl LlmMessageRepository for RecordingForensics {
    fn save_llm_message(&self, message: &LlmMessage) -> Result<(), EngineError> {
        let mut state = self.inner.lock();
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
