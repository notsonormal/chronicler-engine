//! [DOC: docs/architecture/system.md]
//! Error types and result aliases

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmFailure {
    #[error("LLM returned an empty response")]
    EmptyResponse,
    #[error("LLM API returned HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("LLM network error contacting {url}: {detail}")]
    Network { url: String, detail: String },
    #[error("Failed to parse LLM response as {expected_format}")]
    ParseError {
        raw_response: String,
        expected_format: &'static str,
    },
    #[error("LLM request timed out")]
    Timeout,
}

#[derive(Error, Debug)]
pub enum NarrativeFailure {
    #[error("Prompt build failed at stage '{stage}': {reason}")]
    PromptBuild {
        stage: &'static str,
        reason: &'static str,
    },
    #[error("Narration generation failed at stage '{stage}': {reason}")]
    Generation {
        stage: &'static str,
        reason: &'static str,
    },
}

#[derive(Error, Debug)]
#[error("Invariant violated: {invariant}")]
pub struct InternalError {
    pub invariant: String,
}

pub fn internal_error(invariant: impl Into<String>) -> InternalError {
    InternalError {
        invariant: invariant.into(),
    }
}

impl From<InternalError> for EngineError {
    fn from(e: InternalError) -> Self {
        EngineError::Internal(e)
    }
}

impl From<LlmFailure> for EngineError {
    fn from(e: LlmFailure) -> Self {
        EngineError::Llm(e)
    }
}

impl From<NarrativeFailure> for EngineError {
    fn from(e: NarrativeFailure) -> Self {
        EngineError::Narrative(e)
    }
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Serialize error: {0}")]
    Serialize(String),

    #[error("Navigation error: {0}")]
    Navigation(String),

    #[error("LLM error: {0}")]
    Llm(#[source] LlmFailure),

    #[error("Narrative generation error: {0}")]
    Narrative(#[source] NarrativeFailure),

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

    #[error("Internal invariant violated: {0}")]
    Internal(#[source] InternalError),

    #[error("Data loading error in {path}: {source}")]
    DataLoad {
        path: String,
        source: Box<EngineError>,
    },

    #[error("Context overflow: requested {requested} tokens exceeds max {max}")]
    ContextOverflow { requested: usize, max: usize },

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

impl From<std::io::Error> for EngineError {
    fn from(e: std::io::Error) -> Self {
        EngineError::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, EngineError>;
