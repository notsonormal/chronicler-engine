//! Game Flow Tests - Real LLM Version
//!
//! End-to-end tests verifying the game loop with real LLM API calls.
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

    #[tokio::test]
    async fn test_llm_generates_narration_for_free_action() {
        dotenv::dotenv().ok();

        if !has_llm_api_key() {
            eprintln!("Skipping: OPENROUTER_API_KEY not set");
            return;
        }

        eprintln!("Running: OPENROUTER_API_KEY set");

        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let initial_messages: Vec<String> = page
            .evaluate::<(), Vec<String>>(
                "Array.from(document.querySelectorAll('#story-log .log-entry .text')).map(el => el.innerText)",
                None,
            )
            .await
            .unwrap();

        let initial_count = initial_messages.len();
        println!("Initial messages: {}", initial_count);

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

        println!("Waiting for LLM response...");
        let llm_result = wait_for_llm_idle(port, Duration::from_secs(180)).await;
        if llm_result.is_err() {
            let status: String = page
                .evaluate::<(), String>(
                    "document.querySelector('#status-display')?.innerText || 'no status'",
                    None,
                )
                .await
                .unwrap_or_default();
            let error_msg: Option<String> = page
                .evaluate::<(), Option<String>>(
                    "document.querySelector('.error-message')?.innerText || null",
                    None,
                )
                .await
                .unwrap_or(None);
            println!("Warning: LLM did not become idle within timeout. Status: '{status}'");
            if let Some(msg) = error_msg {
                println!("Error message: {msg}");
            }
        }

        let final_count = wait_for_more_messages(&page, initial_count).await;
        println!("Final messages: {}", final_count);

        assert!(
            final_count > initial_count,
            "Should have more messages after LLM processes command. Initial: {}, Final: {}",
            initial_count,
            final_count
        );

        let final_messages: Vec<String> = page
            .evaluate::<(), Vec<String>>(
                "Array.from(document.querySelectorAll('#story-log .log-entry .text')).map(el => el.innerText)",
                None,
            )
            .await
            .unwrap();

        let new_message = final_messages.last().unwrap();
        println!("New message: {}", new_message);

        assert!(
            !new_message.contains("[MockNarration]") && !new_message.contains("[MockGenerated]"),
            "LLM should generate real response, not mock. Got: {}",
            new_message
        );

        assert!(
            new_message.len() > 20,
            "LLM response should be substantial. Got: {}",
            new_message
        );

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_llm_narration_appears_via_polling() {
        dotenv::dotenv().ok();

        if !has_llm_api_key() {
            eprintln!("Skipping: OPENROUTER_API_KEY not set");
            return;
        }

        eprintln!("Running: OPENROUTER_API_KEY set");

        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        // Wait for initial load
        wait_for_status_ready(&page).await;

        // Count initial log entries by type
        let initial_log_state = get_story_log_summary(&page).await;
        println!("Initial story log: {:?}", initial_log_state);

        // Submit a FreeAction that requires LLM narration
        send_action(&page, "look around and describe what you see").await;

        // Verify status transitions to Thinking
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
            "Status should show 'Thinking...' during LLM generation. Got: '{}'",
            status_during
        );

        // Wait for LLM to complete (narration + trigger narration)
        let llm_result = wait_for_llm_idle(port, Duration::from_secs(180)).await;

        // Poll story log until narration count increases (HTMX may need a poll cycle to catch up)
        let final_log_state = wait_for_narration_increase(
            &page,
            initial_log_state.narration_count,
            Duration::from_secs(10),
        )
        .await;
        println!("Final story log: {:?}", final_log_state);

        // Get error message if any
        let error_msg: String = page
            .evaluate::<(), String>(
                "document.querySelector('#error-message')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        // Verify narration was added
        let narration_added = final_log_state.total_entries > initial_log_state.total_entries
            || final_log_state.narration_count > initial_log_state.narration_count;

        if !narration_added && llm_result.is_err() {
            panic!(
                "LLM narration failed: no new entries added after 180s timeout. \
                 Initial: {:?}, Final: {:?}, Error: '{}', LLM idle: Err",
                initial_log_state, final_log_state, error_msg
            );
        }

        if !narration_added && !error_msg.is_empty() {
            panic!(
                "LLM narration failed with error. Initial: {:?}, Final: {:?}, Error: '{}'",
                initial_log_state, final_log_state, error_msg
            );
        }

        assert!(
            narration_added,
            "LLM should have added narration entries. Initial: {:?}, Final: {:?}, LLM idle: {:?}",
            initial_log_state, final_log_state, llm_result
        );

        // Verify at least one Narration-type entry was added (the LLM response)
        assert!(
            final_log_state.narration_count > initial_log_state.narration_count,
            "Expected at least one Narration-type entry from LLM. Initial narration: {}, Final: {}",
            initial_log_state.narration_count,
            final_log_state.narration_count
        );

        // Verify Input entry was logged
        assert!(
            final_log_state.input_count > initial_log_state.input_count,
            "Expected an Input entry for the command. Initial input: {}, Final: {}",
            initial_log_state.input_count,
            final_log_state.input_count
        );

        // Verify status returned to Ready or Error
        let status_after: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();
        println!("Status after LLM: {}", status_after);

        if !status_after.contains("Ready") && !status_after.contains("Error") {
            eprintln!(
                "Status not Ready/Error after 180s. LLM idle result: {:?}. \
                 Story log: {:?}. Error UI: '{}'",
                llm_result, final_log_state, error_msg
            );
        }

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_llm_handles_arrival_narration() {
        dotenv::dotenv().ok();

        if !has_llm_api_key() {
            eprintln!("Skipping: OPENROUTER_API_KEY not set");
            return;
        }

        eprintln!("Running: OPENROUTER_API_KEY set");

        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let initial_status: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();
        println!("Initial status: {}", initial_status);

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

        let status_during: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();
        println!("Status during: {}", status_during);

        let llm_result = wait_for_llm_idle(port, Duration::from_secs(180)).await;
        if llm_result.is_err() {
            let status: String = page
                .evaluate::<(), String>(
                    "document.querySelector('#status-display')?.innerText || 'no status'",
                    None,
                )
                .await
                .unwrap_or_default();
            println!("Warning: LLM did not become idle within timeout. Status: '{status}'");
        }

        let _final_messages = wait_for_more_messages(&page, 0).await;

        let status_after = wait_for_status_not_thinking(&page).await;

        println!(
            "Status after (LLM result: {:?}): {}",
            llm_result, status_after
        );

        assert!(
            !status_after.contains("Thinking")
                || status_after.contains("Ready")
                || status_after.contains("Error"),
            "Status should not be stuck on 'Thinking...'. Got: {} (LLM result: {:?})",
            status_after,
            llm_result
        );

        let _ = browser.close().await;
    }

    // Helper Functions

    /// Helper: Send an action via the form
    async fn send_action(page: &playwright_rs::Page, text: &str) {
        let text_owned = text.to_string();
        let _: Result<(), _> = page
            .evaluate(
                r#"
            (text) => {
                const input = document.querySelector('#command-form input[name="command"]');
                if (input) {
                    input.value = text;
                    input.form?.requestSubmit();
                }
            }
            "#,
                Some(&text_owned),
            )
            .await;
    }

    /// Story log summary for test assertions
    #[derive(Debug)]
    struct StoryLogSummary {
        total_entries: usize,
        narration_count: usize,
        dialogue_count: usize,
        system_count: usize,
        input_count: usize,
    }

    /// Helper: Get a summary of story log entries by type
    async fn get_story_log_summary(page: &playwright_rs::Page) -> StoryLogSummary {
        let result: serde_json::Value = page
            .evaluate::<(), serde_json::Value>(
                r#"
            (() => {
                const entries = document.querySelectorAll('#story-log .log-entry');
                let counts = { total: entries.length, narration: 0, dialogue: 0, system: 0, input: 0 };
                entries.forEach(entry => {
                    if (entry.classList.contains('narration')) counts.narration++;
                    else if (entry.classList.contains('dialogue')) counts.dialogue++;
                    else if (entry.classList.contains('system')) counts.system++;
                    else if (entry.classList.contains('input')) counts.input++;
                });
                return JSON.stringify(counts);
            })()
            "#,
                None,
            )
            .await
            .unwrap_or(serde_json::json!({"total": 0, "narration": 0, "dialogue": 0, "system": 0, "input": 0}));

        let counts: serde_json::Value = if result.is_string() {
            serde_json::from_str(result.as_str().unwrap_or("{}")).unwrap_or_default()
        } else {
            result
        };

        StoryLogSummary {
            total_entries: counts.get("total").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            narration_count: counts
                .get("narration")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize,
            dialogue_count: counts.get("dialogue").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            system_count: counts.get("system").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            input_count: counts.get("input").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        }
    }

    /// Helper: Poll story log until narration count exceeds the initial value
    /// Waits up to `timeout` for HTMX polling to catch up
    async fn wait_for_narration_increase(
        page: &playwright_rs::Page,
        initial_narration: usize,
        timeout: Duration,
    ) -> StoryLogSummary {
        let start = std::time::Instant::now();
        loop {
            let summary = get_story_log_summary(page).await;
            if summary.narration_count > initial_narration {
                return summary;
            }
            if start.elapsed() > timeout {
                return summary;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
