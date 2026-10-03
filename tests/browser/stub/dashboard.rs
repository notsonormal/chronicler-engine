//! Stub-browser tests for dashboard chrome: the error toast, the action-area state machine, and the read-only text-check result. Tagged against `docs/specs/browser_dashboard.md`.

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
    fill_command_input(page, command).await;
    page.evaluate::<(), ()>(
        r#"() => {
            const form = document.getElementById('command-form');
            form.requestSubmit();
        }"#,
        None,
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

/// Stash the command form and status display node identities on the page, so a
/// test can prove a swap did not replace them.
async fn stash_action_area_nodes(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            window.__formBefore = document.getElementById('command-form');
            window.__statusBefore = document.getElementById('status-display');
        })()"#,
        None,
    )
    .await
    .unwrap();
}

/// True when the live command form / status display are the stashed nodes
/// (node identity, not the id string a replacement would also carry).
async fn action_area_nodes_unchanged(page: &playwright_rs::Page) -> (bool, bool) {
    page.evaluate::<(), (bool, bool)>(
        r#"(() => {
            const form = document.getElementById('command-form');
            const status = document.getElementById('status-display');
            return [
                !!form && form === window.__formBefore,
                !!status && status === window.__statusBefore,
            ];
        })()"#,
        None,
    )
    .await
    .unwrap()
}

/// Submit a command the auto-check intercepts and wait for the preview in
/// `#action-preview`.
async fn submit_intercepted_command(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            const input = document.querySelector('#command-form input[name="command"]');
            input.value = 'look at the casle';
            document.querySelector('#command-form button[type="submit"]').click();
        })()"#,
        None,
    )
    .await
    .unwrap();

    wait_until_visible(page, ".text-check-preview", Duration::from_secs(5)).await;
}

/// Submit an intercepted command even while the primary button is disabled: the
/// player can still submit the form mid-generation.
async fn submit_intercepted_command_direct(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            const input = document.querySelector('#command-form input[name="command"]');
            input.value = 'look at the casle';
            document.getElementById('command-form').requestSubmit();
        })()"#,
        None,
    )
    .await
    .unwrap();

    wait_until_visible(page, ".text-check-preview", Duration::from_secs(5)).await;
}

/// Confirm the intercepted preview; the client closes it.
async fn confirm_text_check_preview(page: &playwright_rs::Page) {
    page.locator(".text-check-preview .btn-original")
        .await
        .click(None)
        .await
        .unwrap();

    wait_until_hidden(page, ".text-check-preview", Duration::from_secs(5)).await;
    wait_for_status_generating(page).await;
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

/// True while `selector`'s element is the page's focused element.
async fn active_element_is(page: &playwright_rs::Page, selector: &str) -> bool {
    page.evaluate::<String, bool>(
        r#"(selector) => {
            const el = document.querySelector(selector);
            return !!el && document.activeElement === el;
        }"#,
        Some(&selector.to_string()),
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
        submit_command(&page, "First failure").await;
        wait_for_toast_text(&page, "Error: Failed to process action: First failure").await;
        tokio::time::sleep(Duration::from_millis(2500)).await;
        submit_command(&page, "Second failure").await;
        wait_for_toast_text(&page, "Error: Failed to process action: Second failure").await;

        // Poll across the window in which the first toast's 5s timer fires but
        // the second's 7.5s timer does not. A single read 3.5s after the second
        // toast appeared is load-sensitive: a slow submit shifts the read past
        // the second timer and the toast has already hidden.
        let window_end = std::time::Instant::now() + Duration::from_millis(3500);
        while std::time::Instant::now() < window_end {
            let (visible, text) = read_error_toast(&page).await;
            assert!(
                visible,
                "toast should stay visible until the second error's own timer fires, got text {text:?}"
            );
            assert_eq!(
                text, "Error: Failed to process action: Second failure",
                "#error-notification should show the most recent server error, got {text:?}"
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.9
#[tokio::test]
async fn test_primary_button_locks_and_unlocks_after_confirm() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        async move {
            // Keep the stub generating, so no poll can unlock the button
            // before the assertion reads it.
            status.set(StubStatus::Phase("narrating".to_string()));
            stash_action_area_nodes(&page).await;

            submit_intercepted_command(&page).await;
            confirm_text_check_preview(&page).await;

            // The confirm's status span locks the button as a generating indicator.
            let (disabled, label) = read_submit_button(&page).await;
            assert!(
                disabled,
                "Send should lock while the turn runs (label {label:?})"
            );
            assert!(
                !label.contains("Stop"),
                "the locked button must not claim a Stop action, got {label:?}"
            );
            assert!(
                label.contains("Generating"),
                "the locked button should read as a generating indicator, got {label:?}"
            );

            let (form_same, status_same) = action_area_nodes_unchanged(&page).await;
            assert!(form_same, "confirming the preview replaced #command-form");
            assert!(
                status_same,
                "confirming the preview replaced #status-display"
            );

            // Return the stub to idle, so the next 5s poll clears the status and
            // the button must follow it back to Send.
            status.set(StubStatus::Idle);
            wait_for_status_ready(&page).await;
            let (disabled, label) = read_submit_button(&page).await;
            assert!(
                !disabled,
                "Send should unlock once the status returns to Ready"
            );
            assert!(
                label.contains("Send"),
                "unlocked button should read Send, got {label:?}"
            );
        }
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.10
#[tokio::test]
async fn test_status_error_reaches_toast_after_confirm() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        async move {
            submit_intercepted_command(&page).await;
            confirm_text_check_preview(&page).await;

            // The next poll reports a failed generation; its fragment lands in
            // the same #status-display the confirm swapped the pending span into.
            status.set(StubStatus::Error("narration failed".to_string()));
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(8),
                    Duration::from_millis(100),
                    || async { read_error_toast(&page).await.0 },
                )
                .await,
                "status error should reach the toast after confirming a preview"
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
            // again. If the dedupe state leaked, the repeat stays hidden.
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
async fn test_log_entry_check_is_read_only_and_dismissable() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        stash_action_area_nodes(&page).await;

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

        wait_until_visible(
            &page,
            "#text-check-result:not(:empty)",
            Duration::from_secs(5),
        )
        .await;

        let (form_same, status_same) = action_area_nodes_unchanged(&page).await;
        assert!(form_same, "the text-check result replaced #command-form");
        assert!(
            status_same,
            "the text-check result replaced #status-display"
        );

        let (text, send_buttons, confirm_forms) = page
            .evaluate::<(), (String, usize, usize)>(
                r#"(() => {
                    const result = document.getElementById('text-check-result');
                    const buttons = Array.from(result.querySelectorAll('button'));
                    const sendButtons = buttons.filter((b) => /Send/.test(b.textContent)).length;
                    const confirmForms = result.querySelectorAll('form[hx-post="/action/confirm"]').length;
                    return [result.textContent.trim(), sendButtons, confirmForms];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            text.contains("Checked entry #2"),
            "the result should name the entry it checked, got {text:?}"
        );
        assert_eq!(
            send_buttons, 0,
            "a log-entry check must not offer to send a turn, got {send_buttons} Send button(s)"
        );
        assert_eq!(
            confirm_forms, 0,
            "a log-entry check must not contain a confirm form"
        );

        page.locator(".check-result-dismiss")
            .await
            .click(None)
            .await
            .unwrap();
        assert!(
            wait_for_condition_async(
                Duration::from_secs(3),
                Duration::from_millis(50),
                || async {
                    page.evaluate::<(), bool>(
                        "(() => document.getElementById('text-check-result').innerHTML.trim() === '')()",
                        None,
                    )
                    .await
                    .unwrap_or(false)
                },
            )
            .await,
            "dismissing the result should empty its element"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.13
#[tokio::test]
async fn test_preview_opens_beside_status_display() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        async move {
            status.set(StubStatus::Phase("narrating".to_string()));
            wait_for_status_generating(&page).await;

            // The primary button is a disabled generating indicator now, so
            // submit the form directly (the player's Enter path).
            submit_intercepted_command_direct(&page).await;

            assert!(
                wait_for_condition_async(
                    Duration::from_secs(3),
                    Duration::from_millis(50),
                    || async { active_element_is(&page, "#corrected-textarea").await },
                )
                .await,
                "focus should land in the correction textarea even mid-generation"
            );

            let (preview_visible, status_present, status_text) = page
                .evaluate::<(), (bool, bool, String)>(
                    r#"(() => {
                        const preview = document.querySelector('.text-check-preview');
                        const status = document.getElementById('status-display');
                        return [
                            !!preview,
                            !!status,
                            status ? status.textContent.trim() : '',
                        ];
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(preview_visible, "the send preview should be open");
            assert!(
                status_present,
                "opening the preview must not take the status display off screen"
            );
            assert!(
                status_text.contains("Generating") || status_text.contains("Thinking"),
                "the status display should still show the in-flight phase, got {status_text:?}"
            );
        }
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.14
#[tokio::test]
async fn test_clean_log_entry_check_can_be_dismissed() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        // Drive the shipped check function directly with a clean text and an
        // entry id; the stub keys the clean outcome on the text.
        page.evaluate::<(), ()>(
            "(() => { window.checkText('look at the castle', '2'); })()",
            None,
        )
        .await
        .unwrap();

        wait_until_visible(
            &page,
            "#text-check-result:not(:empty)",
            Duration::from_secs(5),
        )
        .await;

        let text = page
            .evaluate::<(), String>(
                "(() => document.getElementById('text-check-result').textContent.trim())()",
                None,
            )
            .await
            .unwrap();
        assert!(
            text.contains("Checked entry #2"),
            "a clean result should still name the entry it checked, got {text:?}"
        );
        assert!(
            text.contains("No issues found"),
            "a clean result should report no issues, got {text:?}"
        );

        page.locator(".check-result-dismiss")
            .await
            .click(None)
            .await
            .unwrap();
        assert!(
            wait_for_condition_async(
                Duration::from_secs(3),
                Duration::from_millis(50),
                || async {
                    page.evaluate::<(), bool>(
                        "(() => document.getElementById('text-check-result').innerHTML.trim() === '')()",
                        None,
                    )
                    .await
                    .unwrap_or(false)
                },
            )
            .await,
            "dismissing a clean result should empty its element"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.15
#[tokio::test]
async fn test_preview_focus_moves_to_correction_and_back() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        submit_intercepted_command(&page).await;

        assert!(
            wait_for_condition_async(
                Duration::from_secs(3),
                Duration::from_millis(50),
                || async { active_element_is(&page, "#corrected-textarea").await },
            )
            .await,
            "focus should land in the correction textarea when the preview opens"
        );

        page.locator(".preview-cancel")
            .await
            .click(None)
            .await
            .unwrap();
        wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;

        assert!(
            wait_for_condition_async(
                Duration::from_secs(3),
                Duration::from_millis(50),
                || async {
                    active_element_is(&page, "#command-form input[name=\"command\"]").await
                },
            )
            .await,
            "cancelling the preview should return focus to the command input"
        );
    })
    .await;
}
