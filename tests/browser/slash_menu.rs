//! Browser slash-command palette tests: open, filter, navigate, submit flows. Tagged against `docs/specs/browser_slash_menu.md`.

use std::time::Duration;

use super::*;

const COMMAND_INPUT_SELECTOR: &str = r##"#command-form input[name="command"]"##;

/// Type text into the command input one keystroke at a time so the `input`
/// event (which opens the menu) fires for every character.
async fn type_into_command(page: &playwright_rs::Page, text: &str) {
    let input = page.locator(COMMAND_INPUT_SELECTOR).await;
    input.focus().await.unwrap();
    input.press_sequentially(text, None).await.unwrap();
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.1
#[tokio::test]
async fn test_slash_menu_opens_on_slash() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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
        },
    )
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.2
#[tokio::test]
async fn test_slash_menu_filters_by_prefix() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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
        },
    )
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.3
#[tokio::test]
async fn test_slash_menu_arrow_keys_move_active() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
        type_into_command(&page, "/").await;
        wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

        let input = page.locator(COMMAND_INPUT_SELECTOR).await;

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
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            type_into_command(&page, "/").await;
            wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let input = page.locator(COMMAND_INPUT_SELECTOR).await;
            // First suggestion (/impersonate) is active by default.
            input.press("Enter", None).await.unwrap();

            wait_until_hidden(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let value: String = input.input_value(None).await.unwrap_or_default();
            assert_eq!(
                value, "/impersonate ",
                "Enter should populate the input with the highlighted command + trailing space"
            );
        },
    )
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.5
#[tokio::test]
async fn test_slash_menu_escape_closes() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            type_into_command(&page, "/g").await;
            wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let input = page.locator(COMMAND_INPUT_SELECTOR).await;
            input.press("Escape", None).await.unwrap();

            wait_until_hidden(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let value: String = input.input_value(None).await.unwrap_or_default();
            assert_eq!(value, "/g", "Escape should leave the input value unchanged");
        },
    )
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.6
#[tokio::test]
async fn test_slash_menu_click_populates_input() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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
                .locator(COMMAND_INPUT_SELECTOR)
                .await
                .input_value(None)
                .await
                .unwrap_or_default();
            assert_eq!(
                value, "/guide ",
                "Clicking a suggestion should populate the input with its command + trailing space"
            );
        },
    )
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.7
#[tokio::test]
async fn test_slash_menu_reopens_after_action_area_rerender() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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
        },
    )
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.8
#[tokio::test]
async fn test_slash_impersonate_produces_input_entry() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let before = page.locator("#story-log .log-entry.input").await.count().await.unwrap_or(0);

            send_action(&page, "/impersonate hello").await;
            wait_for_status_ready(&page).await;

            let after = page.locator("#story-log .log-entry.input").await.count().await.unwrap_or(0);
            assert_eq!(
                after,
                before + 1,
                "Submitting /impersonate should add one Input entry"
            );

            let input_text: String = page
                .evaluate::<(), String>(
                    r#"(() => {
                        const el = Array.from(document.querySelectorAll('#story-log .log-entry.input .text'))
                            .find(t => t.textContent.includes('/impersonate hello'));
                        return el ? el.textContent : '';
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(
                input_text.is_empty(),
                "/impersonate should not be persisted as a plain Input entry"
            );
        },
    )
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.9
#[tokio::test]
async fn test_slash_guide_does_not_persist_input_entry() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let inputs_before = page
                .locator("#story-log .log-entry.input")
                .await
                .count()
                .await
                .unwrap_or(0);
            let narrations_before = page
                .locator("#story-log .log-entry.narration")
                .await
                .count()
                .await
                .unwrap_or(0);

            send_action(&page, "/guide look around").await;
            wait_for_status_ready(&page).await;

            let inputs_after = page
                .locator("#story-log .log-entry.input")
                .await
                .count()
                .await
                .unwrap_or(0);
            let narrations_after = page
                .locator("#story-log .log-entry.narration")
                .await
                .count()
                .await
                .unwrap_or(0);

            assert_eq!(
                inputs_after, inputs_before,
                "/guide should not add an Input entry"
            );
            assert!(
                narrations_after > narrations_before,
                "/guide should produce at least one Narration entry"
            );
        },
    )
    .await;
}
