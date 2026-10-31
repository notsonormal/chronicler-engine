//! Stub-browser tests for the edit dialog: an htmx swap into its body opens it, Cancel and Escape close it and return focus to the opener, a save closes it only when the engine accepts it and the screen shows it, an answer from outside never replaces its form, and the dialog forms that a screen load never reaches. Tagged against `docs/specs/browser_dashboard.md`, `docs/specs/browser_games.md` and `docs/specs/browser_screen.md`.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;

use crate::test_utils::stub_server::FAILING_TEST_MODEL;

use super::screen_keeps::SUBTAB_ENGAGED;
use super::support::{
    eval_string, open_section, start_current, dialog_body_is_empty, dialog_is_open,
    hold_requests_ending_with, open_settings, release_held_requests_and_wait_for_answers,
    wait_until_dialog_closed, wait_until_focused, StubRunner, NOTICE,
};
use super::*;

const DIALOG_WAIT: Duration = Duration::from_secs(5);

const GAMES_PANEL_KINDS: [ChangeKind; 3] = [
    ChangeKind::GAMES,
    ChangeKind::WORLDS,
    ChangeKind::PROMPT_PRESETS,
];

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

async fn submit_dialog_form(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        "() => document.querySelector('#edit-dialog form').requestSubmit()",
        None,
    )
    .await
    .unwrap();
}

const EDIT_BUTTON: &str = r#"#connection-openrouter-gpt-4o-mini button[hx-get$="/edit"]"#;
const ADD_BUTTON: &str = ".connection-add button";
const TEST_BUTTON: &str = "#edit-dialog .form-actions button:has-text('Test')";

// [docs/specs/browser_dashboard.md] SCENARIO: 16.46
async fn check_a_swap_into_the_dialog_body_opens_the_modal_dialog(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;
    assert!(
        !dialog_is_open(&page).await,
        "the dialog is closed before an opener is activated"
    );

    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;

    assert!(
        dialog_is_open(&page).await,
        "an htmx swap into #edit-dialog-body must open #edit-dialog as a modal"
    );
    assert!(
        wait_until_focused(&page, "#conn_name", Duration::from_secs(2)).await,
        "the first field of the form must have keyboard focus"
    );
    let title = page
        .evaluate::<(), String>(
            r#"() => {
                const dialog = document.getElementById('edit-dialog');
                const heading = document.getElementById(dialog.getAttribute('aria-labelledby'));
                return heading ? heading.textContent.trim() : '';
            }"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        title, "Add Connection",
        "the dialog is named by the heading of its form"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.47
async fn check_closing_the_dialog_returns_focus_to_its_opener(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let stub = stub.changes_handle();
    open_settings(&page).await;

    page.locator(EDIT_BUTTON).await.focus().await.unwrap();
    click_and_settle(&page, EDIT_BUTTON, "#edit-dialog-body").await;
    assert!(dialog_is_open(&page).await, "Edit opens the dialog");
    page.keyboard().press("Escape", None).await.unwrap();
    wait_until_dialog_closed(&page, "Escape must close the dialog").await;
    assert!(
        dialog_body_is_empty(&page).await,
        "closing the dialog discards its form"
    );
    assert!(
        wait_until_focused(&page, EDIT_BUTTON, Duration::from_secs(2)).await,
        "Escape must return keyboard focus to the Edit control that opened the dialog"
    );

    page.locator(ADD_BUTTON).await.focus().await.unwrap();
    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;
    page.mouse().click(2, 2, None).await.unwrap();
    assert!(
        dialog_is_open(&page).await,
        "a click on the backdrop must not close the dialog"
    );
    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .first()
        .click(None)
        .await
        .unwrap();
    wait_until_dialog_closed(&page, "Cancel must close the dialog").await;
    assert!(dialog_body_is_empty(&page).await);
    assert!(
        wait_until_focused(&page, ADD_BUTTON, Duration::from_secs(2)).await,
        "Cancel must return keyboard focus to the Add control that opened the dialog"
    );
    assert_eq!(
        stub.loads("/fragment/connections/new"),
        1,
        "closing the dialog sends no request: the only request was the one that opened it"
    );

    // A click that does not focus its control (Safari and Firefox on macOS do not
    // focus a button on click) leaves the body focused when the dialog opens, so
    // the browser has no control to return focus to.
    page.evaluate::<(), ()>(
        "() => document.activeElement && document.activeElement.blur()",
        None,
    )
    .await
    .unwrap();
    page.evaluate::<(), ()>(
        &format!("() => document.querySelector('{EDIT_BUTTON}').click()"),
        None,
    )
    .await
    .unwrap();
    let opened = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { dialog_is_open(&page).await },
    )
    .await;
    assert!(
        opened,
        "a click that does not focus its control opens the dialog"
    );
    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .first()
        .click(None)
        .await
        .unwrap();
    wait_until_dialog_closed(&page, "Cancel must close the dialog").await;
    assert!(
        wait_until_focused(&page, EDIT_BUTTON, Duration::from_secs(2)).await,
        "Cancel must return keyboard focus to the Edit control, also when it had no focus at the click"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.48
async fn check_a_save_that_the_engine_accepts_closes_the_dialog(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.save_handle().set_connection_saves_succeed(true);
    open_settings(&page).await;

    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;
    submit_dialog_form(&page).await;
    wait_until_dialog_closed(&page, "a save that the engine accepts closes the dialog").await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.49
async fn check_a_late_answer_from_outside_does_not_replace_the_open_dialog(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;
    hold_requests_ending_with(&page, "/test").await;

    let test_button = r#"#connection-openrouter-gpt-4o-mini button[hx-post$="/test"]"#;
    page.locator(test_button).await.click(None).await.unwrap();
    assert!(
        !dialog_is_open(&page).await,
        "the Test request is in flight, so no dialog is open yet"
    );
    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;
    page.locator("#conn_name")
        .await
        .fill("Typed while the test ran", None)
        .await
        .unwrap();

    release_held_requests_and_wait_for_answers(&page).await;

    assert!(dialog_is_open(&page).await, "the dialog stays open");
    let typed = page
        .evaluate::<(), String>(
            "() => { const f = document.getElementById('conn_name'); return f ? f.value : ''; }",
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        typed, "Typed while the test ran",
        "the answer of the Test must not replace the form the player is typing in"
    );
    assert_eq!(
        page.locator("#edit-dialog .connection-test-result")
            .await
            .count()
            .await
            .unwrap_or(0),
        0,
        "the late result is dropped"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.50
async fn check_escape_closes_an_open_error_detail_before_it_closes_the_dialog(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;
    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;
    page.locator("#conn_name")
        .await
        .fill("Typed before the failure", None)
        .await
        .unwrap();
    page.locator("#conn_model")
        .await
        .fill(FAILING_TEST_MODEL, None)
        .await
        .unwrap();
    click_and_settle(&page, TEST_BUTTON, ".connection-test-slot").await;
    let slot = "#edit-dialog .connection-test-slot";
    wait_for_error_disclosure(
        &page,
        slot,
        "a failed Test reports in the form's result slot",
    )
    .await;
    let toggle = format!("{slot} .error-details-toggle");
    // Any scroll closes an open popover, and the scroll event of the click's
    // own scroll-into-view fires a frame after the click.
    page.evaluate::<String, bool>(
        r#"(selector) => new Promise((resolve) => {
            document.querySelector(selector).scrollIntoView({ block: "center" });
            requestAnimationFrame(() => requestAnimationFrame(() => resolve(true)));
        })"#,
        Some(&toggle),
    )
    .await
    .unwrap();
    page.locator(&toggle).await.click(None).await.unwrap();
    assert!(error_details_open(&page, slot).await, "Details opens");

    page.keyboard().press("Escape", None).await.unwrap();

    assert!(
        !error_details_open(&page, slot).await,
        "Escape closes the error detail"
    );
    assert!(
        dialog_is_open(&page).await,
        "the Escape that closed the error detail must not close the dialog"
    );
    let typed = page
        .evaluate::<(), String>("() => document.getElementById('conn_name').value", None)
        .await
        .unwrap();
    assert_eq!(typed, "Typed before the failure", "the draft stays");

    page.keyboard().press("Escape", None).await.unwrap();
    wait_until_dialog_closed(&page, "a second Escape closes the dialog").await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.52
async fn check_the_open_dialog_is_centred_in_the_viewport(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;
    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;
    assert!(dialog_is_open(&page).await, "Add opens the dialog");

    let (left_gap, right_gap, top_gap, bottom_gap): (f64, f64, f64, f64) = page
        .evaluate::<(), (f64, f64, f64, f64)>(
            r#"() => {
                const rect = document.getElementById('edit-dialog').getBoundingClientRect();
                return [rect.left, window.innerWidth - rect.right, rect.top, window.innerHeight - rect.bottom];
            }"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        (left_gap - right_gap).abs() <= 1.0,
        "the dialog is horizontally centred: left gap {left_gap}, right gap {right_gap}"
    );
    assert!(
        (top_gap - bottom_gap).abs() <= 1.0 && top_gap > 0.0,
        "the dialog is vertically centred, off the top edge: top gap {top_gap}, bottom gap {bottom_gap}"
    );
}

const PRESET_EDIT_OPENER: &str = "#preset-edit-open-custom_ref";

// [docs/specs/browser_dashboard.md] SCENARIO: 16.51
async fn check_closing_the_dialog_focuses_the_opener_by_its_id_after_a_load_rewrote_its_card(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    page.locator(r#"[data-tab="prompt-presets"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(&page, "#preset-card-custom_ref", DIALOG_WAIT).await;

    page.locator(PRESET_EDIT_OPENER)
        .await
        .focus()
        .await
        .unwrap();
    click_and_settle(&page, PRESET_EDIT_OPENER, "#edit-dialog-body").await;
    assert!(dialog_is_open(&page).await, "Edit opens the dialog");
    let activate_buttons = r#"() => document.querySelectorAll('#preset-card-custom_ref button[hx-post*="/activate"]').length"#;
    let before = page
        .evaluate::<(), usize>(activate_buttons, None)
        .await
        .unwrap();

    // Set Active leaves the card with one activate button fewer, so the load
    // rewrites the card around the opener.
    let loads = changes.loads("prompt-presets");
    page.evaluate::<(), ()>(
        "() => { fetch('/prompt-presets/custom_ref/activate?mode=novel', { method: 'POST' }); }",
        None,
    )
    .await
    .unwrap();
    let rewritten = wait_for_condition_async(DIALOG_WAIT, Duration::from_millis(25), || async {
        changes.loads("prompt-presets") > loads
            && page
                .evaluate::<(), usize>(activate_buttons, None)
                .await
                .is_ok_and(|now| now < before)
    })
    .await;
    assert!(
        rewritten,
        "a load of the panel rewrites the card of the opener"
    );

    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .first()
        .click(None)
        .await
        .unwrap();
    wait_until_dialog_closed(&page, "Cancel must close the dialog").await;
    assert!(
        wait_until_focused(&page, PRESET_EDIT_OPENER, Duration::from_secs(2)).await,
        "focus goes to the control with the id of the opener"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.78
async fn check_an_opener_keeps_its_id_through_a_load_that_removes_a_row_above_it_and_a_gone_opener_takes_no_focus(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    open_section(&page, "games", ".games-panel").await;
    let rename_of_game_3 = "#game-rename-open-3";
    let remove_game = |id: u32| {
        let script = format!("() => {{ fetch('/games/{id}/delete', {{ method: 'POST' }}); }}");
        let page = page.clone();
        let changes = changes.clone();
        async move {
            let loads = changes.loads("games");
            page.evaluate::<(), ()>(&script, None).await.unwrap();
            let loaded =
                wait_for_condition_async(DIALOG_WAIT, Duration::from_millis(25), || async {
                    changes.loads("games") > loads
                })
                .await;
            assert!(loaded, "the panel loads after the delete of Game {id}");
        }
    };

    page.locator(rename_of_game_3).await.focus().await.unwrap();
    click_and_settle(&page, rename_of_game_3, "#edit-dialog-body").await;
    assert!(dialog_is_open(&page).await, "Rename opens the dialog");
    remove_game(2).await;
    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .first()
        .click(None)
        .await
        .unwrap();
    wait_until_dialog_closed(&page, "Cancel must close the dialog").await;
    assert!(
        wait_until_focused(&page, rename_of_game_3, Duration::from_secs(2)).await,
        "the opener keeps its id through a load that removes the row above it, and focus returns to it"
    );

    click_and_settle(&page, rename_of_game_3, "#edit-dialog-body").await;
    assert!(dialog_is_open(&page).await, "Rename opens the dialog again");
    remove_game(3).await;
    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .first()
        .click(None)
        .await
        .unwrap();
    wait_until_dialog_closed(&page, "Cancel must close the dialog").await;
    assert_eq!(
        eval_string(
            &page,
            "() => document.activeElement === document.body ? '' : 'focus is on ' + document.activeElement.outerHTML.slice(0, 80)"
        )
        .await,
        "",
        "an opener that is gone takes no focus, so focus is on the body"
    );
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
    let before = changes.loads("games");
    for kind in GAMES_PANEL_KINDS {
        changes.bump(kind);
        ask_once(page).await;
        assert_eq!(eval_string(page, check).await, "", "after `{kind}` moved");
    }
    assert_eq!(
        changes.loads("games"),
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
    let before = changes.loads("games");
    assert_eq!(eval_string(&page, SAVE_RENAME).await, "");
    assert_eq!(
        changes.loads("games"),
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

// [docs/specs/browser_screen.md] SCENARIO: 44.17
async fn check_a_save_that_fails_keeps_the_dialog_open_and_reports_in_the_notice(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    stub.games_handle().set_rename_failing(true);
    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    assert_eq!(eval_string(&page, SAVE_RENAME).await, "");
    wait_for_error_disclosure(&page, NOTICE, "a failed save must report in the notice").await;
    let (_, line, raw) = read_error_disclosure(&page, NOTICE).await;
    assert_eq!(line, "The Game was not renamed.", "got {line:?}");
    assert!(raw.contains("Stub rename failure"), "got {raw:?}");
    assert_eq!(
        eval_string(
            &page,
            "() => document.getElementById('edit-dialog-status').textContent.trim()"
        )
        .await,
        "The Game was not renamed.",
        "the dialog repeats the summary in its status line"
    );
    assert!(
        !read_error_disclosure(
            &page,
            "#edit-dialog .game-rename-form form [data-error-slot]"
        )
        .await
        .0,
        "a failure shows in the notice, and the form's slot stays empty"
    );
    assert_eq!(
        eval_string(&page, RENAME_IS_INTACT).await,
        "",
        "the dialog stays open with the same form and the typed name"
    );
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
    let loads = changes.loads("prompt-presets");
    changes.bump(ChangeKind::PROMPT_PRESETS);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("prompt-presets"),
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
    let loads = changes.loads("worlds");
    let renamed = eval_string(
        &page,
        r#"async () => {
            const answered = new Promise((resolve) => {
                document.body.addEventListener(
                    'htmx:afterRequest',
                    (evt) => resolve(String(evt.detail.xhr.status)),
                    { once: true },
                );
            });
            await htmx.ajax('POST', '/games/2/rename', {
                swap: 'none',
                values: { display_name: 'Renamed elsewhere' },
            });
            const status = await answered;
            await window.__waitForScreen();
            return status;
        }"#,
    )
    .await;
    assert_eq!(renamed, "200", "the rename went through");

    assert_eq!(
        eval_string(&page, WORLD_EDITOR_IS_INTACT).await,
        "",
        "a Game rename must not destroy the open World form"
    );
    assert_eq!(
        changes.loads("worlds"),
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
    let loads = changes.loads("settings");
    changes.bump(ChangeKind::SETTINGS);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("settings"),
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

#[tokio::test]
async fn run_dialog_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_swap_into_the_dialog_body_opens_the_modal_dialog,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_closing_the_dialog_returns_focus_to_its_opener,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_save_that_the_engine_accepts_closes_the_dialog,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_late_answer_from_outside_does_not_replace_the_open_dialog,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_escape_closes_an_open_error_detail_before_it_closes_the_dialog,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_closing_the_dialog_focuses_the_opener_by_its_id_after_a_load_rewrote_its_card,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_an_opener_keeps_its_id_through_a_load_that_removes_a_row_above_it_and_a_gone_opener_takes_no_focus,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_the_open_dialog_is_centred_in_the_viewport,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_save_that_fails_keeps_the_dialog_open_and_reports_in_the_notice,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_an_open_rename_dialog_keeps_its_text_and_focus_through_a_load_of_each_kind,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_saving_the_rename_dialog_closes_it_and_the_games_panel_shows_the_name,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_the_new_game_selects_keep_the_players_choice_through_a_load,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_cancelling_the_new_game_dialog_discards_the_choice,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_preset_form_in_the_dialog_survives_a_presets_load_and_cancel_sends_no_request,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_world_form_in_the_dialog_survives_a_game_rename_that_raises_games,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_connection_form_in_the_dialog_survives_a_settings_load_and_keeps_the_sub_tab,
        )
        .await;
    runner.finish().await;
}
