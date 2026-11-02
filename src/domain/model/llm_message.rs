//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM message DTO: the stored record of one model call

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::model::prompt_part::{PromptPart, PromptRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub id: i64,
    pub agent_name: String,
    pub backend_name: String,
    pub model_name: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
    pub parsed_response: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl LlmMessage {
    /// A row from before `raw_request_json` holds two columns in its place, and a
    /// failure before any send holds nothing.
    pub fn sent_parts(&self) -> Vec<PromptPart> {
        let from_payload = serde_json::from_str::<serde_json::Value>(&self.raw_request_json)
            .ok()
            .and_then(|payload| payload.get("messages")?.as_array().cloned())
            .map(|messages| {
                messages
                    .iter()
                    .filter_map(|message| {
                        Some(PromptPart {
                            role: PromptRole::from_wire_name(message.get("role")?.as_str()?)?,
                            text: message.get("content")?.as_str()?.to_string(),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if !from_payload.is_empty() {
            return from_payload;
        }
        [
            PromptPart::system(&self.system_prompt),
            PromptPart::user(&self.user_prompt),
        ]
        .into_iter()
        .filter(|part| !part.text.is_empty())
        .collect()
    }
}
