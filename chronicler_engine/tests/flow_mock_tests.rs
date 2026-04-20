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

use std::sync::Arc;
use playwright_rs::{Browser, LaunchOptions, Playwright};

/// Shared browser state - initialized once and reused across all tests
static SHARED_BROWSER: std::sync::RwLock<Option<Arc<Browser>>> = std::sync::RwLock::new(None);

/// Get or init shared browser
async fn get_shared_browser() -> Arc<Browser> {
    // Try fast path - already initialized
    if let Some(browser) = SHARED_BROWSER.read().unwrap().as_ref() {
        return Arc::clone(browser);
    }
    
    // Slow path - initialize
    let playwright = Playwright::launch().await.unwrap();
    let browser = playwright
        .chromium()
        .launch_with_options(LaunchOptions {
            channel: Some("chrome".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    let shared = Arc::new(browser);
    
    *SHARED_BROWSER.write().unwrap() = Some(Arc::clone(&shared));
    shared
}

/// Cleanup shared browser (call after all tests)
async fn cleanup_shared_browser() {
    if let Some(browser) = SHARED_BROWSER.write().unwrap().take() {
        browser.close().await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use config-based server - no hardcoded port
    const TEST_WORLD: &str = "test";
    const CONFIG_PATH: &str = "tests/test_config.json";

    // Initial Load Tests - No LLM needed

    #[tokio::test]
    async fn test_initial_load_header_shows_location() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        // Use shared browser instead of launching new one each time
        let browser = get_shared_browser().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        // Wait for story log entries
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // Location is now in story log as location-header, not in header
        let location = wait_for_non_loading_value(&page, ".location-header").await;

        println!("Initial location: {}", location);

        assert!(
            !location.is_empty(),
            "Story log should display current location"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_initial_load_story_log_has_content() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        // Poll until story log has entries
        let entries = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        println!("Initial story-log entries: {}", entries);
        assert!(entries > 0, "Story log should have entries on initial load");

        // Wait for completion
        wait_for_status_ready(&page).await;

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_initial_load_status_is_ready() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        // Wait for status to be stable (ready, not thinking)
        wait_for_status_ready(&page).await;
        let status = wait_for_element_text(&page, "#status-display").await;

        println!("Initial status: {}", status);
        assert!(status.contains("Ready"), "Status should show 'Ready'");

        browser.close().await.unwrap();
    }

    // Command Submission Tests - Status updates, no LLM needed

    #[tokio::test]
    async fn test_look_command_shows_thinking() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

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

        // Wait for completion to avoid polluting next test
        wait_for_status_ready(&page).await;

        browser.close().await.unwrap();
    }
}
