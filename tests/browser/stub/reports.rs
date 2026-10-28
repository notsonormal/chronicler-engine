//! Stub-browser tests for where a report shows: the notice place under the header, the slot of a form with typed input, and the Unreachable banner. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;

use super::dialog::OPEN_RENAME;
use super::support::{
    NOTICE, accept_delete_confirmation, ask_once_without_waiting, dialog_is_open, eval_string,
    open_section, read_banner, read_command_input, start_current, wait_until_dialog_closed,
    StubRunner,
};
use super::*;

const NOTICE_CLOSE: &str = "#screen-error [data-close-notice]";
const WAIT: Duration = Duration::from_secs(5);

async fn wait_for_notice(page: &playwright_rs::Page, why: &str) {
    wait_for_error_disclosure(page, NOTICE, why).await;
}

async fn notice_is_shown(page: &playwright_rs::Page) -> bool {
    read_error_disclosure(page, NOTICE).await.0
}

async fn notice_source(page: &playwright_rs::Page) -> String {
    eval_string(
        page,
        "() => document.getElementById('screen-error').dataset.source || ''",
    )
    .await
}

/// The ids of the slots that show a report outside the notice place.
async fn slots_showing_outside_the_notice(page: &playwright_rs::Page) -> String {
    eval_string(
        page,
        r#"() => Array.from(document.querySelectorAll('[data-error-slot]'))
            .filter((slot) => !slot.hidden && !slot.closest('#screen-error'))
            .map((slot) => slot.dataset.errorSlot)
            .join(',')"#,
    )
    .await
}

/// The reports that the whole page shows: a second one in a slot, a row or a dock
/// beside the notice makes this more than one.
async fn reports_on_the_page(page: &playwright_rs::Page) -> String {
    eval_string(
        page,
        "() => String(document.querySelectorAll('.error-disclosure').length)",
    )
    .await
}

/// Waits for the notice, and checks that it holds the report with this first line,
/// that the page shows no other report, and that the Unreachable banner is not raised.
async fn assert_reported_in_the_notice(
    page: &playwright_rs::Page,
    summary: &str,
    raw_contains: &str,
    what: &str,
) {
    wait_for_notice(page, &format!("{what} reports in the notice")).await;
    let (_, line, raw) = read_error_disclosure(page, NOTICE).await;
    assert_eq!(
        line, summary,
        "{what}: the short line is the first line of the body"
    );
    assert!(
        raw.starts_with(summary) && raw.contains(raw_contains),
        "{what}: Details holds the whole text, got {raw:?}"
    );
    assert_eq!(
        reports_on_the_page(page).await,
        "1",
        "{what}: one report on the page"
    );
    assert_eq!(
        slots_showing_outside_the_notice(page).await,
        "",
        "{what}: no slot beside a control shows a report"
    );
    assert_eq!(notice_source(page).await, "action", "{what}: the source");
    assert!(!read_banner(page).await.0, "{what}: the engine answered");
}

async fn click_first_world_delete(page: &playwright_rs::Page) {
    page.locator(".worlds-list .world-item:first-child .btn-danger")
        .await
        .click(None)
        .await
        .unwrap();
}

async fn open_worlds_and_games_ready(page: &playwright_rs::Page) {
    accept_delete_confirmation(page).await;
    open_section(page, "worlds", ".worlds-panel").await;
    wait_until_visible(
        page,
        ".worlds-list .world-item:first-child .btn-danger",
        WAIT,
    )
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.72
async fn check_the_close_control_of_the_notice_hides_it(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_worlds_and_games_ready(&page).await;
    click_first_world_delete(&page).await;
    wait_for_notice(&page, "a failed row action shows a notice").await;

    let close_name = eval_string(
        &page,
        "() => document.querySelector('#screen-error [data-close-notice]').getAttribute('aria-label') || ''",
    )
    .await;
    assert_eq!(
        close_name, "Close the notice",
        "the Close control has an accessible name"
    );

    page.locator(NOTICE_CLOSE).await.click(None).await.unwrap();
    wait_until_hidden(&page, NOTICE, WAIT).await;
    assert_eq!(
        eval_string(
            &page,
            "() => document.activeElement ? document.activeElement.id : ''"
        )
        .await,
        "main-content",
        "Close moves focus to the main content"
    );
    assert_eq!(
        eval_string(
            &page,
            "() => document.querySelector('#screen-error [data-report]').innerHTML.trim()"
        )
        .await,
        "",
        "Close empties the report"
    );
}

async fn save_rename_dialog(page: &playwright_rs::Page) {
    page.locator("#edit-dialog .game-rename-form form button[type=submit]")
        .await
        .click(None)
        .await
        .unwrap();
}

async fn dialog_status(page: &playwright_rs::Page) -> String {
    eval_string(
        page,
        "() => document.getElementById('edit-dialog-status').textContent.trim()",
    )
    .await
}

async fn cancel_dialog(page: &playwright_rs::Page) {
    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_dialog_closed(page, "Cancel closes the dialog").await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.73
async fn check_a_dialog_form_that_the_engine_refuses_reports_in_the_notice_and_stays_open(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;
    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    stub.games_handle().set_rename_refused(true);
    save_rename_dialog(&page).await;

    assert_reported_in_the_notice(
        &page,
        "The Game is busy, so the Game was not renamed.",
        "Still thinking...",
        "a refused dialog form",
    )
    .await;
    assert!(
        dialog_is_open(&page).await,
        "a refused save keeps the dialog open"
    );
    assert_eq!(
        dialog_status(&page).await,
        "The Game is busy, so the Game was not renamed.",
        "the dialog repeats the summary in its status line"
    );
    assert_eq!(
        eval_string(
            &page,
            "() => document.querySelector('#edit-dialog input[name=\"display_name\"]').value"
        )
        .await,
        "A typed name",
        "the dialog keeps the draft"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.73
async fn check_an_invalid_answer_of_a_form_with_typed_input_reports_in_that_form(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(
        "() => { document.getElementById('command-form').requestSubmit(); }",
        None,
    )
    .await
    .unwrap();
    let action = "#command-form [data-error-slot]";
    wait_for_error_disclosure(&page, action, "an invalid Action reports in its form").await;
    let (_, line, raw) = read_error_disclosure(&page, action).await;
    assert_eq!(line, "The Action was not sent: The value is not allowed.");
    assert!(raw.contains("The value is not allowed"), "got {raw:?}");
    assert!(!notice_is_shown(&page).await, "the notice stays empty");
    assert_eq!(
        read_command_input(&page).await,
        "look around",
        "the typed text stays"
    );

    eval_string(
        &page,
        "() => { document.getElementById('autoplay-cycles').value = '50'; return ''; }",
    )
    .await;
    page.locator("#autoplay-btn")
        .await
        .click(None)
        .await
        .unwrap();
    let autoplay = "#autoplay-form [data-error-slot]";
    wait_for_error_disclosure(&page, autoplay, "invalid cycles report in their form").await;
    let (_, line, _) = read_error_disclosure(&page, autoplay).await;
    assert_eq!(
        line,
        "The Autoplay run did not start: The value is not allowed."
    );
    assert!(!notice_is_shown(&page).await, "the notice stays empty");
    assert_eq!(
        eval_string(
            &page,
            "() => document.getElementById('autoplay-cycles').value"
        )
        .await,
        "50",
        "the typed cycles stay"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.74
async fn check_a_notice_stays_until_the_next_action_a_failing_load_works_again_or_close(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    open_worlds_and_games_ready(&page).await;
    start_current(&page).await;

    click_first_world_delete(&page).await;
    wait_for_notice(&page, "a failed delete reports").await;

    let loads = changes.loads("worlds");
    changes.bump(ChangeKind::WORLDS);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("worlds"),
        loads + 1,
        "the panel loads after the report"
    );
    assert!(
        notice_is_shown(&page).await,
        "a good load after an action does not erase the report of that action"
    );
    assert_eq!(notice_source(&page).await, "action");

    open_section(&page, "games", ".games-panel").await;
    assert!(
        notice_is_shown(&page).await,
        "a tab change keeps the notice"
    );
    open_section(&page, "worlds", ".worlds-panel").await;
    wait_until_visible(
        &page,
        ".worlds-list .world-item:first-child .btn-danger",
        WAIT,
    )
    .await;
    assert!(notice_is_shown(&page).await);

    page.locator(r#"button[hx-get="/fragment/worlds/new"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, NOTICE, WAIT).await;
    assert!(
        wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
            dialog_is_open(&page).await
        })
        .await,
        "the opener opens its dialog"
    );
    assert!(
        !notice_is_shown(&page).await,
        "the notice stays clear after the dialog opens"
    );
    cancel_dialog(&page).await;

    click_first_world_delete(&page).await;
    wait_for_notice(&page, "a failed delete reports again").await;
    open_section(&page, "game", "#story-log").await;
    fill_command_input(&page, "look around").await;
    page.locator("#submit-btn").await.click(None).await.unwrap();
    wait_until_hidden(&page, NOTICE, WAIT).await;

    wait_for_screen_current(&page).await;
    changes.set_loads_failing(true);
    changes.bump(ChangeKind::GAMES);
    ask_once_without_waiting(&page).await;
    wait_for_notice(&page, "a failed screen load reports").await;
    assert_eq!(notice_source(&page).await, "screen");
    changes.set_loads_failing(false);
    ask_once_without_waiting(&page).await;
    wait_until_hidden(&page, NOTICE, WAIT).await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.74
async fn check_a_form_report_clears_when_its_form_sends_again(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    eval_string(
        &page,
        r#"() => {
            window.__slotAtSend = [];
            document.body.addEventListener('htmx:beforeRequest', (evt) => {
                const slot = document.querySelector('#command-form [data-error-slot]');
                if (evt.detail.elt.id === 'command-form') window.__slotAtSend.push(slot.hidden);
            });
            return '';
        }"#,
    )
    .await;
    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(
        "() => { document.getElementById('command-form').requestSubmit(); }",
        None,
    )
    .await
    .unwrap();
    wait_for_error_disclosure(&page, "#command-form [data-error-slot]", "the form reports").await;

    page.evaluate::<(), ()>(
        "() => { document.getElementById('command-form').requestSubmit(); }",
        None,
    )
    .await
    .unwrap();
    let sent_again = wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        eval_string(&page, "() => String(window.__slotAtSend.length)").await == "2"
    })
    .await;
    assert!(sent_again, "the form sends a second time");
    assert_eq!(
        eval_string(&page, "() => window.__slotAtSend.join(',')").await,
        "true,true",
        "the slot is empty when the form sends: the earlier report clears at the next send"
    );
}

/// Stops the engine, and returns once the port is closed.
async fn stop_engine(stub: &StubServer) -> StubLifecycleHandle {
    let lifecycle = stub.lifecycle_handle();
    lifecycle.stop();
    wait_until_port_closed(lifecycle.addr()).await;
    lifecycle
}

async fn assert_unreachable_and_nothing_else(page: &playwright_rs::Page, what: &str) {
    assert!(
        wait_for_condition_async(WAIT * 2, Duration::from_millis(50), || async {
            read_banner(page).await.0
        })
        .await,
        "{what}: the banner shows"
    );
    assert!(
        read_banner(page)
            .await
            .2
            .contains("The engine is unreachable"),
        "{what}: the banner says the engine is unreachable"
    );
    assert!(
        !notice_is_shown(page).await,
        "{what}: the notice holds no report"
    );
    assert_eq!(
        slots_showing_outside_the_notice(page).await,
        "",
        "{what}: no slot holds a report"
    );
}

async fn restart_and_wait_for_the_banner_to_clear(
    page: &playwright_rs::Page,
    lifecycle: &StubLifecycleHandle,
    what: &str,
) {
    lifecycle.restart().await;
    load_screen_now(page).await;
    assert!(
        wait_for_condition_async(WAIT, Duration::from_millis(50), || async {
            !read_banner(page).await.0
        })
        .await,
        "{what}: a later answer clears the banner"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.76
async fn check_no_answer_raises_the_banner_and_no_slot_or_notice_holds_a_report(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    open_worlds_and_games_ready(&page).await;
    stop_page_timer(&page).await;

    let lifecycle = stop_engine(&stub).await;
    click_first_world_delete(&page).await;
    assert_unreachable_and_nothing_else(&page, "a row button").await;
    restart_and_wait_for_the_banner_to_clear(&page, &lifecycle, "a row button").await;

    open_section(&page, "games", ".games-panel").await;
    let lifecycle = stop_engine(&stub).await;
    eval_string(
        &page,
        r#"() => {
            const select = document.querySelector('#game-posture-controls select[name="narrative_tense"]');
            select.value = 'present';
            select.dispatchEvent(new Event('change', { bubbles: true }));
            return '';
        }"#,
    )
    .await;
    assert_unreachable_and_nothing_else(&page, "a save-on-change select").await;
    restart_and_wait_for_the_banner_to_clear(&page, &lifecycle, "a save-on-change select").await;

    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    let lifecycle = stop_engine(&stub).await;
    save_rename_dialog(&page).await;
    assert_unreachable_and_nothing_else(&page, "a dialog form").await;
    assert!(dialog_is_open(&page).await, "the dialog stays open");
    assert!(
        dialog_status(&page)
            .await
            .contains("The engine is unreachable"),
        "the dialog repeats the Unreachable text in its status line, got {:?}",
        dialog_status(&page).await
    );
    restart_and_wait_for_the_banner_to_clear(&page, &lifecycle, "a dialog form").await;
    assert!(
        !dialog_status(&page).await.contains("unreachable"),
        "the next answer clears the status line"
    );
    cancel_dialog(&page).await;

    open_section(&page, "game", "#story-log").await;
    page.locator(".log-entry.narration .edit-btn")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(&page, "#edit-textarea", WAIT).await;
    let lifecycle = stop_engine(&stub).await;
    page.locator("[data-editing] .save-btn")
        .await
        .click(None)
        .await
        .unwrap();
    assert_unreachable_and_nothing_else(&page, "a story-log Save").await;
    wait_until_hidden(&page, "#edit-textarea", WAIT).await;
    restart_and_wait_for_the_banner_to_clear(&page, &lifecycle, "a story-log Save").await;
}

const SWAPS_THAT_THROW: &str = r#"() => {
    window.__loadErrors = [];
    document.body.addEventListener('htmx:beforeSwap', (evt) => {
        if (evt.detail.requestConfig.path === '/action/check') {
            evt.detail.shouldSwap = true;
            evt.detail.target = null;
        }
    });
    document.body.addEventListener('htmx:onLoadError', (evt) => {
        window.__loadErrors.push(evt.detail.elt.id || evt.detail.elt.className);
    });
}"#;

async fn wait_for_load_errors(page: &playwright_rs::Page, count: usize) {
    let seen = wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        eval_string(page, "() => String(window.__loadErrors.length)").await == count.to_string()
    })
    .await;
    assert!(seen, "htmx reports {count} load error(s)");
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.77
async fn check_a_load_error_ends_a_check_of_the_command_form_and_not_one_of_another_form(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    wait_until_visible(&page, ".options-regen-form button", WAIT).await;
    page.evaluate::<(), ()>(SWAPS_THAT_THROW, None)
        .await
        .unwrap();
    let state = || {
        eval_string(
            &page,
            "() => document.getElementById('action-area').dataset.state",
        )
    };

    eval_string(&page, "() => { onCommandBeforeRequest(); return ''; }").await;
    assert_eq!(state().await, "checking", "the check is in flight");

    page.locator(".options-regen-form button")
        .await
        .click(None)
        .await
        .unwrap();
    wait_for_load_errors(&page, 1).await;
    assert_eq!(
        state().await,
        "checking",
        "a load error of the new-Options form does not end the check of the command form"
    );

    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(
        "() => { document.getElementById('command-form').requestSubmit(); }",
        None,
    )
    .await
    .unwrap();
    wait_for_load_errors(&page, 2).await;
    assert_ne!(
        state().await,
        "checking",
        "a load error of the command form ends its check"
    );
}

#[tokio::test]
async fn run_report_checks() {
    let mut runner = StubRunner::launch().await;
    let pending = StubActionOutcome::Pending;
    runner
        .run(pending, check_the_close_control_of_the_notice_hides_it)
        .await;
    runner
        .run(
            pending,
            check_a_dialog_form_that_the_engine_refuses_reports_in_the_notice_and_stays_open,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Invalid,
            check_an_invalid_answer_of_a_form_with_typed_input_reports_in_that_form,
        )
        .await;
    runner
        .run(
            pending,
            check_a_notice_stays_until_the_next_action_a_failing_load_works_again_or_close,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Invalid,
            check_a_form_report_clears_when_its_form_sends_again,
        )
        .await;
    runner
        .run_without_convergence(
            "the check stops the engine",
            pending,
            check_no_answer_raises_the_banner_and_no_slot_or_notice_holds_a_report,
        )
        .await;
    runner
        .run(
            pending,
            check_a_load_error_ends_a_check_of_the_command_form_and_not_one_of_another_form,
        )
        .await;
    runner.finish().await;
}
