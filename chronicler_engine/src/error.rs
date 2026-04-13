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

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
