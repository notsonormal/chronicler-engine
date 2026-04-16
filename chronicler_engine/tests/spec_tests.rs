//! UI Test Specification - Derived from dashboard.md
//!
//! This file defines the minimal set of tests needed to verify the dashboard
//! meets its specification. Tests are organized by UI component.
//!
//! Reference: docs/system/dashboard.md

mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use playwright_rs::Playwright;

    const TEST_WORLD: &str = "test";
    const CONFIG_PATH: &str = "tests/test_config.json";

    // HEADER TESTS (dashboard.md Section 1)

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

        // Wait for initial content to load
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
    async fn test_header_displays_location() {
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

        // Location is now in story log, not header - check for location-header class
        let location: String = page
            .evaluate::<(), String>(
                "document.querySelector('.location-header')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        assert!(
            location.len() > 0,
            "Story log should display current location"
        );
        // The location should display the room NAME (e.g., "Test Tavern"), not the ID (e.g., "start")
        assert!(
            location.contains("Test Tavern"),
            "Location should be 'Test Tavern' (the room name from map.json), got: {}",
            location
        );

        browser.close().await.unwrap();
    }

    // STORY LOG TESTS (dashboard.md Section 2)

    #[tokio::test]
    async fn test_story_log_populated_on_load() {
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
    async fn test_story_log_has_correct_styles() {
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

        // Wait specifically for narration class (HTMX applies it after load)
        let has_narration =
            wait_for_element_class(&page, "#story-log .log-entry", "narration", 20).await;

        assert!(has_narration, "Story log should have narration entries");

        // Check dialogue has distinct color (orange/amber)
        let dialogue_color: String = page
            .evaluate::<(), String>(
                "(() => {
                    const el = document.querySelector('#story-log .log-entry.dialogue');
                    if (!el) return '';
                    const style = window.getComputedStyle(el);
                    return style.color;
                })()",
                None,
            )
            .await
            .unwrap();

        // Dialogue should be different from narration (cyan)
        assert_ne!(
            dialogue_color, "rgb(0, 255, 255)",
            "Dialogue should have different color from narration"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_story_log_is_scrollable() {
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

    // VISUAL SIDEBAR TESTS (dashboard.md Section 2)

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

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let has_sidebar: bool = page
            .evaluate::<(), bool>("document.querySelector('.visual-sidebar') !== null", None)
            .await
            .unwrap();

        assert!(has_sidebar, "Visual sidebar should exist");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_location_image_loads() {
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

        let has_image: bool = page
            .evaluate::<(), bool>(
                "document.querySelector('.image-container img') !== null",
                None,
            )
            .await
            .unwrap();

        assert!(has_image, "Location image should load");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_npc_portraits_display() {
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

        let has_portraits: bool = page
            .evaluate::<(), bool>("document.querySelector('.npc-portraits') !== null", None)
            .await
            .unwrap();

        assert!(has_portraits, "NPC portraits container should exist");

        browser.close().await.unwrap();
    }

    // ACTION AREA TESTS (dashboard.md Section 3)

    #[tokio::test]
    async fn test_action_area_has_input() {
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

        let has_input: bool = page
            .evaluate::<(), bool>(
                "document.querySelector('#command-form input') !== null",
                None,
            )
            .await
            .unwrap();

        assert!(has_input, "Action area should have input field");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_action_area_has_submit_button() {
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

        let has_button: bool = page
            .evaluate::<(), bool>(
                "document.querySelector('#command-form button') !== null",
                None,
            )
            .await
            .unwrap();

        assert!(has_button, "Action area should have submit button");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_status_display_exists() {
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

        let has_status: bool = page
            .evaluate::<(), bool>("document.querySelector('#status-display') !== null", None)
            .await
            .unwrap();

        assert!(has_status, "Status display should exist");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_status_shows_ready_initially() {
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

        let status_text: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        assert!(
            status_text.contains("Ready"),
            "Status should show Ready initially, got: {}",
            status_text
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_empty_input_validation() {
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

    // REAL-TIME UPDATES TESTS (dashboard.md Section 4)

    #[tokio::test]
    async fn test_connection_status_indicator_exists() {
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

    // HTMX LOADED TESTS

    #[tokio::test]
    async fn test_htmx_loaded() {
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

        let htmx_loaded: bool = page
            .evaluate::<(), bool>("typeof htmx !== 'undefined'", None)
            .await
            .unwrap();

        assert!(htmx_loaded, "HTMX should be loaded");

        browser.close().await.unwrap();
    }

    // STATIC SHELL VERIFICATION

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

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let form_id_before: String = page
            .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
            .await
            .unwrap();

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

        // Wait for form submission to process
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
}
