//! Stub-browser tests for the options dock: the client-side edit action filling the command input. Tagged against `docs/specs/browser_options.md`.

// `useOption` and `editOption` act on the DOM: the dock shell renders the
// options, and `editOption` copies the option text into the command input and
// focuses it. The stub serves the dock fragment in the real template's shape,
// so the client wiring is exercised unchanged.

use super::*;

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
