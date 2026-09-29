//! Stub-browser tests for dashboard chrome: the error toast, and the action-area handles (Send lock, status error observer) surviving an #action-area swap. Tagged against `docs/specs/browser_dashboard.md`.

// The error toast is a body-level `htmx:beforeSwap` listener with an
// `isError` guard; it reads the response body, strips tags, and shows the
// toast. No engine endpoint produces this event in the test — the test
// dispatches it directly — so the stub server cannot change the behaviour.

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

/// Swap #action-area the way the text-check preview flow does, through the
/// shipped client JS: `saveActionArea()` snapshots the markup, a raw
/// `innerHTML` write replaces it, and `restoreActionArea()` puts the snapshot
/// back. Every node the page-load handles pointed at (form, status display,
/// Send button) is detached; the ids live on in fresh nodes.
async fn swap_action_area_via_restore(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            saveActionArea();
            // Mark the live status display AFTER the snapshot so the restored
            // markup carries no marker and the test can tell the restored
            // node from the original one.
            const display = document.getElementById('status-display');
            if (display) display.dataset.stale = 'true';
            const area = document.getElementById('action-area');
            area.innerHTML =
                '<div class="text-check-preview"><p>preview</p></div>';
            restoreActionArea();
        })()"#,
        None,
    )
    .await
    .unwrap();
}

/// Assert the restore actually produced fresh nodes: the live status display
/// must not carry the stale marker the swap left on the original one.
async fn assert_status_display_restored(page: &playwright_rs::Page) {
    let restored = page
        .evaluate::<(), bool>(
            r#"(() => {
                const display = document.getElementById('status-display');
                return !!display && display.dataset.stale !== 'true';
            })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        restored,
        "the action-area swap did not produce a fresh #status-display"
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
        // serverResponse carries HTML tags so the assertion exercises the
        // handler's tag-stripping path (`response.replace(/<[^>]*>/g, "")`) and
        // proves the toast text is *derived from* the response body, not just
        // non-empty.
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
        // Two errors inside the 5s hide window. The bug was that the first
        // error's hide timer was never cleared, so it hid the toast while the
        // second error's toast was still on screen — the banner slid up and
        // down on a loop. Six seconds after the first event the first timer
        // has fired (5s) but the second (7.5s) has not: a correct toast is
        // still visible.
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
        swap_action_area_via_restore(&page).await;
        assert_status_display_restored(&page).await;

        // Submit through the shipped form. The fresh form's htmx flow acks
        // with the pending status swap, which is when the lock must appear.
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
        swap_action_area_via_restore(&page).await;
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
