use std::time::Duration;

use playwright_rs::expect;

use crate::test_utils::browser::{
    count_log_entries, element_count, send_action, wait_for_element_children,
    wait_for_status_ready, with_test_page,
};
use crate::test_utils::wait::{wait_for_element_exists, wait_for_element_not_exists};

const CONFIG_PATH: &str = "tests/test_config.json";
const TEST_WORLD: &str = "test";

#[tokio::test]
async fn test_edit_button_exists_on_entries() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let edit_buttons = element_count(&page, ".log-entry .edit-btn").await;
        assert!(
            edit_buttons > 0,
            "Edit buttons should exist on story entries"
        );
    })
    .await;
}

#[tokio::test]
async fn test_delete_button_exists_on_entries() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let initial_entries = element_count(&page, "#story-log .log-entry").await;
        send_action(&page, "hello").await;
        wait_for_status_ready(&page).await;
        wait_for_element_children(&page, "#story-log .log-entry", initial_entries as u32 + 1).await;

        let delete_buttons = element_count(&page, ".log-entry .delete-btn").await;
        assert!(
            delete_buttons > 0,
            "Delete buttons should exist on story entries"
        );
    })
    .await;
}

#[tokio::test]
async fn test_edit_mode_activates_on_click() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let clicked = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const btn = document.querySelector('.edit-btn');
                    if (btn) {
                        btn.click();
                        return true;
                    }
                    return false;
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(clicked, "Should find and click an edit button");

        wait_for_element_exists(&page, "#edit-textarea", 10).await;
        // wait_for_element_exists panics if element doesn't appear, so success = element exists
    })
    .await;
}
#[tokio::test]
async fn test_edit_cancel_restores_original() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        // Get original text using locator.inner_text() (avoids playwright-rs JSON issues)
        let original_text = page
            .locator(".log-entry .text")
            .await
            .inner_text()
            .await
            .unwrap_or_default();
        assert!(!original_text.is_empty(), "Should have original text");

        // Click edit button
        page.locator(".edit-btn").await.click(None).await.unwrap();
        wait_for_element_exists(&page, "#edit-textarea", 10).await;

        // Modify text
        let modified = "Modified text for testing";
        page.locator("#edit-textarea")
            .await
            .fill(modified, None)
            .await
            .unwrap();

        // Click Cancel button
        page.locator(".cancel-btn").await.click(None).await.unwrap();
        wait_for_element_not_exists(&page, "#edit-textarea", 10).await;

        // Verify text is restored
        let restored = page
            .locator(".log-entry .text")
            .await
            .inner_text()
            .await
            .unwrap_or_default();

        assert_eq!(
            restored, original_text,
            "Text should be restored to original after cancel"
        );
    })
    .await;
}

#[tokio::test]
async fn test_polling_pauses_during_edit() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        page.evaluate::<(), bool>(
            r#"(() => {
                const btn = document.querySelector('.edit-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        wait_for_element_exists(&page, "#edit-textarea", 10).await;

        tokio::time::sleep(Duration::from_secs(3)).await;

        let edit_exists = element_count(&page, "#edit-textarea").await;
        assert!(
            edit_exists > 0,
            "Edit textarea should persist during polling pause"
        );
    })
    .await;
}

/// Deleting a message sends a POST to /history/delete after the native confirm dialog.
/// Playwright auto-accepts native confirm() dialogs, so we just click and wait for the count to decrease.
#[tokio::test]
async fn test_delete_removes_message() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let initial = count_log_entries(&page).await;
        // Need at least 2 entries for delete button to appear (per dashboard.md: "last entry only, hidden when only one entry exists")
        if initial < 2 {
            // Generate more entries
            send_action(&page, "look").await;
            wait_for_status_ready(&page).await;
        }

        let count_before_delete = count_log_entries(&page).await;
        assert!(
            count_before_delete >= 2,
            "Need at least 2 entries for delete button, have {count_before_delete}"
        );

        // Override native confirm() to always return true
        page.evaluate::<(), ()>(
            r#"(() => {
                window.confirm = () => true;
            })()"#,
            None,
        )
        .await
        .unwrap();

        // Click delete button on last entry
        page.locator(".delete-btn").await.click(None).await.unwrap();

        // Smart wait: poll until count decreases (HTMX swap complete)
        let mut attempts = 0;
        let max_attempts = 40; // 4 seconds at 100ms intervals
        let mut current_count = count_log_entries(&page).await;
        while current_count >= count_before_delete && attempts < max_attempts {
            tokio::time::sleep(Duration::from_millis(100)).await;
            current_count = count_log_entries(&page).await;
            attempts += 1;
        }

        assert!(
            current_count < count_before_delete,
            "Delete should remove the message (expected < {count_before_delete}, got {current_count} after {attempts} attempts)"
        );
    })
    .await;
}

#[tokio::test]
async fn test_no_retry_button_on_last_ai_message() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let initial_entries = element_count(&page, "#story-log .log-entry").await;
        send_action(&page, "look").await;
        wait_for_status_ready(&page).await;
        wait_for_element_children(&page, "#story-log .log-entry", initial_entries as u32 + 2).await;

        let retry_buttons = element_count(&page, ".retry-btn").await;
        assert_eq!(
            retry_buttons, 0,
            "Should not have retry button — swipe controls replace it"
        );
    })
    .await;
}

#[tokio::test]
async fn test_edit_textarea_matches_original_height() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let original_height: f64 = page
            .evaluate::<(), f64>(
                r#"(() => {
                    const text = document.querySelector('.log-entry .text');
                    if (!text) return -1;
                    const rect = text.getBoundingClientRect();
                    return rect.height;
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert!(
            original_height > 0.0,
            "Original text should have a valid height"
        );

        // Click edit
        page.evaluate::<(), bool>(
            r#"(() => {
                const entry = document.querySelector('.log-entry');
                const btn = entry?.querySelector('.edit-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        wait_for_element_exists(&page, "#edit-textarea", 10).await;

        let textarea_height: f64 = page
            .evaluate::<(), f64>(
                r#"(() => {
                    const textarea = document.querySelector('#edit-textarea');
                    if (!textarea) return -1;
                    // Force reflow to ensure styles are applied
                    void textarea.offsetHeight;
                    const rect = textarea.getBoundingClientRect();
                    return rect.height;
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert!(textarea_height > 0.0, "Textarea should have a valid height");


        assert!(
            textarea_height >= original_height,
            "Textarea height ({textarea_height}) should not be smaller than original text height ({original_height})"
        );
        assert!(
            textarea_height <= original_height * 2.0 + 20.0,
            "Textarea height ({textarea_height}) should not be drastically larger than original text height ({original_height})"
        );
    })
    .await;
}

// REMOVED: This test was incorrect. Per docs/system/dashboard.md line 38:
// "Edit button (✎) on all entries" — including input messages.
// Users SHOULD be able to edit their input commands, standard in chat interfaces.

/// Test that delete button appears only on the last entry.
#[tokio::test]
async fn test_delete_button_only_on_last_entry() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let initial_entries = element_count(&page, "#story-log .log-entry").await;
        send_action(&page, "first").await;
        wait_for_status_ready(&page).await;
        wait_for_element_children(&page, "#story-log .log-entry", initial_entries as u32 + 2).await;

        send_action(&page, "second").await;
        wait_for_status_ready(&page).await;
        let _total_entries = wait_for_element_children(&page, "#story-log .log-entry", initial_entries as u32 + 4).await;

        let delete_buttons = element_count(&page, ".log-entry .delete-btn").await;
        assert!(
            delete_buttons <= 2,
            "Should have at most 2 delete buttons (last narration + last input), found: {delete_buttons}"
        );
    })
    .await;
}

/// Test that the status display updates during generation.
/// NOTE: Edit/delete buttons on message entries remain visible during generation.
/// Only the submit button in the action area is disabled (see dashboard.md spec).
#[tokio::test]
async fn test_status_updates_during_generation() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        send_action(&page, "wait").await;

        // Verify status changes from "Ready" during generation
        let status_locator = page.locator("#status-display").await;
        let _ = expect(status_locator)
            .with_timeout(Duration::from_millis(500))
            .not()
            .to_contain_text("Ready")
            .await;

        // Status should show a generating state (Thinking, Narrating, etc.)
        let status_text = page
            .locator("#status-display")
            .await
            .inner_text()
            .await
            .unwrap_or_default();
        assert!(
            status_text.contains("Thinking")
                || status_text.contains("Narrating")
                || status_text.contains("Generating")
                || status_text.contains("Quantifying"),
            "Status should show generating state, got: {status_text}"
        );
    })
    .await;
}
