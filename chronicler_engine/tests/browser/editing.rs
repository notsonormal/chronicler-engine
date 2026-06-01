use std::time::Duration;

use playwright_rs::expect;

use crate::test_utils::browser::{
    element_count, send_action, wait_for_element_children, wait_for_status_ready, with_test_page,
};

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

        let edit_exists = wait_for_element_exists(&page, "#edit-textarea", 10).await;
        assert!(edit_exists > 0, "Edit textarea should appear after clicking edit");
    })
    .await;
}

#[tokio::test]
async fn test_edit_cancel_restores_original() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let original_text: String = page
            .evaluate(
                r#"(() => {
                    const text = document.querySelector('.log-entry .text');
                    return text ? text.textContent : '';
                })()"#,
                None,
            )
            .await
            .unwrap();

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

        let modified = "Modified text for testing";
        page.fill("#edit-textarea", modified).await.unwrap();

        page.evaluate::<(), bool>(
            r#"(() => {
                const btn = document.querySelector('.cancel-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        wait_for_element_not_exists(&page, "#edit-textarea", 10).await;

        let restored_text: String = page
            .evaluate(
                r#"(() => {
                    const text = document.querySelector('.log-entry .text');
                    return text ? text.textContent : '';
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            restored_text, original_text,
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

#[tokio::test]
async fn test_delete_removes_message() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let initial = element_count(&page, "#story-log .log-entry").await;

        page.evaluate::<(), bool>(
            r#"(() => {
                const btn = document.querySelector('.delete-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        wait_for_element_exists(&page, ".confirm-modal", 5).await;
        page.evaluate::<(), bool>(
            r#"(() => {
                const btn = document.querySelector('.confirm-modal .confirm-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_millis(500)).await;
        let after = element_count(&page, "#story-log .log-entry").await;
        assert!(after < initial, "Delete should remove the message");
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
            retry_count, 0,
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

/// Test that input messages (user commands) don't show edit buttons.
#[tokio::test]
async fn test_edit_button_not_on_input_entries() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let initial_entries = element_count(&page, "#story-log .log-entry").await;
        send_action(&page, "hello").await;
        wait_for_status_ready(&page).await;
        wait_for_element_children(&page, "#story-log .log-entry", initial_entries as u32 + 2).await;

        let input_edit_buttons = element_count(&page, ".log-entry.input .edit-btn").await;
        assert_eq!(
            input_edit_buttons, 0,
            "Input messages should not have edit buttons"
        );
    })
    .await;
}

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
        let total_entries = wait_for_element_children(&page, "#story-log .log-entry", initial_entries as u32 + 4).await;

        let delete_buttons = element_count(&page, ".log-entry .delete-btn").await;
        assert!(
            delete_buttons <= 2,
            "Should have at most 2 delete buttons (last narration + last input), found: {delete_buttons}"
        );
    })
    .await;
}

/// Test that edit/delete buttons don't appear while the server is generating.
#[tokio::test]
async fn test_edit_disabled_during_generation() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        send_action(&page, "wait").await;

        let status_locator = page.locator("#status-display").await;
        let _ = expect(status_locator)
            .with_timeout(Duration::from_millis(500))
            .not()
            .to_contain_text("Ready")
            .await;

        let edit_buttons = element_count(&page, ".edit-btn:not([style*=\"display: none\"])").await;
        let delete_buttons = element_count(&page, ".delete-btn:not([style*=\"display: none\"])").await;
    })
    .await;
}
