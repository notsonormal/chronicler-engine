//! [DOC: docs/reference/testing.md]

mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    const TEST_WORLD: &str = "test";
    const CONFIG_PATH: &str = "tests/test_config.json";

    fn has_llm_api_key() -> bool {
        std::env::var("OPENROUTER_API_KEY").is_ok()
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
        let _server = TestServer::new(port, TEST_WORLD).await;

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

            println!("Smoke test: LLM idle={llm_result:?}, status='{status_after}'");

            assert!(
                status_after.contains("Ready") || status_after.contains("Error"),
                "Status should be Ready or Error after LLM completes. Got: {status_after}"
            );
        })
        .await;
    }

    /// Multi-step stability: verify the server remains healthy after two sequential
    /// real LLM calls. This is the only test that exercises real async latency
    /// across multiple generation cycles.
    #[ignore = "slow: requires OPENROUTER_API_KEY"]
    #[tokio::test]
    async fn test_real_llm_multi_step_stability() {
        with_real_llm(|page, port| async move {
            // First command
            send_action(&page, "look around the room").await;
            let result_a = wait_for_llm_idle(port, Duration::from_secs(180)).await;
            let status_a = wait_for_status_ready_or_error(&page).await;
            println!("Step 1: idle={result_a:?}, status='{status_a}'");

            // Second command
            send_action(&page, "describe what you see in detail").await;
            let result_b = wait_for_llm_idle(port, Duration::from_secs(180)).await;
            let status_b = wait_for_status_ready_or_error(&page).await;
            println!("Step 2: idle={result_b:?}, status='{status_b}'");

            // Verify both completed and server is still responsive
            assert!(
                status_a.contains("Ready") || status_a.contains("Error"),
                "Step 1 should complete. Got: {status_a}"
            );
            assert!(
                status_b.contains("Ready") || status_b.contains("Error"),
                "Step 2 should complete. Got: {status_b}"
            );

            // Verify we can still interact with the server
            let entries = count_log_entries(&page).await;
            assert!(
                entries >= 2,
                "Should have at least 2 log entries after two commands"
            );
        })
        .await;
    }
}
