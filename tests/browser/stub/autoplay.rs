//! Stub-browser tests for the Autoplay control: the progress status and the action area while a run holds the Game. Tagged against `docs/specs/browser_autoplay.md`.

use std::time::Duration;

use super::support::{poll_now, StubRunner};
use super::*;

async fn poll_answer(page: &playwright_rs::Page, stub: &StubServer, answer: &str) {
    stub.status_handle()
        .set(StubStatus::Step(answer.to_string()));
    poll_now(page, "#status-display").await;
}

async fn status_text(page: &playwright_rs::Page) -> String {
    page.locator("#status-display")
        .await
        .inner_text()
        .await
        .unwrap_or_default()
        .trim()
        .to_string()
}

async fn is_visible(page: &playwright_rs::Page, selector: &str) -> bool {
    page.locator(selector).await.is_visible().await.unwrap()
}

async fn is_enabled(page: &playwright_rs::Page, selector: &str) -> bool {
    page.locator(selector).await.is_enabled().await.unwrap()
}

const COMMAND_FORM_AND_START: [&str; 3] = ["#command-input", "#submit-btn", "#autoplay-btn"];

// [docs/specs/browser_autoplay.md] SCENARIO: 42.2
async fn check_status_shows_the_run_progress_and_the_action_area_offers_only_stop(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    poll_answer(&page, &stub, "autoplay 2/10 narrating").await;
    assert_eq!(
        status_text(&page).await,
        "Autoplay: cycle 2/10 Generating narration..."
    );
    assert!(
        is_visible(&page, "#stop-btn").await,
        "Stop shows during a run"
    );
    for selector in COMMAND_FORM_AND_START {
        assert!(
            !is_visible(&page, selector).await,
            "{selector} is out of sight while a run holds the Game"
        );
    }

    poll_answer(&page, &stub, "autoplay 2/10").await;
    assert_eq!(status_text(&page).await, "Autoplay: cycle 2/10");
    assert!(
        is_visible(&page, "#stop-btn").await,
        "Stop shows between two Jobs of the run"
    );

    poll_answer(&page, &stub, "idle").await;
    assert_eq!(status_text(&page).await, "Ready");
    assert!(
        !is_visible(&page, "#stop-btn").await,
        "Stop is gone once the run ends"
    );
    for selector in COMMAND_FORM_AND_START {
        assert!(
            is_visible(&page, selector).await && is_enabled(&page, selector).await,
            "{selector} is usable again once the run ends"
        );
    }
}

// [docs/specs/browser_autoplay.md] SCENARIO: 42.6
async fn check_a_save_refused_for_a_held_game_keeps_the_editor_open(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.save_handle().set_save_refused_as_busy(true);
    page.locator(".log-entry.narration .edit-btn")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;
    page.locator("#edit-textarea")
        .await
        .fill("Text typed before the run started", None)
        .await
        .unwrap();
    page.locator(".log-entry.narration .save-btn")
        .await
        .click(None)
        .await
        .unwrap();

    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(50),
            || async { read_error_disclosure(&page, "#story-log-error").await.0 }
        )
        .await,
        "the refused save reports in the story log's error slot"
    );
    let (_, line, detail) = read_error_disclosure(&page, "#story-log-error").await;
    assert!(
        line.contains("busy"),
        "the line names the busy Game, got {line:?}"
    );
    assert!(
        detail.contains("Still thinking..."),
        "the details carry the engine's answer, got {detail:?}"
    );
    let textarea = page.locator("#edit-textarea").await;
    assert_eq!(
        textarea.input_value(None).await.unwrap(),
        "Text typed before the run started",
        "the editor keeps the player's text"
    );
    let trigger: String = page
        .evaluate::<(), String>(
            "(() => document.getElementById('story-log').getAttribute('hx-trigger') || '')()",
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        trigger, "none",
        "the story-log poll stays paused while the editor is open"
    );
}

#[tokio::test]
async fn run_autoplay_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_status_shows_the_run_progress_and_the_action_area_offers_only_stop,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_save_refused_for_a_held_game_keeps_the_editor_open,
        )
        .await;
    runner.finish().await;
}
