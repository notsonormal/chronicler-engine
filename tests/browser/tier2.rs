//! Tier-2 quick-browser tests: browser-only behaviour against a stub server.

// These tests verify what only a browser can observe — DOM shape, JS event
// wiring, client-side rendering — while the server behind them is canned
// (`tests/test_utils/tier2_stub.rs`). They run without an engine process, so
// they avoid the server boot cost the tier-3 tests pay.
//
// Tier placement rule (`tests/STRATEGY.md`): if the server behind the
// behaviour were fake, does the behaviour change? No -> this tier. Yes -> the
// tier-3 full-stack browser tier. A test that asserts server-derived content
// belongs in the HTTP tier instead.

use std::time::Duration;

use super::*;

/// Open the slash palette by typing one keystroke at a time, so the `input`
/// event that opens the menu fires for every character.
async fn type_into_command(page: &playwright_rs::Page, text: &str) {
    let input = page
        .locator(r##"#command-form input[name="command"]"##)
        .await;
    input.focus().await.unwrap();
    input.press_sequentially(text, None).await.unwrap();
}

// The slash palette is pure client-side JS: the shell ships the command list
// and renders the menu from an `input` listener. No engine endpoint is involved,
// so the stub server is a faithful host for the behaviour.
// [docs/specs/browser_slash_menu.md] SCENARIO: 31.1
#[tokio::test]
async fn test_slash_menu_opens_on_slash_tier2() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        type_into_command(&page, "/").await;

        wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

        let cmds: Vec<String> = page
            .locator("#slash-menu .slash-suggestion .slash-cmd")
            .await
            .all_inner_texts()
            .await
            .unwrap_or_default();
        assert_eq!(
            cmds,
            vec![
                "/impersonate".to_string(),
                "/guide".to_string(),
                "/options".to_string()
            ],
            "Menu should list the slash commands in canonical order"
        );
    })
    .await;
}

// The error toast is a body-level `htmx:beforeSwap` listener with an
// `isError` guard; it reads the response body, strips tags, and shows the
// toast. No engine endpoint produces this event in the test — the test
// dispatches it directly — so the stub server cannot change the behaviour.
// [docs/specs/browser_dashboard.md] SCENARIO: 16.7
#[tokio::test]
async fn test_error_toast_on_action_failure_tier2() {
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
