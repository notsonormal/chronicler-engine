//! LLM-driven flow tests: exercises real LLM provider flows end-to-end (ignored by default; run with `python build.py --llm-only`).

#[path = "../test_utils/mod.rs"]
mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    const TEST_WORLD: &str = "test";
    const TEST_PERSONA: &str = "test_player";
    const CONFIG_PATH: &str = "tests/test_config.json";

    fn has_llm_api_key() -> bool {
        std::env::var("OPENROUTER_API_KEY").is_ok()
    }

    /// Fetch the LLM Messages fragment with retries until messages appear or timeout.
    async fn verify_llm_messages_logged(port: u16) {
        let client = reqwest::Client::new();
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(30);

        while start.elapsed() < timeout {
            if let Ok(resp) = client
                .get(format!("http://127.0.0.1:{port}/fragment/llm-messages"))
                .send()
                .await
            {
                if let Ok(text) = resp.text().await {
                    if text.contains("llm-message-list") && !text.contains("No LLM messages yet") {
                        return;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        panic!("LLM messages were not logged within timeout");
    }

    async fn with_real_llm<F, Fut>(test_fn: F)
    where
        F: FnOnce(playwright_rs::Page, u16) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        dotenv::dotenv().ok();

        if !has_llm_api_key() {
            eprintln!("Skipping: OPENROUTER_API_KEY not set");
            return;
        }

        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new(port, TEST_WORLD, TEST_PERSONA).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        wait_for_status_ready(&page).await;

        test_fn(page, port).await;

        let _ = browser.close().await;
    }

    /// Smoke test: verify a single command reaches the real LLM and completes.
    #[ignore = "slow: requires OPENROUTER_API_KEY"]
    #[tokio::test]
    async fn test_real_llm_smoke() {
        with_real_llm(|page, port| async move {
            send_action(&page, "examine the surroundings").await;

            let llm_result = wait_for_llm_idle(port, Duration::from_secs(180)).await;
            let status_after = wait_for_status_ready_or_error(&page).await;

            println!(
                "Smoke test: LLM idle={llm_result:?}, status='{}'",
                status_after.text
            );

            assert!(
                status_after.is_terminal(),
                "Status should be Ready or show an error disclosure after LLM completes. Got: {}",
                status_after.text
            );

            verify_llm_messages_logged(port).await;
        })
        .await;
    }

    /// Multi-step stability: verify the server remains healthy after two sequential
    /// real LLM calls.
    #[ignore = "slow: requires OPENROUTER_API_KEY"]
    #[tokio::test]
    async fn test_real_llm_multi_step_stability() {
        with_real_llm(|page, port| async move {
            send_action(&page, "Look around the room").await;
            let result_a = wait_for_llm_idle(port, Duration::from_secs(180)).await;
            let status_a = wait_for_status_ready_or_error(&page).await;
            println!("Step 1: idle={result_a:?}, status='{}'", status_a.text);

            send_action(&page, "Describe what you see in detail").await;
            let result_b = wait_for_llm_idle(port, Duration::from_secs(180)).await;
            let status_b = wait_for_status_ready_or_error(&page).await;
            println!("Step 2: idle={result_b:?}, status='{}'", status_b.text);

            assert!(
                status_a.is_terminal(),
                "Step 1 should complete with Ready or an error disclosure. Got: {}",
                status_a.text
            );
            assert!(
                status_b.is_terminal(),
                "Step 2 should complete with Ready or an error disclosure. Got: {}",
                status_b.text
            );

            let entries = count_log_entries(&page).await;
            assert!(
                entries >= 2,
                "Should have at least 2 log entries after two commands"
            );

            verify_llm_messages_logged(port).await;
        })
        .await;
    }
}
