//! Stub-browser tests for the slash menu: the client-side command palette rendered from the shipped shell's `input` listener. Tagged against `docs/specs/browser_slash_menu.md`.

// The palette is a document-level `input` listener plus `<body>` child on the
// shipped shell, so the stub's only job is to serve `assets/index.html`.

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

        let exposed: bool = page
            .evaluate::<(), bool>(
                r#"() => {
                    const menu = document.getElementById('slash-menu');
                    const input = document.querySelector('#command-form input[name="command"]');
                    const options = Array.from(menu.querySelectorAll('.slash-suggestion'));
                    return menu.getAttribute('role') === 'listbox'
                        && options.every((o) => o.getAttribute('role') === 'option')
                        && input.getAttribute('aria-expanded') === 'true'
                        && input.getAttribute('aria-controls') === 'slash-menu'
                        && input.getAttribute('aria-activedescendant') === options[0].id
                        && options[0].getAttribute('aria-selected') === 'true';
                }"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            exposed,
            "the open menu must expose a listbox whose first option is active"
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

        let moved: bool = page
            .evaluate::<(), bool>(
                r#"() => {
                    const menu = document.getElementById('slash-menu');
                    const input = document.querySelector('#command-form input[name="command"]');
                    const options = Array.from(menu.querySelectorAll('.slash-suggestion'));
                    return input.getAttribute('aria-activedescendant') === options[1].id
                        && options[1].getAttribute('aria-selected') === 'true'
                        && options[0].getAttribute('aria-selected') === 'false';
                }"#,
                None,
            )
            .await
            .unwrap();
        assert!(moved, "ArrowDown must move the reported active option");

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
