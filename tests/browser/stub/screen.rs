//! Stub-browser tests for the screen loop: the ask, the load of the visible tab, the tab change, the stamp and the failure slots. Tagged against `docs/specs/browser_screen.md`.

// The shell of the stub asks every 100 ms. A check that needs a quiet page pauses
// the timer with `stop_page_timer`, which also waits until no ask or load is in
// flight. `ask_once` then makes one ask, as the page does when it becomes visible,
// and returns when the page shows the screen of the stub.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;

use super::support::{
    ask_once_without_waiting, eval_string, hold_requests_ending_with, hold_screen_loads,
    open_section, read_banner, release_held_requests_and_wait_for_answers, release_screen_loads,
    start_current, wait_for_held_screen_load, StubRunner,
};
use super::*;
use crate::test_utils::screen_part_table::PARTS;

const GAME_TAB_PARTS: [&str; 8] = [
    "header",
    "failure-banner",
    "story-log",
    "visual-sidebar",
    "options-dock",
    "send-control",
    "autoplay-controls",
    "status-display",
];

fn part_loads(changes: &StubChangesHandle) -> Vec<usize> {
    PARTS.iter().map(|part| changes.loads(part.name)).collect()
}

fn loads_since(changes: &StubChangesHandle, before: &[usize]) -> Vec<usize> {
    part_loads(changes)
        .iter()
        .zip(before)
        .map(|(now, then)| now - then)
        .collect()
}

fn one_load_of_each_visible_part() -> Vec<usize> {
    PARTS
        .iter()
        .map(|part| usize::from(part.starts_visible()))
        .collect()
}

async fn part_is_empty(page: &playwright_rs::Page, selector: &str) -> bool {
    page.evaluate::<String, bool>(
        "(selector) => document.querySelector(selector).childNodes.length === 0",
        Some(&selector.to_string()),
    )
    .await
    .unwrap()
}

// [docs/specs/browser_screen.md] SCENARIO: 44.14
async fn check_at_page_start_the_screen_of_the_game_tab_loads_exactly_once(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    assert_eq!(
        changes.screen_requests(),
        vec![GAME_TAB_PARTS.map(str::to_string).to_vec()],
        "one GET /screen names the header, the failure banner and the parts of the Game tab"
    );
    for part in PARTS.iter().filter(|part| !part.starts_visible()) {
        assert!(
            part_is_empty(&page, part.selector).await,
            "{} holds no content while its tab is closed",
            part.selector
        );
    }
}

async fn wait_for_asks(changes: &StubChangesHandle, expected: usize) {
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(20),
            || async { changes.asks() >= expected }
        )
        .await,
        "the page must ask at once, without the timer: {} asks, wanted {expected}",
        changes.asks()
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.15
async fn check_a_request_asks_at_once_when_it_ends(page: playwright_rs::Page, stub: StubServer) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    stub.status_handle().set(StubStatus::Autoplay {
        done: 1,
        budget: 10,
        step: None,
    });
    ask_once(&page).await;

    let asks = changes.asks();
    let status_loads = changes.loads("status-display");
    page.evaluate::<(), ()>("() => document.getElementById('stop-btn').click()", None)
        .await
        .expect("the test presses Stop");
    wait_for_asks(&changes, asks + 1).await;
    wait_for_screen_current(&page).await;
    assert_eq!(
        changes.loads("status-display"),
        status_loads + 1,
        "Stop moved the revision, so the screen loads once and shows the status display"
    );

    stub.save_handle().set_delete_succeeds(true);
    super::support::accept_delete_confirmation(&page).await;
    let asks = changes.asks();
    let entries = page
        .query_selector_all("#story-log .log-entry")
        .await
        .unwrap()
        .len();
    page.locator("#story-log .log-entry:last-child .delete-btn")
        .await
        .click(None)
        .await
        .unwrap();
    wait_for_asks(&changes, asks + 1).await;
    wait_for_screen_current(&page).await;
    assert_eq!(
        page.query_selector_all("#story-log .log-entry")
            .await
            .unwrap()
            .len(),
        entries - 1,
        "the story log loads at once and no longer shows the deleted entry"
    );

    // An action that wrote nothing moves no revision: the page forgets its
    // stamp when the request ends, so the screen still loads.
    let loads = changes.loads("/screen");
    let asks = changes.asks();
    page.locator(".retrigger-btn")
        .await
        .click(None)
        .await
        .unwrap();
    wait_for_asks(&changes, asks + 1).await;
    wait_for_screen_current(&page).await;
    assert_eq!(
        changes.loads("/screen"),
        loads + 1,
        "the Retrigger wrote nothing, and its end loads the screen once"
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.1
async fn check_a_change_loads_the_visible_screen_with_one_request(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    for kind in ChangeKind::ALL {
        let before = part_loads(&changes);
        let requests = changes.loads("/screen");
        changes.bump(kind);
        ask_once(&page).await;
        assert_eq!(
            changes.loads("/screen"),
            requests + 1,
            "one GET /screen for the change of `{kind}`"
        );
        assert_eq!(
            loads_since(&changes, &before),
            one_load_of_each_visible_part(),
            "the header, the failure banner and the Game tab parts load once after `{kind}` moved, and no part of a closed tab loads"
        );
    }
}

// [docs/specs/browser_screen.md] SCENARIO: 44.2
async fn check_an_unchanged_answer_loads_nothing(page: playwright_rs::Page, stub: StubServer) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    let requests = changes.loads("/screen");
    let asks = changes.asks();
    ask_once(&page).await;
    assert!(changes.asks() > asks, "the page must ask");
    assert_eq!(changes.loads("/screen"), requests);
}

// [docs/specs/browser_screen.md] SCENARIO: 44.3
async fn check_a_new_server_id_loads_the_screen(page: playwright_rs::Page, stub: StubServer) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    let before = part_loads(&changes);
    changes.set_server("stub-server-2");
    ask_once(&page).await;
    assert_eq!(
        loads_since(&changes, &before),
        one_load_of_each_visible_part()
    );
    assert_eq!(
        eval_string(
            &page,
            "() => document.documentElement.dataset.revision.split(':')[0]"
        )
        .await,
        "stub-server-2",
        "the stamp of the page names the new server"
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.4
async fn check_a_new_active_game_loads_the_screen_and_names_its_game(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    assert_eq!(
        eval_string(&page, "() => document.documentElement.dataset.game").await,
        "1",
        "the screen names the active Game"
    );

    let before = part_loads(&changes);
    changes.set_game(Some(2));
    ask_once(&page).await;
    assert_eq!(
        loads_since(&changes, &before),
        one_load_of_each_visible_part()
    );
    assert_eq!(
        eval_string(&page, "() => document.documentElement.dataset.game").await,
        "2",
        "the screen names the new active Game"
    );
}

const BANNER_TEXT: &str =
    r#"() => document.getElementById('failure-banner-unreachable').textContent.trim()"#;

// [docs/specs/browser_screen.md] SCENARIO: 44.5
async fn check_a_network_error_shows_unreachable_and_any_answer_clears_it(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let lifecycle = stub.lifecycle_handle();
    start_current(&page).await;

    lifecycle.stop();
    wait_until_port_closed(lifecycle.addr()).await;
    ask_once_without_waiting(&page).await;
    let shown = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { read_banner(&page).await.0 },
    )
    .await;
    assert!(shown, "a network error must show the banner");
    let (_, role, text) = read_banner(&page).await;
    assert_eq!(role, "alert");
    assert!(
        text.contains("The engine is unreachable"),
        "the banner must say that the engine is unreachable, got {text:?}"
    );

    lifecycle.restart().await;
    ask_once(&page).await;
    let (visible, _, text) = read_banner(&page).await;
    assert!(!visible, "an answer must clear the banner, got {text:?}");
    assert_eq!(eval_string(&page, BANNER_TEXT).await, "");
}

const SCREEN_ERROR_TEXT: &str = r#"() => {
    const slot = document.getElementById('notice');
    return slot.hidden ? '' : slot.textContent.trim();
}"#;

// [docs/specs/browser_screen.md] SCENARIO: 44.6
async fn check_a_failed_ask_shows_in_the_screen_error_and_a_good_ask_clears_it(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let failure = stub.failure_handle();
    start_current(&page).await;

    failure.set_asks_and_loads_failing(true);
    ask_once_without_waiting(&page).await;
    let reported = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            eval_string(&page, SCREEN_ERROR_TEXT)
                .await
                .contains("GET /changes answered 500")
        },
    )
    .await;
    assert!(
        reported,
        "got {:?}",
        eval_string(&page, SCREEN_ERROR_TEXT).await
    );
    let (visible, _, text) = read_banner(&page).await;
    assert!(
        !visible,
        "the engine answered, so it is reachable, got {text:?}"
    );
    failure.set_asks_and_loads_failing(false);
    ask_once(&page).await;
    assert_eq!(
        eval_string(&page, SCREEN_ERROR_TEXT).await,
        "",
        "a good ask clears the slot"
    );
}

const SHORTEN_ASK_TIMEOUT: &str =
    "() => { document.documentElement.dataset.askTimeoutMs = '300'; }";

async fn wait_for_unreachable_banner(page: &playwright_rs::Page, why: &str) {
    let shown = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { read_banner(page).await.0 },
    )
    .await;
    assert!(shown, "{why}");
}

// [docs/specs/browser_screen.md] SCENARIO: 44.47
async fn check_an_ask_that_gets_no_answer_shows_unreachable_and_the_next_answer_clears_it(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let failure = stub.failure_handle();
    start_current(&page).await;
    page.evaluate::<(), ()>(SHORTEN_ASK_TIMEOUT, None)
        .await
        .unwrap();

    failure.set_asks_held(true);
    ask_once_without_waiting(&page).await;
    wait_for_unreachable_banner(&page, "an ask with no answer must raise the banner").await;
    let (_, role, text) = read_banner(&page).await;
    assert_eq!(role, "alert");
    assert!(
        text.contains("The engine is unreachable"),
        "the banner must say that the engine is unreachable, got {text:?}"
    );

    failure.set_asks_held(false);
    ask_once(&page).await;
    let (visible, _, text) = read_banner(&page).await;
    assert!(!visible, "an answer must clear the banner, got {text:?}");
}

// [docs/specs/browser_screen.md] SCENARIO: 44.48
async fn check_a_load_that_gets_no_answer_shows_unreachable_and_the_next_answer_loads_the_screen(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let failure = stub.failure_handle();
    start_current(&page).await;
    page.evaluate::<(), ()>(SHORTEN_ASK_TIMEOUT, None)
        .await
        .unwrap();

    failure.set_screen_loads_held(true);
    page.evaluate::<(), ()>("() => fetch('/stub/touch').then(() => {})", None)
        .await
        .unwrap();
    ask_once_without_waiting(&page).await;
    wait_for_unreachable_banner(&page, "a load with no answer must raise the banner").await;

    failure.set_screen_loads_held(false);
    ask_once(&page).await;
    let (visible, _, text) = read_banner(&page).await;
    assert!(!visible, "an answer must clear the banner, got {text:?}");
    wait_for_screen_current(&page).await;
}

const HIDE_PAGE: &str = r#"() => {
    Object.defineProperty(document, 'visibilityState', {
        configurable: true,
        get: () => 'hidden',
    });
}"#;

const SHOW_PAGE: &str = r#"() => {
    Object.defineProperty(document, 'visibilityState', {
        configurable: true,
        get: () => 'visible',
    });
    document.dispatchEvent(new Event('visibilitychange'));
}"#;

// [docs/specs/browser_screen.md] SCENARIO: 44.7
async fn check_a_hidden_page_does_not_ask(page: playwright_rs::Page, stub: StubServer) {
    let changes = stub.changes_handle();
    wait_for_screen_current(&page).await;

    page.evaluate::<(), ()>(HIDE_PAGE, None).await.unwrap();
    let asks = changes.asks();
    // A hidden page skips each tick of the timer, so five ticks of 100 ms pass with no ask.
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(changes.asks(), asks, "a hidden page must not ask");

    page.evaluate::<(), ()>(
        r#"() => {
            Object.defineProperty(document, 'visibilityState', {
                configurable: true,
                get: () => 'visible',
            });
        }"#,
        None,
    )
    .await
    .unwrap();
    let asked = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(20),
        || async { changes.asks() > asks },
    )
    .await;
    assert!(asked, "a visible page asks on its tick");
}

// [docs/specs/browser_screen.md] SCENARIO: 44.8
async fn check_a_page_that_becomes_visible_asks_at_once(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    page.evaluate::<(), ()>(HIDE_PAGE, None).await.unwrap();
    changes.bump(ChangeKind::OPTIONS);
    let asks = changes.asks();
    let before = part_loads(&changes);

    page.evaluate::<(), ()>(SHOW_PAGE, None).await.unwrap();
    wait_for_screen_current(&page).await;
    assert!(
        changes.asks() > asks,
        "the page must ask when it is visible"
    );
    assert_eq!(
        loads_since(&changes, &before),
        one_load_of_each_visible_part(),
        "the revision moved, so the visible screen loads"
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.16
async fn check_a_tab_change_empties_the_closing_tab_and_loads_the_opening_tab_once(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    assert_eq!(changes.loads("games"), 0);

    open_section(&page, "games", ".games-panel").await;
    assert_eq!(
        changes.loads("games"),
        1,
        "opening the tab loads its part exactly once"
    );
    assert_eq!(
        changes.screen_requests().last().cloned().unwrap(),
        ["header", "failure-banner", "games"]
            .map(str::to_string)
            .to_vec(),
        "the screen of the Games tab is the shell and the Games part"
    );
    for selector in [
        "#story-log",
        ".visual-sidebar",
        "#options-dock",
        "#status-display",
    ] {
        assert!(
            part_is_empty(&page, selector).await,
            "{selector} holds no content while the Game tab is closed"
        );
    }

    let story = changes.loads("story-log");
    changes.bump(ChangeKind::STORY);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("story-log"),
        story,
        "a change while the Game tab is closed loads no Game part"
    );

    open_section(&page, "game", "#story-log").await;
    assert_eq!(changes.loads("story-log"), story + 1);
    assert!(
        part_is_empty(&page, ".games-panel").await,
        "the Games part is empty again"
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.28
async fn check_a_tab_change_during_a_load_drops_the_old_answer(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    hold_screen_loads(&page).await;
    changes.bump(ChangeKind::STORY);
    ask_once_without_waiting(&page).await;
    wait_for_held_screen_load(&page).await;

    page.locator(r#"[data-tab="games"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    release_screen_loads(&page).await;
    wait_for_screen_current(&page).await;

    assert!(
        part_is_empty(&page, "#story-log").await,
        "the answer for the Game tab is dropped: its parts stay empty"
    );
    assert!(
        !part_is_empty(&page, ".games-panel").await,
        "the visible tab loads"
    );
    let requests = changes.screen_requests();
    assert_eq!(
        requests.last().cloned().unwrap(),
        ["header", "failure-banner", "games"]
            .map(str::to_string)
            .to_vec(),
        "the loop loads the visible tab after it drops the old answer"
    );
}

const OPEN_EDIT: &str = r#"() => {
    document.querySelector('.log-entry.narration .edit-btn').click();
    const textarea = document.getElementById('edit-textarea');
    textarea.value = 'typed into the edit';
    textarea.focus();
    return document.getElementById('story-log').hasAttribute('data-editing-entry')
        ? '' : 'the story log is not locked';
}"#;

// [docs/specs/browser_screen.md] SCENARIO: 44.29
async fn check_leaving_the_game_tab_discards_the_edit_and_its_lock(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    start_current(&page).await;
    assert_eq!(eval_string(&page, OPEN_EDIT).await, "");

    open_section(&page, "games", ".games-panel").await;
    open_section(&page, "game", "#story-log").await;

    assert_eq!(
        eval_string(
            &page,
            r#"() => {
                if (document.getElementById('edit-textarea')) return 'the edit is still open';
                if (document.getElementById('story-log').hasAttribute('data-editing-entry')) {
                    return 'the Edit buttons are still locked';
                }
                return '';
            }"#
        )
        .await,
        ""
    );
    assert_eq!(
        eval_string(&page, OPEN_EDIT).await,
        "",
        "a new edit opens after the tab came back"
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.36
async fn check_a_reload_restores_the_active_tab_and_the_settings_sub_tab(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    start_current(&page).await;
    open_section(&page, "settings", ".settings-panel").await;
    page.locator("#subtab-text-check")
        .await
        .click(None)
        .await
        .unwrap();

    page.reload(None).await.unwrap();
    wait_for_screen_current(&page).await;
    wait_until_visible(&page, "#subtab-text-check", Duration::from_secs(5)).await;
    let state = eval_string(
        &page,
        r#"() => {
            const tab = document.querySelector('.tab.active').dataset.tab;
            const subtab = document.getElementById('subtab-text-check').getAttribute('aria-selected');
            return tab + ' ' + subtab;
        }"#,
    )
    .await;
    assert_eq!(
        state, "settings true",
        "the reload restores the tab and the sub-tab"
    );

    open_section(&page, "game", "#story-log").await;
    open_section(&page, "settings", ".settings-panel").await;
    assert_eq!(
        eval_string(
            &page,
            "() => document.getElementById('subtab-text-check').getAttribute('aria-selected')"
        )
        .await,
        "true",
        "the part was empty while the tab was closed, and the sub-tab comes back"
    );
}

const SUBMIT_COMMAND: &str = "() => { document.getElementById('command-form').requestSubmit(); }";

async fn command_input_value(page: &playwright_rs::Page) -> String {
    page.evaluate::<(), String>("() => document.getElementById('command-input').value", None)
        .await
        .unwrap()
}

// [docs/specs/browser_screen.md] SCENARIO: 44.35
async fn check_the_action_input_clears_when_the_screen_shows_the_send_and_only_if_unchanged(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    start_current(&page).await;
    hold_screen_loads(&page).await;

    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(SUBMIT_COMMAND, None).await.unwrap();
    wait_for_held_screen_load(&page).await;
    assert_eq!(
        command_input_value(&page).await,
        "look around",
        "the engine took the send, and the screen does not show it yet: the input keeps its text"
    );
    fill_command_input(&page, "look around and wait").await;
    release_screen_loads(&page).await;
    wait_for_screen_current(&page).await;
    assert_eq!(
        command_input_value(&page).await,
        "look around and wait",
        "the player typed on, so the screen showing the send must not clear the new text"
    );

    stub.status_handle().set(StubStatus::Idle);
    hold_screen_loads(&page).await;
    fill_command_input(&page, "wait").await;
    page.evaluate::<(), ()>(SUBMIT_COMMAND, None).await.unwrap();
    wait_for_held_screen_load(&page).await;
    release_screen_loads(&page).await;
    wait_for_screen_current(&page).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async { command_input_value(&page).await.is_empty() }
        )
        .await,
        "an input that still holds the sent text clears once the screen shows the send"
    );
}

const TEST_CONNECTION: &str = "() => document.querySelector('#connection-openrouter-gpt-4o-mini button[hx-post$=\"/test\"]').click()";

async fn wait_for_screen_loads(changes: &StubChangesHandle, expected: usize) {
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(20),
            || async { changes.loads("/screen") >= expected }
        )
        .await,
        "the page must load the screen: {} loads, wanted {expected}",
        changes.loads("/screen")
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.39
async fn check_each_move_of_the_revision_loads_the_screen_once_while_a_request_runs(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "settings", ".settings-panel").await;
    hold_requests_ending_with(&page, "/test").await;
    page.evaluate::<(), ()>(TEST_CONNECTION, None)
        .await
        .unwrap();
    eval_string(
        &page,
        r#"() => {
            window.__testButton = document.querySelector('#connection-openrouter-gpt-4o-mini button[hx-post$="/test"]');
            return '';
        }"#,
    )
    .await;

    let loads = changes.loads("/screen");
    for moved in 1..=3 {
        changes.bump(ChangeKind::SETTINGS);
        ask_once_without_waiting(&page).await;
        wait_for_screen_loads(&changes, loads + moved).await;
    }
    assert_eq!(
        changes.loads("/screen"),
        loads + 3,
        "each move of the revision loads the screen once, though the Test request has not ended"
    );
    assert_eq!(
        eval_string(
            &page,
            r#"() => String(document.querySelector('#connection-openrouter-gpt-4o-mini button[hx-post$="/test"]') === window.__testButton)"#,
        )
        .await,
        "true",
        "the loads keep the Test control that has a request in flight"
    );

    release_held_requests_and_wait_for_answers(&page).await;
    wait_for_screen_current(&page).await;
    assert_eq!(
        changes.loads("/screen"),
        loads + 4,
        "the end of the request loads the screen once more, and the stamp of the page is the stamp of the engine"
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.40
async fn check_a_wait_for_a_stamp_of_another_server_ends_only_at_a_load_that_started_after_it(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    hold_screen_loads(&page).await;
    changes.bump(ChangeKind::STORY);
    ask_once_without_waiting(&page).await;
    wait_for_held_screen_load(&page).await;
    eval_string(
        &page,
        r#"() => {
            window.__waitState = 'waiting';
            afterScreenShows('stub-server-2:1').then(() => { window.__waitState = 'over'; });
            return '';
        }"#,
    )
    .await;

    release_screen_loads(&page).await;
    wait_for_screen_current(&page).await;
    assert_eq!(
        eval_string(&page, "() => window.__waitState").await,
        "waiting",
        "the load that started before the wait may still carry the old server, so it does not end the wait"
    );

    changes.set_server("stub-server-2");
    ask_once(&page).await;
    assert_eq!(
        eval_string(&page, "() => window.__waitState").await,
        "over",
        "a load that started after the wait began and shows another server ends the wait"
    );
}

async fn wait_for_screen_error(page: &playwright_rs::Page, naming: &str) {
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async { eval_string(page, SCREEN_ERROR_TEXT).await.contains(naming) }
        )
        .await,
        "#notice must name {naming}, got {:?}",
        eval_string(page, SCREEN_ERROR_TEXT).await
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.41
async fn check_an_answer_that_the_page_cannot_read_or_show_gives_the_screen_error(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    eval_string(
        &page,
        r#"() => {
            window.__fetchBeforeProbe = window.fetch;
            window.fetch = function (url, init) {
                if (String(url).startsWith('/changes')) {
                    return Promise.resolve(new Response('not json', { status: 200 }));
                }
                return window.__fetchBeforeProbe.apply(this, arguments);
            };
            return '';
        }"#,
    )
    .await;
    ask_once_without_waiting(&page).await;
    wait_for_screen_error(&page, "GET /changes").await;
    eval_string(
        &page,
        "() => { window.fetch = window.__fetchBeforeProbe; return ''; }",
    )
    .await;
    ask_once(&page).await;
    assert_eq!(
        eval_string(&page, SCREEN_ERROR_TEXT).await,
        "",
        "a good ask clears the slot"
    );

    eval_string(
        &page,
        r#"() => {
            window.__morphBeforeProbe = Idiomorph.morph;
            Idiomorph.morph = function () { throw new Error('the morph failed'); };
            return '';
        }"#,
    )
    .await;
    changes.bump(ChangeKind::STORY);
    ask_once_without_waiting(&page).await;
    wait_for_screen_error(&page, "GET /screen").await;
    eval_string(
        &page,
        "() => { Idiomorph.morph = window.__morphBeforeProbe; return ''; }",
    )
    .await;
    ask_once(&page).await;
    assert_eq!(
        eval_string(&page, SCREEN_ERROR_TEXT).await,
        "",
        "a good load clears the slot"
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.44
async fn check_a_screen_failure_that_lasts_keeps_its_open_details_and_its_node(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    changes.set_loads_failing(true);
    changes.bump(ChangeKind::STORY);
    ask_once_without_waiting(&page).await;
    wait_for_screen_error(&page, "GET /screen answered 500").await;
    page.locator("#notice .error-details-toggle")
        .await
        .click(None)
        .await
        .unwrap();
    eval_string(
        &page,
        r#"() => {
            document.querySelector('#notice .error-disclosure').dataset.shownBefore = 'yes';
            return '';
        }"#,
    )
    .await;

    for _ in 0..3 {
        let loads_before = changes.loads("/screen");
        ask_once_without_waiting(&page).await;
        assert!(
            wait_for_condition_async(
                Duration::from_secs(10),
                Duration::from_millis(25),
                || async {
                    changes.loads("/screen") > loads_before
                        && eval_string(
                            &page,
                            "() => document.documentElement.hasAttribute('data-screen-busy') ? 'busy' : ''",
                        )
                        .await
                        .is_empty()
                }
            )
            .await,
            "each ask must make a load that fails and ends"
        );
    }

    assert_eq!(
        eval_string(
            &page,
            r#"() => {
                const disclosure = document.querySelector('#notice .error-disclosure');
                const popover = document.querySelector('#notice .error-detail-popover');
                return (disclosure ? disclosure.dataset.shownBefore || 'replaced' : 'gone') +
                    '/' + (popover && !popover.hidden ? 'open' : 'closed');
            }"#
        )
        .await,
        "yes/open",
        "a failure that repeats must keep the open Details and the node that showed"
    );

    changes.set_loads_failing(false);
    ask_once(&page).await;
}

#[tokio::test]
async fn run_screen_checks() {
    let mut runner = StubRunner::launch().await;
    let outcome = StubActionOutcome::Pending;
    runner
        .run(
            outcome,
            check_at_page_start_the_screen_of_the_game_tab_loads_exactly_once,
        )
        .await;
    runner
        .run(outcome, check_a_request_asks_at_once_when_it_ends)
        .await;
    runner
        .run(
            outcome,
            check_a_change_loads_the_visible_screen_with_one_request,
        )
        .await;
    runner
        .run(outcome, check_an_unchanged_answer_loads_nothing)
        .await;
    runner
        .run(outcome, check_a_new_server_id_loads_the_screen)
        .await;
    runner
        .run(
            outcome,
            check_a_new_active_game_loads_the_screen_and_names_its_game,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_network_error_shows_unreachable_and_any_answer_clears_it,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_failed_ask_shows_in_the_screen_error_and_a_good_ask_clears_it,
        )
        .await;
    runner
        .run(
            outcome,
            check_an_ask_that_gets_no_answer_shows_unreachable_and_the_next_answer_clears_it,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_load_that_gets_no_answer_shows_unreachable_and_the_next_answer_loads_the_screen,
        )
        .await;
    runner.run(outcome, check_a_hidden_page_does_not_ask).await;
    runner
        .run(outcome, check_a_page_that_becomes_visible_asks_at_once)
        .await;
    runner
        .run(
            outcome,
            check_a_tab_change_empties_the_closing_tab_and_loads_the_opening_tab_once,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_tab_change_during_a_load_drops_the_old_answer,
        )
        .await;
    runner
        .run(
            outcome,
            check_leaving_the_game_tab_discards_the_edit_and_its_lock,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_reload_restores_the_active_tab_and_the_settings_sub_tab,
        )
        .await;
    runner
        .run(
            outcome,
            check_the_action_input_clears_when_the_screen_shows_the_send_and_only_if_unchanged,
        )
        .await;
    runner
        .run(
            outcome,
            check_each_move_of_the_revision_loads_the_screen_once_while_a_request_runs,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_wait_for_a_stamp_of_another_server_ends_only_at_a_load_that_started_after_it,
        )
        .await;
    runner
        .run(
            outcome,
            check_an_answer_that_the_page_cannot_read_or_show_gives_the_screen_error,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_screen_failure_that_lasts_keeps_its_open_details_and_its_node,
        )
        .await;
    runner.finish().await;
}
