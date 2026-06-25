use std::time::Duration;

use playwright_rs::expect;

use crate::test_utils::browser::{
    count_log_entries, element_count, send_action, wait_for_element_children,
    wait_for_status_ready, with_test_page,
};
use crate::test_utils::wait::{
    wait_for_element_exists, wait_for_element_not_exists, wait_for_element_persist,
};

const CONFIG_PATH: &str = "tests/test_config.json";
const TEST_WORLD: &str = "test";
const TEST_PERSONA: &str = "test_player";

#[tokio::test]
async fn test_edit_button_exists_on_entries() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let edit_buttons = element_count(&page, ".log-entry .edit-btn").await;
            assert!(
                edit_buttons > 0,
                "Edit buttons should exist on story entries"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_delete_button_exists_on_entries() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let initial_entries = element_count(&page, "#story-log .log-entry").await;
            send_action(&page, "hello").await;
            wait_for_status_ready(&page).await;
            wait_for_element_children(&page, "#story-log .log-entry", initial_entries as u32 + 1)
                .await;

            let delete_buttons = element_count(&page, ".log-entry .delete-btn").await;
            assert!(
                delete_buttons > 0,
                "Delete buttons should exist on story entries"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_edit_mode_activates_on_click() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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
        },
    )
    .await;
}
#[tokio::test]
async fn test_edit_cancel_restores_original() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let original_text = page
                .locator(".log-entry .text")
                .await
                .inner_text()
                .await
                .unwrap_or_default();
            assert!(!original_text.is_empty(), "Should have original text");

            page.locator(".edit-btn").await.click(None).await.unwrap();
            wait_for_element_exists(&page, "#edit-textarea", 10).await;

            let modified = "Modified text for testing";
            page.locator("#edit-textarea")
                .await
                .fill(modified, None)
                .await
                .unwrap();

            page.locator(".cancel-btn").await.click(None).await.unwrap();
            wait_for_element_not_exists(&page, "#edit-textarea", 10).await;

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
        },
    )
    .await;
}

#[tokio::test]
async fn test_polling_pauses_during_edit() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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

            let persisted =
                wait_for_element_persist(&page, "#edit-textarea", Duration::from_secs(3)).await;
            assert!(
                persisted,
                "Edit textarea should persist during polling pause"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_delete_removes_message() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
        let initial = count_log_entries(&page).await;
        if initial < 2 {
            send_action(&page, "look").await;
            wait_for_status_ready(&page).await;
        }

        let count_before_delete = count_log_entries(&page).await;
        assert!(
            count_before_delete >= 2,
            "Need at least 2 entries for delete button, have {count_before_delete}"
        );


        page.evaluate::<(), ()>(
            r#"(() => {
                window.confirm = () => true;
            })()"#,
            None,
        )
        .await
        .unwrap();


        page.locator(".delete-btn").await.click(None).await.unwrap();


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
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let initial_entries = element_count(&page, "#story-log .log-entry").await;
            send_action(&page, "look").await;
            wait_for_status_ready(&page).await;
            wait_for_element_children(&page, "#story-log .log-entry", initial_entries as u32 + 2)
                .await;

            let retry_buttons = element_count(&page, ".retry-btn").await;
            assert_eq!(
                retry_buttons, 0,
                "Should not have retry button — swipe controls replace it"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_edit_textarea_matches_original_height() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
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

/// Test that delete button appears only on the last entry.
#[tokio::test]
async fn test_delete_button_only_on_last_entry() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
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

/// Test status display updates during generation.
#[tokio::test]
async fn test_status_updates_during_generation() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            send_action(&page, "wait").await;

            let status_locator = page.locator("#status-display").await;
            let _ = expect(status_locator)
                .with_timeout(Duration::from_millis(500))
                .not()
                .to_contain_text("Ready")
                .await;

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
        },
    )
    .await;
}
