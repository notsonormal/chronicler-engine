//! Stub-browser tests for the action controls: what a check request does to the controls of the action area, the view states of the action area, and the load that follows a request. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use chronicler_engine::domain::model::state::generation_status::Step;

use super::support::{
    NOTICE, eval_string, hold_requests_ending_with, hold_screen_loads, read_command_input,
    release_held_requests_and_wait_for_answers, release_screen_loads, wait_for_held_screen_load,
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

/// Send looks busy and takes no click while the form that it submits has a request in
/// flight, though the part renders it enabled.
async fn send_takes_clicks(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        "() => getComputedStyle(document.getElementById('submit-btn')).pointerEvents !== 'none'",
        None,
    )
    .await
    .unwrap()
}

async fn press_send(page: &playwright_rs::Page, command: &str) {
    fill_command_input(page, command).await;
    page.locator("#submit-btn").await.click(None).await.unwrap();
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.62
async fn check_a_check_request_disables_the_input_and_shows_send_as_busy_until_it_ends(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    hold_requests_ending_with(&page, "/action/check").await;

    press_send(&page, "look around").await;

    wait_for_controls(
        &page,
        "the request disables the Action input and the area is checking",
        |(_, input, _, state)| *input && state == "checking",
    )
    .await;
    let (send_disabled, ..) = read_controls(&page).await;
    assert!(
        !send_disabled,
        "no script and no request disables Send, the server renders it enabled"
    );
    assert!(
        !send_takes_clicks(&page).await,
        "Send looks busy and takes no click while its form has a request in flight"
    );

    release_held_requests_and_wait_for_answers(&page).await;
    wait_for_controls(
        &page,
        "the end of the request enables the Action input",
        |(send, input, _, state)| !send && !input && state != "checking",
    )
    .await;
    assert!(
        send_takes_clicks(&page).await,
        "Send takes clicks again when the request has ended"
    );
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
async fn check_a_load_during_a_check_request_renders_send_as_the_server_does(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    hold_requests_ending_with(&page, "/action/check").await;
    press_send(&page, "look around").await;
    wait_for_controls(
        &page,
        "the request disables the Action input",
        |(_, input, ..)| *input,
    )
    .await;

    stub.status_handle().set(StubStatus::Step(Step::Narrating));
    wait_for_controls(
        &page,
        "the load that lands during the request renders Send as the server does",
        |(send, _, label, _)| *send && label.contains("Generating"),
    )
    .await;

    release_held_requests_and_wait_for_answers(&page).await;
    wait_for_screen_current(&page).await;
    wait_for_controls(
        &page,
        "the end of the request leaves Send as the server renders it",
        |(send, input, label, _)| *send && !input && label.contains("Generating"),
    )
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.65
async fn check_a_check_request_never_writes_disabled_on_send(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.status_handle().set(StubStatus::Step(Step::Narrating));
    load_screen_now(&page).await;
    wait_for_controls(&page, "the server renders Send disabled", |(send, ..)| {
        *send
    })
    .await;
    page.evaluate::<(), ()>(
        r#"() => {
            window.__sendDisabledWrites = 0;
            new MutationObserver((records) => { window.__sendDisabledWrites += records.length; })
                .observe(document.getElementById('submit-btn'), { attributes: true, attributeFilter: ['disabled'] });
        }"#,
        None,
    )
    .await
    .unwrap();

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
    assert_eq!(
        page.evaluate::<(), f64>("() => window.__sendDisabledWrites", None)
            .await
            .unwrap(),
        0.0,
        "the request wrote `disabled` on Send neither at its start nor at its end"
    );
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
async fn check_a_swap_that_throws_leaves_the_command_form_dead_and_the_screen_loading(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
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
    let loads_before = changes.loads("/screen");

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

    let loaded = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { changes.loads("/screen") > loads_before },
    )
    .await;
    assert!(
        loaded,
        "the end of the failed request forgets the stamp, so the screen loads again with no change"
    );
    wait_for_screen_current(&page).await;
    wait_for_controls(
        &page,
        "a load after the failed swap renders Send enabled, the area is not checking, and the Action input stays disabled",
        |(send, input, label, state)| !send && *input && label == "Send" && state != "checking",
    )
    .await;
    assert!(
        !send_takes_clicks(&page).await,
        "the form keeps its request, so Send still looks busy and takes no click"
    );

    page.evaluate::<(), ()>(
        "() => document.getElementById('command-form').requestSubmit()",
        None,
    )
    .await
    .unwrap();
    // A submit that the dead form let through would reach the stub within a few
    // milliseconds; the window only has to outlast that.
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(
        stub.action_handle().count(),
        1,
        "the dead form drops every later submit until the page reloads"
    );
}

/// The requests that start the pending state of the action area, each from its own
/// control. The retry and the retrigger share one path in the client, and each has
/// its own control.
#[derive(Clone, Copy, Debug)]
enum PendingStarter {
    /// The Send button of the command form.
    Send,
    /// Send Original in the text-check preview.
    PreviewSend,
    /// The Autoplay button.
    AutoplayStart,
    /// The Retry control of the last narration.
    Retry,
    /// The Retrigger control of the last narration.
    Retrigger,
    /// The New options button of the options dock.
    NewOptions,
}

/// The starters that begin the pending state before the answer comes. The other
/// three begin it only when the engine took the request, so a refusal leaves
/// nothing to end.
const STARTERS_BEFORE_THE_ANSWER: [PendingStarter; 3] = [
    PendingStarter::Retry,
    PendingStarter::Retrigger,
    PendingStarter::NewOptions,
];

const PENDING_STARTERS: [PendingStarter; 6] = [
    PendingStarter::Send,
    PendingStarter::PreviewSend,
    PendingStarter::AutoplayStart,
    PendingStarter::Retry,
    PendingStarter::Retrigger,
    PendingStarter::NewOptions,
];

async fn start_pending_request(page: &playwright_rs::Page, starter: PendingStarter) {
    match starter {
        PendingStarter::Send => press_send(page, "look around").await,
        PendingStarter::PreviewSend => {
            press_send(page, "look at the casle").await;
            wait_until_visible(page, ".text-check-preview", Duration::from_secs(5)).await;
            page.locator(".text-check-preview .btn-original")
                .await
                .click(None)
                .await
                .unwrap();
        }
        PendingStarter::AutoplayStart => {
            page.locator("#autoplay-btn")
                .await
                .click(None)
                .await
                .unwrap();
        }
        PendingStarter::Retry => {
            eval_string(
                page,
                "() => { document.querySelector(\".swipe-btn[title='Retry']\").click(); return ''; }",
            )
            .await;
        }
        PendingStarter::Retrigger => {
            eval_string(
                page,
                "() => { document.querySelector('.log-entry.narration .retrigger-btn').click(); return ''; }",
            )
            .await;
        }
        PendingStarter::NewOptions => {
            eval_string(
                page,
                "() => { document.querySelector('.options-regen-form button').click(); return ''; }",
            )
            .await;
        }
    }
}

async fn action_status_text(page: &playwright_rs::Page) -> String {
    eval_string(
        page,
        "() => document.getElementById('action-status').textContent.trim()",
    )
    .await
}

async fn wait_for_action_status(page: &playwright_rs::Page, starter: PendingStarter, shows: &str) {
    let reached = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { action_status_text(page).await == shows },
    )
    .await;
    assert!(
        reached,
        "{starter:?}: #action-status must read {shows:?}, it reads {:?}",
        action_status_text(page).await
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.69
async fn check_a_pending_state_ends_when_the_screen_shows_the_answer_of_its_request(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.save_handle().set_generation_requests_accepted(true);
    wait_for_screen_current(&page).await;
    hold_screen_loads(&page).await;
    for starter in PENDING_STARTERS {
        eval_string(
            &page,
            "() => { window.__holdScreen = true; window.__screenHeld = []; return ''; }",
        )
        .await;
        start_pending_request(&page, starter).await;

        wait_for_action_status(&page, starter, "Thinking...").await;
        wait_for_held_screen_load(&page).await;
        assert_eq!(
            action_status_text(&page).await,
            "Thinking...",
            "{starter:?}: the screen has not shown the answer, so the action area keeps its own state"
        );

        release_screen_loads(&page).await;
        wait_for_action_status(&page, starter, "").await;
        wait_for_screen_current(&page).await;

        stub.status_handle().set(StubStatus::Idle);
        ask_once(&page).await;
    }
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.70
async fn check_a_pending_state_ends_at_once_when_the_engine_does_not_take_its_request(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.save_handle().set_story_writes_refused_as_busy(true);
    wait_for_screen_current(&page).await;
    for starter in STARTERS_BEFORE_THE_ANSWER {
        start_pending_request(&page, starter).await;

        wait_for_error_disclosure(
            &page,
            NOTICE,
            "the request that the engine does not take is reported in the notice",
        )
        .await;
        assert_eq!(
            action_status_text(&page).await,
            "",
            "{starter:?}: the engine did not take the request, so no pending state is left"
        );
    }
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.71
async fn check_a_409_on_the_options_regen_form_shows_exactly_one_refusal_report_in_the_notice(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.save_handle().set_story_writes_refused_as_busy(true);
    wait_for_screen_current(&page).await;
    eval_string(
        &page,
        r#"() => {
            window.__reportsWritten = 0;
            new MutationObserver((records) => {
                for (const record of records) {
                    for (const node of record.addedNodes) {
                        if (node.nodeType === 1 && node.matches('.error-disclosure')) {
                            window.__reportsWritten += 1;
                        }
                    }
                }
            }).observe(document.getElementById('notice'), { childList: true, subtree: true });
            return '';
        }"#,
    )
    .await;
    start_pending_request(&page, PendingStarter::NewOptions).await;
    wait_for_error_disclosure(
        &page,
        NOTICE,
        "the refused request is reported in the notice",
    )
    .await;
    wait_for_screen_current(&page).await;

    let (_, line, _) = read_error_disclosure(&page, NOTICE).await;
    assert!(
        line.contains("the Action was not sent"),
        "the server words the refusal, got {line:?}"
    );
    assert_eq!(
        eval_string(&page, "() => String(window.__reportsWritten)").await,
        "1",
        "one refusal is one report"
    );
    assert_eq!(
        eval_string(
            &page,
            "() => String(document.querySelectorAll('#options-dock [data-error-slot], #options-dock .error-disclosure').length)"
        )
        .await,
        "0",
        "the options dock holds no report"
    );
}

#[tokio::test]
async fn run_action_controls_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_a_check_request_disables_the_input_and_shows_send_as_busy_until_it_ends,
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
            check_a_load_during_a_check_request_renders_send_as_the_server_does,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_a_check_request_never_writes_disabled_on_send,
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
            check_a_swap_that_throws_leaves_the_command_form_dead_and_the_screen_loading,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_pending_state_ends_when_the_screen_shows_the_answer_of_its_request,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_a_pending_state_ends_at_once_when_the_engine_does_not_take_its_request,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_a_409_on_the_options_regen_form_shows_exactly_one_refusal_report_in_the_notice,
        )
        .await;
    runner.finish().await;
}
