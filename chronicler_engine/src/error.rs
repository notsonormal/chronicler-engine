//! [DOC: chronicler_engine/docs/diataxis/reference/architecture_system.md]
//! Error types and result aliases

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmFailure {
    /// Provider returned 200 OK with empty content; no parseable text to feed downstream.
    #[error("LLM returned an empty response")]
    EmptyResponse,
    /// HTTP response carried a non-2xx status; `status`/`body` captured for forensics and retry decisions.
    #[error("LLM API returned HTTP {status}: {body}")]
    Http { status: u16, body: String },
    /// Transport-level failure (DNS, connection refused, TLS); URL unreachable, not an HTTP error.
    #[error("LLM network error contacting {url}: {detail}")]
    Network { url: String, detail: String },
    /// Transport succeeded but response body did not match `expected_format`; raw bytes retained for diagnostics.
    #[error("Failed to parse LLM response as {expected_format}")]
    ParseError {
        raw_response: String,
        expected_format: &'static str,
    },
    /// Request exceeded the configured deadline before any response arrived; circuit-breaker may engage.
    #[error("LLM request timed out")]
    Timeout,
}

#[derive(Error, Debug)]
pub enum NarrativeFailure {
    /// Prompt assembly could not satisfy stage `stage`; reason is a short slug, not user-facing prose.
    #[error("Prompt build failed at stage '{stage}': {reason}")]
    PromptBuild {
        stage: &'static str,
        reason: &'static str,
    },
    /// Narration LLM call completed but post-processing rejected the output as unusable for the scene.
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
    /// Filesystem IO failure outside SQLite/serde paths; message is context, not OS errno.
    #[error("I/O error: {0}")]
    Io(String),

    /// `serde_json` (de)serialization rejected the payload; source carries the line/column.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Hand-rolled parser (scenarios, triggers, templates) rejected input; message names the failing fragment.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Serialization attempted on an incomplete/inconsistent value (not a serde decode failure).
    #[error("Serialize error: {0}")]
    Serialize(String),

    /// Room navigation lookup missed; message is room identifier or context.
    #[error("Navigation error: {0}")]
    Navigation(String),

    /// Narrator/quantifier call failed; inner carries transport-vs-parse distinction.
    #[error("LLM error: {0}")]
    Llm(#[source] LlmFailure),

    /// Narrative pipeline (prompt build or generation post-processing) rejected the output.
    #[error("Narrative generation error: {0}")]
    Narrative(#[source] NarrativeFailure),

    /// Room lookup by identifier failed.
    #[error("Room not found: {0}")]
    RoomNotFound(String),

    /// Message lookup by id failed (deleted, gated, or never persisted).
    #[error("Message not found: {0}")]
    MessageNotFound(u64),

    /// Game id does not correspond to a live game session in storage.
    #[error("Game not found: {0}")]
    GameNotFound(u64),

    /// Persona lookup by name failed.
    #[error("Persona not found: {0}")]
    PersonaNotFound(String),

    /// World lookup by identifier failed.
    #[error("World not found: {0}")]
    WorldNotFound(String),

    /// World cannot be deleted because games still reference it; `game_count` is the blocker count.
    #[error("Cannot delete world with {game_count} games")]
    WorldHasGames { game_count: usize },

    /// Settings/config value missing or invalid; message names the field and constraint.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Template substitution failed (missing var, bad type, recursion limit).
    #[error("Template error: {0}")]
    Template(String),

    /// Render stage (HTMX/Tera) failed downstream of template substitution.
    #[error("Render error: {0}")]
    Render(String),

    /// Engine invariant violated; inner carries the invariant name for triage.
    #[error("Internal invariant violated: {0}")]
    Internal(#[source] InternalError),

    /// Data file loaded from `path` failed to parse/validate; `source` is the wrapped underlying error.
    #[error("Data loading error in {path}: {source}")]
    DataLoad {
        path: String,
        source: Box<EngineError>,
    },

    /// Prompt budget exceeded: `requested` tokens pushed past `max` for the active connection.
    #[error("Context overflow: requested {requested} tokens exceeds max {max}")]
    ContextOverflow { requested: usize, max: usize },

    /// SQLite returned an error; source carries the rusqlite kind.
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

impl From<std::io::Error> for EngineError {
    fn from(e: std::io::Error) -> Self {
        EngineError::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, EngineError>;
