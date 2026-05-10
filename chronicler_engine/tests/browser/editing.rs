use super::*;

const TEST_WORLD: &str = "test";
const CONFIG_PATH: &str = "tests/test_config.json";

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
        // Find first edit button and click it
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

        // Wait for DOM to update with textarea
        let _ = wait_for_element_exists(&page, "#edit-textarea", 10).await;

        assert!(
            element_exists(&page, "#edit-textarea").await,
            "Textarea should appear after clicking edit"
        );
        assert!(
            element_exists(&page, ".save-btn").await,
            "Save button should appear during edit"
        );
        assert!(
            element_exists(&page, ".cancel-btn").await,
            "Cancel button should appear during edit"
        );
    })
    .await;
}

#[tokio::test]
async fn test_edit_cancel_restores_original() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        // Get text from first non-location entry with actual text content
        let original_text: String = page
            .evaluate::<(), String>(
                r#"(() => {
                    const entries = document.querySelectorAll('.log-entry');
                    for (const entry of entries) {
                        if (entry.classList.contains('location')) continue;
                        const text = entry.querySelector('.text');
                        if (text && text.textContent.trim()) {
                            return text.textContent;
                        }
                    }
                    return '';
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert!(
            !original_text.is_empty(),
            "Original text should not be empty, got: '{original_text}'"
        );

        // Click edit on the same non-location entry
        page.evaluate::<(), bool>(
            r#"(() => {
                const entry = document.querySelector('.log-entry:not(.location)');
                const btn = entry?.querySelector('.edit-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        // Wait for edit mode to be ready before clicking cancel
        let _ = wait_for_element_exists(&page, ".cancel-btn", 10).await;

        // Click cancel
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

        // Wait for cancel to complete (textarea should disappear)
        let _ = wait_for_element_not_exists(&page, "#edit-textarea", 10).await;

        // Original text should be restored
        let restored_text: String = page
            .evaluate::<(), String>(
                r#"(() => {
                    const entries = document.querySelectorAll('.log-entry');
                    for (const entry of entries) {
                        if (entry.classList.contains('location')) continue;
                        const text = entry.querySelector('.text');
                        if (text && text.textContent.trim()) {
                            return text.textContent;
                        }
                    }
                    return '';
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            original_text, restored_text,
            "Text should be restored after cancel"
        );
    })
    .await;
}

#[tokio::test]
async fn test_polling_pauses_during_edit() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        // Before edit - hx-trigger should include "every"
        let trigger_before: String = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.getAttribute('hx-trigger') || ''",
                None,
            )
            .await
            .unwrap();

        assert!(
            trigger_before.contains("every"),
            "Should have polling trigger before edit"
        );

        // Click edit on the same non-location entry
        page.evaluate::<(), bool>(
            r#"(() => {
                const entry = document.querySelector('.log-entry:not(.location)');
                const btn = entry?.querySelector('.edit-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        // Wait for edit mode - hx-trigger should change to "none"
        let _ = wait_for_element_exists(&page, "#edit-textarea", 10).await;

        // During edit - hx-trigger should be "none"
        let trigger_during: String = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.getAttribute('hx-trigger') || ''",
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            trigger_during, "none",
            "hx-trigger should be 'none' during edit"
        );

        // Click cancel
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

        // Wait for cancel to complete (textarea should disappear)
        let _ = wait_for_element_not_exists(&page, "#edit-textarea", 10).await;

        // After cancel - hx-trigger should include "every" again
        let trigger_after: String = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.getAttribute('hx-trigger') || ''",
                None,
            )
            .await
            .unwrap();

        assert!(
            trigger_after.contains("every"),
            "Should have polling trigger after cancel"
        );
    })
    .await;
}

#[tokio::test]
async fn test_delete_removes_message() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let initial_count = element_count(&page, "#story-log .log-entry").await;
        assert!(initial_count > 0, "Should have at least one log entry");

        // Get the ID of the first deletable entry (one with a delete button)
        let first_entry_id: String = page
            .evaluate::<(), String>(
                r#"(() => {
                    const entry = document.querySelector('.log-entry[data-id]');
                    return entry ? entry.getAttribute('data-id') : '';
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert!(
            !first_entry_id.is_empty(),
            "Should find an entry with data-id"
        );

        // Override confirm to always return true
        page.evaluate::<(), ()>("(() => { window.confirm = () => true; })()", None)
            .await
            .unwrap();

        // Click the delete button on the first entry
        page.evaluate::<(), bool>(
            r#"(() => {
                const btn = document.querySelector('.log-entry .delete-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        // Wait for the specific entry to disappear
        wait_for_element_not_exists(&page, &format!("[data-id=\"{first_entry_id}\"]"), 50).await;

        let current_count = element_count(&page, "#story-log .log-entry").await;
        assert!(
            current_count < initial_count,
            "Entry count should decrease after delete: {initial_count} -> {current_count}"
        );
    })
    .await;
}

#[tokio::test]
async fn test_retry_button_on_last_ai_message() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let retry_count = element_count(&page, ".retry-btn").await;
        assert_eq!(
            retry_count, 1,
            "Should have exactly one retry button on last AI message"
        );

        let earlier_entries_have_retry: i32 = page
            .evaluate::<(), i32>(
                r#"(() => {
                    const entries = document.querySelectorAll('.log-entry');
                    if (entries.length <= 1) return 0;
                    let count = 0;
                    for (let i = 0; i < entries.length - 1; i++) {
                        if (entries[i].querySelector('.retry-btn')) count++;
                    }
                    return count;
                })()"#,
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            earlier_entries_have_retry, 0,
            "Earlier entries should not have retry buttons"
        );
    })
    .await;
}

#[tokio::test]
async fn test_edit_textarea_matches_original_height() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        // Get the original text element height before edit
        let original_height: f64 = page
            .evaluate::<(), f64>(
                r#"(() => {
                    const text = document.querySelector('.log-entry:not(.location) .text');
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
                const entry = document.querySelector('.log-entry:not(.location)');
                const btn = entry?.querySelector('.edit-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
            None,
        )
        .await
        .unwrap();

        // Wait for edit mode to be ready
        let _ = wait_for_element_exists(&page, "#edit-textarea", 10).await;

        // Get textarea height after edit
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

        // The textarea should not be smaller than the original text, and should not
        // be unreasonably larger. Textareas have inherent minimum heights (padding,
        // border, form control sizing) that make them taller than inline spans for
        // very short text, so we check bounds rather than exact match.
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
