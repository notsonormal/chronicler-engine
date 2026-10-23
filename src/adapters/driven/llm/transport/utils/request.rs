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

#[derive(Debug)]
pub struct ChatCompletionResult {
    pub text: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
}

/// What one chat-completions call asks of the model. The endpoint (URL, key,
/// title) is not part of it.
#[derive(Debug, Clone, Copy)]
pub struct ChatCompletionRequest<'a> {
    pub model: &'a str,
    pub system_prompt: &'a str,
    pub user_text: &'a str,
    /// `None` sends `DEFAULT_MAX_TOKENS`.
    pub max_tokens: Option<u32>,
    /// `None` sends no `reasoning` field, so the model default applies.
    pub reasoning_effort: Option<ReasoningEffort>,
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
        payload["reasoning"] = json!({ "effort": effort });
    }
    let raw_request_json = serde_json::to_string(&payload)
        .unwrap_or_else(|_| format!("{{\"model\":\"{}\"}}", request.model));
    (payload, raw_request_json)
}

pub fn configure_request(
    client: &reqwest::blocking::Client,
    url: &str,
    payload: &serde_json::Value,
    api_key: Option<&str>,
    title: Option<&str>,
) -> reqwest::blocking::RequestBuilder {
    let mut request = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept-Encoding", "gzip, deflate")
        .json(payload);
    if let Some(key) = api_key {
        request = request.header("Authorization", format!("Bearer {key}"));
    }
    if let Some(t) = title {
        request = request.header("X-Title", t);
        request = request.header("HTTP-Referer", "https://github.com/chronicler-engine");
    }
    request
}
