//! Browser behaviour tests: click→DOM change, htmx swap persistence, polling-pause, status wiring. Tagged against `docs/specs/browser.md`.

use std::time::Duration;

use playwright_rs::expect;

use super::*;

// [chronicler_engine/docs/specs/browser.md] SCENARIO: 16.1
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

// [chronicler_engine/docs/specs/browser.md] SCENARIO: 16.2
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

// [chronicler_engine/docs/specs/browser.md] SCENARIO: 16.3
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

// [chronicler_engine/docs/specs/browser.md] SCENARIO: 16.4
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

// [chronicler_engine/docs/specs/browser.md] SCENARIO: 16.5
#[tokio::test]
async fn test_form_stays_static_after_submission() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let form_id_before: String = page
                .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
                .await
                .unwrap();

            page.evaluate::<(), ()>(
                "(() => {
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'look';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()",
                None,
            )
            .await
            .unwrap();

            let _ = wait_for_element_children(&page, "#story-log .log-entry", 2).await;

            let form_id_after: String = page
                .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
                .await
                .unwrap();

            assert_eq!(
                form_id_before, form_id_after,
                "Form should stay in DOM (static shell)"
            );
        },
    )
    .await;
}

// [chronicler_engine/docs/specs/browser.md] SCENARIO: 16.6
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
