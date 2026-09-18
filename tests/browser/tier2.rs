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

/// Type text into the command input one keystroke at a time so the `input`
/// event (which opens the menu) fires for every character.
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
async fn test_slash_menu_opens_on_slash() {
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

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.2
#[tokio::test]
async fn test_slash_menu_filters_by_prefix() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        type_into_command(&page, "/g").await;

        wait_until_visible(
            &page,
            "#slash-menu .slash-suggestion",
            Duration::from_millis(1000),
        )
        .await;

        let cmds: Vec<String> = page
            .locator("#slash-menu .slash-suggestion .slash-cmd")
            .await
            .all_inner_texts()
            .await
            .unwrap_or_default();
        assert_eq!(
            cmds,
            vec!["/guide".to_string()],
            "Only /guide should match the /g prefix"
        );
    })
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.3
#[tokio::test]
async fn test_slash_menu_arrow_keys_move_active() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        type_into_command(&page, "/").await;
        wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

        let input = page
            .locator(r##"#command-form input[name="command"]"##)
            .await;

        let first_active: bool = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const items = document.querySelectorAll('#slash-menu .slash-suggestion');
                    return items.length > 0 && items[0].classList.contains('active');
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(first_active, "First suggestion should start active");

        input.press("ArrowDown", None).await.unwrap();
        let second_active: bool = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const items = document.querySelectorAll('#slash-menu .slash-suggestion');
                    return items.length > 1 && items[1].classList.contains('active') && !items[0].classList.contains('active');
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(second_active, "ArrowDown should move active to the second suggestion");

        input.press("ArrowUp", None).await.unwrap();
        let first_active_again: bool = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const items = document.querySelectorAll('#slash-menu .slash-suggestion');
                    return items.length > 0 && items[0].classList.contains('active');
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(first_active_again, "ArrowUp should move active back to the first suggestion");
    })
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.4
#[tokio::test]
async fn test_slash_menu_enter_populates_input() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        type_into_command(&page, "/").await;
        wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

        let input = page
            .locator(r##"#command-form input[name="command"]"##)
            .await;
        // First suggestion (/impersonate) is active by default.
        input.press("Enter", None).await.unwrap();

        wait_until_hidden(&page, "#slash-menu", Duration::from_millis(1000)).await;

        let value: String = input.input_value(None).await.unwrap_or_default();
        assert_eq!(
            value, "/impersonate ",
            "Enter should populate the input with the highlighted command + trailing space"
        );
    })
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.5
#[tokio::test]
async fn test_slash_menu_escape_closes() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        type_into_command(&page, "/g").await;
        wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

        let input = page
            .locator(r##"#command-form input[name="command"]"##)
            .await;
        input.press("Escape", None).await.unwrap();

        wait_until_hidden(&page, "#slash-menu", Duration::from_millis(1000)).await;

        let value: String = input.input_value(None).await.unwrap_or_default();
        assert_eq!(value, "/g", "Escape should leave the input value unchanged");
    })
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.6
#[tokio::test]
async fn test_slash_menu_click_populates_input() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        type_into_command(&page, "/").await;
        wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

        // Click the /guide suggestion by its command text, independent of menu order.
        page.locator("#slash-menu .slash-suggestion:has(.slash-cmd:text-is('/guide'))")
            .await
            .click(None)
            .await
            .unwrap();

        wait_until_hidden(&page, "#slash-menu", Duration::from_millis(1000)).await;

        let value: String = page
            .locator(r##"#command-form input[name="command"]"##)
            .await
            .input_value(None)
            .await
            .unwrap_or_default();
        assert_eq!(
            value, "/guide ",
            "Clicking a suggestion should populate the input with its command + trailing space"
        );
    })
    .await;
}

// The slash palette's listeners are document-level delegations; what the
// scenario checks is that replacing the action area re-parses the command form
// and typing `/` into the fresh input reopens the menu. The transplant is a
// real DOM replacement (it destroys and recreates the form nodes and fires
// focusout), not a mock of the behaviour, so the stub server changes nothing —
// this test does not read served content.
// [docs/specs/browser_slash_menu.md] SCENARIO: 31.7
#[tokio::test]
async fn test_slash_menu_reopens_after_action_area_rerender() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        type_into_command(&page, "/").await;
        wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

        page.evaluate::<(), ()>(
            r##"(() => {
                const area = document.getElementById('action-area');
                const productionMarkup = area.innerHTML;
                area.innerHTML = productionMarkup;
            })()"##,
            None,
        )
        .await
        .unwrap();

        wait_until_hidden(&page, "#slash-menu", Duration::from_millis(1000)).await;

        type_into_command(&page, "/").await;
        wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

        let count: u32 = page
            .locator("#slash-menu .slash-suggestion")
            .await
            .count()
            .await
            .unwrap_or(0) as u32;
        assert_eq!(
            count, 3,
            "Menu should reopen with all commands after re-render"
        );
    })
    .await;
}

// `useOption` and `editOption` act on the DOM: the dock shell renders the
// options, and `editOption` copies the option text into the command input and
// focuses it. The stub serves the dock fragment in the real template's shape,
// so the client wiring is exercised unchanged.
// [docs/specs/browser_options.md] SCENARIO: 26.3
#[tokio::test]
async fn test_options_dock_edit_fills_without_submitting() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
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
    })
    .await;
}

// `showEditForm` is pure client JS over an existing `.log-entry`: it swaps the
// entry's `.text` for a `#edit-textarea`. The stub serves a canned story-log
// entry in the real template's shape; the edit behaviour does not depend on the
// entry's text.
// [docs/specs/browser_story_log.md] SCENARIO: 30.1
#[tokio::test]
async fn test_edit_mode_activates_on_click() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        let clicked = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const btn = document.querySelector('.edit-btn');
                    if (btn) {
                        btn.click();
                        return true;
                    }
                    return false;
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(clicked, "Should find and click an edit button");

        wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;
    })
    .await;
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.2
#[tokio::test]
async fn test_edit_cancel_restores_original() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        let original_text = page
            .locator(".log-entry .text")
            .await
            .inner_text()
            .await
            .unwrap_or_default();
        assert!(!original_text.is_empty(), "Should have original text");

        page.locator(".edit-btn").await.click(None).await.unwrap();
        wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;

        let modified = "Modified text for testing";
        page.locator("#edit-textarea")
            .await
            .fill(modified, None)
            .await
            .unwrap();

        page.locator(".cancel-btn").await.click(None).await.unwrap();
        wait_until_hidden(&page, "#edit-textarea", Duration::from_millis(500)).await;

        let restored = page
            .locator(".log-entry .text")
            .await
            .inner_text()
            .await
            .unwrap_or_default();

        assert_eq!(
            restored, original_text,
            "Text should be restored to original after cancel"
        );
    })
    .await;
}

// The stub polls `/fragment/story-log` every 2s and returns the canned entry,
// so if `pausePolling` were broken the swap would replace `#story-log`'s
// innerHTML and destroy the textarea. The test therefore still exercises the
// pause: only the paused state lets the textarea survive.
// [docs/specs/browser_story_log.md] SCENARIO: 30.3
#[tokio::test]
async fn test_polling_pauses_during_edit() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        page.evaluate::<(), bool>(
            r#"(() => {
                const btn = document.querySelector('.edit-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;

        let persisted =
            wait_for_element_persist(&page, "#edit-textarea", Duration::from_secs(3)).await;
        assert!(
            persisted,
            "Edit textarea should persist during polling pause"
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
