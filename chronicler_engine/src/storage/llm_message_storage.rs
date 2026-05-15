use std::sync::Mutex;

use chrono::{DateTime, Utc};

use crate::error::EngineError;
use crate::model::llm_message::LlmMessage;
use crate::storage::db::DbPool;

pub trait LlmMessageStorage: Send + Sync {
    fn save(&self, message: &LlmMessage) -> Result<(), EngineError>;
    fn list_latest(&self, limit: usize) -> Result<Vec<LlmMessage>, EngineError>;
}

pub struct SqliteLlmMessageStorage {
    pool: DbPool,
}

impl SqliteLlmMessageStorage {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl LlmMessageStorage for SqliteLlmMessageStorage {
    fn save(&self, message: &LlmMessage) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute(
            "INSERT INTO llm_messages
             (agent_name, backend_name, model_name, system_prompt, user_prompt,
              raw_request_json, raw_response_json, parsed_response, error_message, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                message.agent_name,
                message.backend_name,
                message.model_name,
                message.system_prompt,
                message.user_prompt,
                message.raw_request_json,
                message.raw_response_json,
                message.parsed_response,
                message.error_message,
                message.created_at.to_rfc3339(),
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

    fn list_latest(&self, limit: usize) -> Result<Vec<LlmMessage>, EngineError> {
        let conn = self.pool.conn();
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
            .query_map([limit as i64], row_to_message)
            .map_err(|e| EngineError::Config(format!("Failed to query LLM messages: {e}")))?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row.map_err(|e| {
                EngineError::Config(format!("Failed to read LLM message row: {e}"))
            })?);
        }
        messages.reverse();
        Ok(messages)
    }
}

fn row_to_message(row: &rusqlite::Row) -> Result<LlmMessage, rusqlite::Error> {
    let created_at_str: String = row.get(10)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(10, rusqlite::types::Type::Text, Box::new(e))
        })?
        .with_timezone(&Utc);

    Ok(LlmMessage {
        id: row.get(0)?,
        agent_name: row.get(1)?,
        backend_name: row.get(2)?,
        model_name: row.get(3)?,
        system_prompt: row.get(4)?,
        user_prompt: row.get(5)?,
        raw_request_json: row.get(6)?,
        raw_response_json: row.get(7)?,
        parsed_response: row.get(8)?,
        error_message: row.get(9)?,
        created_at,
    })
}

pub struct InMemoryLlmMessageStorage {
    messages: Mutex<Vec<LlmMessage>>,
}

impl Default for InMemoryLlmMessageStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryLlmMessageStorage {
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(Vec::new()),
        }
    }

    pub fn len(&self) -> usize {
        match self.messages.lock() {
            Ok(guard) => guard.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.messages.lock() {
            Ok(guard) => guard.is_empty(),
            Err(poisoned) => poisoned.into_inner().is_empty(),
        }
    }
}

impl LlmMessageStorage for InMemoryLlmMessageStorage {
    fn save(&self, message: &LlmMessage) -> Result<(), EngineError> {
        let mut msgs = match self.messages.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        msgs.push(message.clone());
        if msgs.len() > 50 {
            msgs.remove(0);
        }
        Ok(())
    }

    fn list_latest(&self, limit: usize) -> Result<Vec<LlmMessage>, EngineError> {
        let msgs = match self.messages.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let start = msgs.len().saturating_sub(limit);
        Ok(msgs[start..].to_vec())
    }
}
