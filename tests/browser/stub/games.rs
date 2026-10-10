//! Stub-browser tests for the Games panel: the keyboard-focus landing after a saved-game delete. Tagged against `docs/specs/browser_games.md`.

use std::time::Duration;

use super::support::wait_until_focused;
use super::*;

// [docs/specs/browser_games.md] SCENARIO: 27.3
#[tokio::test]
async fn test_deleting_a_saved_game_keeps_keyboard_focus_in_the_panel() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
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

        assert!(
            wait_until_focused(&page, ".games-panel :focus", Duration::from_secs(5)).await,
            "deleting the last saved game must leave focus inside the Games panel"
        );
        let rows_after_last_delete = page
            .query_selector_all(".games-list .game-item")
            .await
            .unwrap_or_default()
            .len();
        assert_eq!(
            rows_after_last_delete, 0,
            "the last delete must remove the final row"
        );
    })
    .await;
}
