//! Stub-browser tests for the story log's edit flow: edit mode, the entry locks and where a failed request shows. Tagged against `docs/specs/browser_story_log.md`.

// The stub serves a canned entry in the real template's shape, so the edit
// behaviour does not depend on the entry's text.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;

use super::*;
use super::support::{
    NOTICE, StubRunner, hold_screen_loads, install_story_load_counter, release_screen_loads,
    start_current, story_load_count,
};

/// An open editor marks its entry `data-editing` and the root of the story log
/// `data-editing-entry`.
async fn story_log_marks_an_edit(page: &playwright_rs::Page) -> (bool, bool) {
    page.evaluate::<(), (bool, bool)>(
        "(() => [!!document.querySelector('#story-log .log-entry[data-editing]'), document.getElementById('story-log').hasAttribute('data-editing-entry')])()",
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

    page.locator("[data-editing] .cancel-btn")
        .await
        .click(None)
        .await
        .unwrap();
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

// A moved story number makes the log load on every ask. The load skips the entry
// in edit and shows the rest of the log, so new narration arrives around the draft.
// [docs/specs/browser_story_log.md] SCENARIO: 30.3
async fn check_an_entry_in_edit_survives_loads_of_the_story_log(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    enter_narration_edit(&page).await;
    page.locator("#edit-textarea")
        .await
        .fill("A draft that a load must not erase", None)
        .await
        .unwrap();

    let changes = stub.changes_handle();
    let loads = changes.loads("story-log");
    stub.story_log_handle()
        .append_narration("Narration that arrived during the edit.");
    changes.bump(ChangeKind::STORY);
    let loaded = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            changes.loads("story-log") > loads
                && page
                    .evaluate::<(), bool>(
                        "(() => document.getElementById('story-log').innerText.includes('Narration that arrived during the edit.'))()",
                        None,
                    )
                    .await
                    .unwrap_or(false)
        },
    )
    .await;
    assert!(
        loaded,
        "the story log loads while an entry is in edit, and the new narration shows"
    );

    let textarea = page
        .evaluate::<(), String>(
            "(() => { const t = document.getElementById('edit-textarea'); return t ? t.value : 'the textarea is gone'; })()",
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        textarea, "A draft that a load must not erase",
        "the entry in edit keeps its draft through the load"
    );
    assert_eq!(story_log_marks_an_edit(&page).await, (true, true));
}

// The stub answers the save with a 500, so the shipped save click exercises the
// recovery path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.4
async fn check_failed_save_restores_entry_and_releases_the_edit_marks(
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
        story_log_marks_an_edit(&page).await,
        (true, true),
        "edit mode should mark the entry and the story log"
    );

    page.locator("#edit-textarea")
        .await
        .fill("Modified text that fails to save", None)
        .await
        .unwrap();
    page.locator("[data-editing] .save-btn")
        .await
        .click(None)
        .await
        .unwrap();

    wait_for_error_disclosure(&page, NOTICE, "a failed save should report in the notice").await;
    let (_, save_message, save_raw) = read_error_disclosure(&page, NOTICE).await;
    assert_eq!(
        save_message, "The edit was not saved.",
        "the failed save shows the first line of the server's answer"
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
                    const edit = entry.querySelector('.edit-btn');
                    return !!edit && getComputedStyle(edit).display !== 'none' &&
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

    assert_eq!(
        story_log_marks_an_edit(&page).await,
        (false, false),
        "no entry should be marked as in edit after a failed save"
    );
}

// The stub answers the retry route with a 500, so the shipped retry click
// exercises the recovery path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.5
async fn check_failed_retry_reports_in_the_notice_and_re_enables_send(
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
    wait_for_error_disclosure(&page, NOTICE, "a failed retry should report in the notice").await;
    let (_, message, raw) = read_error_disclosure(&page, NOTICE).await;
    assert_eq!(
        message, "The Retry was not started.",
        "the notice names the failed action"
    );
    assert!(
        raw.contains("Stub retry failure"),
        "the server's own text belongs in the disclosure, got {raw:?}"
    );
    assert!(
        !error_details_open(&page, NOTICE).await,
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
async fn check_escape_cancels_edit(page: playwright_rs::Page, stub: StubServer) {
    install_story_load_counter(&page).await;
    let original_text = narration_text(&page).await;
    assert!(!original_text.is_empty(), "Should have original text");

    enter_narration_edit(&page).await;
    let textarea = page.locator("#edit-textarea").await;
    textarea
        .fill("Modified text that Escape abandons", None)
        .await
        .unwrap();
    // A number moved during the edit is what the log loads once the edit ends.
    stub.changes_handle().bump(ChangeKind::STORY);
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

    let loaded = wait_for_condition_async(
        Duration::from_millis(4000),
        Duration::from_millis(20),
        || async { story_load_count(&page).await >= 1.0 },
    )
    .await;
    assert!(loaded, "the held load never ran after Escape");
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
        "the load after the edit must keep focus on the entry's edit button"
    );
}

// The stub answers the first two saves with a 500, so a recorded request proves the
// shortcut clicked the Save of the entry and sent the typed text; the failure recovery
// is covered by 30.4. The third save succeeds with its screen load held: the old
// textarea is still in the page, and a shortcut there sends nothing.
// [docs/specs/browser_story_log.md] SCENARIO: 30.8
async fn check_keyboard_save_shortcuts_submit_edit(page: playwright_rs::Page, stub: StubServer) {
    page.evaluate::<(), ()>(
        r#"(() => {
                window.__saveRequests = [];
                document.body.addEventListener('htmx:beforeRequest', (evt) => {
                    window.__saveRequests.push(evt.detail.requestConfig.path);
                });
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
    assert_eq!(
        stub.save_handle().saved_texts(),
        ["Ctrl+Enter text", "Cmd+Enter text"],
        "each save posts the text of the textarea"
    );

    stub.save_handle().set_save_succeeds(true);
    wait_for_screen_current(&page).await;
    hold_screen_loads(&page).await;
    enter_narration_edit(&page).await;
    let textarea = page.locator("#edit-textarea").await;
    textarea.fill("Saved text", None).await.unwrap();
    textarea.press("Control+Enter", None).await.unwrap();
    let ended = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { !story_log_marks_an_edit(&page).await.0 },
    )
    .await;
    assert!(
        ended,
        "a done save ends the edit state while the screen load is held"
    );
    textarea.press("Control+Enter", None).await.unwrap();
    release_screen_loads(&page).await;
    wait_until_hidden(&page, "#edit-textarea", Duration::from_secs(5)).await;

    let requests: Vec<String> = page
        .evaluate::<(), Vec<String>>(
            "(() => window.__saveRequests.filter((url) => url.startsWith('/history/')))()",
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        requests.len(),
        3,
        "each saving shortcut sends one save request, and a shortcut after a done save sends none, got {requests:?}"
    );
    assert!(
        requests.iter().all(|url| url.ends_with("/history/1")),
        "save requests should target the edited entry, got {requests:?}"
    );
}

const SWIPE_CONTROLS_ARE_LOCKED: &str = r#"(() => {
    const entry = document.querySelector('.log-entry.narration');
    const buttons = entry.querySelectorAll('.swipe-controls .action-btn');
    return entry.hasAttribute('data-editing') && buttons.length > 0
        && Array.from(buttons).every((b) => getComputedStyle(b).pointerEvents === 'none');
})()"#;

/// The click guard of the story log stops a click on a locked control before htmx sees it.
const CLICK_RETRY_AND_COUNT: &str = r#"(() => {
    if (window.__retries === undefined) {
        window.__retries = 0;
        document.body.addEventListener('htmx:beforeRequest', (evt) => {
            if (evt.detail.requestConfig.path === '/swipe/new') window.__retries += 1;
        });
    }
    const before = window.__retries;
    const retry = document.querySelector('.log-entry.narration .swipe-controls .swipe-btn:last-child');
    retry.click();
    return window.__retries - before;
})()"#;

// [docs/specs/browser_story_log.md] SCENARIO: 30.9
async fn check_edit_locks_entry_controls_and_cancel_restores_them(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    enter_narration_edit(&page).await;

    assert!(
        page.evaluate::<(), bool>(SWIPE_CONTROLS_ARE_LOCKED, None)
            .await
            .unwrap(),
        "the entry's swipe controls should show as locked while editing"
    );
    let clicked = page
        .evaluate::<(), u32>(CLICK_RETRY_AND_COUNT, None)
        .await
        .unwrap();
    assert_eq!(clicked, 0, "a click on a locked swipe control does nothing");
    let edit_hidden = page
        .evaluate::<(), bool>(
            "(() => getComputedStyle(document.querySelector('.log-entry.narration .edit-btn')).display === 'none')()",
            None,
        )
        .await
        .unwrap();
    assert!(
        edit_hidden,
        "the edit control should be hidden while editing"
    );

    // Read in the same tick: the load that follows is async, so this observes
    // `revertEdit`'s synchronous restoration.
    let (edit_back, retry_clicks): (bool, u32) = page
        .evaluate::<(), (bool, u32)>(
            &format!(
                r#"(() => {{
                    document.querySelector('[data-editing] .cancel-btn').click();
                    const entry = document.querySelector('.log-entry.narration');
                    const edit = entry.querySelector('.edit-btn');
                    return [getComputedStyle(edit).display !== 'none', {CLICK_RETRY_AND_COUNT}];
                }})()"#
            ),
            None,
        )
        .await
        .unwrap();
    assert!(
        edit_back,
        "cancel should restore the entry's edit control immediately"
    );
    assert_eq!(
        retry_clicks, 1,
        "cancel should unlock the entry's Retry swipe control"
    );

    wait_until_hidden(&page, "#edit-textarea", Duration::from_millis(500)).await;
}

/// The Save control as the player sees it: whether it looks disabled, and the
/// text its hover shows.
async fn save_control_look(page: &playwright_rs::Page) -> (bool, String) {
    page.locator("[data-editing] .save-btn")
        .await
        .hover(None)
        .await
        .unwrap();
    page.evaluate::<(), (bool, String)>(
        r#"(() => {
            const save = document.querySelector('[data-editing] .save-btn');
            const style = getComputedStyle(save);
            const hover = getComputedStyle(save, '::after').content;
            return [
                Number(style.opacity) < 1 && style.cursor === 'not-allowed',
                hover === 'none' || hover === 'normal' ? '' : hover.replace(/^"|"$/g, ''),
            ];
        })()"#,
        None,
    )
    .await
    .unwrap()
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.24
async fn check_the_save_of_an_edit_looks_disabled_while_the_game_is_busy(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    use chronicler_engine::domain::model::state::generation_status::Step;

    enter_narration_edit(&page).await;
    let status = stub.status_handle();

    for busy in [
        StubStatus::Step(Step::Narrating),
        StubStatus::Autoplay {
            done: 1,
            budget: 3,
            step: None,
        },
    ] {
        status.set(busy);
        let marked = wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async {
                page.evaluate::<(), bool>(
                    "(() => !!document.querySelector('#status-display [data-status=\"thinking\"], #status-display [data-status=\"autoplay\"]'))()",
                    None,
                )
                .await
                .unwrap_or(false)
            },
        )
        .await;
        assert!(marked, "the status display shows the busy Game");
        let (disabled, hover) = save_control_look(&page).await;
        assert!(
            disabled,
            "the Save control looks disabled while the Game is busy"
        );
        assert_eq!(hover, "Wait for the narration to finish");
        status.set(StubStatus::Idle);
        let ready = wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async {
                page.evaluate::<(), bool>(
                    "(() => !!document.querySelector('#status-display [data-status=\"idle\"]'))()",
                    None,
                )
                .await
                .unwrap_or(false)
            },
        )
        .await;
        assert!(ready, "the status display shows Ready again");
        let (disabled, hover) = save_control_look(&page).await;
        assert!(
            !disabled,
            "the Save control looks enabled while the Game is idle"
        );
        assert_eq!(hover, "", "an enabled Save has no waiting text");
    }

    status.set(StubStatus::Error(
        chronicler_engine::domain::model::state::generation_status::GenerationFailureKind::Other,
        "narration failed".to_string(),
    ));
    let failed = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            page.evaluate::<(), bool>(
                "(() => !!document.querySelector('#status-display [data-status=\"failed\"]'))()",
                None,
            )
            .await
            .unwrap_or(false)
        },
    )
    .await;
    assert!(failed, "the status display shows the failure");
    let (disabled, hover) = save_control_look(&page).await;
    assert!(
        !disabled,
        "the Save control does not look disabled when the status display shows a failure"
    );
    assert_eq!(hover, "", "an enabled Save has no waiting text");
}

// The stub answers `/retrigger` with a 500, so the click runs the shipped
// retrigger control against the real route shape.
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
    wait_for_error_disclosure(
        &page,
        NOTICE,
        "a failed retrigger should report in the notice",
    )
    .await;
    let (_, message, raw) = read_error_disclosure(&page, NOTICE).await;
    assert_eq!(
        message, "The retrigger was not started.",
        "the notice names the failed action"
    );
    assert!(
        raw.contains("Stub retrigger failure"),
        "the server's own text belongs in the disclosure, got {raw:?}"
    );
    assert!(
        !error_details_open(&page, NOTICE).await,
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
            document.body.addEventListener('htmx:afterRequest', (evt) => {
                if (evt.detail.requestConfig.path.startsWith('/history/')) window.__saveDone = true;
            });
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
    if (!other) return false;
    const style = getComputedStyle(other);
    return document.getElementById('story-log').hasAttribute('data-editing-entry')
        && style.pointerEvents === 'none'
        && Number(style.opacity) < 1;
})()"#;

const OTHER_EDIT_UNLOCKED: &str = r#"(() => {
    const other = document.querySelector('.log-entry.input .edit-btn');
    if (!other) return false;
    return !document.getElementById('story-log').hasAttribute('data-editing-entry')
        && getComputedStyle(other).pointerEvents !== 'none';
})()"#;

/// A click on the other entry's Edit control, counted instead of run: the click
/// guard stops a click on a locked control before its own handler runs.
const CLICK_OTHER_EDIT_AND_COUNT: &str = r#"(() => {
    window.__editOpens = 0;
    window.showEditForm = () => { window.__editOpens += 1; };
    document.querySelector('.log-entry.input .edit-btn').click();
    return window.__editOpens;
})()"#;

// [docs/specs/browser_story_log.md] SCENARIO: 30.15
async fn check_edit_locks_the_other_entries_edit_controls(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    enter_narration_edit(&page).await;

    assert!(
        page.evaluate::<(), bool>(OTHER_EDIT_LOCKED, None)
            .await
            .unwrap(),
        "the other entries' Edit controls must show as locked during an edit"
    );
    assert_eq!(
        page.evaluate::<(), u32>(CLICK_OTHER_EDIT_AND_COUNT, None)
            .await
            .unwrap(),
        0,
        "a click on a locked Edit control must not open a second editor"
    );

    // Read in the same tick: the load that follows is async.
    let cancel_then_read = format!(
        "(() => {{ document.querySelector('[data-editing] .cancel-btn').click(); return {OTHER_EDIT_UNLOCKED}; }})()"
    );
    let unlocked = page
        .evaluate::<(), bool>(&cancel_then_read, None)
        .await
        .unwrap();
    assert!(
        unlocked,
        "cancel must unlock the other entries' Edit controls immediately"
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
        "the other entries' Edit controls must show as locked during an edit"
    );

    page.locator("[data-editing] .save-btn")
        .await
        .click(None)
        .await
        .unwrap();
    assert!(
        wait_for_save_completion(&page).await,
        "the failed save request never completed"
    );

    let unlocked = page
        .evaluate::<(), bool>(OTHER_EDIT_UNLOCKED, None)
        .await
        .unwrap();
    assert!(
        unlocked,
        "a failed save must unlock the other entries' Edit controls"
    );
}

// The lock must outlive a successful save: releasing it early would let a
// second editor open on the stale textarea.
// [docs/specs/browser_story_log.md] SCENARIO: 30.17
async fn check_successful_save_holds_the_edit_lock_until_the_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let save = stub.save_handle();
    save.set_save_succeeds(true);
    arm_save_completion(&page).await;
    page.evaluate::<(), ()>(
        r#"(() => {
                    window.__earlyUnlock = false;
                    const log = document.getElementById('story-log');
                    const observer = new MutationObserver(() => {
                        if (!document.querySelector('#edit-textarea')) return;
                        const swipe = document.querySelector('.swipe-controls .action-btn');
                        if (
                            !log.hasAttribute('data-editing-entry')
                            || (swipe && getComputedStyle(swipe).pointerEvents !== 'none')
                        ) {
                            window.__earlyUnlock = true;
                        }
                    });
                    observer.observe(log, {
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
        "the other entries' Edit controls must show as locked during an edit"
    );

    page.locator("[data-editing] .save-btn")
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
                                && getComputedStyle(other).pointerEvents !== 'none'
                                && !document.getElementById('story-log').hasAttribute('data-editing-entry');
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
        "the load after the save must re-render the log and unlock the Edit controls"
    );

    let early_unlock = page
        .evaluate::<(), bool>("(() => window.__earlyUnlock === true)()", None)
        .await
        .unwrap();
    assert!(
        !early_unlock,
        "a successful save must hold the edit lock until the load re-renders the log"
    );
}

// The stub answers POST /history/delete with a 500, so the shipped delete click
// exercises the failure path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.18
async fn check_failed_delete_reports_in_the_notice(page: playwright_rs::Page, _stub: StubServer) {
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

    wait_for_error_disclosure(&page, NOTICE, "a failed delete should report in the notice").await;
    let (_, message, raw) = read_error_disclosure(&page, NOTICE).await;
    assert_eq!(
        message, "The Message was not deleted.",
        "the notice names the failed action"
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

// The page keeps the Delete of the entry it rendered until the next load. The stub
// adds a Narration after that render, so the Delete names a Message that is no longer last.
// [docs/specs/browser_story_log.md] SCENARIO: 30.26
async fn check_a_delete_of_a_message_that_is_no_longer_last_reports_in_the_notice(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    page.on_dialog(|dialog| async move { dialog.accept(None).await })
        .await
        .unwrap();
    start_current(&page).await;
    let entries_before = count_log_entries(&page).await;
    assert!(
        entries_before > 1,
        "the fixture must show an entry that carries a delete control"
    );
    // A load after the append would show the new Narration and its Delete, which names
    // the true last Message.
    stop_page_timer(&page).await;
    stub.story_log_handle()
        .append_narration("A Narration that another tab added.");

    page.locator("#story-log .log-entry:last-child .delete-btn")
        .await
        .click(None)
        .await
        .unwrap();

    wait_for_error_disclosure(&page, NOTICE, "a stale delete should report in the notice").await;
    let (_, message, raw) = read_error_disclosure(&page, NOTICE).await;
    assert_eq!(
        message, "The last Message changed, so the Message was not deleted.",
        "the notice names the changed Message"
    );
    assert!(
        raw.contains("The page showed a Message that is no longer the last one."),
        "the engine's line belongs in the disclosure, got {raw:?}"
    );
    load_screen_now(&page).await;
    assert_eq!(
        count_log_entries(&page).await,
        entries_before + 1,
        "the refused delete removed nothing, and the load shows the added Narration"
    );
}

// The stub answers a swipe switch with a 500 once the test asks for the failure
// path, so the shipped previous-swipe click runs the real client path.
// [docs/specs/browser_story_log.md] SCENARIO: 30.19
async fn check_failed_swipe_switch_reports_in_the_notice(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let swipe = stub.swipe_handle();
    swipe.set_two_swipes();
    swipe.set_switch_failing();
    load_screen_now(&page).await;

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

    wait_for_error_disclosure(
        &page,
        NOTICE,
        "a failed swipe switch should report in the notice",
    )
    .await;
    let (_, message, raw) = read_error_disclosure(&page, NOTICE).await;
    assert_eq!(
        message, "The Swipe was not switched.",
        "the notice names the failed action"
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

const EDIT_SLOT: &str = "[data-editing] .edit-actions > [data-error-slot]";

// [docs/specs/browser_story_log.md] SCENARIO: 30.25
async fn check_an_invalid_save_reports_in_the_edit_slot_and_keeps_the_draft(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.save_handle().set_save_invalid(true);
    enter_narration_edit(&page).await;
    page.locator("#edit-textarea")
        .await
        .fill("A draft that the engine does not accept", None)
        .await
        .unwrap();
    page.locator("[data-editing] .save-btn")
        .await
        .click(None)
        .await
        .unwrap();

    wait_for_error_disclosure(
        &page,
        EDIT_SLOT,
        "an invalid save reports in the slot of the entry",
    )
    .await;
    let (_, line, _) = read_error_disclosure(&page, EDIT_SLOT).await;
    assert_eq!(line, "The edit was not saved: The value is not allowed.");
    assert!(
        !read_error_disclosure(&page, NOTICE).await.0,
        "an invalid save of a message edit shows in its slot, not in the notice"
    );
    assert_eq!(
        page.locator("#edit-textarea")
            .await
            .input_value(None)
            .await
            .unwrap(),
        "A draft that the engine does not accept",
        "the draft stays"
    );
    assert_eq!(
        story_log_marks_an_edit(&page).await,
        (true, true),
        "the entry stays in edit and the lock stays"
    );

    page.locator("[data-editing] .cancel-btn")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, "#edit-textarea", Duration::from_millis(500)).await;
    let slot = ".log-entry.narration .edit-actions > [data-error-slot]";
    let cleared = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { !read_error_disclosure(&page, slot).await.0 },
    )
    .await;
    assert!(
        cleared,
        "Cancel clears the slot: the load that follows renders it empty"
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
        .run(
            StubActionOutcome::Pending,
            check_an_entry_in_edit_survives_loads_of_the_story_log,
        )
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
    runner
        .run(
            StubActionOutcome::Pending,
            check_the_save_of_an_edit_looks_disabled_while_the_game_is_busy,
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
            check_failed_save_restores_entry_and_releases_the_edit_marks,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_an_invalid_save_reports_in_the_edit_slot_and_keeps_the_draft,
        )
        .await;

    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_retry_reports_in_the_notice_and_re_enables_send,
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
            check_successful_save_holds_the_edit_lock_until_the_load,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_delete_reports_in_the_notice,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_delete_of_a_message_that_is_no_longer_last_reports_in_the_notice,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_swipe_switch_reports_in_the_notice,
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
