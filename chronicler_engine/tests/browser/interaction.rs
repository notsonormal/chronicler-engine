use super::*;

const TEST_WORLD: &str = "test";
const CONFIG_PATH: &str = "tests/test_config.json";

#[tokio::test]
async fn test_form_submission() {
    let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
    let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

    let (_playwright, browser) = launch_chrome().await;
    let page = browser.new_page().await.unwrap();

    goto_with_connection_check(&page, port)
        .await
        .expect("Failed to connect to server");

    let initial_entries = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

    // Submit a command
    page.evaluate::<(), ()>(
        "(() => { 
            const input = document.querySelector('#command-form input');
            if (input) input.value = 'hello';
            const form = document.querySelector('#command-form');
            if (form) form.requestSubmit();
        })()",
        None,
    )
    .await
    .unwrap();

    // Wait for completion and verify new entries were added
    wait_for_status_ready(&page).await;
    let after_entries =
        wait_for_element_children(&page, "#story-log .log-entry", initial_entries + 1).await;
    assert!(
        after_entries > initial_entries,
        "Form submission should add log entries: {initial_entries} -> {after_entries}"
    );

    let _ = browser.close().await;
}
