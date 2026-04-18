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
        // Load .env before checking for API key
        dotenv::dotenv().ok();

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
        dotenv::dotenv().ok();

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
        dotenv::dotenv().ok();

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

        // Get initial state
        let initial_status: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();
        println!("Initial status: {}", initial_status);

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

        // Wait for any status change
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check if status shows "Thinking..." (LLM is processing)
        let status_during: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();
        println!("Status during: {}", status_during);

        // Wait longer for LLM to complete (if it was triggered)
        let llm_result = wait_for_llm_idle(port, Duration::from_secs(30)).await;

        // Wait for UI to poll and update the DOM after LLM completes
        // Polling happens every 5 seconds, so we need to wait for the next poll
        tokio::time::sleep(Duration::from_secs(6)).await;

        // Final status check
        let status_after: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        println!(
            "Status after (LLM result: {:?}): {}",
            llm_result, status_after
        );

        // Key assertion: After LLM completes (or times out), status should NOT be stuck
        // Either "Ready" (success), "Error" (LLM error), or some other defined state
        // But NOT "Thinking..." which means the flag wasn't reset
        assert!(
            !status_after.contains("Thinking")
                || status_after.contains("Ready")
                || status_after.contains("Error"),
            "Status should not be stuck on 'Thinking...'. Got: {} (LLM result: {:?})",
            status_after,
            llm_result
        );

        browser.close().await.unwrap();
    }
}
