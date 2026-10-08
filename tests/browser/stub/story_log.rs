//! Stub-browser tests for the story log: the client-side edit-mode flow over a canned entry. Tagged against `docs/specs/browser_story_log.md`.

// The stub serves a canned entry in the real template's shape, so the edit
// behaviour does not depend on the entry's text.

use std::time::Duration;

use super::*;

/// `pausePolling` writes "none"; `resumePolling` restores the shell's original
/// trigger.
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

async fn narration_text(page: &playwright_rs::Page) -> String {
    page.locator(".log-entry.narration .text")
        .await
        .inner_text()
        .await
        .unwrap_or_default()
}

/// The fixture's first entry is the narration entry and carries the swipe
/// controls.
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

// The stub's 2s poll would replace `#story-log`'s innerHTML and destroy the
// textarea, so only a working pause lets the textarea persist.
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

// The stub answers the save with a 500, so the shipped save click exercises the
// recovery path.
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

        assert!(
            wait_for_condition_async(
                Duration::from_secs(5),
                Duration::from_millis(50),
                || async { read_error_disclosure(&page, "#story-log-error").await.0 },
            )
            .await,
            "a failed save should report in the story log's error slot"
        );
        let (_, save_message, save_raw) =
            read_error_disclosure(&page, "#story-log-error").await;
        assert!(
            !save_message.is_empty(),
            "the failed save should show a short failure message"
        );
        assert!(
            save_raw.contains("Stub save failure"),
            "the server's own text belongs in the disclosure, got {save_raw:?}"
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

// The stub answers the retry route with a 500, so the shipped retry click
// exercises the recovery path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.5
#[tokio::test]
async fn test_failed_retry_clears_pending_status_and_re_enables_send() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        page.locator(".swipe-btn[title='Retry']")
            .await
            .click(None)
            .await
            .unwrap();

        // Reading the disclosure first also guarantees the synchronous recovery
        // ran before the Send/status assertion below.
        assert!(
            wait_for_condition_async(
                Duration::from_secs(5),
                Duration::from_millis(50),
                || async { read_error_disclosure(&page, "#status-display").await.0 },
            )
            .await,
            "a failed retry should report in the status display"
        );
        let (_, message, raw) = read_error_disclosure(&page, "#status-display").await;
        assert!(
            message.contains("Failed to generate a new swipe"),
            "the status display should name the failed action, got {message:?}"
        );
        assert!(
            raw.contains("Stub retry failure"),
            "the server's own text belongs in the disclosure, got {raw:?}"
        );
        assert!(
            !error_details_open(&page, "#status-display").await,
            "the raw error text must not be on screen until the client opens Details"
        );

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
        assert!(!disabled, "a failed retry should re-enable the Send button");
    })
    .await;
}

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

// [docs/specs/browser_story_log.md] SCENARIO: 30.7
#[tokio::test]
async fn test_escape_cancels_edit() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        install_story_poll_counter(&page).await;
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

        assert!(
            wait_for_condition_async(
                Duration::from_millis(500),
                Duration::from_millis(20),
                || async {
                    page.evaluate::<(), bool>(
                        r#"() => {
                            const btn = document.querySelector('.log-entry.narration .edit-btn');
                            return !!btn && document.activeElement === btn;
                        }"#,
                        None,
                    )
                    .await
                    .unwrap_or(false)
                },
            )
            .await,
            "Escape must put focus on the entry's edit button"
        );

        let polled = wait_for_condition_async(
            Duration::from_millis(4000),
            Duration::from_millis(20),
            || async { story_poll_count(&page).await >= 1.0 },
        )
        .await;
        assert!(polled, "the resumed poll never ran after Escape");
        assert!(
            wait_for_condition_async(
                Duration::from_millis(1000),
                Duration::from_millis(20),
                || async {
                    page.evaluate::<(), bool>(
                        r#"() => {
                            const btn = document.querySelector('.log-entry.narration .edit-btn');
                            return !!btn && document.activeElement === btn;
                        }"#,
                        None,
                    )
                    .await
                    .unwrap_or(false)
                },
            )
            .await,
            "the resumed poll must keep focus on the entry's edit button"
        );
    })
    .await;
}

// The stub answers the save with a 500, so a recorded request proves the
// shortcut reached the shipped `submitEdit`; the failure recovery is covered by
// 30.4.
// [docs/specs/browser_story_log.md] SCENARIO: 30.8
#[tokio::test]
async fn test_keyboard_save_shortcuts_submit_edit() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
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

            // Waiting for the textarea to go is the observable end of the attempt.
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

        // Read in the same tick: the resumed poll is async, so this observes
        // `revertEdit`'s synchronous restoration.
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

// The stub answers `/retrigger` with a 500, so the click runs the shipped
// `submitRetrigger` against the real route shape.
// [docs/specs/browser_story_log.md] SCENARIO: 30.10
#[tokio::test]
async fn test_failed_retrigger_posts_to_retrigger_and_recovers() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let retrigger = stub.retrigger_handle();
        async move {
            page.evaluate::<(), ()>(
                r#"(() => {
                    document.querySelector('.log-entry.narration .retrigger-btn').click();
                })()"#,
                None,
            )
            .await
            .unwrap();

            // Reading the disclosure first also guarantees the recovery ran before
            // the assertions below.
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(5),
                    Duration::from_millis(50),
                    || async { read_error_disclosure(&page, "#status-display").await.0 },
                )
                .await,
                "a failed retrigger should report in the status display"
            );
            let (_, message, raw) = read_error_disclosure(&page, "#status-display").await;
            assert!(
                message.contains("Failed to retrigger the event"),
                "the status display should name the failed action, got {message:?}"
            );
            assert!(
                raw.contains("Stub retrigger failure"),
                "the server's own text belongs in the disclosure, got {raw:?}"
            );
            assert!(
                !error_details_open(&page, "#status-display").await,
                "the raw error text must not be on screen until the client opens Details"
            );

            assert_eq!(
                retrigger.count(),
                1,
                "clicking the retrigger control should send exactly one request"
            );

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
                "a failed retrigger should leave the pending status, got {status:?}"
            );
            assert!(
                !disabled,
                "a failed retrigger should re-enable the Send button"
            );
        }
    })
    .await;
}

async fn story_poll_count(page: &playwright_rs::Page) -> f64 {
    page.evaluate::<(), f64>("(() => window.__storyPolls || 0)()", None)
        .await
        .unwrap_or(0.0)
}

async fn install_story_poll_counter(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            window.__storyPolls = 0;
            document.body.addEventListener('htmx:afterRequest', (evt) => {
                const elt = evt.detail && evt.detail.elt;
                if (elt && elt.id === 'story-log') window.__storyPolls += 1;
            });
        })()"#,
        None,
    )
    .await
    .unwrap();
}

async fn arm_save_completion(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            window.__saveDone = false;
            const original = window.fetch;
            window.fetch = function (input, init) {
                const request = original.call(this, input, init);
                if (String(input).includes('/history/')) {
                    return request.then((response) => {
                        window.__saveDone = true;
                        return response;
                    });
                }
                return request;
            };
        })()"#,
        None,
    )
    .await
    .unwrap();
}

async fn wait_for_save_completion(page: &playwright_rs::Page) -> bool {
    wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(10),
        || async {
            page.evaluate::<(), bool>("(() => window.__saveDone === true)()", None)
                .await
                .unwrap_or(false)
        },
    )
    .await
}

const OTHER_EDIT_LOCKED: &str = r#"(() => {
    const other = document.querySelector('.log-entry.input .edit-btn');
    return !!other && other.disabled;
})()"#;

// An innerHTML swap would replace the node and lose the selection, so the poll
// must morph.
// [docs/specs/browser_story_log.md] SCENARIO: 30.11
#[tokio::test]
async fn test_text_selection_survives_the_poll() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        install_story_poll_counter(&page).await;

        let selected = page
            .evaluate::<(), String>(
                r#"(() => {
                    const text = document.querySelector('.log-entry.narration .text');
                    const range = document.createRange();
                    range.selectNodeContents(text);
                    const selection = window.getSelection();
                    selection.removeAllRanges();
                    selection.addRange(range);
                    window.__selectedNode = text;
                    return selection.toString();
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(!selected.is_empty(), "the test must create a selection");

        let polled = wait_for_condition_async(
            Duration::from_secs(8),
            Duration::from_millis(50),
            || async { story_poll_count(&page).await >= 2.0 },
        )
        .await;
        assert!(polled, "the story log must poll at least twice");

        let (still_selected, same_node) = page
            .evaluate::<(), (bool, bool)>(
                r#"(() => {
                    const selection = window.getSelection();
                    const text = document.querySelector('.log-entry.narration .text');
                    return [
                        !!selection && selection.toString().length > 0,
                        text === window.__selectedNode,
                    ];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            still_selected,
            "a text selection in the log must survive the poll"
        );
        assert!(
            same_node,
            "an idle poll must not replace the entry the selection is in"
        );
    })
    .await;
}

// An innerHTML swap would drop focus to the body, so the poll must morph.
// [docs/specs/browser_story_log.md] SCENARIO: 30.12
#[tokio::test]
async fn test_entry_focus_survives_the_poll() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        install_story_poll_counter(&page).await;

        page.locator(".log-entry.narration .retrigger-btn")
            .await
            .focus()
            .await
            .unwrap();
        page.evaluate::<(), ()>(
            "(() => { window.__focused = document.activeElement; })()",
            None,
        )
        .await
        .unwrap();

        let polled = wait_for_condition_async(
            Duration::from_secs(8),
            Duration::from_millis(50),
            || async { story_poll_count(&page).await >= 2.0 },
        )
        .await;
        assert!(polled, "the story log must poll at least twice");

        let (focused, same_node) = page
            .evaluate::<(), (bool, bool)>(
                r#"(() => {
                    const btn = document.querySelector('.log-entry.narration .retrigger-btn');
                    return [document.activeElement === btn, btn === window.__focused];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(focused, "focus inside a log entry must survive the poll");
        assert!(
            same_node,
            "an idle poll must not replace the focused control"
        );
    })
    .await;
}

// A scroll container needs focus before a keyboard user can arrow-scroll it.
// [docs/specs/browser_story_log.md] SCENARIO: 30.13
#[tokio::test]
async fn test_story_log_is_keyboard_scrollable() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        let (focusable, label, overflows) = page
            .evaluate::<(), (bool, String, bool)>(
                r#"(() => {
                    const log = document.getElementById('story-log');
                    return [
                        log.getAttribute('tabindex') === '0',
                        log.getAttribute('aria-label') || '',
                        log.scrollHeight > log.clientHeight,
                    ];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(focusable, "#story-log must be focusable");
        assert!(!label.is_empty(), "#story-log must have an accessible name");
        assert!(
            overflows,
            "the canned log must overflow its container for the scroll check"
        );

        page.locator("#story-log").await.focus().await.unwrap();
        let focused = page
            .evaluate::<(), bool>("(() => document.activeElement.id === 'story-log')()", None)
            .await
            .unwrap();
        assert!(focused, "focusing #story-log must put the keyboard on it");

        let before = page
            .evaluate::<(), f64>(
                "(() => document.getElementById('story-log').scrollTop)()",
                None,
            )
            .await
            .unwrap();
        // A single synthetic key can be consumed while focus settles, so press
        // until the native scroll moves the log.
        let scrolled = wait_for_condition_async(
            Duration::from_secs(3),
            Duration::from_millis(50),
            || async {
                page.keyboard().press("ArrowDown", None).await.unwrap();
                page.evaluate::<(), f64>(
                    "(() => document.getElementById('story-log').scrollTop)()",
                    None,
                )
                .await
                .unwrap_or(0.0)
                    > before
            },
        )
        .await;
        assert!(scrolled, "ArrowDown must scroll the focused log");
    })
    .await;
}

// The 50-entry cap drops the oldest entry, so the morph's removal matching must
// keep every surviving entry's nodes.
// [docs/specs/browser_story_log.md] SCENARIO: 30.14
#[tokio::test]
async fn test_poll_removal_keeps_the_surviving_entries() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let story = stub.story_log_handle();
        async move {
            story.set_extra_oldest(true);
            let grew = wait_for_condition_async(
                Duration::from_secs(8),
                Duration::from_millis(50),
                || async {
                    page.evaluate::<(), f64>(
                        "(() => document.querySelectorAll('#story-log .log-entry').length)()",
                        None,
                    )
                    .await
                    .unwrap_or(0.0)
                        >= 3.0
                },
            )
            .await;
            assert!(grew, "the poll must render the extra oldest entry");

            page.evaluate::<(), ()>(
                r#"(() => {
                    window.__survivors = Array.from(
                        document.querySelectorAll('#story-log .log-entry'),
                    ).slice(1);
                })()"#,
                None,
            )
            .await
            .unwrap();

            story.set_extra_oldest(false);
            let shrank = wait_for_condition_async(
                Duration::from_secs(8),
                Duration::from_millis(50),
                || async {
                    page.evaluate::<(), f64>(
                        "(() => document.querySelectorAll('#story-log .log-entry').length)()",
                        None,
                    )
                    .await
                    .unwrap_or(0.0)
                        <= 2.0
                },
            )
            .await;
            assert!(shrank, "the poll must drop the extra oldest entry");

            let kept = page
                .evaluate::<(), bool>(
                    r#"(() => {
                        const now = Array.from(
                            document.querySelectorAll('#story-log .log-entry'),
                        );
                        const before = window.__survivors || [];
                        return now.length === before.length
                            && now.every((el, i) => el === before[i]);
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(
                kept,
                "a morph must keep the surviving entries' nodes when one is dropped"
            );
        }
    })
    .await;
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.15
#[tokio::test]
async fn test_edit_locks_the_other_entries_edit_controls() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        enter_narration_edit(&page).await;

        let locked = page
            .evaluate::<(), bool>(OTHER_EDIT_LOCKED, None)
            .await
            .unwrap();
        assert!(
            locked,
            "the other entries' Edit controls must be disabled during an edit"
        );

        // Read in the same tick: the resumed poll is async.
        let unlocked = page
            .evaluate::<(), bool>(
                r#"(() => {
                    document.querySelector('.log-entry.narration .cancel-btn').click();
                    const other = document.querySelector('.log-entry.input .edit-btn');
                    return !!other && !other.disabled;
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            unlocked,
            "cancel must re-enable the other entries' Edit controls immediately"
        );

        wait_until_hidden(&page, "#edit-textarea", Duration::from_millis(500)).await;
    })
    .await;
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.16
#[tokio::test]
async fn test_failed_save_releases_the_edit_lock() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        arm_save_completion(&page).await;
        enter_narration_edit(&page).await;
        assert!(
            page.evaluate::<(), bool>(OTHER_EDIT_LOCKED, None)
                .await
                .unwrap(),
            "the other entries' Edit controls must be disabled during an edit"
        );

        page.locator(".log-entry.narration .save-btn")
            .await
            .click(None)
            .await
            .unwrap();
        assert!(
            wait_for_save_completion(&page).await,
            "the failed save request never completed"
        );

        let unlocked = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const other = document.querySelector('.log-entry.input .edit-btn');
                    return !!other && !other.disabled;
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            unlocked,
            "a failed save must re-enable the other entries' Edit controls"
        );
    })
    .await;
}

// The lock must outlive a successful save: releasing it early would let a
// second editor open on the stale textarea.
// [docs/specs/browser_story_log.md] SCENARIO: 30.17
#[tokio::test]
async fn test_successful_save_holds_the_edit_lock_until_the_poll() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let save = stub.save_handle();
        async move {
            save.set_save_succeeds(true);
            arm_save_completion(&page).await;
            page.evaluate::<(), ()>(
                r#"(() => {
                    window.__earlyUnlock = false;
                    const observer = new MutationObserver(() => {
                        const other = document.querySelector('.log-entry.input .edit-btn');
                        if (
                            document.querySelector('#edit-textarea')
                            && other
                            && !other.disabled
                        ) {
                            window.__earlyUnlock = true;
                        }
                    });
                    observer.observe(document.getElementById('story-log'), {
                        subtree: true,
                        childList: true,
                        attributes: true,
                    });
                })()"#,
                None,
            )
            .await
            .unwrap();

            enter_narration_edit(&page).await;
            assert!(
                page.evaluate::<(), bool>(OTHER_EDIT_LOCKED, None)
                    .await
                    .unwrap(),
                "the other entries' Edit controls must be disabled during an edit"
            );

            page.locator(".log-entry.narration .save-btn")
                .await
                .click(None)
                .await
                .unwrap();
            assert!(
                wait_for_save_completion(&page).await,
                "the successful save request never completed"
            );

            let rerendered = wait_for_condition_async(
                Duration::from_secs(6),
                Duration::from_millis(50),
                || async {
                    page.evaluate::<(), bool>(
                        r#"(() => {
                            const other = document.querySelector('.log-entry.input .edit-btn');
                            return !document.querySelector('#edit-textarea')
                                && !!other
                                && !other.disabled;
                        })()"#,
                        None,
                    )
                    .await
                    .unwrap_or(false)
                },
            )
            .await;
            assert!(
                rerendered,
                "the resumed poll must re-render the log and re-enable the Edit controls"
            );

            let early_unlock = page
                .evaluate::<(), bool>("(() => window.__earlyUnlock === true)()", None)
                .await
                .unwrap();
            assert!(
                !early_unlock,
                "a successful save must hold the edit lock until the poll re-renders the log"
            );
        }
    })
    .await;
}

// The stub answers POST /history/delete with a 500, so the shipped delete click
// exercises the failure path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.18
#[tokio::test]
async fn test_failed_delete_reports_in_the_story_log_slot() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        page.on_dialog(|dialog| async move { dialog.accept(None).await })
            .await
            .unwrap();

        let entries_before = count_log_entries(&page).await;
        assert!(
            entries_before > 1,
            "the fixture must show an entry that carries a delete control"
        );

        page.locator("#story-log .log-entry:last-child .delete-btn")
            .await
            .click(None)
            .await
            .unwrap();

        assert!(
            wait_for_condition_async(
                Duration::from_secs(5),
                Duration::from_millis(50),
                || async { read_error_disclosure(&page, "#story-log-error").await.0 },
            )
            .await,
            "a failed delete should report in the story log's error slot"
        );
        let (_, message, raw) = read_error_disclosure(&page, "#story-log-error").await;
        assert!(
            message.contains("Failed to delete the message"),
            "the slot should name the failed action, got {message:?}"
        );
        assert!(
            raw.contains("Stub delete failure"),
            "the server's own text belongs in the disclosure, got {raw:?}"
        );
        assert_eq!(
            count_log_entries(&page).await,
            entries_before,
            "a failed delete must leave the log as it was"
        );
    })
    .await;
}

// The stub answers a swipe switch with a 500 once the test asks for the failure
// path, so the shipped previous-swipe click runs the real client path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.19
#[tokio::test]
async fn test_failed_swipe_switch_reports_in_the_story_log_slot() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let swipe = stub.swipe_handle();
        async move {
            swipe.set_two_swipes();
            swipe.set_switch_failing();

            assert!(
                wait_for_condition_async(
                    Duration::from_secs(10),
                    Duration::from_millis(200),
                    || async {
                        page.evaluate::<(), bool>(
                            r#"() => !!document.querySelector(".swipe-btn[title='Previous swipe']:not([disabled])")"#,
                            None,
                        )
                        .await
                        .unwrap_or(false)
                    },
                )
                .await,
                "the two-swipe log must offer a Previous-swipe control"
            );

            let entries_before = count_log_entries(&page).await;

            page.evaluate::<(), ()>(
                r#"(() => {
                    const btn = document.querySelector(".swipe-btn[title='Previous swipe']");
                    if (btn) btn.click();
                })()"#,
                None,
            )
            .await
            .unwrap();

            assert!(
                wait_for_condition_async(
                    Duration::from_secs(5),
                    Duration::from_millis(50),
                    || async { read_error_disclosure(&page, "#story-log-error").await.0 },
                )
                .await,
                "a failed swipe switch should report in the story log's error slot"
            );
            let (_, message, raw) = read_error_disclosure(&page, "#story-log-error").await;
            assert!(
                message.contains("Failed to switch the swipe"),
                "the slot should name the failed action, got {message:?}"
            );
            assert!(
                raw.contains("Stub swipe switch failure"),
                "the server's own text belongs in the disclosure, got {raw:?}"
            );
            assert_eq!(
                count_log_entries(&page).await,
                entries_before,
                "a failed swipe switch must leave the log as it was"
            );
        }
    })
    .await;
}
