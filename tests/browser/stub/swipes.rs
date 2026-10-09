//! Stub-browser test for switching swipes: the client's restore handling and the dock it leaves behind.

use std::time::Duration;

use super::*;

async fn read_swipe_counter(page: &playwright_rs::Page) -> String {
    page.evaluate::<(), String>(
        r#"(() => {
            const counter = document.querySelector('#story-log .swipe-counter');
            return counter ? counter.textContent.trim() : '';
        })()"#,
        None,
    )
    .await
    .unwrap_or_default()
}

/// Poll the story log's swipe counter until it reads `expected`.
async fn wait_for_swipe_counter(page: &playwright_rs::Page, expected: &str) {
    let matched = wait_for_condition_async(
        Duration::from_secs(10),
        Duration::from_millis(200),
        || async { read_swipe_counter(page).await == expected },
    )
    .await;
    if !matched {
        let last = read_swipe_counter(page).await;
        capture_failure_state(page, "wait_for_swipe_counter").await;
        panic!("swipe counter never became {expected:?}, last saw {last:?}");
    }
}

async fn option_item_count(page: &playwright_rs::Page) -> usize {
    page.query_selector_all("#options-dock .option-item")
        .await
        .expect("the options dock query should succeed")
        .len()
}

/// Poll the dock until it renders exactly `expected` `.option-item` rows.
async fn wait_for_option_items(page: &playwright_rs::Page, expected: usize) {
    let matched = wait_for_condition_async(
        Duration::from_secs(10),
        Duration::from_millis(200),
        || async { option_item_count(page).await == expected },
    )
    .await;
    if !matched {
        let last = option_item_count(page).await;
        capture_failure_state(page, "wait_for_option_items").await;
        panic!("dock never showed {expected} option item(s), last saw {last}");
    }
}

// [docs/specs/browser_swipes.md] SCENARIO: 37.1
#[tokio::test]
async fn test_switch_swipe_leaves_dashboard_ready_and_drops_options() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let swipe = stub.swipe_handle();
        let status = stub.status_handle();
        async move {
            swipe.set_two_swipes();
            wait_for_swipe_counter(&page, "2 / 2").await;
            wait_for_option_items(&page, 3).await;

            // Reproduce the stuck state: the status poll reports a generation
            // still narrating while the log still offers a switch. Clearing it
            // must not wait for another poll.
            status.set(StubStatus::Phase("narrating".to_string()));
            wait_for_status_generating(&page).await;

            let clicked: bool = page
                .evaluate::<(), bool>(
                    r#"(() => {
                        const btn = document.querySelector(".swipe-btn[title='Previous swipe']");
                        if (!btn) return false;
                        btn.click();
                        return true;
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(
                clicked,
                "the two-swipe log must render a Previous-swipe control"
            );

            wait_for_swipe_counter(&page, "1 / 2").await;

            let (status, disabled): (String, bool) = page
                .evaluate::<(), (String, bool)>(
                    r#"(() => {
                        const status = document.getElementById('status-display');
                        const btn = document.getElementById('submit-btn');
                        return [
                            status ? status.textContent.trim() : '',
                            btn ? btn.disabled : true,
                        ];
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(
                status.contains("Ready"),
                "after a switch the status must be Ready, got {status:?}"
            );
            assert!(!disabled, "Send must be enabled after a swipe switch");

            // The restore is surfaced to the player, not left to the counter.
            let notice: String = page
                .evaluate::<(), String>(
                    r#"(() => {
                        const el = document.getElementById('restore-notice');
                        return el ? el.textContent.trim() : '';
                    })()"#,
                    None,
                )
                .await
                .unwrap_or_default();
            assert!(
                notice.contains("Restored swipe"),
                "the restore must be visible to the player, got {notice:?}"
            );

            // The restored Swipe carries no option set, so the dock empties.
            wait_for_option_items(&page, 0).await;
        }
    })
    .await;
}
