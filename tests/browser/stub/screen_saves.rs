//! Stub-browser tests for a save that waits for the screen: the dialog that says Saving until the screen shows the save. They also cover what a late, repeated, failed or restarted save does. Tagged against `docs/specs/browser_screen.md`.

use std::time::Duration;

use super::dialog::{CANCEL_DIALOG, NEW_GAME_IS_INTACT, OPEN_NEW_GAME, OPEN_RENAME};
use super::support::{
    eval_string, hold_screen_loads, open_section, recorded_requests, record_requests_to,
    release_screen_loads, start_current, wait_for_held_screen_load, StubRunner,
};
use super::*;

const DIALOG_IS_OPEN: &str = "() => document.getElementById('edit-dialog').open ? 'open' : ''";

const SCREEN_ERROR_TEXT: &str = r#"() => {
    const slot = document.getElementById('notice');
    return slot.hidden ? '' : slot.textContent.trim();
}"#;

async fn wait_until_dialog_closed(page: &playwright_rs::Page, why: &str) {
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async { eval_string(page, DIALOG_IS_OPEN).await.is_empty() }
        )
        .await,
        "{why}"
    );
}

async fn wait_until_dialog_says_saving(page: &playwright_rs::Page) {
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async {
                eval_string(page, DIALOG_STATUS)
                    .await
                    .starts_with("Saving…")
            }
        )
        .await,
        "the dialog says Saving… while the screen does not show the save"
    );
}

/// Opens the rename form, holds the screen loads, presses Save, and returns when
/// the engine accepted the save and the load that shows it is held.
async fn save_the_rename_form_with_the_load_held(page: &playwright_rs::Page) {
    start_current(page).await;
    open_section(page, "games", ".games-panel").await;
    assert_eq!(eval_string(page, OPEN_RENAME).await, "");
    hold_screen_loads(page).await;
    page.evaluate::<(), ()>(PRESS_SAVE_IN_RENAME_DIALOG, None)
        .await
        .unwrap();
    wait_for_held_screen_load(page).await;
    wait_until_dialog_says_saving(page).await;
}

// [docs/specs/browser_screen.md] SCENARIO: 44.37
async fn check_a_save_that_the_screen_shows_late_does_not_close_another_dialog(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    save_the_rename_form_with_the_load_held(&page).await;

    page.keyboard().press("Escape", None).await.unwrap();
    wait_until_dialog_closed(&page, "Escape closes the dialog that says Saving…").await;
    assert_eq!(eval_string(&page, OPEN_NEW_GAME).await, "");

    release_screen_loads(&page).await;
    wait_for_screen_current(&page).await;
    let shown = page
        .locator("#game-row-2 .game-name")
        .await
        .inner_text()
        .await
        .unwrap();
    assert_eq!(
        shown, "A typed name",
        "the save went through, and the screen shows it"
    );
    assert_eq!(
        eval_string(&page, NEW_GAME_IS_INTACT).await,
        "",
        "the screen that shows the first save leaves the dialog of the second form open, with its form and its choice"
    );
    assert_eq!(eval_string(&page, CANCEL_DIALOG).await, "");
}

// [docs/specs/browser_screen.md] SCENARIO: 44.38
async fn check_a_second_save_while_the_dialog_says_saving_sends_no_request(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;
    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    record_requests_to(&page, "/games/2/rename").await;
    hold_screen_loads(&page).await;
    page.evaluate::<(), ()>(PRESS_SAVE_IN_RENAME_DIALOG, None)
        .await
        .unwrap();
    wait_for_held_screen_load(&page).await;
    wait_until_dialog_says_saving(&page).await;

    // The page sends a request in the same task as the submit, so a short wait in
    // the page is enough to see whether a second one went out.
    page.evaluate::<(), ()>(
        r#"async () => {
            document.querySelector('#edit-dialog .game-rename-form form button[type=submit]').click();
            document.querySelector('#edit-dialog .game-rename-form form').requestSubmit();
            await new Promise((resolve) => setTimeout(resolve, 150));
        }"#,
        None,
    )
    .await
    .unwrap();
    assert_eq!(
        recorded_requests(&page).await,
        vec!["POST /games/2/rename".to_string()],
        "the engine receives the save once, whatever the client submits while the dialog says Saving…"
    );

    release_screen_loads(&page).await;
    wait_for_screen_current(&page).await;
    wait_until_dialog_closed(&page, "the dialog closes once the screen shows the save").await;
}

// [docs/specs/browser_screen.md] SCENARIO: 44.42
async fn check_a_restart_before_the_load_of_a_save_closes_the_dialog(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    save_the_rename_form_with_the_load_held(&page).await;

    stub.changes_handle().set_server("stub-server-restarted");
    release_screen_loads(&page).await;
    wait_for_screen_current(&page).await;
    assert_eq!(
        eval_string(
            &page,
            "() => document.documentElement.dataset.revision.split(':')[0]"
        )
        .await,
        "stub-server-restarted",
        "the stamp of the page names the new server"
    );
    wait_until_dialog_closed(
        &page,
        "the stamp of the old server cannot be shown by the new server, so the next good load closes the dialog",
    )
    .await;
}

// [docs/specs/browser_screen.md] SCENARIO: 44.43
async fn check_a_save_followed_by_a_failed_load_keeps_the_dialog_at_saving_until_the_next_good_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    save_the_rename_form_with_the_load_held(&page).await;

    changes.set_loads_failing(true);
    release_screen_loads(&page).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async {
                eval_string(&page, SCREEN_ERROR_TEXT)
                    .await
                    .contains("GET /screen answered 500")
            }
        )
        .await,
        "the failed load shows in #notice, got {:?}",
        eval_string(&page, SCREEN_ERROR_TEXT).await
    );
    let status = eval_string(&page, DIALOG_STATUS).await;
    assert!(
        status.starts_with("Saving…") && status.contains("answered 500"),
        "the dialog still says Saving… and names the failure, got {status:?}"
    );
    assert_eq!(
        eval_string(&page, DIALOG_IS_OPEN).await,
        "open",
        "the save is not shown yet, so the dialog stays open"
    );

    changes.set_loads_failing(false);
    ask_once(&page).await;
    wait_until_dialog_closed(&page, "the next good load closes the dialog").await;
    assert_eq!(eval_string(&page, SCREEN_ERROR_TEXT).await, "");
    assert_eq!(eval_string(&page, DIALOG_STATUS).await, "");
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
async fn run_screen_save_checks() {
    let mut runner = StubRunner::launch().await;
    let outcome = StubActionOutcome::Pending;
    runner
        .run(
            outcome,
            check_a_save_that_the_screen_shows_late_does_not_close_another_dialog,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_second_save_while_the_dialog_says_saving_sends_no_request,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_restart_before_the_load_of_a_save_closes_the_dialog,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_save_followed_by_a_failed_load_keeps_the_dialog_at_saving_until_the_next_good_load,
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
