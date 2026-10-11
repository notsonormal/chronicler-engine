//! Stub-browser tests for the story log's edit flow: edit mode, the entry locks and the failure slots. Tagged against `docs/specs/browser_story_log.md`.

// The stub serves a canned entry in the real template's shape, so the edit
// behaviour does not depend on the entry's text.

use std::time::Duration;

use super::*;
use super::support::{StubRunner, install_story_poll_counter, poll_now, story_poll_count};

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
async fn check_edit_mode_activates_on_click(page: playwright_rs::Page, _stub: StubServer) {
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
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.2
async fn check_edit_cancel_restores_original(page: playwright_rs::Page, _stub: StubServer) {
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
}

// The stub's 2s poll would replace `#story-log`'s innerHTML and destroy the
// textarea, so only a working pause lets the textarea persist.
// [docs/specs/browser_story_log.md] SCENARIO: 30.3
async fn check_polling_pauses_during_edit(page: playwright_rs::Page, _stub: StubServer) {
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

    let persisted = wait_for_element_persist(&page, "#edit-textarea", Duration::from_secs(3)).await;
    assert!(
        persisted,
        "Edit textarea should persist during polling pause"
    );
}

// The stub answers the save with a 500, so the shipped save click exercises the
// recovery path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.4
async fn check_failed_save_restores_entry_and_resumes_polling(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
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
    let (_, save_message, save_raw) = read_error_disclosure(&page, "#story-log-error").await;
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
}

// The stub answers the retry route with a 500, so the shipped retry click
// exercises the recovery path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.5
async fn check_failed_retry_clears_pending_status_and_re_enables_send(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
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
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.6
async fn check_edit_textarea_fits_content_and_takes_focus(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
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
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.7
async fn check_escape_cancels_edit(page: playwright_rs::Page, _stub: StubServer) {
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
}

// The stub answers the save with a 500, so a recorded request proves the
// shortcut reached the shipped `submitEdit`; the failure recovery is covered by
// 30.4.
// [docs/specs/browser_story_log.md] SCENARIO: 30.8
async fn check_keyboard_save_shortcuts_submit_edit(page: playwright_rs::Page, _stub: StubServer) {
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
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.9
async fn check_edit_locks_entry_controls_and_cancel_restores_them(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
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
}

// The stub answers `/retrigger` with a 500, so the click runs the shipped
// `submitRetrigger` against the real route shape.
// [docs/specs/browser_story_log.md] SCENARIO: 30.10
async fn check_failed_retrigger_posts_to_retrigger_and_recovers(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let retrigger = stub.retrigger_handle();
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

// [docs/specs/browser_story_log.md] SCENARIO: 30.15
async fn check_edit_locks_the_other_entries_edit_controls(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
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
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.16
async fn check_failed_save_releases_the_edit_lock(page: playwright_rs::Page, _stub: StubServer) {
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
}

// The lock must outlive a successful save: releasing it early would let a
// second editor open on the stale textarea.
// [docs/specs/browser_story_log.md] SCENARIO: 30.17
async fn check_successful_save_holds_the_edit_lock_until_the_poll(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let save = stub.save_handle();
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

// The stub answers POST /history/delete with a 500, so the shipped delete click
// exercises the failure path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.18
async fn check_failed_delete_reports_in_the_story_log_slot(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
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
}

// The stub answers a swipe switch with a 500 once the test asks for the failure
// path, so the shipped previous-swipe click runs the real client path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.19
async fn check_failed_swipe_switch_reports_in_the_story_log_slot(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let swipe = stub.swipe_handle();
    swipe.set_two_swipes();
    swipe.set_switch_failing();
    poll_now(&page, "#story-log").await;

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

// The stub answers POST /history/delete with a 200 and drops the served log's
// last entry, so the shipped delete click runs its success path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.22
async fn check_successful_delete_keeps_focus_in_the_story_log(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let save = stub.save_handle();
    page.on_dialog(|dialog| async move { dialog.accept(None).await })
        .await
        .unwrap();
    save.set_delete_succeeds(true);

    let entries_before = count_log_entries(&page).await;
    assert!(
        entries_before > 1,
        "the fixture must show an entry that carries a delete control"
    );

    let delete = "#story-log .log-entry:last-child .delete-btn";
    page.locator(delete).await.focus().await.unwrap();
    page.locator(delete).await.click(None).await.unwrap();

    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async {
                page.evaluate::<(), bool>(
                    r#"(() => {
                                const entries = document.querySelectorAll('#story-log .log-entry');
                                const last = entries[entries.length - 1];
                                const active = document.activeElement;
                                return entries.length === 1
                                    && !!last && !!active && last.contains(active)
                                    && active.classList.contains('edit-btn');
                            })()"#,
                    None,
                )
                .await
                .unwrap_or(false)
            },
        )
        .await,
        "the entry that becomes last must take focus on its Edit control"
    );
}

#[tokio::test]
async fn run_story_log_edit_mode_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_edit_mode_activates_on_click,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_edit_cancel_restores_original,
        )
        .await;
    runner
        .run(StubActionOutcome::Pending, check_polling_pauses_during_edit)
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_edit_textarea_fits_content_and_takes_focus,
        )
        .await;
    runner
        .run(StubActionOutcome::Pending, check_escape_cancels_edit)
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_keyboard_save_shortcuts_submit_edit,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_edit_locks_entry_controls_and_cancel_restores_them,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_edit_locks_the_other_entries_edit_controls,
        )
        .await;
    runner.finish().await;
}

#[tokio::test]
async fn run_story_log_edit_failure_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_save_restores_entry_and_resumes_polling,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_retry_clears_pending_status_and_re_enables_send,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_retrigger_posts_to_retrigger_and_recovers,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_save_releases_the_edit_lock,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_successful_save_holds_the_edit_lock_until_the_poll,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_delete_reports_in_the_story_log_slot,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_swipe_switch_reports_in_the_story_log_slot,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_successful_delete_keeps_focus_in_the_story_log,
        )
        .await;
    runner.finish().await;
}
