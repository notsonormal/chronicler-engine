// [DOC: docs/system/llm_processing.md]

use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_request_id() -> u64 {
    REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub(crate) const DEFAULT_MAX_TOKENS: u32 = 2048;

#[derive(Debug)]
pub struct ChatCompletionResult {
    pub text: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
}

pub(crate) fn build_request_payload(
    model: &str,
    system_prompt: &str,
    user_text: &str,
    max_tokens: u32,
) -> (serde_json::Value, String) {
    let mut messages = vec![];
    if !system_prompt.is_empty() {
        messages.push(json!({
            "role": "system",
            "content": system_prompt
        }));
    }
    messages.push(json!({
        "role": "user",
        "content": user_text
    }));
    let payload = json!({
        "model": model,
        "messages": messages,
        "stream": false,
        "max_tokens": max_tokens
    });
    let raw_request_json =
        serde_json::to_string(&payload).unwrap_or_else(|_| format!("{{\"model\":\"{model}\"}}"));
    (payload, raw_request_json)
}

pub(crate) fn configure_request(
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
