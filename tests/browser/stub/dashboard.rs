//! Stub-browser tests for dashboard chrome: the error toast, and the action-area handles (Send lock, status error observer) surviving an #action-area swap. Tagged against `docs/specs/browser_dashboard.md`.

// The engine's failing action route answers 500, so htmx fires
// `htmx:beforeSwap` with `isError` on its own. The stub's Error outcome serves
// that 500, and its `/status/generating` serves the spans the status poll
// swaps; no test dispatches either event by hand.

use std::time::Duration;

use super::*;

/// Submit `command` through the shipped command form. Unlike `send_action`,
/// this does not wait for the status span: a 500 is not swapped, so the status
/// stays Ready and the error toast is the observable outcome. The form is
/// submitted directly rather than by clicking Send: the failure path leaves
/// Send locked until the next idle poll, and a second error must land inside
/// the first toast's 5s timer.
async fn submit_command(page: &playwright_rs::Page, command: &str) {
    let command = command.to_string();
    page.evaluate::<String, ()>(
        r#"(command) => {
            const form = document.getElementById('command-form');
            const input = form.querySelector('input[name="command"]');
            input.value = command;
            form.requestSubmit();
        }"#,
        Some(&command),
    )
    .await
    .unwrap();
}

/// Wait until the toast displays `text`, so a timer measurement starts from the
/// toast actually being up.
async fn wait_for_toast_text(page: &playwright_rs::Page, text: &str) {
    let expected = text.to_string();
    let shown = wait_for_condition_async(Duration::from_secs(3), Duration::from_millis(50), || {
        let expected = expected.clone();
        async move { read_error_toast(page).await.1 == expected }
    })
    .await;
    assert!(shown, "toast never displayed {expected:?}");
}

/// Drive the shipped text-check swap: submit a command the auto-check
/// intercepts, so the preview replaces #action-area, then confirm the preview,
/// so `/action/confirm` swaps a fresh #action-area in. Every node the page-load
/// handles pointed at (form, status display, Send button) is detached; the ids
/// live on in fresh nodes.
async fn swap_action_area_via_text_check_confirm(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            window.__statusBeforeSwap = document.getElementById('status-display');
            const input = document.querySelector('#command-form input[name="command"]');
            input.value = 'look at the casle';
            document.querySelector('#command-form button[type="submit"]').click();
        })()"#,
        None,
    )
    .await
    .unwrap();

    wait_until_visible(page, ".text-check-preview", Duration::from_secs(5)).await;

    page.locator(".text-check-preview button:has-text('Send Original')")
        .await
        .click(None)
        .await
        .unwrap();

    // The confirm response replaces #action-area through htmx (outerHTML), so a
    // fresh #status-display arrives.
    wait_until_visible(page, "#status-display", Duration::from_secs(5)).await;
}

/// Assert the confirm swap produced a fresh #status-display node (not the one
/// the page-load handles pointed at).
async fn assert_status_display_restored(page: &playwright_rs::Page) {
    let fresh = page
        .evaluate::<(), bool>(
            r#"(() => {
                const before = window.__statusBeforeSwap;
                const now = document.getElementById('status-display');
                return !!now && now !== before;
            })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        fresh,
        "the confirm swap did not produce a fresh #status-display"
    );
}

/// Read the live Send button's disabled state and label.
async fn read_submit_button(page: &playwright_rs::Page) -> (bool, String) {
    page.evaluate::<(), (bool, String)>(
        r#"(() => {
            const btn = document.getElementById('submit-btn');
            if (!btn) return [false, ''];
            return [btn.disabled, btn.textContent.trim()];
        })()"#,
        None,
    )
    .await
    .unwrap()
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.7
#[tokio::test]
async fn test_error_toast_on_action_failure() {
    with_stub_page(StubActionOutcome::Error, |page, _stub| async move {
        // The server's 500 body is the engine's error render, so the toast text
        // is distinguishably the server's, not the submitted command echoed.
        submit_command(&page, "Internal server error").await;
        let expected = "Error: Failed to process action: Internal server error";
        wait_for_toast_text(&page, expected).await;

        let (visible, text) = read_error_toast(&page).await;
        assert!(
            visible,
            "#error-notification should gain the .visible class on a 500 response"
        );
        assert_eq!(
            text, expected,
            "#error-notification should display the server's response with tags stripped, got {text:?}"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.8
#[tokio::test]
async fn test_newer_error_keeps_toast_visible() {
    with_stub_page(StubActionOutcome::Error, |page, _stub| async move {
        // Second error at 2.5s, checked at 6s: the first toast's timer (5s)
        // has fired but the second's (7.5s) has not, so the toast stays up.
        submit_command(&page, "First failure").await;
        wait_for_toast_text(&page, "Error: Failed to process action: First failure").await;
        tokio::time::sleep(Duration::from_millis(2500)).await;
        submit_command(&page, "Second failure").await;
        wait_for_toast_text(&page, "Error: Failed to process action: Second failure").await;
        tokio::time::sleep(Duration::from_millis(3500)).await;

        let (visible, text) = read_error_toast(&page).await;
        assert!(
            visible,
            "toast should stay visible until the second error's own timer fires, got text {text:?}"
        );
        assert_eq!(
            text, "Error: Failed to process action: Second failure",
            "#error-notification should show the most recent server error, got {text:?}"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.9
#[tokio::test]
async fn test_send_locks_and_unlocks_after_action_area_swap() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        swap_action_area_via_text_check_confirm(&page).await;
        assert_status_display_restored(&page).await;

        // The form's htmx ack swaps in the pending status; that is when the
        // lock must appear.
        send_action(&page, "wait").await;

        let (disabled, label) = read_submit_button(&page).await;
        assert!(
            disabled,
            "Send should lock during generation after an action-area swap (label {label:?})"
        );
        assert!(
            label.contains("Stop"),
            "locked button should read Stop after the swap, got {label:?}"
        );

        // The stub poll answers "idle", so the fresh status display returns
        // to Ready within one 5s poll cycle — and the button must follow.
        wait_for_status_ready(&page).await;
        let (disabled, label) = read_submit_button(&page).await;
        assert!(
            !disabled,
            "Send should unlock once the status returns to Ready after the swap"
        );
        assert!(
            label.contains("Send"),
            "unlocked button should read Send after the swap, got {label:?}"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.10
#[tokio::test]
async fn test_status_error_reaches_observer_after_action_area_swap() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        async move {
            swap_action_area_via_text_check_confirm(&page).await;
            assert_status_display_restored(&page).await;

            // The stub's /status/generating answers the error span a failed
            // generation renders; the fresh #status-display's own poll swaps it in
            // and the body-level observer toasts it — the toast's beforeSwap
            // listener is not involved.
            status.set(StubStatus::Error("narration failed".to_string()));
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(8),
                    Duration::from_millis(100),
                    || async { read_error_toast(&page).await.0 },
                )
                .await,
                "status error should reach the observer after an action-area swap"
            );
            let (_, text) = read_error_toast(&page).await;
            assert_eq!(
                text, "Error: narration failed",
                "#error-notification should show the status error, got {text:?}"
            );

            // A Ready poll clears lastStatusError; wait for it, then for the first
            // toast's 5s hide timer to run out, so visibility alone proves the
            // second show call.
            status.set(StubStatus::Idle);
            wait_for_status_ready(&page).await;
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(12),
                    Duration::from_millis(100),
                    || async { !read_error_toast(&page).await.0 },
                )
                .await,
                "toast should have hidden before the dedupe check"
            );

            // The Ready status reset the dedupe, so the SAME error must toast
            // again. If the observer survived the swap but its dedupe state leaked
            // across it, the repeat stays hidden.
            status.set(StubStatus::Error("narration failed".to_string()));
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(8),
                    Duration::from_millis(100),
                    || async { read_error_toast(&page).await.0 },
                )
                .await,
                "the same error should toast again after a Ready status reset the dedupe"
            );
            let (_, text) = read_error_toast(&page).await;
            assert_eq!(
                text, "Error: narration failed",
                "#error-notification should show the status error again, got {text:?}"
            );
        }
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.11
#[tokio::test]
async fn test_text_check_result_keeps_command_form() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        // Stash the live nodes; the result must not replace them.
        page.evaluate::<(), ()>(
            r#"(() => {
                window.__formBefore = document.getElementById('command-form');
                window.__statusBefore = document.getElementById('status-display');
            })()"#,
            None,
        )
        .await
        .unwrap();

        let clicked = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const btn = document.querySelector('.check-btn');
                    if (btn) { btn.click(); return true; }
                    return false;
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(clicked, "Should find and click a log-entry check button");

        // The result renders in its own element beside the form. Waiting on
        // the element being non-empty asserts the shipped contract without
        // asserting the stub's canned body.
        wait_until_visible(
            &page,
            "#text-check-result:not(:empty)",
            Duration::from_secs(5),
        )
        .await;

        // Node identity, not the id string: a fresh #command-form would carry
        // the same id and pass a string check while the form was replaced.
        let (form_same, status_same, result_text) = page
            .evaluate::<(), (bool, bool, String)>(
                r#"(() => {
                    const form = document.getElementById('command-form');
                    const status = document.getElementById('status-display');
                    const result = document.getElementById('text-check-result');
                    return [
                        !!form && form === window.__formBefore,
                        !!status && status === window.__statusBefore,
                        result ? result.textContent.trim() : '',
                    ];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(form_same, "the text-check result replaced #command-form");
        assert!(
            status_same,
            "the text-check result replaced #status-display"
        );
        assert!(
            !result_text.is_empty(),
            "#text-check-result should be non-empty, got {result_text:?}"
        );
    })
    .await;
}
