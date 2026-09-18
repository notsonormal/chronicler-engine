//! Browser story-log tests: delete. Tagged against `docs/specs/browser_story_log.md`.

use std::time::Duration;

use super::*;

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
