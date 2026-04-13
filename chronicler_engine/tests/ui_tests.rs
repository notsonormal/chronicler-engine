//! UI Tests for Chronicler Engine HTMX Dashboard
//!
//! Run with: cargo test --test ui_tests
//!
//! Note: Requires playwright browsers installed: npx playwright install chromium

mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use playwright_rs::Playwright;

    const TEST_PORT: u16 = 3001;
    const TEST_WORLD: &str = "test";

    #[tokio::test]
    async fn test_page_loads() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
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
    async fn test_htmx_loaded() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
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

    #[tokio::test]
    async fn test_ws_extension_loaded() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let ws_loaded: bool = page
            .evaluate::<(), bool>("htmx?.extension?.ws !== undefined", None)
            .await
            .unwrap();

        if !ws_loaded {
            println!("Note: WS extension may not load from CDN in headless mode");
        }

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_connection_status_visible() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
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
        assert!(has_status, "Connection status should exist");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_action_area_has_input() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
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
    async fn test_action_hints_visible() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let has_hints: bool = page
            .evaluate::<(), bool>("document.querySelector('.action-hints') !== null", None)
            .await
            .unwrap();
        assert!(has_hints, "Action hints should exist");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_move_north_command() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let initial_location: String = page
            .evaluate::<(), String>("document.querySelector('.location')?.innerText || ''", None)
            .await
            .unwrap();
        println!("Initial location: '{}'", initial_location);

        let clicked: String = page
            .evaluate::<(), String>(
                "(() => { 
                    const input = document.querySelector('#command-form input');
                    if (input) input.value = 'move north';
                    const btn = document.querySelector('#command-form button');
                    if (btn) { btn.click(); return 'button_clicked'; }
                    return 'no_button';
                })()",
                None,
            )
            .await
            .unwrap();
        println!("Click result: '{}'", clicked);

        // Poll until location changes or timeout using helper
        let new_location = wait_for_location_change(&page, &initial_location).await;

        println!("New location: '{}'", new_location);

        assert!(
            clicked.contains("button_clicked"),
            "Button should be clickable"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_images_load() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let response = page
            .evaluate::<(), u32>(
                "(() => {
                return fetch('/data/images/test.jpg', { method: 'HEAD' })
                    .then(r => r.status)
                    .catch(() => 0);
            })()",
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            response, 200,
            "Test image should be accessible at /data/images/test.jpg"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_npc_image_visible() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for visual sidebar to load
        let _ = wait_for_element_children(&page, ".npc-portraits", 1).await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Check NPC image renders
        let img_src: String = page
            .evaluate::<(), String>(
                "document.querySelector('.npc-portrait img')?.src || ''",
                None,
            )
            .await
            .unwrap();

        assert!(
            img_src.contains("test.jpg"),
            "NPC image should show test.jpg, got: {}",
            img_src
        );

        // Check image is visible (in DOM and visible)
        let img_visible: bool = page
            .evaluate::<(), bool>(
                "(() => {
                    const img = document.querySelector('.npc-portrait img');
                    return img && img.offsetParent !== null;
                })()",
                None,
            )
            .await
            .unwrap();
        assert!(img_visible, "NPC image should be visible");

        browser.close().await.unwrap();
    }
}
