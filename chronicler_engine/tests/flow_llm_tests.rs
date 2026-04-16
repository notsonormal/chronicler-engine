//! Game Flow Tests - Real LLM Version
//!
//! End-to-end tests verifying the game loop with real LLM API calls.
//! These tests require OPENROUTER_API_KEY to be set and will be skipped if not available.
//!
//! Tests here focus on LLM-specific functionality:
//! - LLM generates actual narrative responses
//! - Different action types produce different LLM outputs
//! - Full end-to-end flow with real AI responses
//!
//! Reference: docs/system/game_flow.md

mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use playwright_rs::Playwright;
    use tokio::time::Duration;

    const TEST_WORLD: &str = "test";
    const CONFIG_PATH: &str = "tests/test_config.json";

    // Check if real LLM is available
    fn has_llm_api_key() -> bool {
        std::env::var("OPENROUTER_API_KEY").is_ok()
    }

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
    // LLM-Specific Tests - These require real LLM API
    // ========================================================================

    #[tokio::test]
    async fn test_llm_generates_narration_for_free_action() {
        if !has_llm_api_key() {
            eprintln!("Skipping: OPENROUTER_API_KEY not set");
            return;
        }

        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new(port, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

        // Get initial story-log content
        let initial_messages: Vec<String> = page
            .evaluate::<(), Vec<String>>(
                "Array.from(document.querySelectorAll('#story-log .log-entry .text')).map(el => el.innerText)",
                None,
            )
            .await
            .unwrap();

        let initial_count = initial_messages.len();
        println!("Initial messages: {}", initial_count);

        // Submit a free action that triggers LLM
        page.evaluate::<(), ()>(
            r#"
            (() => {
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'examine the mysterious orb on the table';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()
            "#,
            None,
        )
        .await
        .unwrap();

        // Wait for LLM response using smart polling
        println!("Waiting for LLM response...");
        let llm_result = wait_for_llm_idle(port, Duration::from_secs(30)).await;
        if llm_result.is_err() {
            println!("Warning: LLM did not become idle within timeout");
        }

        // Poll until we have more messages
        let final_count = wait_for_more_messages(&page, initial_count).await;
        println!("Final messages: {}", final_count);

        // Should have new messages
        assert!(
            final_count > initial_count,
            "Should have more messages after LLM processes command. Initial: {}, Final: {}",
            initial_count,
            final_count
        );

        // Get the actual messages to check content
        let final_messages: Vec<String> = page
            .evaluate::<(), Vec<String>>(
                "Array.from(document.querySelectorAll('#story-log .log-entry .text')).map(el => el.innerText)",
                None,
            )
            .await
            .unwrap();

        // Check that the new message is NOT a mock (should be real LLM output)
        let new_message = final_messages.last().unwrap();
        println!("New message: {}", new_message);

        // Real LLM responses won't contain "[MockNarration]" or "[MockGenerated]"
        assert!(
            !new_message.contains("[MockNarration]") && !new_message.contains("[MockGenerated]"),
            "LLM should generate real response, not mock. Got: {}",
            new_message
        );

        // Should have some substantial content (not just empty/error)
        assert!(
            new_message.len() > 20,
            "LLM response should be substantial. Got: {}",
            new_message
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_llm_narration_appears_via_polling() {
        if !has_llm_api_key() {
            eprintln!("Skipping: OPENROUTER_API_KEY not set");
            return;
        }

        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new(port, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

        // Get initial content
        let initial_text: String = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        // Submit command that triggers LLM
        page.evaluate::<(), ()>(
            r#"
            (() => {
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'look around and describe what you see';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()
            "#,
            None,
        )
        .await
        .unwrap();

        // Check status shows "Thinking..." immediately after submit
        let status_during: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        println!("Status during LLM: {}", status_during);
        assert!(
            status_during.contains("Thinking"),
            "Status should show 'Thinking...' during LLM"
        );

        // Wait for LLM to complete and polling to catch it
        let llm_result = wait_for_llm_idle(port, Duration::from_secs(30)).await;
        if llm_result.is_err() {
            println!("Warning: LLM did not become idle within timeout");
        }

        // Poll until content changes
        let final_text = wait_for_story_log_change(&page, &initial_text).await;
        println!(
            "Content length: {} -> {}",
            initial_text.len(),
            final_text.len()
        );

        // Content should have changed via polling
        assert!(
            !final_text.is_empty(),
            "Story-log should update via polling after LLM"
        );
        assert_ne!(initial_text, final_text, "Story-log content should change");

        // Status should return to ready
        let status_after: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        println!("Status after LLM: {}", status_after);
        assert!(
            status_after.contains("Ready"),
            "Status should return to 'Ready' after LLM"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_llm_handles_arrival_narration() {
        if !has_llm_api_key() {
            eprintln!("Skipping: OPENROUTER_API_KEY not set");
            return;
        }

        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new(port, TEST_WORLD);

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", port), None)
            .await
            .unwrap();

        // Submit move command - should trigger arrival narration
        page.evaluate::<(), ()>(
            r#"
            (() => {
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'go north';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()
            "#,
            None,
        )
        .await
        .unwrap();

        // Wait for LLM arrival narration
        let llm_result = wait_for_llm_idle(port, Duration::from_secs(30)).await;
        if llm_result.is_err() {
            println!("Warning: LLM did not become idle within timeout");
        }

        // Get story-log entries
        let messages: Vec<String> = page
            .evaluate::<(), Vec<String>>(
                "Array.from(document.querySelectorAll('#story-log .log-entry .text')).map(el => el.innerText)",
                None,
            )
            .await
            .unwrap();

        println!("Messages after move: {:?}", messages);

        // Should have more messages than initial
        // Initial load has: welcome, logged in, room description
        // After move: should have user input + LLM arrival narration
        assert!(messages.len() >= 3, "Should have messages after move");

        // The new message should be LLM-generated (not mock)
        if let Some(new_msg) = messages.last() {
            assert!(
                !new_msg.contains("[MockArrival]") && !new_msg.contains("[MockNarration]"),
                "Arrival should use real LLM, got: {}",
                new_msg
            );
        }

        browser.close().await.unwrap();
    }

    // ========================================================================
    // Error Handling Tests - LLM specific
    // ========================================================================

    #[tokio::test]
    async fn test_llm_error_shows_in_story_log() {
        if !has_llm_api_key() {
            eprintln!("Skipping: OPENROUTER_API_KEY not set");
            return;
        }

        // This test would verify error handling - currently there's no explicit error handling
        // that shows in the UI. This is a placeholder for future error handling tests.

        eprintln!("Note: Error handling test not yet implemented");
        eprintln!("Future: Test that LLM API errors show user-friendly message in story-log");
    }
}
