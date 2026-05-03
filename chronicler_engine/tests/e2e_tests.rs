//! [DOC: docs/reference/testing.md]

mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use playwright_rs::Playwright;

    const TEST_WORLD: &str = "test";
    const CONFIG_PATH: &str = "tests/test_config.json";

    // UI Structure Tests

    #[tokio::test]
    async fn test_page_loads() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let title = page.title().await.unwrap();
        assert_eq!(title, "Chronicler Engine");

        let has_header: bool = page
            .evaluate::<(), bool>("document.querySelector('.header') !== null", None)
            .await
            .unwrap();
        assert!(has_header, "Header should exist");

        let has_story_log: bool = page
            .evaluate::<(), bool>("document.querySelector('#story-log') !== null", None)
            .await
            .unwrap();
        assert!(has_story_log, "Story log should exist");

        let has_action_area: bool = page
            .evaluate::<(), bool>("document.querySelector('.action-area') !== null", None)
            .await
            .unwrap();
        assert!(has_action_area, "Action area should exist");

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_header_displays_game_title() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let title: String = page
            .evaluate::<(), String>(
                "document.querySelector('.game-title')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            title, "Chronicler Engine",
            "Header should display game title"
        );

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_connection_status_indicator() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let has_status: bool = page
            .evaluate::<(), bool>(
                "document.querySelector('#connection-status') !== null",
                None,
            )
            .await
            .unwrap();
        assert!(has_status, "Connection status indicator should exist");

        let _ = browser.close().await;
    }

    // Action Area Tests

    #[tokio::test]
    async fn test_action_area_elements() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let has_input: bool = page
            .evaluate::<(), bool>(
                "document.querySelector('#command-form input') !== null",
                None,
            )
            .await
            .unwrap();
        assert!(has_input, "Input field should exist");

        let has_button: bool = page
            .evaluate::<(), bool>(
                "document.querySelector('#command-form button') !== null",
                None,
            )
            .await
            .unwrap();
        assert!(has_button, "Submit button should exist");

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_input_validation_required() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Check input has required attribute
        let has_required: bool = page
            .evaluate::<(), bool>(
                "document.querySelector('#command-form input')?.hasAttribute('required')",
                None,
            )
            .await
            .unwrap();
        assert!(
            has_required,
            "Input should have required attribute for validation"
        );

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_form_submission() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Submit a command
        page.evaluate::<(), ()>(
            "(() => { 
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'look';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()",
            None,
        )
        .await
        .unwrap();

        // Wait for completion to avoid polluting next test
        wait_for_status_ready(&page).await;

        let _ = browser.close().await;
    }

    // Story Log Tests

    #[tokio::test]
    async fn test_story_log_populated() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let log_entries: u32 = page
            .evaluate::<(), u32>(
                "document.querySelectorAll('#story-log .log-entry').length",
                None,
            )
            .await
            .unwrap();
        assert!(
            log_entries > 0,
            "Story log should have entries on initial load"
        );

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_story_log_scrollable() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let overflow_y: String = page
            .evaluate::<(), String>(
                "(() => {
                    const el = document.querySelector('#story-log');
                    return window.getComputedStyle(el).overflowY;
                })()",
                None,
            )
            .await
            .unwrap();

        assert!(
            overflow_y == "auto" || overflow_y == "scroll",
            "Story log should be scrollable"
        );

        let _ = browser.close().await;
    }

    // Layout Tests

    #[tokio::test]
    async fn test_no_horizontal_overflow() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let has_overflow = page
            .evaluate::<(), bool>(
                r#"() => {
                    const body = document.body;
                    const html = document.documentElement;
                    return body.scrollWidth > html.clientWidth || body.clientWidth > html.clientWidth;
                }"#,
                None,
            )
            .await
            .unwrap();

        assert!(!has_overflow, "Page should not have horizontal overflow");

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_element_positioning() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let header_top = page
            .evaluate::<(), f64>(
                "document.querySelector('.header')?.getBoundingClientRect().top || -1",
                None,
            )
            .await
            .unwrap();
        let story_log_top = page
            .evaluate::<(), f64>(
                "document.querySelector('#story-log')?.getBoundingClientRect().top || -1",
                None,
            )
            .await
            .unwrap();
        let action_area_top = page
            .evaluate::<(), f64>(
                "document.querySelector('.action-area')?.getBoundingClientRect().top || -1",
                None,
            )
            .await
            .unwrap();

        assert!(
            story_log_top > header_top,
            "Story log should be below header"
        );
        assert!(
            action_area_top > story_log_top,
            "Action area should be below story log"
        );

        let _ = browser.close().await;
    }

    // Visual Sidebar Tests

    #[tokio::test]
    async fn test_visual_sidebar_exists() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let has_sidebar: bool = page
            .evaluate::<(), bool>("document.querySelector('.visual-sidebar') !== null", None)
            .await
            .unwrap();
        assert!(has_sidebar, "Visual sidebar should exist");

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_action_hints_visible() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let has_hints: bool = page
            .evaluate::<(), bool>("document.querySelector('.action-hints') !== null", None)
            .await
            .unwrap();
        assert!(has_hints, "Action hints should exist");

        let _ = browser.close().await;
    }

    // Static Shell Test

    #[tokio::test]
    async fn test_form_stays_static_after_submission() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let form_id_before: String = page
            .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
            .await
            .unwrap();

        page.evaluate::<(), ()>(
            "(() => { 
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'look';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()",
            None,
        )
        .await
        .unwrap();

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 2).await;

        let form_id_after: String = page
            .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
            .await
            .unwrap();

        assert_eq!(
            form_id_before, form_id_after,
            "Form should stay in DOM (static shell)"
        );

        let _ = browser.close().await;
    }

    // CSS Tests

    #[tokio::test]
    async fn test_css_valid() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let css_content: String = page
            .evaluate::<(), String>(
                r#"(async () => {
                    const response = await fetch('/assets/styles.css');
                    return await response.text();
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert!(
            css_content.contains(":root"),
            "CSS should contain :root for CSS variables"
        );
        assert!(
            css_content.contains("var(--"),
            "CSS should use CSS custom properties (var())"
        );
        assert!(
            css_content.contains("@media"),
            "CSS should contain @media queries for responsive breakpoints"
        );

        let _ = browser.close().await;
    }

    // Scrollbar & NPC Portrait Layout Tests

    #[tokio::test]
    async fn test_scrollbar_styled() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Verify CSS contains scrollbar styles
        let css_content: String = page
            .evaluate::<(), String>(
                r#"(async () => {
                    const response = await fetch('/assets/styles.css');
                    return await response.text();
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert!(
            css_content.contains("::-webkit-scrollbar"),
            "CSS should contain custom scrollbar styling"
        );
        assert!(
            css_content.contains("scrollbar-width"),
            "CSS should contain Firefox scrollbar-width"
        );

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_npc_portraits_horizontal_layout() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Check flex-wrap is nowrap
        let flex_wrap: String = page
            .evaluate::<(), String>(
                r#"(() => {
                    const el = document.querySelector('.npc-portraits');
                    if (!el) return 'no-element';
                    return window.getComputedStyle(el).flexWrap;
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert_eq!(
            flex_wrap, "nowrap",
            "NPC portraits should have flex-wrap: nowrap"
        );

        // Check overflow-x is auto
        let overflow_x: String = page
            .evaluate::<(), String>(
                r#"(() => {
                    const el = document.querySelector('.npc-portraits');
                    if (!el) return 'no-element';
                    return window.getComputedStyle(el).overflowX;
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert_eq!(
            overflow_x, "auto",
            "NPC portraits should have overflow-x: auto"
        );

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_npc_portraits_fixed_width() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Check portrait image has fixed width (not 100%)
        let width: f64 = page
            .evaluate::<(), f64>(
                r#"(() => {
                    const el = document.querySelector('.image-container.npc-portrait img');
                    if (!el) return 0;
                    const rect = el.getBoundingClientRect();
                    return rect.width;
                })()"#,
                None,
            )
            .await
            .unwrap();

        // Fixed width should be around 80px (not 100% which would be much smaller)
        assert!(
            width > 50.0 && width < 120.0,
            "NPC portrait should have fixed width around 80px, got {width}"
        );

        browser.close().await.unwrap();
    }

    // ============ Edit/Retry UI Tests ============

    #[tokio::test]
    async fn test_edit_button_exists_on_entries() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Edit buttons should exist on non-location entries
        let edit_buttons: i32 = page
            .evaluate::<(), i32>(
                "document.querySelectorAll('.log-entry:not(.location) .edit-btn').length",
                None,
            )
            .await
            .unwrap();

        // There should be at least one edit button (entry with text, not location header)
        assert!(
            edit_buttons > 0,
            "Edit buttons should exist on story entries"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_edit_mode_activates_on_click() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Find first edit button and click it
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

        // Wait for DOM to update with textarea
        let _ = wait_for_element_exists(&page, "#edit-textarea", 10).await;

        // Textarea should now exist
        let textarea_exists: bool = page
            .evaluate::<(), bool>("document.querySelector('#edit-textarea') !== null", None)
            .await
            .unwrap();

        assert!(
            textarea_exists,
            "Textarea should appear after clicking edit"
        );

        // Save/cancel buttons should exist
        let save_exists: bool = page
            .evaluate::<(), bool>("document.querySelector('.save-btn') !== null", None)
            .await
            .unwrap();
        assert!(save_exists, "Save button should appear during edit");

        let cancel_exists: bool = page
            .evaluate::<(), bool>("document.querySelector('.cancel-btn') !== null", None)
            .await
            .unwrap();
        assert!(cancel_exists, "Cancel button should appear during edit");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_edit_cancel_restores_original() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Get text from first non-location entry with actual text content
        let original_text: String = page
            .evaluate::<(), String>(
                r#"(() => {
                    const entries = document.querySelectorAll('.log-entry');
                    for (const entry of entries) {
                        if (entry.classList.contains('location')) continue;
                        const text = entry.querySelector('.text');
                        if (text && text.textContent.trim()) {
                            return text.textContent;
                        }
                    }
                    return '';
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert!(
            !original_text.is_empty(),
            "Original text should not be empty, got: '{original_text}'"
        );

        // Click edit
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

        // Wait for edit mode to be ready before clicking cancel
        let _ = wait_for_element_exists(&page, ".cancel-btn", 10).await;

        // Click cancel
        page.evaluate::<(), bool>(
            r#"(() => {
                const btn = document.querySelector('.cancel-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        // Wait for cancel to complete (textarea should disappear)
        let _ = wait_for_element_not_exists(&page, "#edit-textarea", 10).await;

        // Original text should be restored
        let restored_text: String = page
            .evaluate::<(), String>(
                r#"(() => {
                    const entries = document.querySelectorAll('.log-entry');
                    for (const entry of entries) {
                        if (entry.classList.contains('location')) continue;
                        const text = entry.querySelector('.text');
                        if (text && text.textContent.trim()) {
                            return text.textContent;
                        }
                    }
                    return '';
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            original_text, restored_text,
            "Text should be restored after cancel"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_polling_pauses_during_edit() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Before edit - hx-trigger should include "every"
        let trigger_before: String = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.getAttribute('hx-trigger') || ''",
                None,
            )
            .await
            .unwrap();

        assert!(
            trigger_before.contains("every"),
            "Should have polling trigger before edit"
        );

        // Click edit
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

        // Wait for edit mode - hx-trigger should change to "none"
        let _ = wait_for_element_exists(&page, "#edit-textarea", 10).await;

        // During edit - hx-trigger should be "none"
        let trigger_during: String = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.getAttribute('hx-trigger') || ''",
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            trigger_during, "none",
            "hx-trigger should be 'none' during edit"
        );

        // Click cancel
        page.evaluate::<(), bool>(
            r#"(() => {
                const btn = document.querySelector('.cancel-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        // Wait for cancel to complete (textarea should disappear)
        let _ = wait_for_element_not_exists(&page, "#edit-textarea", 10).await;

        // After cancel - hx-trigger should include "every" again
        let trigger_after: String = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.getAttribute('hx-trigger') || ''",
                None,
            )
            .await
            .unwrap();

        assert!(
            trigger_after.contains("every"),
            "Should have polling trigger after cancel"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_retry_button_on_last_ai_message() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Count retry buttons - should be exactly 1
        let retry_count: i32 = page
            .evaluate::<(), i32>("document.querySelectorAll('.retry-btn').length", None)
            .await
            .unwrap();

        assert_eq!(
            retry_count, 1,
            "Should have exactly one retry button on last AI message"
        );

        // Earlier entries should NOT have retry buttons
        let earlier_entries_have_retry: i32 = page
            .evaluate::<(), i32>(
                r#"(() => {
                    const entries = document.querySelectorAll('.log-entry');
                    if (entries.length <= 1) return 0;
                    let count = 0;
                    for (let i = 0; i < entries.length - 1; i++) {
                        if (entries[i].querySelector('.retry-btn')) count++;
                    }
                    return count;
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            earlier_entries_have_retry, 0,
            "Earlier entries should not have retry buttons"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_edit_textarea_matches_original_height() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Get the original text element height before edit
        let original_height: f64 = page
            .evaluate::<(), f64>(
                r#"(() => {
                    const text = document.querySelector('.log-entry:not(.location) .text');
                    if (!text) return -1;
                    const rect = text.getBoundingClientRect();
                    return rect.height;
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert!(
            original_height > 0.0,
            "Original text should have a valid height"
        );

        // Click edit
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

        // Wait for edit mode to be ready
        let _ = wait_for_element_exists(&page, "#edit-textarea", 10).await;

        // Get textarea height after edit
        let textarea_height: f64 = page
            .evaluate::<(), f64>(
                r#"(() => {
                    const textarea = document.querySelector('#edit-textarea');
                    if (!textarea) return -1;
                    // Force reflow to ensure styles are applied
                    void textarea.offsetHeight;
                    const rect = textarea.getBoundingClientRect();
                    return rect.height;
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert!(textarea_height > 0.0, "Textarea should have a valid height");

        // The textarea height should match the original text height
        // Allow tolerance for rendering differences, borders, and padding (15px)
        let diff = (textarea_height - original_height).abs();
        assert!(
            diff < 15.0,
            "Textarea height ({textarea_height}) should match original text height ({original_height})"
        );

        browser.close().await.unwrap();
    }
}
