//! Browser story-log tests: edit-in-place, polling persistence, delete. Tagged against `docs/specs/browser_story_log.md`.

use std::time::Duration;

use super::*;

// [docs/specs/browser_story_log.md] SCENARIO: 30.1
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

            wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;
        },
    )
    .await;
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.2
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
            wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;

            let modified = "Modified text for testing";
            page.locator("#edit-textarea")
                .await
                .fill(modified, None)
                .await
                .unwrap();

            page.locator(".cancel-btn").await.click(None).await.unwrap();
            wait_until_hidden(&page, "#edit-textarea", Duration::from_millis(500)).await;

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

// [docs/specs/browser_story_log.md] SCENARIO: 30.3
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

            wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;

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

// [docs/specs/browser_story_log.md] SCENARIO: 30.4
#[tokio::test]
async fn test_delete_removes_message() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let initial = count_log_entries(&page).await;
            if initial < 2 {
                send_action(&page, "look").await;
                wait_for_status_ready(&page).await;
            }

            // The story log renders on its own 2s poll, independent of the 5s
            // status poll — status "Ready" does not mean the log shows the
            // completed turn. Wait for the DOM to catch up before reading the
            // baseline.
            wait_for_element_children(&page, "#story-log .log-entry", 2).await;

            // Target one specific entry: capture the last entry's data-id, then
            // wait for THAT id to leave the DOM. A count-vs-baseline comparison
            // cannot distinguish a successful delete from a concurrent log
            // refresh that adds entries.
            let deleted_id: Option<String> = page
                .evaluate::<(), Option<String>>(
                    r#"(() => {
                    const entries = document.querySelectorAll('#story-log .log-entry');
                    const last = entries[entries.length - 1];
                    return last ? last.getAttribute('data-id') : null;
                })()"#,
                    None,
                )
                .await
                .unwrap();
            let deleted_id = deleted_id.expect("log must contain an entry to delete");

            page.evaluate::<(), ()>(
                r#"(() => {
                window.confirm = () => true;
            })()"#,
                None,
            )
            .await
            .unwrap();

            page.locator(".delete-btn").await.click(None).await.unwrap();

            // Detached counts as hidden — the log refresh replaces #story-log's
            // innerHTML, dropping the deleted entry from the DOM.
            wait_until_hidden(
                &page,
                &format!(".log-entry[data-id='{deleted_id}']"),
                Duration::from_secs(10),
            )
            .await;
        },
    )
    .await;
}
