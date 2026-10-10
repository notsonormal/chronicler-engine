//! Stub-browser tests for the options dock: the client-side edit action filling the command input. Tagged against `docs/specs/browser_options.md`.

use std::time::Duration;

use super::*;
use super::support::StubRunner;

/// How long a click the guard must swallow still gets to reach the stub. The
/// fixed client sends nothing at all, so the window only has to outlast a
/// submit that a regressed client would have fired immediately.
const SUBMIT_SETTLE: Duration = Duration::from_millis(500);

// [docs/specs/browser_options.md] SCENARIO: 26.5
async fn check_option_click_during_generation_does_not_submit(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let actions = stub.action_handle();
    wait_for_element_children(&page, "#options-dock .option-item", 3).await;

    // The stub's acknowledgement lands "Thinking..." in the status
    // display, so the dock's options outlive their turn.
    send_action(&page, "look").await;
    let requests_before = actions.count();
    assert_eq!(
        requests_before, 1,
        "the command submit is the only /action/check request so far"
    );

    let generating: bool = page
        .evaluate::<(), bool>(
            r#"(() => {
                    const status = document.getElementById('status-display');
                    return !!status && !!status.querySelector('.status.thinking');
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(generating, "the turn must be in flight before the click");

    let clicked: bool = page
        .evaluate::<(), bool>(
            r#"(() => {
                    const btn = document.querySelector('#options-dock .option-btn');
                    if (!btn) return false;
                    btn.click();
                    return true;
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(clicked, "the dock must render an option button");

    // Give the (buggy) submit a chance to reach the stub.
    tokio::time::sleep(SUBMIT_SETTLE).await;
    assert_eq!(
        actions.count(),
        requests_before,
        "an option click while a generation is in flight must not submit"
    );
}

// [docs/specs/browser_options.md] SCENARIO: 26.3
async fn check_options_dock_edit_fills_without_submitting(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_element_children(&page, "#options-dock .option-item", 3).await;
    let option_text: String = page
        .evaluate::<(), String>(
            r#"(() => document.querySelector('#options-dock .option-btn').textContent)()"#,
            None,
        )
        .await
        .unwrap();

    let entries_before = page
        .locator("#story-log .log-entry")
        .await
        .count()
        .await
        .unwrap_or(0);

    let clicked: bool = page
        .evaluate::<(), bool>(
            r#"(() => {
                    const btn = document.querySelector('#options-dock .option-item .mini-btn');
                    if (!btn) return false;
                    btn.click();
                    return true;
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(clicked, "Edit click must find the edit button");

    let focused: bool = page
        .evaluate::<(), bool>(
            r#"(() => {
                    const input = document.querySelector('#command-form input[name="command"]');
                    return input.value.length > 0 && document.activeElement === input;
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(focused, "Edit must fill the command input and focus it");

    let input_value: String = page
        .evaluate::<(), String>(
            r#"(() => document.querySelector('#command-form input[name="command"]').value)()"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(input_value, option_text, "Edit must fill the option text");

    let entries_after = page
        .locator("#story-log .log-entry")
        .await
        .count()
        .await
        .unwrap_or(0);
    assert_eq!(
        entries_after, entries_before,
        "Edit must not submit or create a story-log entry"
    );
}

#[tokio::test]
async fn run_options_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_option_click_during_generation_does_not_submit,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_options_dock_edit_fills_without_submitting,
        )
        .await;
    runner.finish().await;
}
