//! Stub-browser tests for the Games panel: the keyboard-focus landing after a saved-game delete, and the pages of saved Games. Tagged against `docs/specs/browser_games.md`.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;

use super::support::{
    ask_once_without_waiting, hold_screen_loads, release_screen_loads, start_current,
    wait_for_held_screen_load, wait_until_focused,
};
use super::*;
use super::support::StubRunner;

// [docs/specs/browser_games.md] SCENARIO: 27.3
async fn check_deleting_a_saved_game_keeps_keyboard_focus_in_the_panel(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    page.on_dialog(|dialog| async move { dialog.accept(None).await })
        .await
        .unwrap();

    open_games_tab(&page).await;

    let rows_before = page
        .query_selector_all(".games-list .game-item")
        .await
        .unwrap_or_default()
        .len();
    assert!(
        rows_before > 1,
        "the fixture must show more than one saved game, got {rows_before}"
    );

    let first_delete = ".games-list .game-item:first-child .btn-danger";
    page.locator(first_delete).await.focus().await.unwrap();
    page.locator(first_delete).await.click(None).await.unwrap();

    assert!(
        wait_until_focused(
            &page,
            ".games-list .game-item .game-actions > button.btn-cyan",
            Duration::from_secs(5),
        )
        .await,
        "deleting a saved game must move focus to the first control of the surviving row, its Rename control"
    );
    let rows_after_first_delete = page
        .query_selector_all(".games-list .game-item")
        .await
        .unwrap_or_default()
        .len();
    assert_eq!(
        rows_after_first_delete, 1,
        "the delete must leave the one surviving row behind"
    );

    let last_delete = ".games-list .game-item .btn-danger";
    page.locator(last_delete).await.focus().await.unwrap();
    page.locator(last_delete).await.click(None).await.unwrap();

    // The clicked control is already inside the panel, so a focus wait is
    // satisfied before the load lands. Wait for the removal itself first.
    let removed = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            page.query_selector_all(".games-list .game-item")
                .await
                .unwrap_or_default()
                .is_empty()
        },
    )
    .await;
    assert!(removed, "the last delete must remove the final row");
    assert!(
        wait_until_focused(&page, ".games-panel", Duration::from_secs(5)).await,
        "deleting the last saved game must leave focus inside the Games panel"
    );
}

const SAVED_GAME_ROWS: &str = ".games-list .game-item";
const PAGE_SIZE: usize = 50;

async fn saved_game_rows(page: &playwright_rs::Page) -> usize {
    page.query_selector_all(SAVED_GAME_ROWS)
        .await
        .unwrap_or_default()
        .len()
}

async fn wait_for_saved_game_rows(page: &playwright_rs::Page, rows: usize) {
    let shown = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { saved_game_rows(page).await == rows },
    )
    .await;
    assert!(
        shown,
        "the Saved Games list must show {rows} Games, shows {}",
        saved_game_rows(page).await
    );
}

async fn pager_position(page: &playwright_rs::Page) -> String {
    page.locator(".games-page-position")
        .await
        .inner_text()
        .await
        .unwrap_or_default()
}

async fn pager_is_enabled(page: &playwright_rs::Page, control: &str) -> bool {
    page.locator(&format!(".games-page-{control}"))
        .await
        .is_enabled()
        .await
        .unwrap()
}

// [docs/specs/browser_games.md] SCENARIO: 27.8
async fn check_next_and_previous_move_between_the_pages_of_saved_games(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.games_handle().set_saved_games(PAGE_SIZE as u64 + 1);
    open_games_tab(&page).await;
    wait_for_saved_game_rows(&page, PAGE_SIZE).await;
    assert_eq!(pager_position(&page).await, "Page 1 of 2");
    assert!(!pager_is_enabled(&page, "previous").await);
    assert!(pager_is_enabled(&page, "next").await);

    page.locator(".games-page-next")
        .await
        .click(None)
        .await
        .unwrap();
    wait_for_saved_game_rows(&page, 1).await;
    assert_eq!(pager_position(&page).await, "Page 2 of 2");
    assert!(!pager_is_enabled(&page, "next").await);
    assert!(pager_is_enabled(&page, "previous").await);

    load_screen_now(&page).await;
    wait_for_saved_game_rows(&page, 1).await;
    assert_eq!(
        pager_position(&page).await,
        "Page 2 of 2",
        "a load of the screen keeps the page"
    );

    page.locator(".games-page-previous")
        .await
        .click(None)
        .await
        .unwrap();
    wait_for_saved_game_rows(&page, PAGE_SIZE).await;
    assert_eq!(pager_position(&page).await, "Page 1 of 2");
}

// [docs/specs/browser_games.md] SCENARIO: 27.9
async fn check_the_page_of_saved_games_resets_when_the_client_leaves_the_games_tab(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.games_handle().set_saved_games(PAGE_SIZE as u64 + 1);
    open_games_tab(&page).await;
    wait_for_saved_game_rows(&page, PAGE_SIZE).await;
    page.locator(".games-page-next")
        .await
        .click(None)
        .await
        .unwrap();
    wait_for_saved_game_rows(&page, 1).await;
    assert_eq!(pager_position(&page).await, "Page 2 of 2");

    open_worlds_tab(&page).await;
    open_games_tab(&page).await;

    wait_for_saved_game_rows(&page, PAGE_SIZE).await;
    assert_eq!(pager_position(&page).await, "Page 1 of 2");
}

// [docs/specs/browser_games.md] SCENARIO: 27.10
async fn check_a_click_on_next_during_a_load_drops_the_old_answer(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.games_handle().set_saved_games(PAGE_SIZE as u64 + 1);
    start_current(&page).await;
    open_games_tab(&page).await;
    wait_for_saved_game_rows(&page, PAGE_SIZE).await;
    hold_screen_loads(&page).await;
    stub.changes_handle().bump(ChangeKind::GAMES);
    ask_once_without_waiting(&page).await;
    wait_for_held_screen_load(&page).await;

    page.locator(".games-page-next")
        .await
        .click(None)
        .await
        .unwrap();
    release_screen_loads(&page).await;

    wait_for_saved_game_rows(&page, 1).await;
    assert_eq!(
        pager_position(&page).await,
        "Page 2 of 2",
        "the old answer, which asked for page 1, must not hide page 2"
    );
}

// [docs/specs/browser_games.md] SCENARIO: 27.11
async fn check_next_on_the_last_page_keeps_keyboard_focus_and_does_nothing(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.games_handle().set_saved_games(PAGE_SIZE as u64 + 1);
    open_games_tab(&page).await;
    wait_for_saved_game_rows(&page, PAGE_SIZE).await;

    page.locator(".games-page-next")
        .await
        .focus()
        .await
        .unwrap();
    page.keyboard().press("Enter", None).await.unwrap();
    wait_for_saved_game_rows(&page, 1).await;
    assert_eq!(pager_position(&page).await, "Page 2 of 2");
    assert!(
        wait_until_focused(&page, ".games-page-next", Duration::from_secs(5)).await,
        "Next must keep keyboard focus when it reaches the last page"
    );

    let requests_before = stub.changes_handle().screen_requests().len();
    page.keyboard().press("Enter", None).await.unwrap();
    load_screen_now(&page).await;
    wait_for_saved_game_rows(&page, 1).await;

    assert_eq!(
        stub.changes_handle().screen_requests().len(),
        requests_before + 1,
        "a press on Next at the last page must ask for no screen load; only the load of this check"
    );
    assert_eq!(pager_position(&page).await, "Page 2 of 2");
    assert!(
        wait_until_focused(&page, ".games-page-next", Duration::from_secs(5)).await,
        "Next must keep keyboard focus after a press that does nothing"
    );
}

#[tokio::test]
async fn run_games_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_deleting_a_saved_game_keeps_keyboard_focus_in_the_panel,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_next_and_previous_move_between_the_pages_of_saved_games,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_the_page_of_saved_games_resets_when_the_client_leaves_the_games_tab,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_click_on_next_during_a_load_drops_the_old_answer,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_next_on_the_last_page_keeps_keyboard_focus_and_does_nothing,
        )
        .await;
    runner.finish().await;
}
