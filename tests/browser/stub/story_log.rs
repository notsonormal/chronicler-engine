//! Stub-browser tests for the story log: the client-side edit-mode flow over a canned entry. Tagged against `docs/specs/browser_story_log.md`.

// `showEditForm` is pure client JS over an existing `.log-entry`: it swaps the
// entry's `.text` for a `#edit-textarea`. The stub serves a canned story-log
// entry in the real template's shape; the edit behaviour does not depend on the
// entry's text.

use std::time::Duration;

use super::*;

/// The `hx-trigger` the story-log poller currently carries. `pausePolling`
/// writes "none"; `resumePolling` restores the shell's original trigger.
async fn story_log_trigger(page: &playwright_rs::Page) -> String {
    page.evaluate::<(), String>(
        r#"(() => {
            const el = document.getElementById('story-log');
            return el ? (el.getAttribute('hx-trigger') || '') : '';
        })()"#,
        None,
    )
    .await
    .unwrap()
}

/// The text the error toast currently shows.
async fn error_toast_text(page: &playwright_rs::Page) -> String {
    page.evaluate::<(), String>(
        r#"(() => {
            const el = document.getElementById('error-notification');
            return el ? (el.textContent || '') : '';
        })()"#,
        None,
    )
    .await
    .unwrap()
}

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
            .locator(".log-entry.narration .text")
            .await
            .inner_text()
            .await
            .unwrap_or_default();
        assert!(!original_text.is_empty(), "Should have original text");

        page.locator(".log-entry.narration .edit-btn")
            .await
            .click(None)
            .await
            .unwrap();
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
            .locator(".log-entry.narration .text")
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

// The save route fails (stub: 500) so the shipped save click exercises the
// recovery path: report through the toast, restore the pre-edit entry, and
// unpause the log. Without the fix the textarea and the `hx-trigger="none"`
// both survive and the error is silent.
// [docs/specs/browser_story_log.md] SCENARIO: 30.4
#[tokio::test]
async fn test_failed_save_restores_entry_and_resumes_polling() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        let original_text = page
            .locator(".log-entry.narration .text")
            .await
            .inner_text()
            .await
            .unwrap_or_default();
        assert!(!original_text.is_empty(), "Should have original text");

        page.locator(".log-entry.narration .edit-btn")
            .await
            .click(None)
            .await
            .unwrap();
        wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;

        // Entering edit mode pauses the log; the recovery must put it back.
        assert_eq!(
            story_log_trigger(&page).await,
            "none",
            "edit mode should pause story-log polling"
        );

        page.locator("#edit-textarea")
            .await
            .fill("Modified text that fails to save", None)
            .await
            .unwrap();
        page.locator(".log-entry.narration .save-btn")
            .await
            .click(None)
            .await
            .unwrap();

        wait_until_visible(&page, "#error-notification.visible", Duration::from_secs(5)).await;
        assert!(
            !error_toast_text(&page).await.is_empty(),
            "the failed save should show an error message"
        );

        wait_until_hidden(&page, "#edit-textarea", Duration::from_millis(500)).await;

        let restored = page
            .locator(".log-entry.narration .text")
            .await
            .inner_text()
            .await
            .unwrap_or_default();
        assert_eq!(
            restored, original_text,
            "the entry should return to its pre-edit text after a failed save"
        );

        let trigger = story_log_trigger(&page).await;
        assert_ne!(
            trigger, "none",
            "story-log polling should resume after a failed save"
        );
        assert!(
            trigger.contains("every 2s"),
            "resumed poll trigger should be the shell's original, got {trigger:?}"
        );
    })
    .await;
}

// The retry route fails (stub: 500) so the shipped retry click exercises the
// recovery path: report through the toast and leave the status display no
// longer stuck on the pending state with Send disabled.
// [docs/specs/browser_story_log.md] SCENARIO: 30.5
#[tokio::test]
async fn test_failed_retry_clears_pending_status_and_re_enables_send() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        page.locator(".swipe-btn[title='Retry']")
            .await
            .click(None)
            .await
            .unwrap();

        // The toast is what the fix adds; reading it first also guarantees the
        // synchronous recovery ran before the Send/status assertion below.
        wait_until_visible(&page, "#error-notification.visible", Duration::from_secs(5)).await;

        let (status, disabled) = page
            .evaluate::<(), (String, bool)>(
                r#"(() => {
                    const status = document.getElementById('status-display');
                    const btn = document.getElementById('submit-btn');
                    return [
                        status ? status.textContent.trim() : '',
                        btn ? btn.disabled : true,
                    ];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            !status.contains("Thinking"),
            "a failed retry should leave the pending status, got {status:?}"
        );
        assert!(
            status.contains("Ready"),
            "a failed retry should reset the status to Ready, got {status:?}"
        );
        assert!(!disabled, "a failed retry should re-enable the Send button");
    })
    .await;
}
