//! Browser games-panel tests: posture fragment render, auto-save, mode switch. Tagged against `docs/specs/browser_games.md`.

use std::time::Duration;

use playwright_rs::expect;

use super::*;

// [docs/specs/browser_games.md] SCENARIO: 27.1
#[tokio::test]
async fn test_games_panel_renders_posture_fragment() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            page.locator(r#".tab[data-tab="games"]"#)
                .await
                .click(None)
                .await
                .unwrap();
            wait_until_visible(&page, "#game-posture-controls", Duration::from_millis(1000)).await;

            let mode = page
                .locator(r#"#game-posture-controls select[name="narrator_mode"]"#)
                .await
                .input_value(None)
                .await
                .unwrap_or_default();
            assert_eq!(mode, "novel", "active game starts in Novel mode");

            let perspective = page
                .locator(r#"#game-posture-controls select[name="narrative_perspective"]"#)
                .await
                .input_value(None)
                .await
                .unwrap_or_default();
            assert_eq!(perspective, "third", "active game starts in Third person");

            let tense = page
                .locator(r#"#game-posture-controls select[name="narrative_tense"]"#)
                .await
                .input_value(None)
                .await
                .unwrap_or_default();
            assert_eq!(tense, "past", "active game starts in Past tense");

            for name in [
                "system_preset_id",
                "quantifier_preset_id",
                "impersonate_preset_id",
            ] {
                let select = page
                    .locator(&format!(r#"#game-posture-controls select[name="{name}"]"#))
                    .await;
                assert!(
                    select.is_visible().await.unwrap_or(false),
                    "{name} select is rendered"
                );
            }
        },
    )
    .await;
}

// [docs/specs/browser_games.md] SCENARIO: 27.2
#[tokio::test]
async fn test_games_tense_change_autosaves_and_rerenders() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            page.locator(r#".tab[data-tab="games"]"#)
                .await
                .click(None)
                .await
                .unwrap();
            wait_until_visible(&page, "#game-posture-controls", Duration::from_millis(1000)).await;

            page.locator(r#"#game-posture-controls select[name="narrative_tense"]"#)
                .await
                .select_option("present", None)
                .await
                .unwrap();

            // The POST /games/:id/posture response swaps the fragment
            // (outerHTML), so the locator re-resolves to the fresh select.
            let tense_select = page
                .locator(r#"#game-posture-controls select[name="narrative_tense"]"#)
                .await;
            if let Err(e) = expect(tense_select)
                .with_timeout(std::time::Duration::from_secs(5))
                .to_have_value("present")
                .await
            {
                panic!("fragment should re-render with tense present after auto-save: {e}");
            }
        },
    )
    .await;
}

// [docs/specs/browser_games.md] SCENARIO: 27.3
#[tokio::test]
async fn test_games_mode_switch_retargets_and_nudges() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            page.locator(r#".tab[data-tab="games"]"#)
                .await
                .click(None)
                .await
                .unwrap();
            wait_until_visible(&page, "#game-posture-controls", Duration::from_millis(1000)).await;

            page.locator(r#"#game-posture-controls select[name="narrator_mode"]"#)
                .await
                .select_option("interactive_fiction", None)
                .await
                .unwrap();

            let perspective_select = page
                .locator(r#"#game-posture-controls select[name="narrative_perspective"]"#)
                .await;
            if let Err(e) = expect(perspective_select)
                .with_timeout(std::time::Duration::from_secs(5))
                .to_have_value("second")
                .await
            {
                panic!("mode switch should nudge perspective to the IF default: {e}");
            }

            let system_select = page
                .locator(r#"#game-posture-controls select[name="system_preset_id"]"#)
                .await;
            if let Err(e) = expect(system_select)
                .with_timeout(std::time::Duration::from_secs(5))
                .to_have_value("system_if_default")
                .await
            {
                panic!("mode switch should retarget the system preset to the IF bundle: {e}");
            }
        },
    )
    .await;
}
