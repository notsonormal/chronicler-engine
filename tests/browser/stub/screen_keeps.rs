//! Stub-browser tests for what a screen load keeps: the story-log entry in edit, the engaged state of each part, and an inline error that shows. Tagged against `docs/specs/browser_screen.md`.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;
use chronicler_engine::domain::model::state::generation_status::GenerationFailureKind;

use super::support::{
    ask_once_without_waiting, eval_string, hold_screen_loads, open_section, read_banner,
    release_screen_loads, start_current, wait_for_held_screen_load, StubRunner,
};
use super::*;
use crate::test_utils::screen_part_table::PARTS;

const OPEN_EDIT: &str = r#"() => {
    document.querySelector('.log-entry.narration .edit-btn').click();
    const textarea = document.getElementById('edit-textarea');
    textarea.value = 'typed into the edit';
    textarea.focus();
    window.__editTextarea = textarea;
    return '';
}"#;

const EDIT_IS_INTACT: &str = r#"() => {
    const textarea = document.getElementById('edit-textarea');
    if (!textarea) return 'the textarea is gone';
    if (textarea !== window.__editTextarea) return 'the textarea is a new element';
    if (textarea.value !== 'typed into the edit') return 'the text changed: ' + textarea.value;
    if (document.activeElement !== textarea) return 'the textarea lost focus';
    return '';
}"#;

// [docs/specs/browser_screen.md] SCENARIO: 44.9
async fn check_a_load_skips_the_entry_in_edit_and_shows_the_rest_of_the_log(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    eval_string(&page, OPEN_EDIT).await;
    stub.story_log_handle()
        .append_narration("Narration that arrived during the edit.");
    let before = changes.loads("story-log");
    for _ in 0..3 {
        changes.bump(ChangeKind::STORY);
        ask_once(&page).await;
    }
    assert_eq!(
        changes.loads("story-log") - before,
        3,
        "the story log loads for each move, edit or no edit"
    );
    assert_eq!(eval_string(&page, EDIT_IS_INTACT).await, "");
    assert!(
        eval_string(
            &page,
            "() => document.getElementById('story-log').innerText.includes('Narration that arrived during the edit.') ? '' : 'the new narration is missing'"
        )
        .await
        .is_empty(),
        "the rest of the log loads around the entry in edit"
    );

    page.evaluate::<(), ()>("() => cancelEdit()", None)
        .await
        .unwrap();
    wait_for_screen_current(&page).await;
    assert_eq!(
        eval_string(
            &page,
            "() => document.getElementById('edit-textarea') ? 'the textarea is still there' : ''"
        )
        .await,
        ""
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.10
async fn check_a_load_in_flight_when_an_edit_began_skips_the_entry_in_edit(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    stub.story_log_handle()
        .append_narration("Narration in the answer that was in flight.");
    hold_screen_loads(&page).await;
    let before = changes.loads("story-log");
    changes.bump(ChangeKind::STORY);
    ask_once_without_waiting(&page).await;
    wait_for_held_screen_load(&page).await;
    assert_eq!(
        eval_string(&page, OPEN_EDIT).await,
        "",
        "the edit must open while the load is in flight"
    );
    release_screen_loads(&page).await;
    wait_for_screen_current(&page).await;

    assert_eq!(changes.loads("story-log"), before + 1);
    assert_eq!(eval_string(&page, EDIT_IS_INTACT).await, "");
    assert!(
        eval_string(
            &page,
            "() => document.getElementById('story-log').innerText.includes('Narration in the answer that was in flight.') ? '' : 'the answer was not swapped in'"
        )
        .await
        .is_empty(),
        "the answer is swapped in around the entry in edit"
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.11
async fn check_a_disclosure_that_lists_its_open_state_stays_open_through_a_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    stub.options_handle().set_failure(true);
    start_current(&page).await;

    eval_string(
        &page,
        r#"() => {
            const details = document.querySelector('#options-dock details');
            details.open = true;
            window.__keptDetails = details;
            return '';
        }"#,
    )
    .await;
    let before = changes.loads("options-dock");
    changes.bump(ChangeKind::OPTIONS);
    ask_once(&page).await;
    assert_eq!(changes.loads("options-dock"), before + 1);
    let state = eval_string(
        &page,
        r#"() => {
            const details = document.querySelector('#options-dock details');
            if (details !== window.__keptDetails) return 'a new element';
            return details.open ? '' : 'closed';
        }"#,
    )
    .await;
    assert_eq!(
        state, "",
        "the Details disclosure must be the same open element"
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.12
async fn check_focus_moves_to_a_surviving_row_when_a_load_removes_the_focused_node(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    page.locator("#edit-entry-1").await.focus().await.unwrap();
    stub.story_log_handle().remove_newest();
    let before = changes.loads("story-log");
    ask_once(&page).await;
    assert_eq!(changes.loads("story-log"), before + 1);

    let landing = eval_string(
        &page,
        r#"() => {
            if (document.getElementById('entry-1')) return 'the newest entry is still there';
            const focused = document.activeElement;
            const entry = focused && focused.closest ? focused.closest('#story-log .log-entry') : null;
            return entry ? entry.id : 'focus is on ' + (focused ? focused.tagName : 'nothing');
        }"#,
    )
    .await;
    assert_eq!(
        landing, "entry-2",
        "focus must land in the entry that is now the newest"
    );
}

/// What a part keeps through a load, and how the check engages it. `engage`
/// puts the controls of the part in their engaged state and may remember
/// nodes in `window`. `check` answers "" when the engaged state is as it was.
/// `end` ends the edits the engaged state holds.
pub(super) struct Engaged {
    part: &'static str,
    kinds: &'static [ChangeKind],
    pub(super) engage: &'static str,
    pub(super) check: &'static str,
    end: Option<&'static str>,
}

async fn assert_engaged_state_survives(
    page: &playwright_rs::Page,
    changes: &StubChangesHandle,
    engaged: &Engaged,
) {
    let part = PARTS
        .iter()
        .find(|part| part.name == engaged.part)
        .expect("the engaged part is in the table");
    let problem = eval_string(page, engaged.engage).await;
    assert_eq!(problem, "", "{}: engaging", part.selector);

    let before = changes.loads(part.name);
    for kind in engaged.kinds {
        changes.bump(*kind);
        ask_once(page).await;
        let problem = eval_string(page, engaged.check).await;
        assert_eq!(problem, "", "{} after `{kind}` moved", part.selector);
    }
    let loaded = changes.loads(part.name) - before;
    assert_eq!(
        loaded,
        engaged.kinds.len(),
        "{} must load once for each change",
        part.selector
    );
    if let Some(end) = engaged.end {
        eval_string(page, end).await;
        wait_for_screen_current(page).await;
    }
}

const HEADER_ENGAGED: Engaged = Engaged {
    part: "header",
    kinds: &[ChangeKind::GAMES],
    engage: "() => { window.__headerNode = document.querySelector('#header > *'); return ''; }",
    check: "() => document.querySelector('#header > *') === window.__headerNode ? '' : 'a new element'",
    end: None,
};

const STORY_LOG_ENGAGED: Engaged = Engaged {
    part: "story-log",
    kinds: &[
        ChangeKind::STORY,
        ChangeKind::BUSY_STATE,
        ChangeKind::WORLDS,
    ],
    engage: OPEN_EDIT,
    check: EDIT_IS_INTACT,
    end: Some("() => { cancelEdit(); return ''; }"),
};

const SIDEBAR_ENGAGED: Engaged = Engaged {
    part: "visual-sidebar",
    kinds: &[ChangeKind::STORY, ChangeKind::WORLDS],
    engage: r#"() => {
        const label = document.querySelector('.visual-sidebar .image-label');
        const range = document.createRange();
        range.selectNodeContents(label);
        const selection = window.getSelection();
        selection.removeAllRanges();
        selection.addRange(range);
        window.__sidebarLabel = label;
        return selection.toString() === '' ? 'no selection' : '';
    }"#,
    check: r#"() => {
        const label = document.querySelector('.visual-sidebar .image-label');
        if (label !== window.__sidebarLabel) return 'a new element';
        return window.getSelection().toString() === label.textContent ? '' : 'the selection changed';
    }"#,
    end: None,
};

const OPTION_ENGAGED: Engaged = Engaged {
    part: "options-dock",
    kinds: &[ChangeKind::OPTIONS, ChangeKind::BUSY_STATE],
    engage: r#"() => {
        const option = document.querySelector('#options-dock .option-btn');
        option.focus();
        window.__optionButton = option;
        return document.activeElement === option ? '' : 'no focus';
    }"#,
    check: r#"() => {
        const option = document.querySelector('#options-dock .option-btn');
        if (option !== window.__optionButton) return 'a new element';
        return document.activeElement === option ? '' : 'the focus moved';
    }"#,
    end: None,
};

const OPTIONS_FAILURE_ENGAGED: Engaged = Engaged {
    part: "options-dock",
    kinds: &[ChangeKind::OPTIONS, ChangeKind::BUSY_STATE],
    engage: r#"() => {
        const details = document.querySelector('#options-dock details');
        details.open = true;
        window.__optionsDetails = details;
        return '';
    }"#,
    check: r#"() => {
        const details = document.querySelector('#options-dock details');
        if (details !== window.__optionsDetails) return 'a new element';
        return details.open ? '' : 'closed';
    }"#,
    end: None,
};

const STATUS_ENGAGED: Engaged = Engaged {
    part: "status-display",
    kinds: &[ChangeKind::BUSY_STATE],
    engage: r#"() => {
        document.querySelector('#status-display .error-details-toggle').click();
        const popover = document.querySelector('#status-display .error-detail-popover');
        window.__statusPopover = popover;
        return popover.hidden ? 'the popover did not open' : '';
    }"#,
    check: r#"() => {
        const popover = document.querySelector('#status-display .error-detail-popover');
        if (popover !== window.__statusPopover) return 'a new element';
        return popover.hidden ? 'closed' : '';
    }"#,
    end: None,
};

const LLM_ENGAGED: Engaged = Engaged {
    part: "llm-messages",
    kinds: &[ChangeKind::LLM_MESSAGES],
    engage: r#"() => {
        const open = document.querySelector('.llm-message-open');
        open.focus();
        window.__llmOpen = open;
        return document.activeElement === open ? '' : 'the Open control did not take focus';
    }"#,
    check: r#"() => {
        const open = document.querySelector('.llm-message-open');
        if (open !== window.__llmOpen) return 'the Open control is a new element';
        return document.activeElement === open ? '' : 'the focus moved';
    }"#,
    end: None,
};

const BANNER_ENGAGED: Engaged = Engaged {
    part: "failure-banner",
    kinds: &[ChangeKind::SETTINGS, ChangeKind::LLM_MESSAGES],
    engage: r#"() => {
        const toggle = document.querySelector('#failure-banner-degraded .error-details-toggle');
        toggle.click();
        toggle.focus();
        window.__bannerToggle = toggle;
        window.__bannerPopover = document.getElementById('failure-banner-popover');
        return window.__bannerPopover.hidden ? 'the popover did not open' : '';
    }"#,
    check: r#"() => {
        const toggle = document.querySelector('#failure-banner-degraded .error-details-toggle');
        const popover = document.getElementById('failure-banner-popover');
        if (toggle !== window.__bannerToggle) return 'the toggle is a new element';
        if (popover !== window.__bannerPopover) return 'the popover is a new element';
        if (popover.hidden) return 'the popover closed';
        if (toggle.getAttribute('aria-expanded') !== 'true') return 'aria-expanded changed';
        if (!popover.style.left) return 'the position of the popover was lost';
        return document.activeElement === toggle ? '' : 'the focus moved';
    }"#,
    end: None,
};

pub(super) const SUBTAB_ENGAGED: Engaged = Engaged {
    part: "settings",
    kinds: &[ChangeKind::SETTINGS, ChangeKind::LLM_MESSAGES],
    engage: r#"() => {
        const tab = document.getElementById('subtab-text-check');
        tab.click();
        window.__subtab = tab;
        window.__textPanel = document.getElementById('settings-text-check');
        return tab.getAttribute('aria-selected') === 'true' ? '' : 'the sub-tab did not open';
    }"#,
    check: r#"() => {
        const tab = document.getElementById('subtab-text-check');
        const textPanel = document.getElementById('settings-text-check');
        if (tab !== window.__subtab) return 'the sub-tab is a new element';
        if (textPanel !== window.__textPanel) return 'the Text Check panel is a new element';
        if (tab.getAttribute('aria-selected') !== 'true') return 'the sub-tab lost its selection';
        if (document.getElementById('subtab-connections').getAttribute('aria-selected') !== 'false') {
            return 'the Connections sub-tab is selected again';
        }
        if (textPanel.hidden) return 'the Text Check panel is hidden';
        return document.getElementById('settings-connections').hidden ? '' : 'the Connections panel shows';
    }"#,
    end: None,
};

// [docs/specs/browser_screen.md] SCENARIO: 44.13
async fn check_a_load_keeps_the_engaged_state_of_each_part(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    assert_engaged_state_survives(&page, &changes, &HEADER_ENGAGED).await;
    assert_engaged_state_survives(&page, &changes, &STORY_LOG_ENGAGED).await;
    assert_engaged_state_survives(&page, &changes, &SIDEBAR_ENGAGED).await;
    assert_engaged_state_survives(&page, &changes, &OPTION_ENGAGED).await;

    stub.options_handle().set_failure(true);
    ask_once(&page).await;
    assert_engaged_state_survives(&page, &changes, &OPTIONS_FAILURE_ENGAGED).await;

    stub.status_handle().set(StubStatus::Error(
        GenerationFailureKind::Other,
        "narration failed".to_string(),
    ));
    ask_once(&page).await;
    assert_engaged_state_survives(&page, &changes, &STATUS_ENGAGED).await;

    open_section(&page, "llm-messages", ".llm-messages-panel").await;
    assert_engaged_state_survives(&page, &changes, &LLM_ENGAGED).await;

    stub.failure_handle().set_header_degraded(true);
    ask_once(&page).await;
    // The load settles a moment after its request ends, and the banner shows then.
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async { read_banner(&page).await.0 },
        )
        .await,
        "the degraded banner must show before its disclosure is engaged"
    );
    assert_engaged_state_survives(&page, &changes, &BANNER_ENGAGED).await;

    open_section(&page, "settings", ".settings-panel").await;
    assert_engaged_state_survives(&page, &changes, &SUBTAB_ENGAGED).await;
}

const TYPE_TENSE: &str = r#"async () => {
    const select = document.querySelector('#game-posture-controls select[name="narrative_tense"]');
    const ended = new Promise((resolve) =>
        select.addEventListener('htmx:afterRequest', resolve, { once: true }));
    select.value = 'present';
    select.dispatchEvent(new Event('change', { bubbles: true }));
    await ended;
    await new Promise((resolve) => setTimeout(resolve, 0));
    return select.value;
}"#;

const SAVE_TENSE_UNCHANGED: &str = r#"async () => {
    const select = document.querySelector('#game-posture-controls select[name="narrative_tense"]');
    const ended = new Promise((resolve) =>
        select.addEventListener('htmx:afterRequest', resolve, { once: true }));
    select.dispatchEvent(new Event('change', { bubbles: true }));
    await ended;
    await new Promise((resolve) => setTimeout(resolve, 0));
    return select.value;
}"#;

// [docs/specs/browser_screen.md] SCENARIO: 44.18
async fn check_a_control_that_saves_on_change_goes_back_to_its_rendered_value_when_the_save_fails(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(
        eval_string(&page, TYPE_TENSE).await,
        "past",
        "the Tense select must show the rendered value again"
    );
    let (shown, line, _) =
        read_error_disclosure(&page, "#game-posture-controls [data-error-slot]").await;
    assert!(shown && line.contains("action failed"), "got {line:?}");
    ask_once(&page).await;
}

// [docs/specs/browser_screen.md] SCENARIO: 44.19
async fn check_a_load_keeps_an_inline_error_that_is_showing(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    let slot = "#game-posture-controls [data-error-slot]";
    eval_string(&page, TYPE_TENSE).await;
    let (shown, _, raw) = read_error_disclosure(&page, slot).await;
    assert!(shown, "the failed save must report in the control's slot");

    let before = changes.loads("games");
    changes.bump(ChangeKind::GAMES);
    ask_once(&page).await;
    assert_eq!(changes.loads("games"), before + 1, "the panel loads");
    let (still_shown, _, still_raw) = read_error_disclosure(&page, slot).await;
    assert!(still_shown, "a load must keep the error that is showing");
    assert_eq!(still_raw, raw);

    stub.games_handle().set_posture_saves_succeed(true);
    eval_string(&page, SAVE_TENSE_UNCHANGED).await;
    let (shown, _, _) = read_error_disclosure(&page, slot).await;
    assert!(!shown, "a save that goes through clears the error");
}

#[tokio::test]
async fn run_screen_keep_checks() {
    let mut runner = StubRunner::launch().await;
    let outcome = StubActionOutcome::Pending;
    runner
        .run(
            outcome,
            check_a_load_skips_the_entry_in_edit_and_shows_the_rest_of_the_log,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_load_in_flight_when_an_edit_began_skips_the_entry_in_edit,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_disclosure_that_lists_its_open_state_stays_open_through_a_load,
        )
        .await;
    runner
        .run(
            outcome,
            check_focus_moves_to_a_surviving_row_when_a_load_removes_the_focused_node,
        )
        .await;
    runner
        .run(outcome, check_a_load_keeps_the_engaged_state_of_each_part)
        .await;
    runner.run(outcome, check_a_control_that_saves_on_change_goes_back_to_its_rendered_value_when_the_save_fails).await;
    runner
        .run(outcome, check_a_load_keeps_an_inline_error_that_is_showing)
        .await;
    runner.finish().await;
}
