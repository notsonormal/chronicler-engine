//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM request building

use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;

use crate::domain::model::settings::ReasoningEffort;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_request_id() -> u64 {
    REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub const DEFAULT_MAX_TOKENS: u32 = 2048;

/// OpenCode Go asks each client to name itself.
const USER_AGENT: &str = concat!("chronicler-engine/", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub struct ChatCompletionResult {
    pub text: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ChatCompletionsEndpoint<'a> {
    pub base_url: &'a str,
    pub api_key: Option<&'a str>,
    /// Sent in `X-Title`, with an `HTTP-Referer`.
    pub title: Option<&'a str>,
    /// The header name and its value.
    pub session: Option<(&'a str, &'a str)>,
}

/// What one chat-completions call asks of the model.
#[derive(Debug, Clone, Copy)]
pub struct ChatCompletionRequest<'a> {
    pub model: &'a str,
    pub system_prompt: &'a str,
    pub user_text: &'a str,
    /// `None` sends `DEFAULT_MAX_TOKENS`.
    pub max_tokens: Option<u32>,
    /// Sent as the flat `reasoning_effort` field: OpenCode Go ignores the
    /// nested `reasoning: {effort}` form. `None` sends no field, so the model
    /// default applies.
    pub reasoning_effort: Option<ReasoningEffort>,
    /// `None` sends no `provider` field, so the provider chooses the host.
    pub provider_routing: Option<ProviderRouting<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ProviderRouting<'a> {
    pub order: &'a [String],
    pub allow_fallbacks: bool,
}

impl ChatCompletionRequest<'_> {
    pub fn resolved_max_tokens(&self) -> u32 {
        self.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS)
    }
}

pub fn build_request_payload(request: &ChatCompletionRequest<'_>) -> (serde_json::Value, String) {
    let mut messages = vec![];
    if !request.system_prompt.is_empty() {
        messages.push(json!({
            "role": "system",
            "content": request.system_prompt
        }));
    }
    messages.push(json!({
        "role": "user",
        "content": request.user_text
    }));
    let mut payload = json!({
        "model": request.model,
        "messages": messages,
        "stream": false,
        "max_tokens": request.resolved_max_tokens()
    });
    if let Some(effort) = request.reasoning_effort {
        payload["reasoning_effort"] = json!(effort);
    }
    if let Some(routing) = request.provider_routing {
        payload["provider"] = json!({
            "order": routing.order,
            "allow_fallbacks": routing.allow_fallbacks,
        });
    }
    let raw_request_json = serde_json::to_string(&payload)
        .unwrap_or_else(|_| format!("{{\"model\":\"{}\"}}", request.model));
    (payload, raw_request_json)
}

pub fn configure_request(
    client: &reqwest::blocking::Client,
    url: &str,
    payload: &serde_json::Value,
    endpoint: &ChatCompletionsEndpoint<'_>,
) -> reqwest::blocking::RequestBuilder {
    let mut request = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .header("User-Agent", USER_AGENT)
        .json(payload);
    if let Some(key) = endpoint.api_key {
        request = request.header("Authorization", format!("Bearer {key}"));
    }
    if let Some(t) = endpoint.title {
        request = request.header("X-Title", t);
        request = request.header("HTTP-Referer", "https://github.com/chronicler-engine");
    }
    if let Some((name, value)) = endpoint.session {
        request = request.header(name, value);
    }
    request
}
