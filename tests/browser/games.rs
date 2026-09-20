//! Browser games-panel tests: per-game posture auto-save wiring guard. Tagged against `docs/specs/browser_games.md`.

use super::*;

/// The browser-only question: does the `change` event on a posture select
/// inside the swap-loaded fragment reach the server. The fragment render is
/// covered at HTTP (`tests/http/games_fragment.rs` SCENARIO 20.8).
// [docs/specs/browser_games.md] SCENARIO: 27.1
#[tokio::test]
async fn test_games_posture_change_reaches_server() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            open_games_tab(&page).await;

            // The posture selects auto-save to `#game-posture-controls` with
            // `hx-swap="outerHTML"`, so the fragment is its own swap target.
            select_option_and_settle(
                &page,
                r#"#game-posture-controls select[name="narrative_tense"]"#,
                "present",
                "#game-posture-controls",
            )
            .await;

            // Reload so the fragment re-renders from the persisted game.
            page.reload(None)
                .await
                .expect("reload after the posture change");
            open_games_tab(&page).await;
            let selected: String = page
                .evaluate::<(), String>(
                    r#"(() => {
                        const sel = document.querySelector('#game-posture-controls select[name="narrative_tense"]');
                        return sel ? sel.value : '';
                    })()"#,
                    None,
                )
                .await
                .unwrap_or_default();
            assert_eq!(
                selected, "present",
                "the tense change must reach the server: the re-rendered fragment should show the persisted tense"
            );
        },
    )
    .await;
}
