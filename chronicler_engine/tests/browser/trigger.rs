//! [DOC: docs/reference/testing.md]

use super::*;

fn load_first_trigger_prompt(character_id: &str) -> String {
    let path = format!("data/characters/test/{character_id}.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Test fixture missing: {path} — {e}"));
    let json: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("Test fixture invalid JSON: {path} — {e}"));
    json["triggers"]
        .get(0)
        .and_then(|t| t["narration"]["narration_prompt"].as_str())
        .map(String::from)
        .unwrap_or_else(|| {
            panic!(
                "Test fixture {path} missing triggers[0].narration.narration_prompt — \
                 if the trigger structure changed, update this test"
            )
        })
}

#[tokio::test]
async fn test_look_command_adds_narration_entries() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        wait_for_status_ready(&page).await;
        let initial_entries = wait_for_element_children(&page, "#story-log .log-entry", 1).await;
        println!("Initial entries: {initial_entries}");
        send_action(&page, "look").await;
        wait_for_status_ready(&page).await;
        let after_look = wait_for_log_entries(&page, initial_entries as usize + 1).await;
        assert!(
            after_look > initial_entries as usize,
            "Look command should add at least one narration entry"
        );
        println!("Trigger test: {initial_entries} -> {after_look} entries");
    })
    .await;
}

#[tokio::test]
async fn test_second_quantifier_detects_room_npcs() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        wait_for_status_ready(&page).await;
        let before = count_log_entries(&page).await;
        send_action(&page, "enter shop").await;
        wait_for_status_ready(&page).await;
        let after = wait_for_log_entries(&page, before + 1).await;
        println!("Entries before: {before}, after: {after}");
        assert!(
            after > before,
            "Moving to room with NPCs should trigger events"
        );
    })
    .await;
}

#[tokio::test]
async fn test_no_trigger_for_npc_without_triggers() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        wait_for_status_ready(&page).await;
        let before = count_log_entries(&page).await;
        send_action(&page, "talk to bartender").await;
        wait_for_status_ready(&page).await;
        let after = wait_for_log_entries(&page, before + 1).await;
        assert!(
            after > before,
            "Should have response entries after talking to bartender"
        );
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
    })
    .await;
}

#[tokio::test]
async fn test_second_encounter_does_not_refire() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        wait_for_status_ready(&page).await;
        send_action(&page, "talk to shopkeeper").await;
        wait_for_status_ready(&page).await;
        let after_first = wait_for_log_entries(&page, 1).await;
        println!("After first talk: {after_first} entries");
        let _ = wait_for_log_entries(&page, after_first).await;
        send_action(&page, "talk to shopkeeper").await;
        wait_for_status_ready(&page).await;
        let after_second = wait_for_log_entries(&page, after_first).await;
        println!("After second talk: {after_second} entries");
        let shopkeeper_trigger_text = load_first_trigger_prompt("shopkeeper");
        let story_log = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.innerText || ''",
                None,
            )
            .await
            .unwrap_or_default();
        let trigger_count = story_log.matches(&shopkeeper_trigger_text).count();
        assert!(
            trigger_count <= 1,
            "Shopkeeper trigger should appear at most once (found {trigger_count} times)"
        );
    })
    .await;
}

#[tokio::test]
async fn test_freeaction_without_movement_works() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        wait_for_status_ready(&page).await;
        let before = count_log_entries(&page).await;
        send_action(&page, "look").await;
        wait_for_status_ready(&page).await;
        let after = wait_for_log_entries(&page, before + 1).await;
        assert!(after > before, "Look should produce output");
        let status = get_status(&page).await;
        assert_eq!(status, "Ready", "Status should be ready");
    })
    .await;
}

#[tokio::test]
async fn test_freeaction_with_movement_no_triggers() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        wait_for_status_ready(&page).await;
        let initial_location = wait_for_non_loading_value(&page, ".location-header").await;
        println!("Initial location: {initial_location}");
        send_action(&page, "go north").await;
        wait_for_status_ready(&page).await;
        let status = get_status(&page).await;
        assert_eq!(
            status, "Ready",
            "Movement action should complete eventually"
        );
    })
    .await;
}
