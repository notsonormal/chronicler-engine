//! [DOC: docs/reference/testing.md]

mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    // Use config-based server - no hardcoded port
    const TEST_WORLD: &str = "test";
    const CONFIG_PATH: &str = "tests/test_config.json";

    // Initial Load Tests - No LLM needed

    #[tokio::test]
    async fn test_initial_load_story_log_displays_location() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        // Wait for story log entries
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        // The location is rendered inside the story log as a location-header entry
        let location = wait_for_non_loading_value(&page, ".location-header").await;

        println!("Initial location: {location}");

        assert!(
            !location.is_empty(),
            "Story log should display current location"
        );

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_initial_load_story_log_has_content() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        // Poll until story log has entries
        let entries = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        println!("Initial story-log entries: {entries}");
        assert!(entries > 0, "Story log should have entries on initial load");

        // Wait for completion
        wait_for_status_ready(&page).await;

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_initial_load_status_is_ready() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        // Wait for status to be stable (ready, not thinking)
        wait_for_status_ready(&page).await;
        let status = wait_for_element_text(&page, "#status-display").await;

        println!("Initial status: {status}");
        assert!(status.contains("Ready"), "Status should show 'Ready'");

        let _ = browser.close().await;
    }

    // Command Submission Tests - Status updates, no LLM needed

    #[tokio::test]
    async fn test_look_command_completes_to_ready() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

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

        // Look is processed synchronously — wait for Ready then verify
        wait_for_status_ready(&page).await;
        let status: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap();
        assert!(
            status.contains("Ready"),
            "Status should be Ready after look command: {status}"
        );

        // A new narration entry should have been added
        let entries = wait_for_log_entries(&page, 1).await;
        assert!(entries >= 1, "Look command should add narration entries");

        let _ = browser.close().await;
    }

    #[tokio::test]
    async fn test_double_submit_protection() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{port}/action/check");

        // Submit two commands in rapid succession
        let req1 = client
            .post(&url)
            .form(&[("command", "alpha-beta-unique-1")])
            .send();
        let req2 = client
            .post(&url)
            .form(&[("command", "gamma-delta-unique-2")])
            .send();

        let (res1, res2) = tokio::join!(req1, req2);
        assert!(res1.is_ok(), "First request should succeed");
        assert!(res2.is_ok(), "Second request should succeed");

        // Poll story log until both commands appear (targeted wait, max ~3s)
        let mut story = String::new();
        for _ in 0..30 {
            story = client
                .get(format!("http://127.0.0.1:{port}/fragment/story-log"))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            if story.contains("alpha-beta-unique-1") && story.contains("gamma-delta-unique-2") {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        println!("Story log: {story}");

        let has_first = story.contains("alpha-beta-unique-1");
        let has_second = story.contains("gamma-delta-unique-2");

        assert!(has_first, "First command should appear in story log");
        // Without mutex, both requests are accepted; server processes both
        assert!(has_second, "Second command should also appear in story log");
    }
}
