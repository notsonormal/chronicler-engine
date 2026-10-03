//! HTTP E2E tests for the text-check endpoint (POST /check-text).

use chronicler_engine::domain::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::test_support::TestAppBuilder;

use crate::support::http_requests::{post_form, response_body};

// [docs/specs/text_check.md] SCENARIO: 34.1
#[tokio::test]
async fn test_log_entry_check_returns_read_only_result_panel() {
    let settings = AppSettings {
        text_check: TextCheckSettings {
            mode: TextCheckMode::Spell,
            enable_auto_check: true,
            ignored_words: vec![],
        },
        ..Default::default()
    };
    let app = TestAppBuilder::default_test().settings(settings).build();

    let resp = post_form(&app, "/check-text", "command=go+to+the+casle&entry_id=2").await;
    assert!(resp.status().is_success());
    let body = response_body(resp).await;

    assert!(
        body.contains("text-check-result-panel"),
        "a log-entry check must render the read-only result panel: {body}"
    );
    assert!(
        body.contains("Checked entry #2"),
        "the result panel must name the entry it checked: {body}"
    );
    assert!(
        !body.contains("text-check-preview"),
        "a log-entry check must not render the send preview: {body}"
    );
    assert!(
        !body.contains("/action/confirm") && !body.contains("Send"),
        "the result panel must offer no send form: {body}"
    );
}
