//! Browser dashboard-chrome tests: static command form, status display. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use playwright_rs::expect;

use super::*;

// [docs/specs/browser_dashboard.md] SCENARIO: 16.5
#[tokio::test]
async fn test_form_stays_static_after_submission() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let form_id_before: String = page
                .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
                .await
                .unwrap();

            // `#command-form` sits outside every swap target, so it is never
            // re-registered and has no registering settle to race.
            send_action(&page, "look").await;

            wait_for_element_children(&page, "#story-log .log-entry", 2).await;

            let form_id_after: String = page
                .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
                .await
                .unwrap();

            assert_eq!(
                form_id_before, form_id_after,
                "Form should stay in DOM (static shell)"
            );
        },
    )
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.6
#[tokio::test]
async fn test_status_updates_during_generation() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            send_action(&page, "wait").await;

            let status_locator = page.locator("#status-display").await;
            let _ = expect(status_locator)
                .with_timeout(Duration::from_millis(500))
                .not()
                .to_contain_text("Ready")
                .await;

            let status_text = page
                .locator("#status-display")
                .await
                .inner_text()
                .await
                .unwrap_or_default();
            assert!(
                status_text.contains("Thinking")
                    || status_text.contains("Narrating")
                    || status_text.contains("Generating")
                    || status_text.contains("Quantifying"),
                "Status should show generating state, got: {status_text}"
            );
        },
    )
    .await;
}
