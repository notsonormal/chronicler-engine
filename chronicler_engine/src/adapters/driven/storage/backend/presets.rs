//! [DOC: docs/system/storage.md]
//! Preset storage operations

use crate::error::EngineError;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::adapters::driven::storage::backend::{Backend, Storage};
use crate::adapters::driven::storage::models::prompt_preset::DbPromptPreset;

impl Storage {
    pub fn list_presets(&self, preset_type: PresetType) -> Result<Vec<PromptPreset>, EngineError> {
        self.with_backend_mut("list_presets", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at
                         FROM prompt_presets
                         WHERE preset_type = ?1
                         ORDER BY updated_at DESC",
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

                let rows = stmt
                    .query_map([preset_type.as_str()], DbPromptPreset::from_row)
                    .map_err(|e| EngineError::Config(format!("Failed to query presets: {e}")))?;

                let mut presets = Vec::new();
                for row in rows {
                    let db =
                        row.map_err(|e| EngineError::Config(format!("Failed to read preset row: {e}")))?;
                    presets.push(db_preset_to_preset(db));
                }
                Ok(presets)
            }
            Backend::InMemory(data) => {
                Ok(data.presets
                    .iter()
                    .filter(|p| p.preset_type == preset_type)
                    .cloned()
                    .collect())
            }
        })
    }

    pub fn get_preset(&self, id: &str) -> Result<Option<PromptPreset>, EngineError> {
        self.with_backend_mut("get_preset", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at
                         FROM prompt_presets
                         WHERE id = ?1",
                    )
                    .map_err(|e| EngineError::Config(format!("Failed to prepare query: {e}")))?;

                let mut rows = stmt
                    .query_map([id], DbPromptPreset::from_row)
                    .map_err(|e| EngineError::Config(format!("Failed to query preset: {e}")))?;

                match rows.next() {
                    Some(row) => {
                        let db = row
                            .map_err(|e| EngineError::Config(format!("Failed to read preset row: {e}")))?;
                        Ok(Some(db_preset_to_preset(db)))
                    }
                    None => Ok(None),
                }
            }
            Backend::InMemory(data) => {
                Ok(data.presets.iter().find(|p| p.id == id).cloned())
            }
        })
    }

    pub fn save_preset(&self, preset: &PromptPreset) -> Result<(), EngineError> {
        self.with_backend_mut("save_preset", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = chrono::Utc::now().to_rfc3339();
                let is_default = if preset.is_default { 1 } else { 0 };

                conn.execute(
                    "INSERT INTO prompt_presets (id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                     ON CONFLICT(id) DO UPDATE SET
                         name = excluded.name,
                         preset_type = excluded.preset_type,
                         role = excluded.role,
                         instructions = excluded.instructions,
                         writing_style = excluded.writing_style,
                         output_format = excluded.output_format,
                         is_default = excluded.is_default,
                         updated_at = excluded.updated_at",
                    rusqlite::params![
                        preset.id,
                        preset.name,
                        preset.preset_type.as_str(),
                        preset.role,
                        preset.instructions,
                        preset.writing_style,
                        preset.output_format,
                        is_default,
                        now,
                        now,
                    ],
                )
                .map_err(|e| EngineError::Config(format!("Failed to save preset: {e}")))?;

                Ok(())
            }
            Backend::InMemory(data) => {
                if let Some(idx) = data.presets.iter().position(|p| p.id == preset.id) {
                    data.presets[idx] = preset.clone();
                } else {
                    data.presets.push(preset.clone());
                }
                Ok(())
            }
        })
    }

    pub fn delete_preset(&self, id: &str) -> Result<(), EngineError> {
        self.with_backend_mut("delete_preset", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute("DELETE FROM prompt_presets WHERE id = ?1", [id])
                    .map_err(|e| EngineError::Config(format!("Failed to delete preset: {e}")))?;
                Ok(())
            }
            Backend::InMemory(data) => {
                data.presets.retain(|p| p.id != id);
                Ok(())
            }
        })
    }
}

fn db_preset_to_preset(db: DbPromptPreset) -> PromptPreset {
    let preset_type = match db.preset_type.as_str() {
        "quantifier" => PresetType::Quantifier,
        _ => PresetType::System,
    };
    PromptPreset {
        id: db.id,
        name: db.name,
        role: db.role,
        instructions: db.instructions,
        writing_style: db.writing_style,
        output_format: db.output_format,
        is_default: db.is_default != 0,
        preset_type,
    }
}
