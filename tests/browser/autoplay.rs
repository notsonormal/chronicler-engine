//! Browser Autoplay tests: the Autoplay control's full-stack hop, and the view-only Game while a run holds it. A run that must hold its Game narrates through a `SilentModel`. Tagged against `docs/specs/browser_autoplay.md`.

use std::time::Duration;

use super::*;

const RUN_STATUS: &str = "Autoplay: cycle 1/10";

async fn wait_for<Fut>(description: &str, condition: impl Fn() -> Fut)
where
    Fut: std::future::Future<Output = bool>,
{
    wait_for_within(Duration::from_secs(10), description, condition).await;
}

async fn wait_for_within<Fut>(timeout: Duration, description: &str, condition: impl Fn() -> Fut)
where
    Fut: std::future::Future<Output = bool>,
{
    assert!(
        wait_for_condition_async(timeout, Duration::from_millis(100), condition).await,
        "timed out waiting for: {description}"
    );
}

/// An empty log fails, so the check cannot pass vacuously.
async fn story_controls_all(page: &playwright_rs::Page, disabled: bool) -> bool {
    page.evaluate::<bool, bool>(
        r#"(disabled) => ['.edit-btn', '.delete-btn', '.swipe-btn[aria-label="Retry"]'].every((selector) => {
            const controls = Array.from(document.querySelectorAll('#story-log ' + selector));
            return controls.length > 0 && controls.every((control) => control.disabled === disabled);
        })"#,
        Some(&disabled),
    )
    .await
    .unwrap()
}

async fn config_selects_all(page: &playwright_rs::Page, disabled: bool) -> bool {
    let ids: Vec<String> = GAME_CONFIG_SELECT_IDS
        .iter()
        .map(|id| id.to_string())
        .collect();
    page.evaluate::<Vec<String>, bool>(
        &format!(
            r#"(ids) => ids.every((id) => {{
                const select = document.getElementById(id);
                return !!select && select.disabled === {disabled};
            }})"#
        ),
        Some(&ids),
    )
    .await
    .unwrap()
}

async fn show_tab(page: &playwright_rs::Page, tab: &str) {
    page.evaluate::<String, ()>(
        "(tab) => document.querySelector(`.tab[data-tab=\"${tab}\"]`).click()", // settle-guard-exempt: a tab switch makes no htmx request.
        Some(&tab.to_string()),
    )
    .await
    .unwrap();
}

async fn control_is_usable(page: &playwright_rs::Page, selector: &str) -> bool {
    let locator = page.locator(selector).await;
    locator.is_visible().await.unwrap() && locator.is_enabled().await.unwrap()
}

async fn start_a_held_run(page: &playwright_rs::Page, port: u16) -> SilentModel {
    send_action(page, "look").await;
    wait_for_status_ready(page).await;
    wait_for_element_children(page, "#story-log .log-entry", 3).await;
    assert!(
        story_controls_all(page, false).await,
        "before the run, the controls are enabled"
    );

    let silent_model = inject_silent_narrator(port).await;
    click_and_settle(page, "#autoplay-btn", "#status-display")
        .await
        .expect_settled("start the run");
    wait_for("the status display shows the run", || async {
        status_display_text(page).await.starts_with(RUN_STATUS)
    })
    .await;
    silent_model
}

// [docs/specs/browser_autoplay.md] SCENARIO: 42.1
#[tokio::test]
async fn test_the_autoplay_control_starts_a_run_with_the_default_budget() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, port| async move {
            let budget: String = page
                .evaluate::<(), String>(
                    "(() => document.getElementById('autoplay-cycles').value)()",
                    None,
                )
                .await
                .unwrap();
            assert_eq!(budget, "10", "the budget field defaults to 10");
            wait_for_status_ready(&page).await;
            assert_eq!(
                status_display_text(&page).await,
                "Ready",
                "a fresh Game starts from Ready"
            );

            let _silent_model = inject_silent_narrator(port).await;
            click_and_settle(&page, "#autoplay-btn", "#status-display")
                .await
                .expect_settled("start the run");

            // The status display loads only when a number moves, so a run that shows within 3
            // seconds shows because the start asks at once.
            wait_for_within(
                Duration::from_secs(3),
                "the status display shows the run with its budget",
                || async {
                    status_display_text(&page).await
                        == "Autoplay: cycle 1/10 Generating narration..."
                },
            )
            .await;
        },
    )
    .await;
}

// [docs/specs/browser_autoplay.md] SCENARIO: 42.3
#[tokio::test]
async fn test_while_a_run_holds_the_game_the_story_changing_controls_are_disabled_until_stop() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, port| async move {
            let _silent_model = start_a_held_run(&page, port).await;

            wait_for("the story log renders its controls disabled", || async {
                story_controls_all(&page, true).await
            })
            .await;
            // The Games panel is in a section that is hidden, so it loads when
            // the section opens, and renders what the run holds by then.
            open_games_tab(&page).await;
            wait_for(
                "the Games panel renders its config controls disabled",
                || async { config_selects_all(&page, true).await },
            )
            .await;
            show_tab(&page, "game").await;

            click_and_settle(&page, "#stop-btn", "#status-display")
                .await
                .expect_settled("stop the run");

            wait_for_status_ready(&page).await;
            assert!(control_is_usable(&page, "#command-input").await);
            assert!(control_is_usable(&page, "#submit-btn").await);
            wait_for("the story log renders its controls enabled", || async {
                story_controls_all(&page, false).await
            })
            .await;
            show_tab(&page, "games").await;
            wait_for(
                "the Games panel renders its config controls enabled",
                || async { config_selects_all(&page, false).await },
            )
            .await;
        },
    )
    .await;
}

// [docs/specs/browser_autoplay.md] SCENARIO: 42.4
#[tokio::test]
async fn test_the_games_list_marks_a_game_that_a_run_holds() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, port| async move {
            open_games_tab(&page).await;
            assert_eq!(
                page.locator("#games-tab .run-badge")
                    .await
                    .count()
                    .await
                    .unwrap(),
                0,
                "no run holds a Game yet"
            );
            show_tab(&page, "game").await;

            let _silent_model = inject_silent_narrator(port).await;
            click_and_settle(&page, "#autoplay-btn", "#status-display")
                .await
                .expect_settled("start the run");
            show_tab(&page, "games").await;
            wait_for("the loaded Games panel marks the Game", || async {
                page.locator("#games-tab .game-item.active .run-badge")
                    .await
                    .count()
                    .await
                    .unwrap()
                    == 1
            })
            .await;

            show_tab(&page, "game").await;
            click_and_settle(&page, "#stop-btn", "#status-display")
                .await
                .expect_settled("stop the run");
            show_tab(&page, "games").await;
            wait_for("the Games panel drops the mark", || async {
                page.locator("#games-tab .run-badge")
                    .await
                    .count()
                    .await
                    .unwrap()
                    == 0
            })
            .await;
        },
    )
    .await;
}
