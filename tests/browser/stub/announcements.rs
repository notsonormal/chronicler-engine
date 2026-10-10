//! Stub-browser tests for the live-region announcements: each change is announced exactly once. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use playwright_rs::AriaRole;

use chronicler_engine::domain::model::state::generation_status::GenerationFailureKind;

use super::support::poll_now;
use super::*;
use super::support::StubRunner;

async fn watch_live_region(page: &playwright_rs::Page, region_id: &str) {
    page.evaluate::<String, ()>(
        r#"(id) => {
            const region = document.getElementById(id);
            window.__liveCounts = window.__liveCounts || {};
            window.__liveCounts[id] = 0;
            new MutationObserver(() => {
                window.__liveCounts[id] += 1;
            }).observe(region, { childList: true, characterData: true, subtree: true });
        }"#,
        Some(&region_id.to_string()),
    )
    .await
    .unwrap();
}

async fn live_region_changes(page: &playwright_rs::Page, region_id: &str) -> i64 {
    page.evaluate::<String, i64>(
        r#"(id) => (window.__liveCounts && window.__liveCounts[id]) || 0"#,
        Some(&region_id.to_string()),
    )
    .await
    .unwrap()
}

async fn read_live_region(page: &playwright_rs::Page, region_id: &str) -> (String, String) {
    page.evaluate::<String, (String, String)>(
        r#"(id) => {
            const region = document.getElementById(id);
            if (!region) return ['', ''];
            return [region.getAttribute('role') || '', region.textContent.trim()];
        }"#,
        Some(&region_id.to_string()),
    )
    .await
    .unwrap()
}

async fn assert_live_region_role(page: &playwright_rs::Page, region_id: &str, role: AriaRole) {
    let by_role = page.get_by_role(role, None).await;
    let by_id = page.locator(&format!("#{region_id}")).await;
    let matches = by_role.and_(&by_id).count().await.unwrap_or(0);
    assert_eq!(
        matches, 1,
        "#{region_id} must compute to the {role:?} accessibility role"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.23
async fn check_status_changes_are_announced_once(page: playwright_rs::Page, stub: StubServer) {
    let status = stub.status_handle();
    assert_live_region_role(&page, "status-announcer", AriaRole::Status).await;
    assert_live_region_role(&page, "status-error-announcer", AriaRole::Alert).await;
    let (_, phase_text) = read_live_region(&page, "status-announcer").await;
    assert_eq!(phase_text, "", "the phase announcer starts empty");

    watch_live_region(&page, "status-announcer").await;
    watch_live_region(&page, "status-error-announcer").await;

    status.set(StubStatus::Phase("narrating".to_string()));
    poll_now(&page, "#status-display").await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(8),
            Duration::from_millis(100),
            || async {
                read_live_region(&page, "status-announcer")
                    .await
                    .1
                    .contains("Generating narration")
            }
        )
        .await,
        "the phase transition should be announced"
    );
    assert_eq!(
        live_region_changes(&page, "status-announcer").await,
        1,
        "the phase change should be announced exactly once"
    );

    poll_now(&page, "#status-display").await;
    assert_eq!(
        live_region_changes(&page, "status-announcer").await,
        1,
        "an unchanged phase must not be re-announced on a later poll"
    );

    status.set(StubStatus::Error(
        GenerationFailureKind::Other,
        "narration failed".to_string(),
    ));
    poll_now(&page, "#status-display").await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(8),
            Duration::from_millis(100),
            || async {
                !read_live_region(&page, "status-error-announcer")
                    .await
                    .1
                    .is_empty()
            }
        )
        .await,
        "the generation error should be announced"
    );
    let (_, error_text) = read_live_region(&page, "status-error-announcer").await;
    assert_eq!(
        error_text, "The last turn failed to generate.",
        "the error announcer must carry the short user-facing line, not the raw text"
    );

    status.set(StubStatus::Idle);
    poll_now(&page, "#status-display").await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(8),
            Duration::from_millis(100),
            || async { read_live_region(&page, "status-announcer").await.1 == "Ready" }
        )
        .await,
        "the return to Ready should be announced"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.24
async fn check_new_narration_and_options_are_announced_once(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let log = stub.story_log_handle();
    let options = stub.options_handle();
    assert_live_region_role(&page, "narration-announcer", AriaRole::Status).await;
    assert_live_region_role(&page, "options-announcer", AriaRole::Status).await;

    watch_live_region(&page, "narration-announcer").await;
    watch_live_region(&page, "options-announcer").await;

    assert_eq!(
        read_live_region(&page, "narration-announcer").await.1,
        "",
        "loading the page must not announce the whole log"
    );

    log.append_narration("A new dawn breaks over the courtyard.");
    poll_now(&page, "#story-log").await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(8),
            Duration::from_millis(100),
            || async {
                read_live_region(&page, "narration-announcer")
                    .await
                    .1
                    .contains("A new dawn breaks over the courtyard.")
            }
        )
        .await,
        "the new narration should be announced"
    );
    let narration = read_live_region(&page, "narration-announcer").await.1;
    assert!(
        !narration.contains("Welcome to the Test World"),
        "the announcement must be scoped to the changed entry, got {narration:?}"
    );
    assert_eq!(
        live_region_changes(&page, "narration-announcer").await,
        1,
        "the new narration should be announced exactly once"
    );

    wait_for_element_children(&page, "#options-dock .option-item", 3).await;
    options.serve_alternate_set(true);
    poll_now(&page, "#options-dock").await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(8),
            Duration::from_millis(100),
            || async {
                read_live_region(&page, "options-announcer")
                    .await
                    .1
                    .contains("Search the cellar")
            }
        )
        .await,
        "the changed option set should be announced"
    );
    assert_eq!(
        live_region_changes(&page, "options-announcer").await,
        1,
        "the changed option set should be announced exactly once"
    );

    poll_now(&page, "#story-log").await;
    poll_now(&page, "#options-dock").await;
    assert_eq!(
        live_region_changes(&page, "narration-announcer").await,
        1,
        "a later poll must not re-announce the narration"
    );
    assert_eq!(
        live_region_changes(&page, "options-announcer").await,
        1,
        "a later poll must not re-announce the options"
    );
}

#[tokio::test]
async fn run_announcements_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_status_changes_are_announced_once,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_new_narration_and_options_are_announced_once,
        )
        .await;
    runner.finish().await;
}
