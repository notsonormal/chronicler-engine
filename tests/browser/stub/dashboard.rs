//! Stub-browser tests for dashboard chrome: the error toast's response-body handling. Tagged against `docs/specs/browser_dashboard.md`.

// The error toast is a body-level `htmx:beforeSwap` listener with an
// `isError` guard; it reads the response body, strips tags, and shows the
// toast. No engine endpoint produces this event in the test — the test
// dispatches it directly — so the stub server cannot change the behaviour.

use super::*;

// [docs/specs/browser_dashboard.md] SCENARIO: 16.7
#[tokio::test]
async fn test_error_toast_on_action_failure() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        // serverResponse carries HTML tags so the assertion exercises the
        // handler's tag-stripping path (`response.replace(/<[^>]*>/g, "")`) and
        // proves the toast text is *derived from* the response body, not just
        // non-empty.
        page.evaluate::<(), ()>(
            r#"(() => {
                const evt = new CustomEvent('htmx:beforeSwap', {
                    bubbles: true,
                    cancelable: true,
                    detail: {
                        isError: true,
                        serverResponse: '<p>Internal server error</p>',
                        target: document.getElementById('action-area'),
                    },
                });
                document.body.dispatchEvent(evt);
            })()"#,
            None,
        )
        .await
        .unwrap();

        let toast_state: (bool, String) = page
            .evaluate::<(), (bool, String)>(
                r#"(() => {
                    const el = document.getElementById('error-notification');
                    if (!el) return [false, ''];
                    return [el.classList.contains('visible'), el.textContent || ''];
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert!(
            toast_state.0,
            "#error-notification should gain .visible class on a 500 htmx:beforeSwap event"
        );
        assert_eq!(
            toast_state.1, "Internal server error",
            "#error-notification should display the response body with tags stripped, got {:?}",
            toast_state.1
        );
    })
    .await;
}
