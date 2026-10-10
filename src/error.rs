//! [DOC: docs/diataxis/reference/architecture_system.md]
//! Error types and result aliases

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmFailure {
    /// The provider answered 200 OK with a blank, null or missing `content`; no answer to feed
    /// downstream.
    #[error("LLM returned an empty response")]
    EmptyResponse { raw_response: String },
    /// The model spent its whole output budget before writing an answer (`finish_reason: "length"`
    /// with an empty `content`). The counts are `None` when the provider reported no `usage`.
    #[error(
        "LLM returned no answer after spending its whole token budget{}",
        LlmFailure::token_counts_suffix(*completion_tokens, *reasoning_tokens)
    )]
    TokenBudgetSpent {
        raw_response: String,
        completion_tokens: Option<u64>,
        reasoning_tokens: Option<u64>,
    },
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

impl InternalError {
    pub fn new(invariant: impl Into<String>) -> Self {
        Self {
            invariant: invariant.into(),
        }
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

impl LlmFailure {
    pub fn raw_response_body(&self) -> Option<&str> {
        match self {
            Self::EmptyResponse { raw_response }
            | Self::TokenBudgetSpent { raw_response, .. }
            | Self::ParseError { raw_response, .. } => Some(raw_response),
            Self::Http { .. } | Self::Network { .. } | Self::Timeout => None,
        }
    }

    fn token_counts_suffix(
        completion_tokens: Option<u64>,
        reasoning_tokens: Option<u64>,
    ) -> String {
        match (completion_tokens, reasoning_tokens) {
            (Some(completion), Some(reasoning)) => {
                format!(" ({completion} completion tokens, {reasoning} of them reasoning)")
            }
            (Some(completion), None) => format!(" ({completion} completion tokens)"),
            _ => String::new(),
        }
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

    /// The active prompt preset is absent from the preset library; the id names
    /// the slot's stored value.
    #[error("Prompt preset not found: {0}")]
    PresetNotFound(String),

    /// A world with this key already exists; the user-facing create path refuses to replace it.
    #[error("A world with key '{0}' already exists")]
    WorldAlreadyExists(String),

    /// World cannot be deleted because games still reference it; `game_count` is the blocker count.
    #[error("Cannot delete world with {game_count} games")]
    WorldHasGames { game_count: usize },

    /// Settings/config value missing or invalid; message names the field and constraint.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Use-case guard rejected the request (e.g. no scene to act on); message is user-facing.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Template substitution failed (missing var, bad type, recursion limit).
    #[error("Template error: {0}")]
    Template(String),

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

impl EngineError {
    pub fn raw_response_body(&self) -> Option<&str> {
        match self {
            EngineError::Llm(failure) => failure.raw_response_body(),
            _ => None,
        }
    }

    pub fn llm_error_string(&self) -> String {
        match self {
            EngineError::Llm(LlmFailure::Timeout) => "LLM Error: request timed out".to_string(),
            EngineError::Llm(LlmFailure::Network { url, detail }) => {
                format!("LLM Error: network error ({url}) \u{2014} {detail}")
            }
            EngineError::Llm(LlmFailure::ParseError {
                expected_format, ..
            }) => {
                format!("LLM Error: unexpected response format (expected {expected_format})")
            }
            EngineError::Llm(LlmFailure::EmptyResponse { .. }) => {
                "LLM Error: empty response".to_string()
            }
            EngineError::Llm(LlmFailure::TokenBudgetSpent {
                completion_tokens,
                reasoning_tokens,
                ..
            }) => format!(
                "LLM Error: the model spent its whole token budget before writing an answer{}",
                LlmFailure::token_counts_suffix(*completion_tokens, *reasoning_tokens)
            ),
            EngineError::Llm(LlmFailure::Http { status, body }) => {
                format!("LLM Error: HTTP {status} \u{2014} {body}")
            }
            EngineError::Narrative(nf) => format!("LLM Error: {nf}"),
            _ => format!("LLM Error: {self}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, EngineError>;
