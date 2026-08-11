//! [DOC: docs/diataxis/reference/storage.md]
//! Prompt preset model

pub struct DbPromptPreset {
    pub id: String,
    pub name: String,
    pub preset_type: String,
    pub role: Option<String>,
    pub instructions: Option<String>,
    pub writing_style: Option<String>,
    pub output_format: Option<String>,
    pub is_default: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl DbPromptPreset {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbPromptPreset {
            id: row.get(0)?,
            name: row.get(1)?,
            preset_type: row.get(2)?,
            role: row.get(3)?,
            instructions: row.get(4)?,
            writing_style: row.get(5)?,
            output_format: row.get(6)?,
            is_default: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}
