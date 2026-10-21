//! [DOC: docs/diataxis/reference/storage.md]
//! Preset storage operations

use crate::error::{DatabaseContext, EngineError};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::utils::name_is_available;
use crate::adapters::driven::storage::{Backend, Storage, StorageCall};
use crate::adapters::driven::storage::models::prompt_preset::DbPromptPreset;

impl Storage {
    pub fn list_presets(&self, preset_type: PresetType) -> Result<Vec<PromptPreset>, EngineError> {
        self.with_backend_mut(StorageCall::ListPresets, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at, allowed_modes
                         FROM prompt_presets
                         WHERE preset_type = ?1
                         ORDER BY updated_at DESC",
                    )
                    .db_context("Failed to prepare the presets query")?;

                let rows = stmt
                    .query_map([preset_type.as_str()], DbPromptPreset::from_row)
                    .db_context("Failed to query presets")?;

                let mut presets = Vec::new();
                for row in rows {
                    let db = row.db_context("Failed to read preset row")?;
                    presets.push(db.into_preset()?);
                }
                Ok(presets)
            }
        })
    }

    pub fn get_preset(&self, id: &str) -> Result<Option<PromptPreset>, EngineError> {
        self.with_backend_mut(StorageCall::GetPreset, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare(
                        "SELECT id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at, allowed_modes
                         FROM prompt_presets
                         WHERE id = ?1",
                    )
                    .db_context("Failed to prepare the preset query")?;

                let mut rows = stmt
                    .query_map([id], DbPromptPreset::from_row)
                    .db_context("Failed to query preset")?;

                match rows.next() {
                    Some(row) => {
                        let db = row.db_context("Failed to read the preset row")?;
                        Ok(Some(db.into_preset()?))
                    }
                    None => Ok(None),
                }
            }
        })
    }

    /// Runs inside the same backend acquisition as the write in `save_preset`,
    /// so check-and-write is one step and two concurrent creates cannot both
    /// pass. The check is a scan, not a `UNIQUE` constraint, so a legacy
    /// database that already holds duplicates still opens.
    fn ensure_preset_name_available(
        backend: &Backend,
        preset: &PromptPreset,
    ) -> Result<(), EngineError> {
        let available = match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn
                    .prepare("SELECT id, name FROM prompt_presets WHERE preset_type = ?1")
                    .db_context("Failed to prepare the preset names query")?;

                let rows = stmt
                    .query_map([preset.preset_type.as_str()], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })
                    .db_context("Failed to query preset names")?;

                let mut existing = Vec::new();
                for row in rows {
                    existing.push(row.db_context("Failed to read preset name")?);
                }

                name_is_available(
                    existing
                        .iter()
                        .map(|(id, name)| (id.as_str(), name.as_str())),
                    &preset.name,
                    Some(preset.id.as_str()),
                )
            }
        };

        if available {
            Ok(())
        } else {
            Err(EngineError::Validation(format!(
                "A {} preset named '{}' already exists",
                preset.preset_type.as_str(),
                preset.name.trim()
            )))
        }
    }

    pub fn save_preset(&self, preset: &PromptPreset) -> Result<(), EngineError> {
        self.with_backend_mut(StorageCall::SavePreset, |backend| {
            Self::ensure_preset_name_available(backend, preset)?;
            Self::write_preset(backend, preset)
        })
    }

    fn write_preset(backend: &mut Backend, preset: &PromptPreset) -> Result<(), EngineError> {
        match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = chrono::Utc::now().to_rfc3339();
                let is_default = if preset.is_default { 1 } else { 0 };

                conn.execute(
                    "INSERT INTO prompt_presets (id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at, allowed_modes)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                     ON CONFLICT(id) DO UPDATE SET
                         name = excluded.name,
                         preset_type = excluded.preset_type,
                         role = excluded.role,
                         instructions = excluded.instructions,
                         writing_style = excluded.writing_style,
                         output_format = excluded.output_format,
                         is_default = excluded.is_default,
                         updated_at = excluded.updated_at,
                         allowed_modes = excluded.allowed_modes",
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
                        serde_json::to_string(&preset.allowed_modes)
                            .map_err(|e| EngineError::Parse(format!(
                                "Failed to serialize allowed_modes: {e}"
                            )))?,
                    ],
                )
                .db_context("Failed to save preset")?;

                Ok(())
            }
        }
    }

    pub fn delete_preset(&self, id: &str) -> Result<(), EngineError> {
        self.with_backend_mut(StorageCall::DeletePreset, |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                conn.execute("DELETE FROM prompt_presets WHERE id = ?1", [id])
                    .db_context("Failed to delete preset")?;
                Ok(())
            }
        })
    }
}
