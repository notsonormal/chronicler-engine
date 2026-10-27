//! Stub-browser tests for the action controls: the disabled state that a check request gives its own controls, the view states of the action area, and the load that follows a request. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use chronicler_engine::domain::model::state::generation_status::Step;

use super::support::{
    hold_requests_ending_with, read_command_input, release_held_requests_and_wait_for_answers,
    StubRunner,
};
use super::*;

/// The Send button disabled, the Action input disabled, the Send label, and the
/// state of the action area.
async fn read_controls(page: &playwright_rs::Page) -> (bool, bool, String, String) {
    page.evaluate::<(), (bool, bool, String, String)>(
        r#"(() => {
            const send = document.getElementById('submit-btn');
            const input = document.getElementById('command-input');
            const area = document.getElementById('action-area');
            return [send.disabled, input.disabled, send.textContent.trim(), area.dataset.state || ''];
        })()"#,
        None,
    )
    .await
    .unwrap()
}

async fn wait_for_controls(
    page: &playwright_rs::Page,
    shows: &str,
    wanted: impl Fn(&(bool, bool, String, String)) -> bool,
) {
    let reached = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { wanted(&read_controls(page).await) },
    )
    .await;
    assert!(
        reached,
        "{shows}; the controls read {:?}",
        read_controls(page).await
    );
}

async fn press_send(page: &playwright_rs::Page, command: &str) {
    fill_command_input(page, command).await;
    page.locator("#submit-btn").await.click(None).await.unwrap();
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.62
async fn check_a_check_request_disables_send_and_the_input_until_it_ends(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    hold_requests_ending_with(&page, "/action/check").await;

    press_send(&page, "look around").await;

    wait_for_controls(
        &page,
        "the request disables Send and the Action input and the area is checking",
        |(send, input, _, state)| *send && *input && state == "checking",
    )
    .await;

    release_held_requests_and_wait_for_answers(&page).await;
    wait_for_controls(
        &page,
        "the end of the request enables Send and the Action input",
        |(send, input, _, state)| !send && !input && state != "checking",
    )
    .await;
    assert_eq!(
        read_command_input(&page).await,
        "look around",
        "the engine refused the command, so the input keeps it"
    );
}

async fn held_requests_seen(page: &playwright_rs::Page) -> f64 {
    page.evaluate::<(), f64>("() => window.heldRequestsSeen", None)
        .await
        .unwrap()
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.63
async fn check_a_send_while_a_check_request_runs_is_not_sent_twice(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    hold_requests_ending_with(&page, "/action/check").await;

    press_send(&page, "look around").await;
    let first_held = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(10),
        || async { held_requests_seen(&page).await >= 1.0 },
    )
    .await;
    assert!(first_held, "the first send leaves the page and is held");

    page.evaluate::<(), ()>(
        r#"() => {
            document.getElementById('submit-btn').click();
            document.getElementById('command-form').requestSubmit();
            document.getElementById('command-input').focus();
        }"#,
        None,
    )
    .await
    .unwrap();
    page.keyboard().press("Enter", None).await.unwrap();

    assert_eq!(
        held_requests_seen(&page).await,
        1.0,
        "a click, a submit and Enter while the request runs send nothing more"
    );

    release_held_requests_and_wait_for_answers(&page).await;
    wait_for_screen_current(&page).await;
    assert_eq!(
        held_requests_seen(&page).await,
        1.0,
        "the end of the request sends no second request that htmx queued"
    );
    page.evaluate::<(), ()>("() => window.releaseHeldRequests()", None)
        .await
        .unwrap();
    wait_for_screen_current(&page).await;
    assert_eq!(
        stub.action_handle().count(),
        1,
        "the engine received one request that checks a command"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.64
async fn check_a_load_during_a_check_request_leaves_the_controls_to_the_load_after_it(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    hold_requests_ending_with(&page, "/action/check").await;
    press_send(&page, "look around").await;
    wait_for_controls(&page, "the request disables Send", |(send, ..)| *send).await;

    let screen_requests = stub.changes_handle().screen_requests().len();
    stub.status_handle().set(StubStatus::Step(Step::Narrating));
    let loaded = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { stub.changes_handle().screen_requests().len() > screen_requests },
    )
    .await;
    assert!(loaded, "the page loads the screen while the request runs");
    assert!(
        read_controls(&page).await.0,
        "Send is still disabled while the request runs"
    );

    release_held_requests_and_wait_for_answers(&page).await;
    wait_for_screen_current(&page).await;
    wait_for_controls(
        &page,
        "the load after the request renders Send as the server does",
        |(send, _, label, _)| *send && label.contains("Generating"),
    )
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.65
async fn check_a_check_request_that_starts_with_send_rendered_disabled_leaves_it_disabled(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.status_handle().set(StubStatus::Step(Step::Narrating));
    load_screen_now(&page).await;
    wait_for_controls(&page, "the server renders Send disabled", |(send, ..)| {
        *send
    })
    .await;

    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(
        "() => document.getElementById('command-form').requestSubmit()",
        None,
    )
    .await
    .unwrap();
    let sent = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { stub.action_handle().count() == 1 },
    )
    .await;
    assert!(
        sent,
        "the engine receives the one request that checks a command"
    );
    wait_for_screen_current(&page).await;

    wait_for_controls(
        &page,
        "the end of the request leaves Send disabled and generating",
        |(send, _, label, _)| *send && label.contains("Generating"),
    )
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.66
async fn check_the_action_area_is_in_the_preview_state_while_the_preview_is_open(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_screen_current(&page).await;

    press_send(&page, "look at the casle").await;
    wait_until_visible(&page, ".text-check-preview", Duration::from_secs(5)).await;
    wait_for_controls(&page, "the area is in the preview state", |(.., state)| {
        state == "preview"
    })
    .await;

    page.locator(".text-check-preview .btn-original")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;
    wait_for_controls(&page, "the area leaves the preview state", |(.., state)| {
        state != "preview"
    })
    .await;
}

/// The ids of the controls that Tab reaches from the Action input, one press each.
async fn tab_order_from_the_action_input(
    page: &playwright_rs::Page,
    presses: usize,
) -> Vec<String> {
    page.evaluate::<(), ()>(
        "() => document.getElementById('command-input').focus()",
        None,
    )
    .await
    .unwrap();
    let mut reached = Vec::new();
    for _ in 0..presses {
        page.keyboard().press("Tab", None).await.unwrap();
        reached.push(
            page.evaluate::<(), String>("() => document.activeElement.id", None)
                .await
                .unwrap(),
        );
    }
    reached
}

/// The ids of the action area's controls, from the left edge to the right.
async fn visual_order_of_the_controls(page: &playwright_rs::Page) -> Vec<String> {
    page.evaluate::<(), Vec<String>>(
        r#"(() => {
            const ids = ['command-input', 'submit-btn', 'autoplay-cycles', 'autoplay-btn'];
            return ids
                .map((id) => [id, document.getElementById(id).getBoundingClientRect().left])
                .sort((a, b) => a[1] - b[1])
                .map((entry) => entry[0]);
        })()"#,
        None,
    )
    .await
    .unwrap()
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.67
async fn check_tab_follows_the_visual_order_of_the_action_controls(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    wait_for_controls(&page, "the screen shows Send", |_| true).await;

    assert_eq!(
        visual_order_of_the_controls(&page).await,
        [
            "command-input",
            "submit-btn",
            "autoplay-cycles",
            "autoplay-btn"
        ],
        "from left to right: the Action input, Send, the cycles field, Autoplay"
    );
    assert_eq!(
        tab_order_from_the_action_input(&page, 3).await,
        ["submit-btn", "autoplay-cycles", "autoplay-btn"],
        "Tab from the Action input reaches Send, the cycles field, then Autoplay"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.68
async fn check_a_swap_that_throws_does_not_leave_the_loads_waiting(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    // The engine refuses the command and changes nothing, so no change moves the
    // stamp: only the end of the failed request can ask for the load.
    page.evaluate::<(), ()>(
        r#"() => {
            window.__loadErrors = 0;
            document.body.addEventListener('htmx:beforeSwap', (evt) => {
                if (evt.detail.requestConfig.path === '/action/check') {
                    evt.detail.shouldSwap = true;
                    evt.detail.target = null;
                }
            });
            document.body.addEventListener('htmx:onLoadError', () => { window.__loadErrors += 1; });
        }"#,
        None,
    )
    .await
    .unwrap();

    press_send(&page, "look around").await;
    let failed = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            page.evaluate::<(), f64>("() => window.__loadErrors", None)
                .await
                .unwrap_or(0.0)
                >= 1.0
        },
    )
    .await;
    assert!(
        failed,
        "the swap of the check answer throws, and htmx says so"
    );

    wait_for_controls(
        &page,
        "a load after the failed swap renders Send enabled and the area leaves checking",
        |(send, _, label, state)| !send && label == "Send" && state != "checking",
    )
    .await;
}

#[tokio::test]
async fn run_action_controls_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_a_check_request_disables_send_and_the_input_until_it_ends,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_a_send_while_a_check_request_runs_is_not_sent_twice,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_a_load_during_a_check_request_leaves_the_controls_to_the_load_after_it,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_a_check_request_that_starts_with_send_rendered_disabled_leaves_it_disabled,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_the_action_area_is_in_the_preview_state_while_the_preview_is_open,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_tab_follows_the_visual_order_of_the_action_controls,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_a_swap_that_throws_does_not_leave_the_loads_waiting,
        )
        .await;
    runner.finish().await;
}
