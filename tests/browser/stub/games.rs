//! Stub-browser tests for the Games panel: the keyboard-focus landing after a saved-game delete. Tagged against `docs/specs/browser_games.md`.

use std::time::Duration;

use super::support::wait_until_focused;
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
            ".games-list .game-item .game-actions > button.btn-primary",
            Duration::from_secs(5),
        )
        .await,
        "deleting a saved game must move focus to the surviving row's Switch control"
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
    // satisfied before the swap lands. Wait for the removal itself first.
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
        wait_until_focused(&page, ".games-panel :focus", Duration::from_secs(5)).await,
        "deleting the last saved game must leave focus inside the Games panel"
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
    runner.finish().await;
}
