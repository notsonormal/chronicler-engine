//! Stub-browser tests for what a screen load keeps: the story-log entry in edit, the engaged state of each part, an inline error that shows, and the dialog forms that a load never reaches. Tagged against `docs/specs/browser_screen.md`, `docs/specs/browser_games.md` and `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;
use chronicler_engine::domain::model::state::generation_status::GenerationFailureKind;

use super::support::{
    ask_once_without_waiting, eval_string, hold_screen_loads, open_section, read_banner,
    release_screen_loads, start_current, wait_for_held_screen_load, StubRunner,
};
use super::*;
use crate::test_utils::screen_part_table::PARTS;

const GAMES_PANEL_KINDS: [ChangeKind; 3] = [
    ChangeKind::GAMES,
    ChangeKind::WORLDS,
    ChangeKind::PROMPT_PRESETS,
];

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
    let before = changes.loads("/fragment/story-log");
    for _ in 0..3 {
        changes.bump(ChangeKind::STORY);
        ask_once(&page).await;
    }
    assert_eq!(
        changes.loads("/fragment/story-log") - before,
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
    let before = changes.loads("/fragment/story-log");
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

    assert_eq!(changes.loads("/fragment/story-log"), before + 1);
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
    let before = changes.loads("/fragment/options-dock");
    changes.bump(ChangeKind::OPTIONS);
    ask_once(&page).await;
    assert_eq!(changes.loads("/fragment/options-dock"), before + 1);
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
    let before = changes.loads("/fragment/story-log");
    ask_once(&page).await;
    assert_eq!(changes.loads("/fragment/story-log"), before + 1);

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
struct Engaged {
    part: &'static str,
    kinds: &'static [ChangeKind],
    engage: &'static str,
    check: &'static str,
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

    let before = changes.loads(part.path);
    for kind in engaged.kinds {
        changes.bump(*kind);
        ask_once(page).await;
        let problem = eval_string(page, engaged.check).await;
        assert_eq!(problem, "", "{} after `{kind}` moved", part.selector);
    }
    let loaded = changes.loads(part.path) - before;
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

/// Runs `action`, and returns once the htmx swap that it started has settled.
/// htmx settles a swap 20 ms after it lands, and the page moves focus into a new
/// editor then, so a check that engages the editor waits for that settle first.
macro_rules! after_settle {
    () => {
        r#"const afterSettle = (action) => new Promise((resolve) => {
        document.body.addEventListener('htmx:afterSettle', function once(evt) {
                        document.body.removeEventListener('htmx:afterSettle', once);
            resolve();
        });
        action();
    });"#
    };
}

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

const SUBTAB_ENGAGED: Engaged = Engaged {
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

/// The dialog forms of the Games panel, opened the way the player does: a click
/// on the control, and a wait until htmx has settled the swap into the dialog body.
pub(super) const OPEN_RENAME: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('#game-row-2 button[hx-get$="/rename"]').click());
    const input = document.querySelector('#edit-dialog input[name="display_name"]');
    if (!input) return 'the dialog did not open with the rename form';
    input.value = 'A typed name';
    input.focus();
    window.__renameForm = document.querySelector('#edit-dialog .game-rename-form');
    window.__renameInput = input;
    return document.activeElement === input ? '' : 'the name field has no focus';
}"#
);

pub(super) const RENAME_IS_INTACT: &str = r#"() => {
    const dialog = document.getElementById('edit-dialog');
    const form = document.querySelector('#edit-dialog .game-rename-form');
    const input = form ? form.querySelector('input[name="display_name"]') : null;
    if (!dialog.open) return 'the dialog closed';
    if (form !== window.__renameForm) return 'the rename form is a new element';
    if (input !== window.__renameInput) return 'the name field is a new element';
    if (input.value !== 'A typed name') return 'the typed name changed: ' + input.value;
    if (document.activeElement !== input) return 'the name field lost focus';
    return '';
}"#;

/// Presses Save in the rename dialog, and returns once the request has ended and
/// every ask and load that followed has settled. The wait for a macrotask lets the
/// page's own `afterRequest` listener run first: it sits on the body, and this
/// listener sits on the form.
const SAVE_RENAME: &str = r#"async () => {
    const form = document.querySelector('#edit-dialog .game-rename-form form');
    const ended = new Promise((resolve) =>
        form.addEventListener('htmx:afterRequest', resolve, { once: true }));
    form.querySelector('button[type=submit]').click();
    await ended;
    await new Promise((resolve) => setTimeout(resolve, 0));
    await window.__waitForScreen();
    return '';
}"#;

pub(super) const OPEN_NEW_GAME: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('button[hx-get="/fragment/games/new"]').click());
    const world = document.getElementById('new-game-world');
    const persona = document.getElementById('new-game-persona');
    if (!world || !persona) return 'the dialog did not open with the New Game form';
    world.value = 'test';
    persona.value = 'test_player';
    window.__newGameForm = document.querySelector('#edit-dialog .new-game-form-dialog');
    window.__newGameWorld = world;
    window.__newGamePersona = persona;
    return world.value === 'test' && persona.value === 'test_player' ? '' : 'the choice did not take';
}"#
);

pub(super) const NEW_GAME_IS_INTACT: &str = r#"() => {
    const dialog = document.getElementById('edit-dialog');
    const world = document.getElementById('new-game-world');
    const persona = document.getElementById('new-game-persona');
    if (!dialog.open) return 'the dialog closed';
    if (document.querySelector('#edit-dialog .new-game-form-dialog') !== window.__newGameForm) return 'the New Game form is a new element';
    if (world !== window.__newGameWorld) return 'the World select is a new element';
    if (persona !== window.__newGamePersona) return 'the Persona select is a new element';
    if (world.value !== 'test') return 'the World choice changed: ' + world.value;
    if (persona.value !== 'test_player') return 'the Persona choice changed: ' + persona.value;
    return '';
}"#;

const NEW_GAME_IS_RENDERED: &str = r#"() => {
    const world = document.getElementById('new-game-world');
    const persona = document.getElementById('new-game-persona');
    if (world.selectedIndex !== 0) return 'the World select keeps the choice: ' + world.value;
    if (persona.selectedIndex !== 0) return 'the Persona select keeps the choice: ' + persona.value;
    return '';
}"#;

pub(super) const CANCEL_DIALOG: &str = r#"async () => {
    const dialog = document.getElementById('edit-dialog');
    if (!dialog.open) return 'the dialog is not open';
    const closed = new Promise((resolve) =>
        dialog.addEventListener('close', resolve, { once: true }));
    dialog.querySelector('[data-close-dialog]').click();
    await closed;
    await window.__waitForScreen();
    if (dialog.open) return 'the dialog is still open';
    return document.getElementById('edit-dialog-body').children.length === 0
        ? '' : 'the dialog body still holds the form';
}"#;

async fn assert_games_panel_loads_around_the_dialog(
    page: &playwright_rs::Page,
    changes: &StubChangesHandle,
    check: &str,
) {
    let before = changes.loads("/fragment/games");
    for kind in GAMES_PANEL_KINDS {
        changes.bump(kind);
        ask_once(page).await;
        assert_eq!(eval_string(page, check).await, "", "after `{kind}` moved");
    }
    assert_eq!(
        changes.loads("/fragment/games"),
        before + GAMES_PANEL_KINDS.len(),
        "the panel loads for each kind that moved"
    );
}

// [docs/specs/browser_games.md] SCENARIO: 27.4
async fn check_an_open_rename_dialog_keeps_its_text_and_focus_through_a_load_of_each_kind(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    assert_games_panel_loads_around_the_dialog(&page, &changes, RENAME_IS_INTACT).await;
}

// [docs/specs/browser_games.md] SCENARIO: 27.5
async fn check_saving_the_rename_dialog_closes_it_and_the_games_panel_shows_the_name(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    let before = changes.loads("/fragment/games");
    assert_eq!(eval_string(&page, SAVE_RENAME).await, "");
    assert_eq!(
        changes.loads("/fragment/games"),
        before + 1,
        "the saved rename must load the panel exactly once"
    );
    assert_eq!(
        eval_string(
            &page,
            "() => document.getElementById('edit-dialog').open ? 'the dialog is still open' : ''"
        )
        .await,
        "",
        "an accepted save closes the dialog"
    );
    let shown = page
        .locator("#game-row-2 .game-name")
        .await
        .inner_text()
        .await
        .unwrap();
    assert_eq!(shown, "A typed name", "the load shows the renamed Game");
}

// [docs/specs/browser_games.md] SCENARIO: 27.6
async fn check_the_new_game_selects_keep_the_players_choice_through_a_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(eval_string(&page, OPEN_NEW_GAME).await, "");
    assert_games_panel_loads_around_the_dialog(&page, &changes, NEW_GAME_IS_INTACT).await;
}

// [docs/specs/browser_games.md] SCENARIO: 27.7
async fn check_cancelling_the_new_game_dialog_discards_the_choice(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(eval_string(&page, OPEN_NEW_GAME).await, "");
    changes.bump(ChangeKind::WORLDS);
    ask_once(&page).await;
    assert_eq!(eval_string(&page, NEW_GAME_IS_INTACT).await, "");

    assert_eq!(eval_string(&page, CANCEL_DIALOG).await, "");
    assert_eq!(
        eval_string(&page, OPEN_NEW_GAME_WITHOUT_CHOOSING).await,
        "",
        "the New Game control opens a fresh form"
    );
    assert_eq!(
        eval_string(&page, NEW_GAME_IS_RENDERED).await,
        "",
        "the form that opens again shows the first options, not the discarded choice"
    );
}

const OPEN_NEW_GAME_WITHOUT_CHOOSING: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('button[hx-get="/fragment/games/new"]').click());
    return document.getElementById('new-game-world') ? '' : 'the dialog did not open with the New Game form';
}"#
);

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

// [docs/specs/browser_screen.md] SCENARIO: 44.17
async fn check_a_save_that_fails_keeps_the_dialog_open_with_its_typed_values(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    stub.games_handle().set_rename_failing(true);
    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    assert_eq!(eval_string(&page, SAVE_RENAME).await, "");
    let (shown, line, raw) = read_error_disclosure(
        &page,
        "#edit-dialog .game-rename-form form [data-error-slot]",
    )
    .await;
    assert!(shown, "a failed save must report in the form's own slot");
    assert!(line.contains("action failed"), "got {line:?}");
    assert!(raw.contains("Stub rename failure"), "got {raw:?}");
    assert_eq!(
        eval_string(&page, RENAME_IS_INTACT).await,
        "",
        "the dialog stays open with the same form and the typed name"
    );
}

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

    let before = changes.loads("/fragment/games");
    changes.bump(ChangeKind::GAMES);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("/fragment/games"),
        before + 1,
        "the panel loads"
    );
    let (still_shown, _, still_raw) = read_error_disclosure(&page, slot).await;
    assert!(still_shown, "a load must keep the error that is showing");
    assert_eq!(still_raw, raw);

    stub.games_handle().set_posture_saves_succeed(true);
    eval_string(&page, SAVE_TENSE_UNCHANGED).await;
    let (shown, _, _) = read_error_disclosure(&page, slot).await;
    assert!(!shown, "a save that goes through clears the error");
}

const OPEN_PRESET_EDIT: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('button[hx-get="/fragment/prompt-presets/custom_ref/edit"]').click());
    const input = document.getElementById('edit-name-custom_ref');
    if (!input) return 'the dialog did not open with the edit form';
    input.value = 'A typed preset name';
    input.focus();
    window.__presetForm = document.querySelector('#edit-dialog .preset-form');
    window.__presetInput = input;
    return document.activeElement === input ? '' : 'the name field has no focus';
}"#
);

const PRESET_EDIT_IS_INTACT: &str = r#"() => {
    const form = document.querySelector('#edit-dialog .preset-form');
    const input = document.getElementById('edit-name-custom_ref');
    if (!document.getElementById('edit-dialog').open) return 'the dialog closed';
    if (form !== window.__presetForm) return 'the form is a new element';
    if (input !== window.__presetInput) return 'the name field is a new element';
    if (input.value !== 'A typed preset name') return 'the typed name changed: ' + input.value;
    return document.activeElement === input ? '' : 'the name field lost focus';
}"#;

// [docs/specs/browser_screen.md] SCENARIO: 44.20
async fn check_a_preset_form_in_the_dialog_survives_a_presets_load_and_cancel_sends_no_request(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "prompt-presets", ".prompt-presets-panel").await;

    assert_eq!(eval_string(&page, OPEN_PRESET_EDIT).await, "");
    let loads = changes.loads("/fragment/prompt-presets");
    changes.bump(ChangeKind::PROMPT_PRESETS);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("/fragment/prompt-presets"),
        loads + 1,
        "an open form holds no part, so the panel loads"
    );
    assert_eq!(
        eval_string(&page, PRESET_EDIT_IS_INTACT).await,
        "",
        "the load of the panel leaves the dialog alone"
    );

    start_counting_requests(&page).await;
    assert_eq!(eval_string(&page, CANCEL_DIALOG).await, "");
    assert_eq!(requests_counted(&page).await, 0, "Cancel sends no request");
}

const OPEN_WORLD_EDITOR: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('#world-test button[hx-get$="/edit"]').click());
    const input = document.querySelector('#edit-dialog input[name="name"]');
    if (!input) return 'the dialog did not open with the World form';
    input.value = 'A typed world name';
    input.focus();
    window.__worldForm = document.getElementById('world-form-container');
    window.__worldInput = input;
    return document.activeElement === input ? '' : 'the name field has no focus';
}"#
);

const WORLD_EDITOR_IS_INTACT: &str = r#"() => {
    const form = document.getElementById('world-form-container');
    const input = document.querySelector('#edit-dialog input[name="name"]');
    if (!document.getElementById('edit-dialog').open) return 'the dialog closed';
    if (form !== window.__worldForm) return 'the form is a new element';
    if (input !== window.__worldInput) return 'the name field is a new element';
    if (input.value !== 'A typed world name') return 'the typed name changed: ' + input.value;
    return document.activeElement === input ? '' : 'the name field lost focus';
}"#;

// [docs/specs/browser_screen.md] SCENARIO: 44.21
async fn check_a_world_form_in_the_dialog_survives_a_game_rename_that_raises_games(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "worlds", ".worlds-panel").await;

    assert_eq!(eval_string(&page, OPEN_WORLD_EDITOR).await, "");
    let loads = changes.loads("/fragment/worlds");
    let renamed = eval_string(
        &page,
        r#"async () => {
            const response = await actionFetch('/games/2/rename', {
                method: 'POST',
                headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                body: 'display_name=Renamed+elsewhere',
            });
            await window.__waitForScreen();
            return response.ok ? '' : 'the rename failed';
        }"#,
    )
    .await;
    assert_eq!(renamed, "");

    assert_eq!(
        eval_string(&page, WORLD_EDITOR_IS_INTACT).await,
        "",
        "a Game rename must not destroy the open World form"
    );
    assert_eq!(
        changes.loads("/fragment/worlds"),
        loads + 1,
        "the Worlds panel loads for the moved number"
    );
    assert_eq!(eval_string(&page, CANCEL_DIALOG).await, "");
}

// [docs/specs/browser_screen.md] SCENARIO: 44.22
async fn check_a_connection_form_in_the_dialog_survives_a_settings_load_and_keeps_the_sub_tab(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "settings", ".settings-panel").await;

    assert_eq!(eval_string(&page, SUBTAB_ENGAGED.engage).await, "");
    let opened = eval_string(
        &page,
        concat!(
            "async () => {",
            after_settle!(),
            r#"
            await afterSettle(() =>
                document.querySelector('#connection-openrouter-gpt-4o-mini button[hx-get$="/edit"]').click());
            const input = document.getElementById('conn_name');
            if (!input) return 'the form did not open';
            input.value = 'A typed connection';
            input.focus();
            window.__form = document.querySelector('#edit-dialog .connection-form');
            window.__formInput = input;
            return document.activeElement === input ? '' : 'the name field has no focus';
        }"#
        ),
    )
    .await;
    assert_eq!(opened, "");
    let loads = changes.loads("/fragment/settings");
    changes.bump(ChangeKind::SETTINGS);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("/fragment/settings"),
        loads + 1,
        "an open form holds no part, so the panel loads"
    );
    let intact = eval_string(
        &page,
        r#"() => {
            const form = document.querySelector('#edit-dialog .connection-form');
            const input = document.getElementById('conn_name');
            if (form !== window.__form) return 'the form is a new element';
            if (input !== window.__formInput) return 'the name field is a new element';
            if (input.value !== 'A typed connection') return 'the typed name changed: ' + input.value;
            return document.activeElement === input ? '' : 'the name field lost focus';
        }"#,
    )
    .await;
    assert_eq!(intact, "", "the load of the panel leaves the dialog alone");

    start_counting_requests(&page).await;
    assert_eq!(eval_string(&page, CANCEL_DIALOG).await, "");
    assert_eq!(requests_counted(&page).await, 0, "Cancel sends no request");
    assert_eq!(
        eval_string(&page, SUBTAB_ENGAGED.check).await,
        "",
        "the sub-tab is still the one the player chose, after the dialog and the load"
    );
}

pub(super) const DIALOG_STATUS: &str =
    "() => document.getElementById('edit-dialog-status').textContent.trim()";

pub(super) const PRESS_SAVE_IN_RENAME_DIALOG: &str = r#"() => {
    document.querySelector('#edit-dialog .game-rename-form form button[type=submit]').click();
}"#;

// [docs/specs/browser_screen.md] SCENARIO: 44.34
async fn check_the_dialog_says_saving_until_the_screen_shows_the_save(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;
    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    hold_screen_loads(&page).await;

    page.evaluate::<(), ()>(PRESS_SAVE_IN_RENAME_DIALOG, None)
        .await
        .unwrap();
    wait_for_held_screen_load(&page).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async { eval_string(&page, DIALOG_STATUS).await == "Saving…" }
        )
        .await,
        "the dialog says Saving… while the screen does not show the save"
    );
    assert_eq!(
        eval_string(
            &page,
            "() => document.getElementById('edit-dialog').open ? '' : 'closed'"
        )
        .await,
        "",
        "the accepted save does not close the dialog before the screen shows it"
    );

    release_screen_loads(&page).await;
    wait_for_screen_current(&page).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async {
                eval_string(
                    &page,
                    "() => document.getElementById('edit-dialog').open ? 'open' : ''",
                )
                .await
                .is_empty()
            }
        )
        .await,
        "the dialog closes once the screen shows the save"
    );
    assert_eq!(eval_string(&page, DIALOG_STATUS).await, "");
    let shown = page
        .locator("#game-row-2 .game-name")
        .await
        .inner_text()
        .await
        .unwrap();
    assert_eq!(shown, "A typed name", "the screen shows the renamed Game");
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
    runner
        .run(
            outcome,
            check_a_save_that_fails_keeps_the_dialog_open_with_its_typed_values,
        )
        .await;
    runner.run(outcome, check_a_control_that_saves_on_change_goes_back_to_its_rendered_value_when_the_save_fails).await;
    runner
        .run(outcome, check_a_load_keeps_an_inline_error_that_is_showing)
        .await;
    runner
        .run(
            outcome,
            check_an_open_rename_dialog_keeps_its_text_and_focus_through_a_load_of_each_kind,
        )
        .await;
    runner
        .run(
            outcome,
            check_saving_the_rename_dialog_closes_it_and_the_games_panel_shows_the_name,
        )
        .await;
    runner
        .run(
            outcome,
            check_the_new_game_selects_keep_the_players_choice_through_a_load,
        )
        .await;
    runner
        .run(
            outcome,
            check_cancelling_the_new_game_dialog_discards_the_choice,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_preset_form_in_the_dialog_survives_a_presets_load_and_cancel_sends_no_request,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_world_form_in_the_dialog_survives_a_game_rename_that_raises_games,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_connection_form_in_the_dialog_survives_a_settings_load_and_keeps_the_sub_tab,
        )
        .await;
    runner
        .run(
            outcome,
            check_the_dialog_says_saving_until_the_screen_shows_the_save,
        )
        .await;
    runner.finish().await;
}
