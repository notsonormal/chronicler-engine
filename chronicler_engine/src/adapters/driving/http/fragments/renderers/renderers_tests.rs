use std::sync::Arc;

use crate::domain::model::llm_message::LlmMessageBuilder;
use crate::adapters::driving::http::fragments::{html_escape, render_error, render_llm_messages};
use crate::storage::Storage;
use crate::test_support::TestAppBuilder;

fn make_test_app_state(
    llm_storage: Option<Arc<Storage>>,
) -> crate::adapters::driving::http::AppState {
    // Use TestAppBuilder for proper world/persona setup, then swap in custom storage if needed
    let mut builder = TestAppBuilder::default_test();
    if let Some(storage) = llm_storage {
        builder = builder.storage(storage);
    }
    builder.build_app_state()
}

#[test]
fn test_html_escape_basic() {
    assert_eq!(html_escape("<test>"), "&lt;test&gt;");
}

#[test]
fn test_html_escape_ampersand() {
    assert_eq!(html_escape("foo & bar"), "foo &amp; bar");
}

#[test]
fn test_html_escape_quotes() {
    assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
}

#[test]
fn test_html_escape_all() {
    assert_eq!(
        html_escape("<foo & \"bar'\">"),
        "&lt;foo &amp; &quot;bar&#x27;&quot;&gt;"
    );
}

#[test]
fn test_html_escape_empty() {
    assert_eq!(html_escape(""), "");
}

#[test]
fn test_html_escape_newline() {
    // [DOC: docs/reference/testing.md]
    // Newlines should be preserved (not converted to &lt;br&gt;)
    assert_eq!(html_escape("line1\nline2"), "line1\nline2");
}

#[test]
fn test_html_escape_backtick() {
    // [DOC: docs/reference/testing.md]
    // Backticks should be preserved
    assert_eq!(html_escape("`code`"), "`code`");
}

#[test]
fn test_html_escape_unicode_characters() {
    // [DOC: docs/reference/testing.md]
    // Unicode should be preserved
    assert_eq!(html_escape("日本語"), "日本語");
}

#[test]
fn test_html_escape_multiple_special_chars() {
    assert_eq!(
        html_escape("<div class=\"test\">Hello & \"World\"</div>"),
        "&lt;div class=&quot;test&quot;&gt;Hello &amp; &quot;World&quot;&lt;/div&gt;"
    );
}

#[test]
fn test_html_escape_repeated_escaping() {
    // [DOC: docs/reference/testing.md]
    // html_escape is NOT idempotent - running it twice double-encodes
    let escaped = html_escape("<&>");
    assert_eq!(escaped, "&lt;&amp;&gt;");
    assert_eq!(html_escape(&escaped), "&amp;lt;&amp;amp;&amp;gt;");
}

#[test]
fn test_html_escape_only_ampersand() {
    assert_eq!(html_escape("&"), "&amp;");
}

#[test]
fn test_html_escape_only_lt() {
    assert_eq!(html_escape("<"), "&lt;");
}

#[test]
fn test_html_escape_only_gt() {
    assert_eq!(html_escape(">"), "&gt;");
}

#[test]
fn test_render_error_basic() {
    let result = render_error("Test error message");
    assert!(result.contains("error-message"));
    assert!(result.contains("Test error message"));
}

#[test]
fn test_render_error_html_escaped() {
    let result = render_error("<script>alert('xss')</script>");
    assert!(!result.contains("<script>"));
    assert!(result.contains("&lt;script&gt;"));
}

#[test]
fn test_render_error_empty_message() {
    let result = render_error("");
    assert!(result.contains("error-message"));
    assert!(result.contains("Error:"));
}

#[test]
fn test_render_error_long_message() {
    let long_msg = "x".repeat(10000).to_string();
    let result = render_error(&long_msg);
    assert!(result.len() > 10000);
    assert!(result.contains(&long_msg[..100]));
}

#[test]
fn test_render_llm_messages_empty() {
    let app_state = make_test_app_state(None);
    let html = render_llm_messages(&app_state).unwrap();
    assert!(html.contains("llm-message-list"));
    assert!(html.contains("No LLM messages yet"));
}

#[test]
fn test_render_llm_messages_with_data() {
    let llm_storage = Arc::new(Storage::new_in_memory());
    let msg = LlmMessageBuilder::new()
        .agent_name("narrator")
        .backend_name("OpenRouter")
        .model_name("gpt-4")
        .system_prompt("sys")
        .user_prompt("user")
        .raw_request_json("req")
        .raw_response_json("res")
        .parsed_response("hello")
        .error_message(None::<String>)
        .build();
    llm_storage.save_llm_message(&msg).unwrap();

    let app_state = make_test_app_state(Some(llm_storage));
    let html = render_llm_messages(&app_state).unwrap();
    assert!(html.contains("llm-message-list"));
    assert!(html.contains("narrator"));
    assert!(html.contains("OpenRouter"));
    assert!(html.contains("gpt-4"));
    assert!(html.contains("hello"));
}
