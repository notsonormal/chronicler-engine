//! Stub-browser tests for dashboard chrome: the error toast's response-body handling and hide-timer behaviour. Tagged against `docs/specs/browser_dashboard.md`.

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
