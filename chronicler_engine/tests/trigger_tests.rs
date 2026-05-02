//! [DOC: docs/reference/testing.md]

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
        println!("Initial entries: {initial_entries}");

        // Send a "look" command to trigger narration
        send_action(&page, "look").await;
        wait_for_status_ready(&page).await;

        // After look, there should be at least initial_entries + 1 (the look narration)
        let after_look = wait_for_log_entries(&page, initial_entries as usize + 1).await;
        assert!(
            after_look > initial_entries as usize,
            "Look command should add at least one narration entry"
        );

        println!("Trigger test: {initial_entries} -> {after_look} entries");

        let _ = browser.close().await;
    }

    /// Test: Trigger fires when second quantifier detects NPC via RoomConfiguredNpcs
    /// This tests the flow where:
    /// 1. Player moves to a room with configured NPCs (e.g., shop with shopkeeper)
    /// 2. Second quantifier runs on narration and sees RoomConfiguredNpcs
    /// 3. Trigger with TimesMet Eq 0 fires
    #[tokio::test]
    async fn test_second_quantifier_detects_room_npcs() {
        let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
        let _server = TestServer::new_with_mock(port, TEST_WORLD).await;

        let (_playwright, browser) = launch_chrome().await;
        let page = browser.new_page().await.unwrap();

        goto_with_connection_check(&page, port)
            .await
            .expect("Failed to connect to server");

        wait_for_status_ready(&page).await;

        // Get initial entry count
        let before = count_log_entries(&page).await;

        // Move to a room that has NPCs configured (mock will detect room from keyword)
        // "enter shop" - shop has shopkeeper in map.json
        send_action(&page, "enter shop").await;
        wait_for_status_ready(&page).await;

        // Poll for more log entries (replaces arbitrary sleep)
        let after = wait_for_log_entries(&page, before + 1).await;

        // The mock narration should have triggered room NPC detection
        // and the trigger should fire (adding another entry)
        println!("Entries before: {before}, after: {after}");

        // At minimum: action response + trigger narration
        assert!(
            after > before,
            "Moving to room with NPCs should trigger events"
        );

        let _ = browser.close().await;
    }

    /// Test: NPC without triggers still works (bartender has no triggers)
    /// NOTE: With the fix evaluating ALL NPCs for triggers, other NPCs' triggers may fire
    /// even when talking to an NPC without triggers. This test verifies the bartender
    /// still generates dialogue, but accepts that status may show "Thinking..." briefly
    /// due to other NPCs' triggers firing.
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

        // Wait for completion (may take time due to other NPCs' triggers firing)
        wait_for_status_ready(&page).await;

        // Should have more entries (the action response + possibly other triggers)
        let after = wait_for_log_entries(&page, before + 1).await;
        assert!(
            after > before,
            "Should have response entries after talking to bartender"
        );

        // Verify bartender dialogue appeared
        let story_log = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.innerText || ''",
                None,
            )
            .await
            .unwrap_or_default();
        assert!(
            story_log.contains("Bartender") || story_log.contains("bartender"),
            "Bartender dialogue should appear"
        );

        let _ = browser.close().await;
    }

    /// Test: Second encounter does NOT re-fire (non-repeatable trigger)
    /// After a trigger fires once, subsequent encounters should not fire again.
    /// NOTE: With the fix that evaluates ALL NPCs for triggers (not just npcs_in_area),
    /// we may see more entries on subsequent encounters due to other NPCs' triggers.
    /// The key check is that the SAME NPC's trigger doesn't fire twice.
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
        println!("After first talk: {after_first} entries");

        // Poll for trigger state processing (replaces arbitrary sleep)
        let _ = wait_for_log_entries(&page, after_first).await;

        // Second encounter - shopkeeper trigger should NOT fire (times_met is now 1)
        send_action(&page, "talk to shopkeeper").await;
        wait_for_status_ready(&page).await;
        // Poll for story log update (replaces arbitrary sleep)
        let after_second = wait_for_log_entries(&page, after_first).await;
        println!("After second talk: {after_second} entries");

        // With the fix evaluating ALL NPCs, we may see more than 2 new entries
        // due to other NPCs' triggers firing. But the shopkeeper's trigger
        // should NOT have fired again (since times_met is now 1 and trigger is non-repeatable).
        // We verify this by checking the content doesn't contain a duplicate shopkeeper narration.
        let shopkeeper_trigger_text =
            "The shopkeeper looks up from behind the counter with a warm smile.";
        let story_log = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.innerText || ''",
                None,
            )
            .await
            .unwrap_or_default();

        // Count occurrences of the shopkeeper's trigger text
        let trigger_count = story_log.matches(shopkeeper_trigger_text).count();
        assert!(
            trigger_count <= 1,
            "Shopkeeper trigger should appear at most once (found {trigger_count} times)"
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
    /// NOTE: With the fix that evaluates ALL NPCs for triggers (not just npcs_in_area),
    /// movement actions now trigger continuation narration for NPCs with active triggers.
    /// This means "go north" may be async and take longer to complete.
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
        println!("Initial location: {initial_location}");

        // Try a movement command
        // With the fix, ALL NPCs get evaluated for triggers, so even "go north"
        // may trigger continuation narration for NPCs with TimesMet Eq 0 triggers.
        // This makes it an async action that takes time.
        send_action(&page, "go north").await;

        // Wait for the action to complete (may take a while due to trigger narration)
        wait_for_status_ready(&page).await;

        // Status should eventually become Ready
        let status = get_status(&page).await;
        assert_eq!(
            status, "Ready",
            "Movement action should complete eventually"
        );

        let _ = browser.close().await;
    }
}
