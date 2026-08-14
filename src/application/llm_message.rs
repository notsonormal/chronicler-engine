//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM recorder save seam

use std::sync::Arc;

use crate::domain::model::llm_message::LlmMessage;
use crate::error::EngineError;

pub type SaveLlmMessageFn = Arc<dyn Fn(&LlmMessage) -> Result<(), EngineError> + Send + Sync>;
