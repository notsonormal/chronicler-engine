use std::sync::Mutex;

use crate::error::EngineError;
use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::storage::db::DbPool;
use crate::storage::models::prompt_preset::DbPromptPreset;

/// [DOC: docs/architecture/system.md]
pub trait PromptPresetStorage: Send + Sync {
    fn list(&self, preset_type: PresetType) -> Result<Vec<PromptPreset>, EngineError>;
    fn get(&self, id: &str) -> Result<Option<PromptPreset>, EngineError>;
    fn save(&self, preset: &PromptPreset) -> Result<(), EngineError>;
    fn delete(&self, id: &str) -> Result<(), EngineError>;
}

pub struct SqlitePromptPresetStorage {
    pool: DbPool,
}

impl SqlitePromptPresetStorage {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl PromptPresetStorage for SqlitePromptPresetStorage {
    fn list(&self, preset_type: PresetType) -> Result<Vec<PromptPreset>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, preset_type, prompt_text, is_default, created_at, updated_at
                 FROM prompt_presets
                 WHERE preset_type = ?1
                 ORDER BY updated_at DESC",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        let rows = stmt
            .query_map([preset_type.as_str()], |row| {
                Ok(DbPromptPreset {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    preset_type: row.get(2)?,
                    prompt_text: row.get(3)?,
                    is_default: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| EngineError::Config(format!("Failed to query presets: {e}")))?;

        let mut presets = Vec::new();
        for row in rows {
            let db =
                row.map_err(|e| EngineError::Config(format!("Failed to read preset row: {e}")))?;
            presets.push(from_db(db));
        }
        Ok(presets)
    }

    fn get(&self, id: &str) -> Result<Option<PromptPreset>, EngineError> {
        let conn = self.pool.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, preset_type, prompt_text, is_default, created_at, updated_at
                 FROM prompt_presets
                 WHERE id = ?1",
            )
            .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

        let mut rows = stmt
            .query_map([id], |row| {
                Ok(DbPromptPreset {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    preset_type: row.get(2)?,
                    prompt_text: row.get(3)?,
                    is_default: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| EngineError::Config(format!("Failed to query preset: {e}")))?;

        match rows.next() {
            Some(row) => {
                let db = row
                    .map_err(|e| EngineError::Config(format!("Failed to read preset row: {e}")))?;
                Ok(Some(from_db(db)))
            }
            None => Ok(None),
        }
    }

    fn save(&self, preset: &PromptPreset) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        let now = chrono::Utc::now().to_rfc3339();
        let is_default = if preset.is_default { 1 } else { 0 };

        conn.execute(
            "INSERT INTO prompt_presets (id, name, preset_type, prompt_text, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 preset_type = excluded.preset_type,
                 prompt_text = excluded.prompt_text,
                 is_default = excluded.is_default,
                 updated_at = excluded.updated_at",
            rusqlite::params![
                preset.id,
                preset.name,
                preset.preset_type.as_str(),
                preset.prompt_text,
                is_default,
                now,
                now,
            ],
        )
        .map_err(|e| EngineError::Config(format!("Failed to save preset: {e}")))?;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), EngineError> {
        let conn = self.pool.conn();
        conn.execute("DELETE FROM prompt_presets WHERE id = ?1", [id])
            .map_err(|e| EngineError::Config(format!("Failed to delete preset: {e}")))?;
        Ok(())
    }
}

pub struct InMemoryPromptPresetStorage {
    presets: Mutex<Vec<PromptPreset>>,
}

impl Default for InMemoryPromptPresetStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryPromptPresetStorage {
    pub fn new() -> Self {
        Self {
            presets: Mutex::new(Vec::new()),
        }
    }
}

impl PromptPresetStorage for InMemoryPromptPresetStorage {
    fn list(&self, preset_type: PresetType) -> Result<Vec<PromptPreset>, EngineError> {
        let presets = match self.presets.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        Ok(presets
            .iter()
            .filter(|p| p.preset_type == preset_type)
            .cloned()
            .collect())
    }

    fn get(&self, id: &str) -> Result<Option<PromptPreset>, EngineError> {
        let presets = match self.presets.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        Ok(presets.iter().find(|p| p.id == id).cloned())
    }

    fn save(&self, preset: &PromptPreset) -> Result<(), EngineError> {
        let mut presets = match self.presets.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(idx) = presets.iter().position(|p| p.id == preset.id) {
            presets[idx] = preset.clone();
        } else {
            presets.push(preset.clone());
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), EngineError> {
        let mut presets = match self.presets.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        presets.retain(|p| p.id != id);
        Ok(())
    }
}

pub(crate) fn from_db(db: DbPromptPreset) -> PromptPreset {
    use crate::model::prompt_preset::PresetType;
    let preset_type = match db.preset_type.as_str() {
        "quantifier" => PresetType::Quantifier,
        _ => PresetType::System,
    };
    PromptPreset {
        id: db.id,
        name: db.name,
        prompt_text: db.prompt_text,
        is_default: db.is_default != 0,
        preset_type,
    }
}
