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
}
