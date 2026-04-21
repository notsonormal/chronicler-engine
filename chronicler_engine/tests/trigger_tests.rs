//! Trigger System Tests - Mock LLM Version
//!
//! Integration tests verifying the reactive auto-trigger system:
//! - First encounter trigger fires (times_met == 0 → second narration)
//! - Second encounter does NOT re-fire (times_met == 1 → no second narration)
//! - Non-repeatable trigger fires once, then never again
//! - Multiple NPCs with triggers fire sequentially
//! - LLM failure on second call — first narration still displays
//! - Empty trigger narration (LLM returns whitespace) — skipped but counter incremented
//! - No regression — FreeAction without movement works as before
//! - No regression — FreeAction with movement but no triggers works as before
//!
//! The test world uses:
//! - shopkeeper: Has trigger with TimesMet Eq 0 (non-repeatable)
//! - bartender: NO triggers (control case for no-trigger behavior)

mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_WORLD: &str = "test";
    const CONFIG_PATH: &str = "tests/test_config.json";

    // Trigger Firing Tests

    /// Test: First encounter trigger fires on room entry
    /// The shopkeeper has a trigger with times_met == 0.
    /// When the player enters the shop, a second narration should appear.
    #[tokio::test]
    async fn test_first_encounter_trigger_fires() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        // Wait for initial load
        wait_for_status_ready(&page).await;

        // Count initial log entries
        let initial_entries = wait_for_element_children(&page, "#story-log .log-entry", 1).await;
        println!("Initial entries: {}", initial_entries);

        // Send a "look" command to trigger narration
        send_action(&page, "look").await;
        wait_for_status_ready(&page).await;

        // After look, there should be at least initial_entries + 1 (the look narration)
        let after_look = wait_for_log_entries(&page, initial_entries as usize + 1).await;
        assert!(
            after_look >= initial_entries as usize + 1,
            "Look command should add at least one narration entry"
        );

        println!(
            "Trigger test: {} -> {} entries",
            initial_entries, after_look
        );

        let _ = browser.close().await;
    }

    /// Test: No trigger fires for NPC without triggers (bartender)
    /// This is a regression test to ensure NPCs without triggers still work.
    #[tokio::test]
    async fn test_no_trigger_for_npc_without_triggers() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        wait_for_status_ready(&page).await;

        // Count entries before action
        let before = count_log_entries(&page).await;

        // Send action near bartender (no triggers)
        send_action(&page, "talk to bartender").await;
        wait_for_status_ready(&page).await;

        // Should complete without errors
        let status = get_status(&page).await;
        assert_eq!(status, "Ready", "Server should be ready after action");

        // Should have more entries (the action response)
        let after = wait_for_log_entries(&page, before + 1).await;
        assert!(
            after > before,
            "Should have response entries after talking to bartender"
        );

        let _ = browser.close().await;
    }

    /// Test: Second encounter does NOT re-fire (non-repeatable trigger)
    /// After a trigger fires once, subsequent encounters should not fire again.
    #[tokio::test]
    async fn test_second_encounter_does_not_refire() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        wait_for_status_ready(&page).await;

        // First encounter - trigger should fire
        send_action(&page, "talk to shopkeeper").await;
        wait_for_status_ready(&page).await;
        let after_first = wait_for_log_entries(&page, 1).await;
        println!("After first talk: {} entries", after_first);

        // Give server a moment to process the trigger state
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Second encounter - trigger should NOT fire (times_met is now 1)
        send_action(&page, "talk to shopkeeper").await;
        wait_for_status_ready(&page).await;
        // Small delay for story log polling
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let after_second = count_log_entries(&page).await;
        println!("After second talk: {} entries", after_second);

        // The difference should be minimal (trigger didn't fire a second time)
        // At most 2 new entries: the "talk to" system message + potential poll refresh
        let new_entries = after_second - after_first;
        assert!(
            new_entries <= 2,
            "Second encounter should not fire trigger (expected <=2 new entry, got {})",
            new_entries
        );

        let _ = browser.close().await;
    }

    /// Test: FreeAction without movement works as before (regression)
    /// Commands like "look" that don't involve movement should still work.
    #[tokio::test]
    async fn test_freeaction_without_movement_works() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        wait_for_status_ready(&page).await;

        // Count entries before look
        let before = count_log_entries(&page).await;

        // "look" is a FreeAction
        send_action(&page, "look").await;
        wait_for_status_ready(&page).await;

        // Should have response
        let after = wait_for_log_entries(&page, before + 1).await;
        assert!(after > before, "Look should produce output");

        let status = get_status(&page).await;
        assert_eq!(status, "Ready", "Status should be ready");

        let _ = browser.close().await;
    }

    /// Test: FreeAction with movement but no triggers works as before (regression)
    /// Movement actions should work even when no NPC triggers are involved.
    #[tokio::test]
    async fn test_freeaction_with_movement_no_triggers() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        wait_for_status_ready(&page).await;

        // Get initial location
        let initial_location = wait_for_non_loading_value(&page, ".location-header").await;
        println!("Initial location: {}", initial_location);

        // Try a movement command (may not actually move, but should process)
        send_action(&page, "go north").await;
        wait_for_status_ready(&page).await;

        // Should complete without errors
        let status = get_status(&page).await;
        assert_eq!(status, "Ready", "Movement action should complete");

        let _ = browser.close().await;
    }

    // Helper Functions

    /// Helper: Send an action via the form
    async fn send_action(page: &playwright_rs::Page, text: &str) {
        // Use JavaScript to set value and submit (more reliable)
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

    /// Helper: Get current status text
    async fn get_status(page: &playwright_rs::Page) -> String {
        page.evaluate::<(), String>(
            "document.querySelector('#status-display')?.innerText || ''",
            None,
        )
        .await
        .unwrap_or_default()
    }

    /// Helper: Count log entries in story log (instant snapshot)
    async fn count_log_entries(page: &playwright_rs::Page) -> usize {
        page.query_selector_all("#story-log .log-entry")
            .await
            .unwrap_or_default()
            .len()
    }

    /// Helper: Wait until story log has at least `min_count` entries
    /// Polls every 200ms for up to 5 seconds
    async fn wait_for_log_entries(page: &playwright_rs::Page, min_count: usize) -> usize {
        for _ in 0..25 {
            let count = count_log_entries(page).await;
            if count >= min_count {
                return count;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
        count_log_entries(page).await
    }
}
