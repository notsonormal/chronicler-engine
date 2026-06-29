//! [DOC: docs/system/llm_processing.md]
//! LLM message storage

use crate::error::EngineError;
use crate::domain::model::llm_message::LlmMessage;
use crate::storage::backend::{Backend, Storage};
use crate::storage::models::llm_message::DbLlmMessage;

impl Storage {
    pub fn save_llm_message(&self, message: &LlmMessage) -> Result<(), EngineError> {
        self.with_backend_mut("save_llm_message", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let db_msg = DbLlmMessage::from(message);
                conn.execute(
                    "INSERT INTO llm_messages
                     (agent_name, backend_name, model_name, system_prompt, user_prompt,
                      raw_request_json, raw_response_json, parsed_response, error_message, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    rusqlite::params![
                        db_msg.agent_name,
                        db_msg.backend_name,
                        db_msg.model_name,
                        db_msg.system_prompt,
                        db_msg.user_prompt,
                        db_msg.raw_request_json,
                        db_msg.raw_response_json,
                        db_msg.parsed_response,
                        db_msg.error_message,
                        db_msg.created_at,
                    ],
                )
                .map_err(|e| EngineError::Config(format!("Failed to save LLM message: {e}")))?;

                conn.execute(
                    "DELETE FROM llm_messages
                     WHERE id NOT IN (
                         SELECT id FROM llm_messages ORDER BY created_at DESC LIMIT 50
                     )",
                    [],
                )
                .map_err(|e| EngineError::Config(format!("Failed to prune LLM messages: {e}")))?;

                Ok(())
            }
            Backend::InMemory(data) => {
                data.llm_messages.push(message.clone());
                if data.llm_messages.len() > 50 {
                    data.llm_messages.remove(0);
                }
                Ok(())
            }
        })
    }

    pub fn list_latest_llm_messages(&self, limit: usize) -> Result<Vec<LlmMessage>, EngineError> {
        self.with_backend_mut("list_latest_llm_messages", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, agent_name, backend_name, model_name, system_prompt, user_prompt,
                                raw_request_json, raw_response_json, parsed_response, error_message, created_at
                         FROM llm_messages
                         ORDER BY created_at DESC
                         LIMIT ?1",
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

                let rows = stmt
                    .query_map([limit as i64], DbLlmMessage::from_row)
                    .map_err(|e| EngineError::Config(format!("Failed to query LLM messages: {e}")))?;

                let mut messages = Vec::new();
                for row in rows {
                    let db_msg = row
                        .map_err(|e| EngineError::Config(format!("Failed to read LLM message row: {e}")))?;
                    messages.push(LlmMessage::try_from(&db_msg)?);
                }
                messages.reverse();
                Ok(messages)
            }
            Backend::InMemory(data) => {
                let start = data.llm_messages.len().saturating_sub(limit);
                Ok(data.llm_messages[start..].to_vec())
            }
        })
    }
}
