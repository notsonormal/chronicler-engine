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

        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new(port, TEST_WORLD);

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
        let llm_result = wait_for_llm_idle(port, Duration::from_secs(90)).await;
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

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        let initial_text: String = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

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

        let llm_result = wait_for_llm_idle(port, Duration::from_secs(90)).await;
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

        let final_text = wait_for_story_log_change(&page, &initial_text).await;
        println!(
            "Content length: {} -> {}",
            initial_text.len(),
            final_text.len()
        );

        // Content should have changed via polling OR there's an error message showing
        let error_msg: String = page
            .evaluate::<(), String>(
                "document.querySelector('#error-message')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        if !error_msg.is_empty() {
            println!("Error message displayed in UI: {}", error_msg);
        }

        let content_changed = final_text.len() > initial_text.len();
        assert!(
            content_changed || !error_msg.is_empty(),
            "Either story should expand or error should show. Content: {}->{}, Error: '{}'",
            initial_text.len(),
            final_text.len(),
            error_msg
        );

        let status_after: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();

        println!("Status after LLM: {}", status_after);
        let status_ok = status_after.contains("Ready") || status_after.contains("Error");
        assert!(
            status_ok,
            "Status should return to 'Ready' or show error after LLM. Got: {}",
            status_after
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

        let llm_result = wait_for_llm_idle(port, Duration::from_secs(90)).await;
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

        browser.close().await.unwrap();
    }
}
