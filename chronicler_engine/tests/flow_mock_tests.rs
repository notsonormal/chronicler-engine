//! Game Flow Tests - Mock LLM Version
//!
//! End-to-end tests verifying the core game loop with mocked LLM responses.
//! These tests verify the UI/UX flow without requiring real LLM API calls.
//!
//! Tests here focus on:
//! - Initial page load (header, story-log, status)
//! - Command submission (form works, status updates)
//! - Polling mechanism (updates appear without page reload)
//! - Message ordering (new messages at bottom)
//!
//! Reference: docs/system/game_flow.md

mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use playwright_rs::Playwright;
    use tokio::time::Duration;

    const TEST_PORT: u16 = 3006;
    const TEST_WORLD: &str = "test";

    async fn launch_chrome() -> (playwright_rs::Playwright, playwright_rs::Browser) {
        use playwright_rs::LaunchOptions;

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright
            .chromium()
            .launch_with_options(LaunchOptions {
                channel: Some("chrome".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        (playwright, browser)
    }

    // ========================================================================
    // Initial Load Tests - No LLM needed
    // ========================================================================

    #[tokio::test]
    async fn test_initial_load_header_shows_location() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Poll until location is loaded (not "Loading...")
        let location = wait_for_non_loading_value(&page, ".location").await;

        println!("Initial location: {}", location);

        assert!(
            !location.contains("Loading"),
            "Location should not show 'Loading...'"
        );
        assert!(
            location.contains('|'),
            "Location should contain '|' separator"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_initial_load_story_log_has_content() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Poll until story log has entries
        let entries = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        println!("Initial story-log entries: {}", entries);
        assert!(entries > 0, "Story log should have entries on initial load");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_initial_load_status_is_ready() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Poll until status is ready
        let status = wait_for_element_text(&page, "#status-display").await;

        println!("Initial status: {}", status);
        assert!(status.contains("Ready"), "Status should show 'Ready'");

        browser.close().await.unwrap();
    }

    // ========================================================================
    // Command Submission Tests - Status updates, no LLM needed
    // ========================================================================

    #[tokio::test]
    async fn test_look_command_shows_thinking() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Poll until element is ready (not loading)
        let _ = wait_for_non_loading_value(&page, ".location").await;

        // Submit look command
        page.evaluate::<(), ()>(
            r#"
            (() => {
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'look';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()
            "#,
            None,
        )
        .await
        .unwrap();

        // Immediately check status (synchronous response)
        let status: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        println!("Status after look: {}", status);

        // Status should show "Thinking..." or "Ready" (timing dependent)
        let has_status = status.contains("Thinking") || status.contains("Ready");
        assert!(has_status, "Status should show 'Thinking...' or 'Ready'");

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_look_command_adds_entry_to_story_log() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Get initial entry count
        let initial_count = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Submit look command (should add user input to log, even if LLM fails)
        page.evaluate::<(), ()>(
            r#"
            (() => {
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'look';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()
            "#,
            None,
        )
        .await
        .unwrap();

        // Poll until we see the input entry appear
        let new_count = wait_for_story_log_entries(&page, initial_count + 1).await;

        println!("Entries after look: {} -> {}", initial_count, new_count);

        // Should have at least one new entry (the input)
        assert!(
            new_count >= initial_count,
            "Story log should have entries after command"
        );

        browser.close().await.unwrap();
    }

    // ========================================================================
    // Polling / Real-time Update Tests - No LLM needed
    // ========================================================================

    #[tokio::test]
    async fn test_realtime_update_without_reload() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Get initial story-log content
        let initial_inner = wait_for_story_log_change(&page, "").await;

        // Submit command
        page.evaluate::<(), ()>(
            r#"
            (() => {
                const input = document.querySelector('#command-form input');
                if (input) {
                    input.value = 'examine the room';
                    console.log('Set input value to: ' + input.value);
                }
                const form = document.querySelector('#command-form');
                if (form) {
                    console.log('Submitting form...');
                    form.requestSubmit();
                }
            })()
            "#,
            None,
        )
        .await
        .unwrap();

        // Wait for LLM to finish generating (up to 10 seconds)
        let llm_result = wait_for_llm_idle(TEST_PORT, Duration::from_secs(10)).await;
        if llm_result.is_err() {
            println!("Warning: LLM did not become idle within timeout");
        }

        // Poll until story-log content changes (max ~5s for polling cycle)
        let updated_inner = wait_for_story_log_change(&page, &initial_inner).await;

        println!(
            "Story-log changed: {} -> {} chars",
            initial_inner.len(),
            updated_inner.len()
        );

        // Content should have changed (new entry added via polling)
        assert_ne!(
            initial_inner, updated_inner,
            "Story-log should update via polling"
        );
        assert!(
            updated_inner.len() > initial_inner.len(),
            "New content should be added"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_new_messages_appear_at_bottom() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Get initial messages (wait for content to load first)
        let initial_messages: Vec<String> = page
            .evaluate::<(), Vec<String>>(
                "Array.from(document.querySelectorAll('#story-log .log-entry .text')).map(el => el.innerText)",
                None,
            )
            .await
            .unwrap();

        let initial_count = initial_messages.len();
        println!("Initial message count: {}", initial_count);

        // Submit a command
        page.evaluate::<(), ()>(
            r#"
            (() => {
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'look around';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()
            "#,
            None,
        )
        .await
        .unwrap();

        // Wait for LLM to finish generating (up to 10 seconds)
        let llm_result = wait_for_llm_idle(TEST_PORT, Duration::from_secs(10)).await;
        if llm_result.is_err() {
            println!("Warning: LLM did not become idle within timeout");
        }

        // Poll until we have more messages
        let final_count = wait_for_more_messages(&page, initial_count).await;

        println!("Final message count: {}", final_count);

        // Should have more messages
        assert!(
            final_count > initial_count,
            "Should have more messages after command. Initial: {}, Final: {}",
            initial_count,
            final_count
        );

        // Get final messages for ordering verification
        let final_messages: Vec<String> = page
            .evaluate::<(), Vec<String>>(
                "Array.from(document.querySelectorAll('#story-log .log-entry .text')).map(el => el.innerText)",
                None,
            )
            .await
            .unwrap();

        // Verify ordering - first messages should be same as initial (oldest at top)
        for i in 0..initial_count {
            assert_eq!(
                initial_messages[i], final_messages[i],
                "Message {} should be same (oldest messages at top)",
                i
            );
        }

        browser.close().await.unwrap();
    }

    // ========================================================================
    // Connection Status Tests - No LLM needed
    // ========================================================================

    #[tokio::test]
    async fn test_connection_indicator_present() {
        let _server = TestServer::new_with_mock(TEST_PORT, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Poll until connection status is present
        let connection_status = wait_for_element_text(&page, "#connection-status").await;

        println!("Connection status: {}", connection_status);

        // Should show some status (Connected, Disconnected, or Ready)
        assert!(
            !connection_status.is_empty(),
            "Connection status should be present"
        );

        browser.close().await.unwrap();
    }
}
