//! End-to-End Tests - Browser-Based
//!
//! Merged from: spec_tests.rs, behavior_tests.rs, ui_tests.rs, layout_tests.rs
//! Duplicates removed, pointless tests removed.
//!
//! Runtime: ~60 seconds
//!
//! Run with: cargo test --test e2e_tests

mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use playwright_rs::Playwright;

    const TEST_WORLD: &str = "test";
    const CONFIG_PATH: &str = "tests/test_config.json";

    // ========================================================================
    // UI Structure Tests (from spec_tests.rs, ui_tests.rs)
    // ========================================================================

    #[tokio::test]
    async fn test_page_loads() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_header_displays_game_title() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_connection_status_indicator() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let has_status: bool = page
            .evaluate::<(), bool>(
                "document.querySelector('#connection-status') !== null",
                None,
            )
            .await
            .unwrap();
        assert!(has_status, "Connection status indicator should exist");

        browser.close().await.unwrap();
    }

    // ========================================================================
    // Action Area Tests (from spec_tests.rs, behavior_tests.rs)
    // ========================================================================

    #[tokio::test]
    async fn test_action_area_elements() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_input_validation_required() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_form_submission() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    // ========================================================================
    // Story Log Tests (from spec_tests.rs, layout_tests.rs)
    // ========================================================================

    #[tokio::test]
    async fn test_story_log_populated() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_story_log_scrollable() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    // ========================================================================
    // Layout Tests (from layout_tests.rs) - Critical Ones Only
    // ========================================================================

    #[tokio::test]
    async fn test_no_horizontal_overflow() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_element_positioning() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    // ========================================================================
    // Visual Sidebar Tests (from spec_tests.rs, ui_tests.rs)
    // ========================================================================

    #[tokio::test]
    async fn test_visual_sidebar_exists() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let has_sidebar: bool = page
            .evaluate::<(), bool>("document.querySelector('.visual-sidebar') !== null", None)
            .await
            .unwrap();
        assert!(has_sidebar, "Visual sidebar should exist");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_action_hints_visible() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let has_hints: bool = page
            .evaluate::<(), bool>("document.querySelector('.action-hints') !== null", None)
            .await
            .unwrap();
        assert!(has_hints, "Action hints should exist");

        browser.close().await.unwrap();
    }

    // ========================================================================
    // Static Shell Test (from spec_tests.rs)
    // ========================================================================

    #[tokio::test]
    async fn test_form_stays_static_after_submission() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    // ========================================================================
    // CSS Tests
    // ========================================================================

    #[tokio::test]
    async fn test_css_file_loads() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

        // Fetch the CSS file via fetch API
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

        // Check :root exists in CSS (CSS custom properties declaration)
        assert!(
            css_content.contains(":root"),
            "CSS should contain :root for CSS variables"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_css_variables_used() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

        // Fetch the CSS file
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

        // Check CSS variables (var(-- ) are used
        assert!(
            css_content.contains("var(--"),
            "CSS should use CSS custom properties (var())"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_responsive_breakpoints() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

        // Fetch the CSS file
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

        // Check @media queries exist for responsive design
        assert!(
            css_content.contains("@media"),
            "CSS should contain @media queries for responsive breakpoints"
        );

        browser.close().await.unwrap();
    }

    // ========================================================================
    // Scrollbar & NPC Portrait Layout Tests (new in ui-improvements plan)
    // ========================================================================

    #[tokio::test]
    async fn test_scrollbar_styled() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_npc_portraits_horizontal_layout() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_npc_portraits_fixed_width() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

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
            "NPC portrait should have fixed width around 80px, got {}",
            width
        );

        browser.close().await.unwrap();
    }
}
