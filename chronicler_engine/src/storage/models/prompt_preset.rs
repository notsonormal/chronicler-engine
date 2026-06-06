//! [DOC: docs/system/storage.md]
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
