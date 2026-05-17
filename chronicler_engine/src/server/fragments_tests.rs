use std::sync::{Arc, RwLock, atomic::AtomicBool};
use tokio_util::sync::CancellationToken;

use crate::model::llm_message::LlmMessageBuilder;
use crate::model::settings::AppSettings;
use crate::server::fragments::{ActionForm, html_escape, render_error};
use crate::storage::llm_message_storage::{InMemoryLlmMessageStorage, LlmMessageStorage};
use crate::test_support::InMemoryGameStorage;
use crate::test_support::{TestMap, TestPlayer, TestWorld};

fn make_test_app_state(
    llm_storage: Option<Arc<InMemoryLlmMessageStorage>>,
) -> crate::server::AppState {
    let llm_storage: Arc<dyn LlmMessageStorage> = match llm_storage {
        Some(s) => s,
        None => Arc::new(InMemoryLlmMessageStorage::new()),
    };
    let game_service_storage = Arc::clone(&llm_storage);
    let storage = Arc::new(InMemoryGameStorage::new());
    crate::server::AppState {
        snapshot_storage: storage.clone(),
        message_storage: storage,
        llm_message_storage: llm_storage,
        world: Arc::new(TestWorld::minimal()),
        map: Arc::new(TestMap::single_room("start")),
        player: Arc::new(TestPlayer::standard()),
        npcs: Arc::new(std::collections::HashMap::new()),
        game_service: Arc::new(
            crate::application::game_service::DefaultGameService::with_storage(
                Some(game_service_storage),
                Arc::new(RwLock::new(AppSettings::default())),
            ),
        ) as Arc<dyn crate::application::game_service::GameService>,
        settings: Arc::new(RwLock::new(AppSettings::default())),
        cancel_token: Arc::new(RwLock::new(CancellationToken::new())),
        is_generating: Arc::new(AtomicBool::new(false)),
    }
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
        html_escape("<foo & \"bar\">"),
        "&lt;foo &amp; &quot;bar&quot;&gt;"
    );
}

#[test]
fn test_html_escape_empty() {
    assert_eq!(html_escape(""), "");
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
fn test_action_form_deserialization() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "look"}"#).unwrap();
    assert_eq!(form.command, "look");
}

#[test]
fn test_action_form_empty_command() {
    let form: ActionForm = serde_json::from_str(r#"{"command": ""}"#).unwrap();
    assert!(form.command.is_empty());
}

#[test]
fn test_edit_history_form_deserialization() {
    let form: crate::server::fragments::EditHistoryForm =
        serde_json::from_str(r#"{"text": "Modified text"}"#).unwrap();
    assert_eq!(form.text, "Modified text");
}

#[test]
fn test_render_error_empty_message() {
    let result = render_error("");
    assert!(result.contains("error-message"));
    assert!(result.contains("Error:"));
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
fn test_action_form_with_whitespace_command() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "  look  "}"#).unwrap();
    assert_eq!(form.command, "  look  ");
}

#[test]
fn test_action_form_with_special_characters() {
    let form: ActionForm =
        serde_json::from_str(r#"{"command": "go north & talk to guard"}"#).unwrap();
    assert_eq!(form.command, "go north & talk to guard");
}

#[test]
fn test_edit_history_form_empty_text() {
    let form: crate::server::fragments::EditHistoryForm =
        serde_json::from_str(r#"{"text": ""}"#).unwrap();
    assert!(form.text.is_empty());
}

#[test]
fn test_edit_history_form_with_newlines() {
    let form: crate::server::fragments::EditHistoryForm =
        serde_json::from_str(r#"{"text": "Line1\nLine2\nLine3"}"#).unwrap();
    assert!(form.text.contains('\n'));
}

#[test]
fn test_action_form_deserialize_unicode() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "こんにちは"}"#).unwrap();
    assert_eq!(form.command, "こんにちは");
}

#[test]
fn test_render_error_long_message() {
    let long_msg = "x".repeat(10000).to_string();
    let result = render_error(&long_msg);
    assert!(result.len() > 10000);
    assert!(result.contains(&long_msg[..100]));
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
fn test_action_form_roundtrip() {
    let original = ActionForm {
        command: "test command".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: ActionForm = serde_json::from_str(&json).unwrap();
    assert_eq!(original.command, parsed.command);
}

#[test]
fn test_edit_history_form_roundtrip() {
    let original = crate::server::fragments::EditHistoryForm {
        text: "new text".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: crate::server::fragments::EditHistoryForm = serde_json::from_str(&json).unwrap();
    assert_eq!(original.text, parsed.text);
}

#[test]
fn test_render_llm_messages_empty() {
    let app_state = make_test_app_state(None);
    let html = crate::server::fragments::render_llm_messages(&app_state).unwrap();
    assert!(html.contains("llm-message-list"));
    assert!(html.contains("No LLM messages yet"));
}

#[test]
fn test_render_llm_messages_with_data() {
    let llm_storage = Arc::new(InMemoryLlmMessageStorage::new());
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
    llm_storage.save(&msg).unwrap();

    let app_state = make_test_app_state(Some(llm_storage));
    let html = crate::server::fragments::render_llm_messages(&app_state).unwrap();
    assert!(html.contains("llm-message-list"));
    assert!(html.contains("narrator"));
    assert!(html.contains("OpenRouter"));
    assert!(html.contains("gpt-4"));
    assert!(html.contains("hello"));
}
