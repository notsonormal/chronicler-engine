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

/// The narration entry's current visible text, the value a revert restores.
async fn narration_text(page: &playwright_rs::Page) -> String {
    page.locator(".log-entry.narration .text")
        .await
        .inner_text()
        .await
        .unwrap_or_default()
}

/// Open edit mode on the narration entry (the fixture's first, swipe-bearing
/// entry) and wait for the textarea.
async fn enter_narration_edit(page: &playwright_rs::Page) {
    page.locator(".log-entry.narration .edit-btn")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(page, "#edit-textarea", Duration::from_millis(500)).await;
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
        let original_text = narration_text(&page).await;
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
        let original_text = narration_text(&page).await;
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
            !read_error_toast(&page).await.1.is_empty(),
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

        let controls_restored = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const entry = document.querySelector('.log-entry.narration');
                    if (!entry) return false;
                    return !!entry.querySelector('.edit-btn') &&
                        !!entry.querySelector('.swipe-controls .swipe-btn:not([disabled])');
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            controls_restored,
            "the entry's pre-edit action controls, including an enabled Retry, should be available again after a failed save"
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

// The textarea auto-grows with its content and takes focus on activation.
// The growth is capped: content below the cap shows no inner scrollbar, and
// content past it scrolls inside the textarea.
// [docs/specs/browser_story_log.md] SCENARIO: 30.6
#[tokio::test]
async fn test_edit_textarea_fits_content_and_takes_focus() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        enter_narration_edit(&page).await;

        let focused: bool = page
            .evaluate::<(), bool>(
                r#"(() => document.activeElement === document.querySelector('#edit-textarea'))()"#,
                None,
            )
            .await
            .unwrap();
        assert!(focused, "the edit textarea should take focus");

        let textarea = page.locator("#edit-textarea").await;

        // Content below the 50vh cap fits, so the textarea does not scroll.
        textarea.fill("A short edit.", None).await.unwrap();
        let (short_client, short_scroll, viewport): (f64, f64, f64) = page
            .evaluate::<(), (f64, f64, f64)>(
                r#"(() => {
                    const textarea = document.querySelector('#edit-textarea');
                    return [textarea.clientHeight, textarea.scrollHeight, window.innerHeight];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            short_scroll <= short_client + 1.0,
            "content below the cap should not scroll inside the textarea \
             (client height {short_client}, scroll height {short_scroll})"
        );

        // Content past the cap stops at 50vh and scrolls inside the textarea.
        let long_text = "A line of text for the cap check.\n".repeat(120);
        textarea.fill(&long_text, None).await.unwrap();
        let (long_client, long_scroll): (f64, f64) = page
            .evaluate::<(), (f64, f64)>(
                r#"(() => {
                    const textarea = document.querySelector('#edit-textarea');
                    return [textarea.clientHeight, textarea.scrollHeight];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            long_client <= viewport * 0.5 + 1.0,
            "the textarea should stop at the 50vh cap \
             (client height {long_client}, viewport {viewport})"
        );
        assert!(
            long_scroll > long_client,
            "content past the cap should scroll inside the textarea \
             (client height {long_client}, scroll height {long_scroll})"
        );
    })
    .await;
}

// Escape is the keyboard twin of the ✗ button: it must abandon the edit and
// restore the original text.
// [docs/specs/browser_story_log.md] SCENARIO: 30.7
#[tokio::test]
async fn test_escape_cancels_edit() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        let original_text = narration_text(&page).await;
        assert!(!original_text.is_empty(), "Should have original text");

        enter_narration_edit(&page).await;
        let textarea = page.locator("#edit-textarea").await;
        textarea
            .fill("Modified text that Escape abandons", None)
            .await
            .unwrap();
        textarea.press("Escape", None).await.unwrap();

        wait_until_hidden(&page, "#edit-textarea", Duration::from_millis(500)).await;

        let restored = page
            .locator(".log-entry.narration .text")
            .await
            .inner_text()
            .await
            .unwrap_or_default();
        assert_eq!(
            restored, original_text,
            "Escape should restore the original text"
        );
    })
    .await;
}

// Ctrl+Enter and Cmd+Enter are the keyboard save shortcuts. The stub answers
// the save with a 500, so a recorded request proves the shortcut reached the
// shipped `submitEdit`; the surrounding failure recovery is covered by 30.4.
// [docs/specs/browser_story_log.md] SCENARIO: 30.8
#[tokio::test]
async fn test_keyboard_save_shortcuts_submit_edit() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        // Record the save URLs without changing the stub's canned failure.
        page.evaluate::<(), ()>(
            r#"(() => {
                window.__saveRequests = [];
                const original = window.fetch;
                window.fetch = function (input, init) {
                    window.__saveRequests.push(String(input));
                    return original.call(this, input, init);
                };
            })()"#,
            None,
        )
        .await
        .unwrap();

        for (modifier, text) in [
            ("Control+Enter", "Ctrl+Enter text"),
            ("Meta+Enter", "Cmd+Enter text"),
        ] {
            enter_narration_edit(&page).await;
            let textarea = page.locator("#edit-textarea").await;
            textarea.fill(text, None).await.unwrap();
            textarea.press(modifier, None).await.unwrap();

            // The failed save reverts and raises the toast; waiting for the
            // textarea to go is the observable end of the attempt.
            wait_until_hidden(&page, "#edit-textarea", Duration::from_millis(500)).await;
        }

        let requests: Vec<String> = page
            .evaluate::<(), Vec<String>>("(() => window.__saveRequests)()", None)
            .await
            .unwrap();
        assert_eq!(
            requests.len(),
            2,
            "Ctrl+Enter and Cmd+Enter should each send one save request, got {requests:?}"
        );
        assert!(
            requests.iter().all(|url| url.ends_with("/history/1")),
            "save requests should target the edited entry, got {requests:?}"
        );
    })
    .await;
}

// While editing, the swipe controls must stop responding and the action
// cluster becomes save/cancel. Cancel must put the pre-edit controls back
// immediately, not wait for the resumed poll.
// [docs/specs/browser_story_log.md] SCENARIO: 30.9
#[tokio::test]
async fn test_edit_locks_entry_controls_and_cancel_restores_them() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        enter_narration_edit(&page).await;

        let (swipe_disabled, edit_replaced): (bool, bool) = page
            .evaluate::<(), (bool, bool)>(
                r#"(() => {
                    const entry = document.querySelector('.log-entry.narration');
                    const buttons = entry.querySelectorAll('.swipe-controls .action-btn');
                    return [
                        buttons.length > 0 && Array.from(buttons).every((b) => b.disabled),
                        !entry.querySelector('.edit-btn'),
                    ];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            swipe_disabled,
            "the entry's swipe controls should be disabled while editing"
        );
        assert!(
            edit_replaced,
            "the edit control should be replaced while editing"
        );

        // Cancel and read the entry in the same tick: the resumed poll is
        // async, so this observes `revertEdit`'s synchronous restoration.
        let (edit_back, retry_enabled): (bool, bool) = page
            .evaluate::<(), (bool, bool)>(
                r#"(() => {
                    document.querySelector('.log-entry.narration .cancel-btn').click();
                    const entry = document.querySelector('.log-entry.narration');
                    const retry = entry.querySelector('.swipe-controls .swipe-btn:not([disabled])');
                    return [!!entry.querySelector('.edit-btn'), !!retry];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            edit_back,
            "cancel should restore the entry's edit control immediately"
        );
        assert!(
            retry_enabled,
            "cancel should re-enable the entry's Retry swipe control"
        );

        wait_until_hidden(&page, "#edit-textarea", Duration::from_millis(500)).await;
    })
    .await;
}
