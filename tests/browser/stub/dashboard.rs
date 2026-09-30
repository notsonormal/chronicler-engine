//! Stub-browser tests for dashboard chrome: the error toast, and the action-area handles (Send lock, status error observer) surviving an #action-area swap. Tagged against `docs/specs/browser_dashboard.md`.

// No engine endpoint produces `htmx:beforeSwap` with `isError`; the test
// dispatches the event itself, so the stub server cannot change the behaviour.

use std::time::Duration;

use super::*;

/// Dispatch the synthetic `htmx:beforeSwap` error the toast listens for.
/// The message is wrapped in a tag so the handler's tag-stripping path runs.
async fn dispatch_error_toast(page: &playwright_rs::Page, message: &str) {
    let script = format!(
        r#"(() => {{
            const evt = new CustomEvent('htmx:beforeSwap', {{
                bubbles: true,
                cancelable: true,
                detail: {{
                    isError: true,
                    serverResponse: '<p>{message}</p>',
                    target: document.getElementById('action-area'),
                }},
            }});
            document.body.dispatchEvent(evt);
        }})()"#
    );
    page.evaluate::<(), ()>(&script, None).await.unwrap();
}

/// Read the toast's `.visible` state and displayed text.
async fn read_error_toast(page: &playwright_rs::Page) -> (bool, String) {
    page.evaluate::<(), (bool, String)>(
        r#"(() => {
            const el = document.getElementById('error-notification');
            if (!el) return [false, ''];
            return [el.classList.contains('visible'), el.textContent || ''];
        })()"#,
        None,
    )
    .await
    .unwrap()
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

/// Write `html` into the live #status-display, the way htmx swaps the
/// /status/generating poll response into it. The HTML is bound as an
/// evaluate argument, not spliced into the script source, so callers may
/// pass markup containing quotes.
async fn inject_status_html(page: &playwright_rs::Page, html: &str) {
    let html_owned = html.to_string();
    page.evaluate::<String, ()>(
        r#"(html) => {
            const display = document.getElementById('status-display');
            if (!display) throw new Error('no live #status-display');
            display.innerHTML = html;
        }"#,
        Some(&html_owned),
    )
    .await
    .unwrap();
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.7
#[tokio::test]
async fn test_error_toast_on_action_failure() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        // `dispatch_error_toast` wraps the message in a tag, so the assertion
        // matching the bare text proves the handler's tag-stripping path ran.
        dispatch_error_toast(&page, "Internal server error").await;

        let (visible, text) = read_error_toast(&page).await;
        assert!(
            visible,
            "#error-notification should gain .visible class on a 500 htmx:beforeSwap event"
        );
        assert_eq!(
            text, "Internal server error",
            "#error-notification should display the response body with tags stripped, got {text:?}"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.8
#[tokio::test]
async fn test_newer_error_keeps_toast_visible() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        // Second error at 2.5s, checked at 6s: the first toast's timer (5s)
        // has fired but the second's (7.5s) has not, so the toast stays up.
        dispatch_error_toast(&page, "First failure").await;
        tokio::time::sleep(Duration::from_millis(2500)).await;
        dispatch_error_toast(&page, "Second failure").await;
        tokio::time::sleep(Duration::from_millis(3500)).await;

        let (visible, text) = read_error_toast(&page).await;
        assert!(
            visible,
            "toast should stay visible until the second error's own timer fires, got text {text:?}"
        );
        assert_eq!(
            text, "Second failure",
            "#error-notification should show the most recent error, got {text:?}"
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
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        swap_action_area_via_text_check_confirm(&page).await;
        assert_status_display_restored(&page).await;

        // The engine's /status/generating poll returns exactly this span when
        // a generation failed; htmx swaps it into #status-display. The raw
        // innerHTML write here is the same childList mutation that swap
        // produces, so a live observer must toast it — the toast's own
        // beforeSwap listener is not involved.
        inject_status_html(
            &page,
            r#"<span class="status error">Error: narration failed</span>"#,
        )
        .await;
        let (visible, text) = read_error_toast(&page).await;
        assert!(
            visible,
            "status error should reach the observer after an action-area swap, got {text:?}"
        );
        assert_eq!(
            text, "Error: narration failed",
            "#error-notification should show the status error, got {text:?}"
        );

        // Let the first toast's 5s hide timer run out so visibility alone
        // proves the second show call.
        tokio::time::sleep(Duration::from_millis(5500)).await;
        let (visible, text) = read_error_toast(&page).await;
        assert!(
            !visible,
            "toast should have hidden before the dedupe check, got {text:?}"
        );

        // A Ready status clears lastStatusError, so the SAME error must toast
        // again. If the observer survived the swap but its dedupe state
        // leaked across it, the repeat stays hidden.
        inject_status_html(&page, r#"<span class="status ready">Ready</span>"#).await;
        inject_status_html(
            &page,
            r#"<span class="status error">Error: narration failed</span>"#,
        )
        .await;
        let (visible, text) = read_error_toast(&page).await;
        assert!(
            visible,
            "the same error should toast again after a Ready status reset the dedupe, got {text:?}"
        );
        assert_eq!(
            text, "Error: narration failed",
            "#error-notification should show the status error again, got {text:?}"
        );
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
