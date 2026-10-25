//! [DOC: docs/diataxis/reference/storage.md]
//! LLM message storage

use crate::error::{DatabaseContext, EngineError};
use crate::adapters::driven::storage::{Backend, Storage, StorageCall};
use crate::adapters::driven::storage::models::llm_message::DbLlmMessage;
use crate::domain::model::llm_message::LlmMessage;

/// How many LLM message rows a database keeps. A read of the whole retained
/// window asks for this, and so sees every row retention left behind.
pub const LLM_MESSAGE_RETENTION_LIMIT: usize = 50;

impl Storage {
    pub fn save_llm_message(&self, message: &LlmMessage) -> Result<(), EngineError> {
        self.with_backend_mut(StorageCall::SaveLlmMessage, |backend| match backend {
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
                .db_context("Failed to save LLM message")?;

                conn.execute(
                    "DELETE FROM llm_messages
                     WHERE id NOT IN (
                         SELECT id FROM llm_messages ORDER BY created_at DESC LIMIT ?1
                     )",
                    [LLM_MESSAGE_RETENTION_LIMIT as i64],
                )
                .db_context("Failed to prune LLM messages")?;

                Ok(())
            }
        })
    }

    pub fn list_latest_llm_messages(&self, limit: usize) -> Result<Vec<LlmMessage>, EngineError> {
        self.with_backend_mut(StorageCall::ListLatestLlmMessages, |backend| match backend {
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
                    .db_context("Failed to prepare the LLM messages query")?;

                let rows = stmt
                    .query_map([limit as i64], DbLlmMessage::from_row)
                    .db_context("Failed to query LLM messages")?;

                let mut messages = Vec::new();
                for row in rows {
                    let db_msg = row.db_context("Failed to read LLM message row")?;
                    messages.push(LlmMessage::try_from(&db_msg)?);
                }
                messages.reverse();
                Ok(messages)
            }
        })
    }
}
