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

            // The save answers nothing: the games number moves, and the Games
            // panel is the region whose load shows the saved value.
            select_option_and_settle(
                &page,
                r#"#game-posture-controls select[name="narrative_tense"]"#,
                "present",
                ".games-panel",
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

// [docs/specs/browser_games.md] SCENARIO: 27.2
#[tokio::test]
async fn test_games_posture_change_keeps_focus_on_the_select() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            open_games_tab(&page).await;

            let selector = r#"#game-posture-controls select[name="narrative_tense"]"#;
            page.locator(selector).await.focus().await.unwrap();
            page.evaluate::<String, ()>(
                r#"(selector) => { window.__tenseSelect = document.querySelector(selector); }"#,
                Some(&selector.to_string()),
            )
            .await
            .unwrap();

            select_option_and_settle(&page, selector, "present", ".games-panel").await;

            let kept = wait_for_condition_async(
                std::time::Duration::from_secs(2),
                std::time::Duration::from_millis(25),
                || async {
                    page.evaluate::<String, bool>(
                        r#"(selector) => {
                            const el = document.querySelector(selector);
                            return !!el && el === window.__tenseSelect
                                && el.value === 'present' && document.activeElement === el;
                        }"#,
                        Some(&selector.to_string()),
                    )
                    .await
                    .unwrap_or(false)
                },
            )
            .await;
            assert!(
                kept,
                "the posture select must stay the same element, showing the saved value, with focus, across the Games panel load"
            );
        },
    )
    .await;
}
