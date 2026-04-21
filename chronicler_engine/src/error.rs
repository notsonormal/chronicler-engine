use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Navigation error: {0}")]
    Navigation(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Narrative generation error: {0}")]
    Narrative(String),

    #[error("Room not found: {0}")]
    RoomNotFound(String),

    #[error("NPC not found: {0}")]
    NpcNotFound(String),

    #[error("World not found: {0}")]
    WorldNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Data loading error in {path}: {source}")]
    DataLoad {
        path: String,
        source: Box<EngineError>,
    },

    #[error("Context overflow: requested {requested} tokens exceeds max {max}")]
    ContextOverflow { requested: usize, max: usize },
}

pub type Result<T> = std::result::Result<T, EngineError>;
