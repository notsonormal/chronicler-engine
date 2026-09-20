//! Stub-browser tests for the story log: the client-side edit-mode flow over a canned entry. Tagged against `docs/specs/browser_story_log.md`.

// `showEditForm` is pure client JS over an existing `.log-entry`: it swaps the
// entry's `.text` for a `#edit-textarea`. The stub serves a canned story-log
// entry in the real template's shape; the edit behaviour does not depend on the
// entry's text.

use std::time::Duration;

use super::*;

// [docs/specs/browser_story_log.md] SCENARIO: 30.1
#[tokio::test]
async fn test_edit_mode_activates_on_click() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
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
    })
    .await;
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.2
#[tokio::test]
async fn test_edit_cancel_restores_original() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
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
    })
    .await;
}

// The stub polls `/fragment/story-log` every 2s and returns the canned entry,
// so if `pausePolling` were broken the swap would replace `#story-log`'s
// innerHTML and destroy the textarea. The test therefore still exercises the
// pause: only the paused state lets the textarea survive.
// [docs/specs/browser_story_log.md] SCENARIO: 30.3
#[tokio::test]
async fn test_polling_pauses_during_edit() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
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
    })
    .await;
}
