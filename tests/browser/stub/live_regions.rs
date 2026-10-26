//! Stub-browser tests for the live regions: the ask loop, the loads it starts, the entry in edit that a load skips, and the state a load keeps. The dialog forms that survive a load are here too. Tagged against `docs/specs/browser_live_regions.md`.

// The timer of the page does not run in these checks. `chroniclerLive.stopTimer()`
// stops it and waits until no ask or load is in flight. `chroniclerLive.askOnce()`
// then runs one ask and returns when every load of that ask has settled, so each
// check decides when the page asks and what the stub answers.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;
use chronicler_engine::domain::model::state::generation_status::GenerationFailureKind;

use super::support::{ask_once, read_banner, stop_page_timer, StubRunner};
use super::*;
use crate::test_utils::live_region_table::REGIONS;

const STORY_LOG: usize = 1;
const GAMES_PANEL: usize = 5;
const LLM_PANEL: usize = 6;
const FAILURE_BANNER: usize = 7;
const SETTINGS_PANEL: usize = 8;

fn expected_loads(kind: ChangeKind) -> Vec<usize> {
    REGIONS
        .iter()
        .map(|region| usize::from(region.starts_visible() && region.kinds.contains(&kind)))
        .collect()
}

fn one_load_of_each_visible_region() -> Vec<usize> {
    REGIONS
        .iter()
        .map(|region| usize::from(region.starts_visible()))
        .collect()
}

fn region_loads(changes: &StubChangesHandle) -> Vec<usize> {
    REGIONS
        .iter()
        .map(|region| changes.loads(region.path))
        .collect()
}

fn loads_since(changes: &StubChangesHandle, before: &[usize]) -> Vec<usize> {
    region_loads(changes)
        .iter()
        .zip(before)
        .map(|(now, then)| now - then)
        .collect()
}

async fn eval_string(page: &playwright_rs::Page, script: &str) -> String {
    page.evaluate::<(), String>(script, None)
        .await
        .unwrap_or_else(|e| panic!("evaluate failed: {e}\n{script}"))
}

async fn settle(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>("() => window.chroniclerLive.settle()", None)
        .await
        .expect("the page settles");
}

async fn live_state(page: &playwright_rs::Page, selector: &str) -> String {
    page.evaluate::<String, String>(
        "(selector) => document.querySelector(selector).dataset.liveState || ''",
        Some(&selector.to_string()),
    )
    .await
    .unwrap()
}

/// The first ask of the page already loaded each region once, so the ask here
/// loads none: it only makes every region current under the numbers the stub holds.
async fn start_current(page: &playwright_rs::Page) {
    stop_page_timer(page).await;
    ask_once(page).await;
    for region in &REGIONS {
        let expected = if region.starts_visible() {
            "current"
        } else {
            "stale"
        };
        assert_eq!(
            live_state(page, region.selector).await,
            expected,
            "{} must be {expected} after the first ask",
            region.selector
        );
    }
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.14
async fn check_at_page_start_each_region_loads_exactly_once(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    // The runner opened the page and let its first ask settle. The timer has
    // not run since: it stops here, so a later ask cannot add a load.
    stop_page_timer(&page).await;
    for region in &REGIONS {
        let (loads, state) = if region.starts_visible() {
            (1, "current")
        } else {
            (0, "stale")
        };
        assert_eq!(
            changes.loads(region.path),
            loads,
            "{} must load {loads} times at page start",
            region.selector
        );
        assert_eq!(live_state(&page, region.selector).await, state);
    }
}

async fn wait_for_asks(changes: &StubChangesHandle, expected: usize) {
    assert!(
        wait_for_condition_async(
            std::time::Duration::from_secs(5),
            std::time::Duration::from_millis(20),
            || async { changes.asks() >= expected }
        )
        .await,
        "the page must ask at once, without the timer: {} asks, wanted {expected}",
        changes.asks()
    );
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.15
async fn check_an_action_asks_at_once_when_its_request_ends(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    let asks = changes.asks();
    let status_loads = changes.loads("/status/busy");
    page.evaluate::<(), ()>("() => document.getElementById('stop-btn').click()", None)
        .await
        .expect("the test presses Stop");
    wait_for_asks(&changes, asks + 1).await;
    settle(&page).await;
    assert_eq!(
        changes.loads("/status/busy"),
        status_loads + 1,
        "Stop moved the busy state, so the status display loads once"
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
    settle(&page).await;
    assert_eq!(
        page.query_selector_all("#story-log .log-entry")
            .await
            .unwrap()
            .len(),
        entries - 1,
        "the story log loads at once and no longer shows the deleted entry"
    );
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.1
async fn check_a_moved_number_loads_the_regions_that_list_its_kind(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    for kind in ChangeKind::ALL {
        let before = region_loads(&changes);
        changes.bump(kind);
        ask_once(&page).await;
        let moved = loads_since(&changes, &before);
        assert_eq!(
            moved,
            expected_loads(kind),
            "loads of [header, story log, sidebar, dock, status, games, llm, banner, settings, presets, worlds] after `{kind}` moved"
        );
    }
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.2
async fn check_an_unchanged_answer_loads_nothing(page: playwright_rs::Page, stub: StubServer) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    let before = region_loads(&changes);
    let asks = changes.asks();
    ask_once(&page).await;
    assert_eq!(changes.asks(), asks + 1, "the page must ask once");
    assert_eq!(loads_since(&changes, &before), vec![0; REGIONS.len()]);
    for region in REGIONS.iter().filter(|region| region.starts_visible()) {
        assert_eq!(live_state(&page, region.selector).await, "current");
    }
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.3
async fn check_a_new_server_id_loads_every_region(page: playwright_rs::Page, stub: StubServer) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    let before = region_loads(&changes);
    changes.set_server("stub-server-2");
    ask_once(&page).await;
    assert_eq!(
        loads_since(&changes, &before),
        one_load_of_each_visible_region()
    );
    assert_eq!(live_state(&page, ".games-panel").await, "stale");
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.4
async fn check_a_new_active_game_loads_every_region(page: playwright_rs::Page, stub: StubServer) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    let before = region_loads(&changes);
    changes.set_game(Some(2));
    ask_once(&page).await;
    assert_eq!(
        loads_since(&changes, &before),
        one_load_of_each_visible_region()
    );
    assert_eq!(live_state(&page, ".llm-messages-panel").await, "stale");
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.5
async fn check_a_failed_ask_shows_unreachable_and_a_good_ask_clears_it(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    let failure = stub.failure_handle();
    start_current(&page).await;

    let before = region_loads(&changes);
    failure.set_asks_and_loads_failing(true);
    ask_once(&page).await;
    let (visible, role, text) = read_banner(&page).await;
    assert!(visible, "a failed ask must show the banner");
    assert_eq!(role, "alert");
    assert!(
        text.contains("The engine is unreachable"),
        "the banner must say that the engine is unreachable, got {text:?}"
    );
    assert_eq!(loads_since(&changes, &before), vec![0; REGIONS.len()]);

    failure.set_asks_and_loads_failing(false);
    ask_once(&page).await;
    let (visible, _, text) = read_banner(&page).await;
    assert!(!visible, "a good ask must clear the banner, got {text:?}");
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.6
async fn check_a_failed_region_load_is_tried_again_on_the_next_ask(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    let before = changes.loads("/fragment/story-log");
    changes.set_loads_failing(true);
    stub.story_log_handle()
        .append_narration("A line that arrives late.");
    ask_once(&page).await;
    assert_eq!(changes.loads("/fragment/story-log"), before + 1);
    assert_eq!(live_state(&page, "#story-log").await, "due");
    let shown = page.locator("#story-log").await.inner_text().await.unwrap();
    assert!(
        !shown.contains("A line that arrives late."),
        "a failed load must leave the log as it was"
    );

    changes.set_loads_failing(false);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("/fragment/story-log"),
        before + 2,
        "the next ask must try the load again with no number moved"
    );
    assert_eq!(live_state(&page, "#story-log").await, "current");
    let shown = page.locator("#story-log").await.inner_text().await.unwrap();
    assert!(shown.contains("A line that arrives late."), "got {shown:?}");
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

// [docs/specs/browser_live_regions.md] SCENARIO: 44.7
async fn check_a_hidden_page_does_not_ask(page: playwright_rs::Page, stub: StubServer) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    page.evaluate::<(), ()>(HIDE_PAGE, None).await.unwrap();
    let asks = changes.asks();
    page.evaluate::<(), ()>("async () => { await window.chroniclerLive.tick(); }", None)
        .await
        .unwrap();
    settle(&page).await;
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
    page.evaluate::<(), ()>("async () => { await window.chroniclerLive.tick(); }", None)
        .await
        .unwrap();
    settle(&page).await;
    assert_eq!(changes.asks(), asks + 1, "a visible page asks on its tick");
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.8
async fn check_a_page_that_becomes_visible_asks_at_once(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    page.evaluate::<(), ()>(HIDE_PAGE, None).await.unwrap();
    changes.bump(ChangeKind::OPTIONS);
    let asks = changes.asks();
    let before = region_loads(&changes);

    page.evaluate::<(), ()>(SHOW_PAGE, None).await.unwrap();
    settle(&page).await;
    assert_eq!(
        changes.asks(),
        asks + 1,
        "the page must ask when it is visible"
    );
    assert_eq!(
        loads_since(&changes, &before),
        expected_loads(ChangeKind::OPTIONS),
        "only the dock lists `options`"
    );
}

const OPEN_EDIT: &str = r#"() => {
    document.querySelector('.log-entry.narration .edit-btn').click();
    const textarea = document.getElementById('edit-textarea');
    textarea.value = 'typed into the edit';
    textarea.focus();
    window.__editTextarea = textarea;
    return '';
}"#;

const EDIT_IS_INTACT: &str = r#"() => {
    const textarea = document.getElementById('edit-textarea');
    if (!textarea) return 'the textarea is gone';
    if (textarea !== window.__editTextarea) return 'the textarea is a new element';
    if (textarea.value !== 'typed into the edit') return 'the text changed: ' + textarea.value;
    if (document.activeElement !== textarea) return 'the textarea lost focus';
    return '';
}"#;

// [docs/specs/browser_live_regions.md] SCENARIO: 44.9
async fn check_a_load_skips_the_entry_in_edit_and_shows_the_rest_of_the_log(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    eval_string(&page, OPEN_EDIT).await;
    stub.story_log_handle()
        .append_narration("Narration that arrived during the edit.");
    let before = region_loads(&changes);
    for _ in 0..3 {
        changes.bump(ChangeKind::STORY);
        ask_once(&page).await;
    }
    assert_eq!(
        loads_since(&changes, &before)[STORY_LOG],
        3,
        "the region loads for each move of its kind, edit or no edit"
    );
    assert_eq!(live_state(&page, "#story-log").await, "current");
    assert_eq!(eval_string(&page, EDIT_IS_INTACT).await, "");
    assert!(
        eval_string(
            &page,
            "() => document.getElementById('story-log').innerText.includes('Narration that arrived during the edit.') ? '' : 'the new narration is missing'"
        )
        .await
        .is_empty(),
        "the rest of the log loads around the entry in edit"
    );

    page.evaluate::<(), ()>("() => cancelEdit()", None)
        .await
        .unwrap();
    settle(&page).await;
    assert_eq!(live_state(&page, "#story-log").await, "current");
    assert_eq!(
        eval_string(
            &page,
            "() => document.getElementById('edit-textarea') ? 'the textarea is still there' : ''"
        )
        .await,
        ""
    );
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.10
async fn check_a_load_in_flight_when_an_edit_began_skips_the_entry_in_edit(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    stub.story_log_handle()
        .append_narration("Narration in the answer that was in flight.");
    changes.set_story_log_delay_ms(400);
    changes.bump(ChangeKind::STORY);
    let before = changes.loads("/fragment/story-log");

    let script = format!(
        r#"async () => {{
            const asked = window.chroniclerLive.askOnce();
            const log = document.getElementById('story-log');
            const deadline = Date.now() + 3000;
            while (log.dataset.liveState !== 'loading') {{
                if (Date.now() > deadline) return 'the load never began';
                await new Promise((resolve) => setTimeout(resolve, 5));
            }}
            const problem = ({OPEN_EDIT})();
            await asked;
            return problem;
        }}"#
    );
    let outcome = eval_string(&page, &script).await;
    assert_eq!(
        outcome, "",
        "the edit must open while the load is in flight"
    );

    assert_eq!(changes.loads("/fragment/story-log"), before + 1);
    assert_eq!(live_state(&page, "#story-log").await, "current");
    assert_eq!(eval_string(&page, EDIT_IS_INTACT).await, "");
    assert!(
        eval_string(
            &page,
            "() => document.getElementById('story-log').innerText.includes('Narration in the answer that was in flight.') ? '' : 'the answer was not swapped in'"
        )
        .await
        .is_empty(),
        "the answer is swapped in around the entry in edit"
    );
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.11
async fn check_a_disclosure_that_lists_its_open_state_stays_open_through_a_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    stub.options_handle().set_failure(true);
    start_current(&page).await;

    eval_string(
        &page,
        r#"() => {
            const details = document.querySelector('#options-dock details');
            details.open = true;
            window.__keptDetails = details;
            return '';
        }"#,
    )
    .await;
    let before = changes.loads("/fragment/options-dock");
    changes.bump(ChangeKind::OPTIONS);
    ask_once(&page).await;
    assert_eq!(changes.loads("/fragment/options-dock"), before + 1);
    let state = eval_string(
        &page,
        r#"() => {
            const details = document.querySelector('#options-dock details');
            if (details !== window.__keptDetails) return 'a new element';
            return details.open ? '' : 'closed';
        }"#,
    )
    .await;
    assert_eq!(
        state, "",
        "the Details disclosure must be the same open element"
    );
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.12
async fn check_focus_moves_to_a_surviving_row_when_a_load_removes_the_focused_node(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    page.locator("#edit-entry-1").await.focus().await.unwrap();
    stub.story_log_handle().remove_newest();
    let before = changes.loads("/fragment/story-log");
    ask_once(&page).await;
    assert_eq!(changes.loads("/fragment/story-log"), before + 1);

    let landing = eval_string(
        &page,
        r#"() => {
            if (document.getElementById('entry-1')) return 'the newest entry is still there';
            const focused = document.activeElement;
            const entry = focused && focused.closest ? focused.closest('#story-log .log-entry') : null;
            return entry ? entry.id : 'focus is on ' + (focused ? focused.tagName : 'nothing');
        }"#,
    )
    .await;
    assert_eq!(
        landing, "entry-2",
        "focus must land in the entry that is now the newest"
    );
}

/// What a region keeps through a load, and how the check engages it. `engage`
/// puts the controls of the region in their engaged state and may remember
/// nodes in `window`. `check` answers "" when the engaged state is as it was.
/// `end` ends the edits the engaged state holds.
struct Engaged {
    region: usize,
    engage: &'static str,
    check: &'static str,
    end: Option<&'static str>,
}

async fn assert_engaged_state_survives(
    page: &playwright_rs::Page,
    changes: &StubChangesHandle,
    engaged: &Engaged,
) {
    let region = &REGIONS[engaged.region];
    let problem = eval_string(page, engaged.engage).await;
    assert_eq!(problem, "", "{}: engaging", region.selector);

    let before = changes.loads(region.path);
    for kind in region.kinds {
        changes.bump(*kind);
        ask_once(page).await;
        let problem = eval_string(page, engaged.check).await;
        assert_eq!(problem, "", "{} after `{kind}` moved", region.selector);
    }
    let loaded = changes.loads(region.path) - before;
    assert_eq!(
        loaded,
        region.kinds.len(),
        "{} must load once for each kind it lists",
        region.selector
    );
    if let Some(end) = engaged.end {
        eval_string(page, end).await;
        settle(page).await;
    }
    assert_eq!(live_state(page, region.selector).await, "current");
}

const HEADER_ENGAGED: Engaged = Engaged {
    region: 0,
    engage: "() => { window.__headerNode = document.querySelector('#header > *'); return ''; }",
    check: "() => document.querySelector('#header > *') === window.__headerNode ? '' : 'a new element'",
    end: None,
};

const STORY_LOG_ENGAGED: Engaged = Engaged {
    region: STORY_LOG,
    engage: OPEN_EDIT,
    check: EDIT_IS_INTACT,
    end: Some("() => { cancelEdit(); return ''; }"),
};

const SIDEBAR_ENGAGED: Engaged = Engaged {
    region: 2,
    engage: r#"() => {
        const label = document.querySelector('.visual-sidebar .image-label');
        const range = document.createRange();
        range.selectNodeContents(label);
        const selection = window.getSelection();
        selection.removeAllRanges();
        selection.addRange(range);
        window.__sidebarLabel = label;
        return selection.toString() === '' ? 'no selection' : '';
    }"#,
    check: r#"() => {
        const label = document.querySelector('.visual-sidebar .image-label');
        if (label !== window.__sidebarLabel) return 'a new element';
        return window.getSelection().toString() === label.textContent ? '' : 'the selection changed';
    }"#,
    end: None,
};

const OPTION_ENGAGED: Engaged = Engaged {
    region: 3,
    engage: r#"() => {
        const option = document.querySelector('#options-dock .option-btn');
        option.focus();
        window.__optionButton = option;
        return document.activeElement === option ? '' : 'no focus';
    }"#,
    check: r#"() => {
        const option = document.querySelector('#options-dock .option-btn');
        if (option !== window.__optionButton) return 'a new element';
        return document.activeElement === option ? '' : 'the focus moved';
    }"#,
    end: None,
};

const OPTIONS_FAILURE_ENGAGED: Engaged = Engaged {
    region: 3,
    engage: r#"() => {
        const details = document.querySelector('#options-dock details');
        details.open = true;
        window.__optionsDetails = details;
        return '';
    }"#,
    check: r#"() => {
        const details = document.querySelector('#options-dock details');
        if (details !== window.__optionsDetails) return 'a new element';
        return details.open ? '' : 'closed';
    }"#,
    end: None,
};

const STATUS_ENGAGED: Engaged = Engaged {
    region: 4,
    engage: r#"() => {
        document.querySelector('#status-display .error-details-toggle').click();
        const popover = document.querySelector('#status-display .error-detail-popover');
        window.__statusPopover = popover;
        return popover.hidden ? 'the popover did not open' : '';
    }"#,
    check: r#"() => {
        const popover = document.querySelector('#status-display .error-detail-popover');
        if (popover !== window.__statusPopover) return 'a new element';
        return popover.hidden ? 'closed' : '';
    }"#,
    end: None,
};

const LLM_ENGAGED: Engaged = Engaged {
    region: LLM_PANEL,
    engage: r#"() => {
        document.querySelector('.llm-message-header').click();
        const system = document.querySelector('.llm-message-prompt-details');
        system.open = true;
        window.__llmHeader = document.querySelector('.llm-message-header');
        window.__llmSystem = system;
        return window.__llmHeader.getAttribute('aria-expanded') === 'true' ? '' : 'the row did not expand';
    }"#,
    check: r#"() => {
        const header = document.querySelector('.llm-message-header');
        const system = document.querySelector('.llm-message-prompt-details');
        if (header !== window.__llmHeader) return 'the header is a new element';
        if (system !== window.__llmSystem) return 'the System disclosure is a new element';
        if (header.getAttribute('aria-expanded') !== 'true') return 'the row collapsed';
        if (document.querySelector('.llm-message-body').hidden) return 'the body is hidden';
        return system.open ? '' : 'the System disclosure closed';
    }"#,
    end: None,
};

/// Runs `action`, and returns once the htmx swap that it started has settled.
/// htmx settles a swap 20 ms after it lands, and the page moves focus into a new
/// editor then, so a check that engages the editor waits for that settle first.
/// A load of a region settles with the region as its element, and does not count.
macro_rules! after_settle {
    () => {
        r#"const afterSettle = (action) => new Promise((resolve) => {
        document.body.addEventListener('htmx:afterSettle', function once(evt) {
            const elt = evt.detail && evt.detail.elt;
            if (elt && elt.hasAttribute && elt.hasAttribute('data-live')) return;
            document.body.removeEventListener('htmx:afterSettle', once);
            resolve();
        });
        action();
    });"#
    };
}

const BANNER_ENGAGED: Engaged = Engaged {
    region: FAILURE_BANNER,
    engage: r#"() => {
        const toggle = document.querySelector('#failure-banner-degraded .error-details-toggle');
        toggle.click();
        toggle.focus();
        window.__bannerToggle = toggle;
        window.__bannerPopover = document.getElementById('failure-banner-popover');
        return window.__bannerPopover.hidden ? 'the popover did not open' : '';
    }"#,
    check: r#"() => {
        const toggle = document.querySelector('#failure-banner-degraded .error-details-toggle');
        const popover = document.getElementById('failure-banner-popover');
        if (toggle !== window.__bannerToggle) return 'the toggle is a new element';
        if (popover !== window.__bannerPopover) return 'the popover is a new element';
        if (popover.hidden) return 'the popover closed';
        if (toggle.getAttribute('aria-expanded') !== 'true') return 'aria-expanded changed';
        if (!popover.style.left) return 'the position of the popover was lost';
        return document.activeElement === toggle ? '' : 'the focus moved';
    }"#,
    end: None,
};

const SUBTAB_ENGAGED: Engaged = Engaged {
    region: SETTINGS_PANEL,
    engage: r#"() => {
        const tab = document.getElementById('subtab-text-check');
        tab.click();
        window.__subtab = tab;
        window.__textPanel = document.getElementById('settings-text-check');
        return tab.getAttribute('aria-selected') === 'true' ? '' : 'the sub-tab did not open';
    }"#,
    check: r#"() => {
        const tab = document.getElementById('subtab-text-check');
        const textPanel = document.getElementById('settings-text-check');
        if (tab !== window.__subtab) return 'the sub-tab is a new element';
        if (textPanel !== window.__textPanel) return 'the Text Check panel is a new element';
        if (tab.getAttribute('aria-selected') !== 'true') return 'the sub-tab lost its selection';
        if (document.getElementById('subtab-connections').getAttribute('aria-selected') !== 'false') {
            return 'the Connections sub-tab is selected again';
        }
        if (textPanel.hidden) return 'the Text Check panel is hidden';
        return document.getElementById('settings-connections').hidden ? '' : 'the Connections panel shows';
    }"#,
    end: None,
};

// [docs/specs/browser_live_regions.md] SCENARIO: 44.13
async fn check_a_load_keeps_the_engaged_state_of_each_region(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    assert_engaged_state_survives(&page, &changes, &HEADER_ENGAGED).await;
    assert_engaged_state_survives(&page, &changes, &STORY_LOG_ENGAGED).await;
    assert_engaged_state_survives(&page, &changes, &SIDEBAR_ENGAGED).await;
    assert_engaged_state_survives(&page, &changes, &OPTION_ENGAGED).await;

    stub.options_handle().set_failure(true);
    ask_once(&page).await;
    assert_engaged_state_survives(&page, &changes, &OPTIONS_FAILURE_ENGAGED).await;

    stub.status_handle().set(StubStatus::Error(
        GenerationFailureKind::Other,
        "narration failed".to_string(),
    ));
    ask_once(&page).await;
    assert_engaged_state_survives(&page, &changes, &STATUS_ENGAGED).await;

    open_section(&page, "llm-messages", ".llm-messages-panel").await;
    assert_engaged_state_survives(&page, &changes, &LLM_ENGAGED).await;

    stub.failure_handle().set_header_degraded(true);
    ask_once(&page).await;
    // The load settles a moment after its request ends, and the banner shows then.
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async { read_banner(&page).await.0 },
        )
        .await,
        "the degraded banner must show before its disclosure is engaged"
    );
    assert_engaged_state_survives(&page, &changes, &BANNER_ENGAGED).await;

    open_section(&page, "settings", ".settings-panel").await;
    assert_engaged_state_survives(&page, &changes, &SUBTAB_ENGAGED).await;
}

/// Opens a dashboard section the way the player does, and waits until every
/// load that the opening started has settled, and until htmx has processed the
/// content of the region of that section: htmx attaches the listeners of new
/// controls at the end of the settle task, so a click before it is lost.
async fn open_section(page: &playwright_rs::Page, tab: &str, region: &str) {
    page.locator(&format!(r#"[data-tab="{tab}"]"#))
        .await
        .click(None)
        .await
        .unwrap_or_else(|e| panic!("click tab '{tab}' failed: {e}"));
    settle(page).await;
    await_panel_ready(page, region).await.expect_settled(tab);
}

/// The dialog forms of the Games panel, opened the way the player does: a click
/// on the control, and a wait until htmx has settled the swap into the dialog body.
const OPEN_RENAME: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('#game-row-2 button[hx-get$="/rename"]').click());
    const input = document.querySelector('#edit-dialog input[name="display_name"]');
    if (!input) return 'the dialog did not open with the rename form';
    input.value = 'A typed name';
    input.focus();
    window.__renameForm = document.querySelector('#edit-dialog .game-rename-form');
    window.__renameInput = input;
    return document.activeElement === input ? '' : 'the name field has no focus';
}"#
);

const RENAME_IS_INTACT: &str = r#"() => {
    const dialog = document.getElementById('edit-dialog');
    const form = document.querySelector('#edit-dialog .game-rename-form');
    const input = form ? form.querySelector('input[name="display_name"]') : null;
    if (!dialog.open) return 'the dialog closed';
    if (form !== window.__renameForm) return 'the rename form is a new element';
    if (input !== window.__renameInput) return 'the name field is a new element';
    if (input.value !== 'A typed name') return 'the typed name changed: ' + input.value;
    if (document.activeElement !== input) return 'the name field lost focus';
    return '';
}"#;

/// Presses Save in the rename dialog, and returns once the request has ended and
/// every ask and load that followed has settled. The wait for a macrotask lets the
/// page's own `afterRequest` listener run first: it sits on the body, and this
/// listener sits on the form.
const SAVE_RENAME: &str = r#"async () => {
    const form = document.querySelector('#edit-dialog .game-rename-form form');
    const ended = new Promise((resolve) =>
        form.addEventListener('htmx:afterRequest', resolve, { once: true }));
    form.querySelector('button[type=submit]').click();
    await ended;
    await new Promise((resolve) => setTimeout(resolve, 0));
    await window.chroniclerLive.settle();
    return '';
}"#;

const OPEN_NEW_GAME: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('button[hx-get="/fragment/games/new"]').click());
    const world = document.getElementById('new-game-world');
    const persona = document.getElementById('new-game-persona');
    if (!world || !persona) return 'the dialog did not open with the New Game form';
    world.value = 'test';
    persona.value = 'test_player';
    window.__newGameForm = document.querySelector('#edit-dialog .new-game-form-dialog');
    window.__newGameWorld = world;
    window.__newGamePersona = persona;
    return world.value === 'test' && persona.value === 'test_player' ? '' : 'the choice did not take';
}"#
);

const NEW_GAME_IS_INTACT: &str = r#"() => {
    const dialog = document.getElementById('edit-dialog');
    const world = document.getElementById('new-game-world');
    const persona = document.getElementById('new-game-persona');
    if (!dialog.open) return 'the dialog closed';
    if (document.querySelector('#edit-dialog .new-game-form-dialog') !== window.__newGameForm) return 'the New Game form is a new element';
    if (world !== window.__newGameWorld) return 'the World select is a new element';
    if (persona !== window.__newGamePersona) return 'the Persona select is a new element';
    if (world.value !== 'test') return 'the World choice changed: ' + world.value;
    if (persona.value !== 'test_player') return 'the Persona choice changed: ' + persona.value;
    return '';
}"#;

const NEW_GAME_IS_RENDERED: &str = r#"() => {
    const world = document.getElementById('new-game-world');
    const persona = document.getElementById('new-game-persona');
    if (world.selectedIndex !== 0) return 'the World select keeps the choice: ' + world.value;
    if (persona.selectedIndex !== 0) return 'the Persona select keeps the choice: ' + persona.value;
    return '';
}"#;

/// Cancels the open dialog with a click, and returns once the dialog is closed
/// and its body is empty.
const CANCEL_DIALOG: &str = r#"async () => {
    const dialog = document.getElementById('edit-dialog');
    if (!dialog.open) return 'the dialog is not open';
    const closed = new Promise((resolve) =>
        dialog.addEventListener('close', resolve, { once: true }));
    dialog.querySelector('[data-close-dialog]').click();
    await closed;
    await window.chroniclerLive.settle();
    if (dialog.open) return 'the dialog is still open';
    return document.getElementById('edit-dialog-body').children.length === 0
        ? '' : 'the dialog body still holds the form';
}"#;

/// Bumps each kind of the Games panel, one at a time, and asks after each: the
/// panel loads, and the dialog above it keeps its form.
async fn assert_games_panel_loads_around_the_dialog(
    page: &playwright_rs::Page,
    changes: &StubChangesHandle,
    check: &str,
) {
    let before = changes.loads("/fragment/games");
    for kind in REGIONS[GAMES_PANEL].kinds {
        changes.bump(*kind);
        ask_once(page).await;
        assert_eq!(eval_string(page, check).await, "", "after `{kind}` moved");
    }
    assert_eq!(
        changes.loads("/fragment/games"),
        before + REGIONS[GAMES_PANEL].kinds.len(),
        "the panel loads for each kind that moved"
    );
    assert_eq!(live_state(page, ".games-panel").await, "current");
}

// [docs/specs/browser_games.md] SCENARIO: 27.4
async fn check_an_open_rename_dialog_keeps_its_text_and_focus_through_a_load_of_each_kind(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;
    assert_eq!(live_state(&page, ".games-panel").await, "current");

    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    assert_games_panel_loads_around_the_dialog(&page, &changes, RENAME_IS_INTACT).await;
}

// [docs/specs/browser_games.md] SCENARIO: 27.5
async fn check_saving_the_rename_dialog_closes_it_and_the_games_panel_shows_the_name(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    let before = changes.loads("/fragment/games");
    assert_eq!(eval_string(&page, SAVE_RENAME).await, "");
    assert_eq!(
        changes.loads("/fragment/games"),
        before + 1,
        "the saved rename must load the panel exactly once"
    );
    assert_eq!(live_state(&page, ".games-panel").await, "current");
    assert_eq!(
        eval_string(
            &page,
            "() => document.getElementById('edit-dialog').open ? 'the dialog is still open' : ''"
        )
        .await,
        "",
        "an accepted save closes the dialog"
    );
    let shown = page
        .locator("#game-row-2 .game-name")
        .await
        .inner_text()
        .await
        .unwrap();
    assert_eq!(shown, "A typed name", "the load shows the renamed Game");
}

// [docs/specs/browser_games.md] SCENARIO: 27.6
async fn check_the_new_game_selects_keep_the_players_choice_through_a_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(eval_string(&page, OPEN_NEW_GAME).await, "");
    assert_games_panel_loads_around_the_dialog(&page, &changes, NEW_GAME_IS_INTACT).await;
}

// [docs/specs/browser_games.md] SCENARIO: 27.7
async fn check_cancelling_the_new_game_dialog_discards_the_choice(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(eval_string(&page, OPEN_NEW_GAME).await, "");
    changes.bump(ChangeKind::WORLDS);
    ask_once(&page).await;
    assert_eq!(eval_string(&page, NEW_GAME_IS_INTACT).await, "");

    assert_eq!(eval_string(&page, CANCEL_DIALOG).await, "");
    assert_eq!(
        eval_string(&page, OPEN_NEW_GAME_WITHOUT_CHOOSING).await,
        "",
        "the New Game control opens a fresh form"
    );
    assert_eq!(
        eval_string(&page, NEW_GAME_IS_RENDERED).await,
        "",
        "the form that opens again shows the first options, not the discarded choice"
    );
}

const OPEN_NEW_GAME_WITHOUT_CHOOSING: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('button[hx-get="/fragment/games/new"]').click());
    return document.getElementById('new-game-world') ? '' : 'the dialog did not open with the New Game form';
}"#
);

// [docs/specs/browser_live_regions.md] SCENARIO: 44.16
async fn check_a_region_in_a_hidden_section_waits_and_the_section_loads_it_once(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    assert_eq!(changes.loads("/fragment/games"), 0);
    assert_eq!(changes.loads("/fragment/llm-messages"), 0);

    let header = changes.loads("/fragment/header");
    changes.bump(ChangeKind::GAMES);
    ask_once(&page).await;
    assert_eq!(changes.loads("/fragment/header"), header + 1);
    assert_eq!(changes.loads("/fragment/games"), 0, "a hidden region waits");
    assert_eq!(live_state(&page, ".games-panel").await, "stale");

    changes.bump(ChangeKind::LLM_MESSAGES);
    ask_once(&page).await;
    assert_eq!(changes.loads("/fragment/llm-messages"), 0);
    assert_eq!(live_state(&page, ".llm-messages-panel").await, "stale");

    open_section(&page, "games", ".games-panel").await;
    assert_eq!(
        changes.loads("/fragment/games"),
        1,
        "opening the section loads its region exactly once"
    );
    assert_eq!(live_state(&page, ".games-panel").await, "current");
    for region in REGIONS.iter().filter(|region| region.section == "game") {
        assert_eq!(live_state(&page, region.selector).await, "current");
    }

    // The Game tab is a section too: it is hidden now.
    let story = changes.loads("/fragment/story-log");
    changes.bump(ChangeKind::STORY);
    ask_once(&page).await;
    assert_eq!(changes.loads("/fragment/story-log"), story);
    assert_eq!(live_state(&page, "#story-log").await, "stale");
    open_section(&page, "game", "#story-log").await;
    assert_eq!(changes.loads("/fragment/story-log"), story + 1);
    assert_eq!(live_state(&page, "#story-log").await, "current");
}

const TYPE_TENSE: &str = r#"async () => {
    const select = document.querySelector('#game-posture-controls select[name="narrative_tense"]');
    const ended = new Promise((resolve) =>
        select.addEventListener('htmx:afterRequest', resolve, { once: true }));
    select.value = 'present';
    select.dispatchEvent(new Event('change', { bubbles: true }));
    await ended;
    await new Promise((resolve) => setTimeout(resolve, 0));
    return select.value;
}"#;

/// The Tense select sends its save again with the value it already shows.
const SAVE_TENSE_UNCHANGED: &str = r#"async () => {
    const select = document.querySelector('#game-posture-controls select[name="narrative_tense"]');
    const ended = new Promise((resolve) =>
        select.addEventListener('htmx:afterRequest', resolve, { once: true }));
    select.dispatchEvent(new Event('change', { bubbles: true }));
    await ended;
    await new Promise((resolve) => setTimeout(resolve, 0));
    return select.value;
}"#;

// [docs/specs/browser_live_regions.md] SCENARIO: 44.17
async fn check_a_save_that_fails_keeps_the_dialog_open_with_its_typed_values(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    stub.games_handle().set_rename_failing(true);
    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    assert_eq!(eval_string(&page, SAVE_RENAME).await, "");
    let (shown, line, raw) = read_error_disclosure(
        &page,
        "#edit-dialog .game-rename-form form [data-error-slot]",
    )
    .await;
    assert!(shown, "a failed save must report in the form's own slot");
    assert!(line.contains("action failed"), "got {line:?}");
    assert!(raw.contains("Stub rename failure"), "got {raw:?}");
    assert_eq!(
        eval_string(&page, RENAME_IS_INTACT).await,
        "",
        "the dialog stays open with the same form and the typed name"
    );
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.18
async fn check_a_control_that_saves_on_change_goes_back_to_its_rendered_value_when_the_save_fails(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(
        eval_string(&page, TYPE_TENSE).await,
        "past",
        "the Tense select must show the rendered value again"
    );
    let (shown, line, _) =
        read_error_disclosure(&page, "#game-posture-controls [data-error-slot]").await;
    assert!(shown && line.contains("action failed"), "got {line:?}");
    ask_once(&page).await;
    assert_eq!(live_state(&page, ".games-panel").await, "current");
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.19
async fn check_a_load_keeps_an_inline_error_that_is_showing(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    let slot = "#game-posture-controls [data-error-slot]";
    eval_string(&page, TYPE_TENSE).await;
    let (shown, _, raw) = read_error_disclosure(&page, slot).await;
    assert!(shown, "the failed save must report in the control's slot");

    let before = changes.loads("/fragment/games");
    changes.bump(ChangeKind::GAMES);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("/fragment/games"),
        before + 1,
        "the panel loads"
    );
    let (still_shown, _, still_raw) = read_error_disclosure(&page, slot).await;
    assert!(still_shown, "a load must keep the error that is showing");
    assert_eq!(still_raw, raw);

    stub.games_handle().set_posture_saves_succeed(true);
    eval_string(&page, SAVE_TENSE_UNCHANGED).await;
    let (shown, _, _) = read_error_disclosure(&page, slot).await;
    assert!(!shown, "a save that goes through clears the error");
}

const OPEN_PRESET_EDIT: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('button[hx-get="/fragment/prompt-presets/custom_ref/edit"]').click());
    const input = document.getElementById('edit-name-custom_ref');
    if (!input) return 'the dialog did not open with the edit form';
    input.value = 'A typed preset name';
    input.focus();
    window.__presetForm = document.querySelector('#edit-dialog .preset-form');
    window.__presetInput = input;
    return document.activeElement === input ? '' : 'the name field has no focus';
}"#
);

const PRESET_EDIT_IS_INTACT: &str = r#"() => {
    const form = document.querySelector('#edit-dialog .preset-form');
    const input = document.getElementById('edit-name-custom_ref');
    if (!document.getElementById('edit-dialog').open) return 'the dialog closed';
    if (form !== window.__presetForm) return 'the form is a new element';
    if (input !== window.__presetInput) return 'the name field is a new element';
    if (input.value !== 'A typed preset name') return 'the typed name changed: ' + input.value;
    return document.activeElement === input ? '' : 'the name field lost focus';
}"#;

// [docs/specs/browser_live_regions.md] SCENARIO: 44.20
async fn check_a_preset_form_in_the_dialog_survives_a_presets_load_and_cancel_sends_no_request(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "prompt-presets", ".prompt-presets-panel").await;

    assert_eq!(eval_string(&page, OPEN_PRESET_EDIT).await, "");
    let loads = changes.loads("/fragment/prompt-presets");
    changes.bump(ChangeKind::PROMPT_PRESETS);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("/fragment/prompt-presets"),
        loads + 1,
        "an open form holds no region, so the panel loads"
    );
    assert_eq!(live_state(&page, ".prompt-presets-panel").await, "current");
    assert_eq!(
        eval_string(&page, PRESET_EDIT_IS_INTACT).await,
        "",
        "the load of the panel leaves the dialog alone"
    );

    start_counting_requests(&page).await;
    assert_eq!(eval_string(&page, CANCEL_DIALOG).await, "");
    assert_eq!(requests_counted(&page).await, 0, "Cancel sends no request");
}

const OPEN_WORLD_EDITOR: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('#world-test button[hx-get$="/edit"]').click());
    const input = document.querySelector('#edit-dialog input[name="name"]');
    if (!input) return 'the dialog did not open with the World form';
    input.value = 'A typed world name';
    input.focus();
    window.__worldForm = document.getElementById('world-form-container');
    window.__worldInput = input;
    return document.activeElement === input ? '' : 'the name field has no focus';
}"#
);

const WORLD_EDITOR_IS_INTACT: &str = r#"() => {
    const form = document.getElementById('world-form-container');
    const input = document.querySelector('#edit-dialog input[name="name"]');
    if (!document.getElementById('edit-dialog').open) return 'the dialog closed';
    if (form !== window.__worldForm) return 'the form is a new element';
    if (input !== window.__worldInput) return 'the name field is a new element';
    if (input.value !== 'A typed world name') return 'the typed name changed: ' + input.value;
    return document.activeElement === input ? '' : 'the name field lost focus';
}"#;

// [docs/specs/browser_live_regions.md] SCENARIO: 44.21
async fn check_a_world_form_in_the_dialog_survives_a_game_rename_that_raises_games(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "worlds", ".worlds-panel").await;

    assert_eq!(eval_string(&page, OPEN_WORLD_EDITOR).await, "");
    let loads = changes.loads("/fragment/worlds");
    let renamed = eval_string(
        &page,
        r#"async () => {
            const response = await actionFetch('/games/2/rename', {
                method: 'POST',
                headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                body: 'display_name=Renamed+elsewhere',
            });
            await window.chroniclerLive.settle();
            return response.ok ? '' : 'the rename failed';
        }"#,
    )
    .await;
    assert_eq!(renamed, "");

    assert_eq!(
        eval_string(&page, WORLD_EDITOR_IS_INTACT).await,
        "",
        "a Game rename must not destroy the open World form"
    );
    assert_eq!(
        changes.loads("/fragment/worlds"),
        loads + 1,
        "the Worlds panel loads for the moved number"
    );
    assert_eq!(live_state(&page, ".worlds-panel").await, "current");
    assert_eq!(eval_string(&page, CANCEL_DIALOG).await, "");
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.22
async fn check_a_connection_form_in_the_dialog_survives_a_settings_load_and_keeps_the_sub_tab(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "settings", ".settings-panel").await;

    assert_eq!(eval_string(&page, SUBTAB_ENGAGED.engage).await, "");
    let opened = eval_string(
        &page,
        concat!(
            "async () => {",
            after_settle!(),
            r#"
            await afterSettle(() =>
                document.querySelector('#connection-openrouter-gpt-4o-mini button[hx-get$="/edit"]').click());
            const input = document.getElementById('conn_name');
            if (!input) return 'the form did not open';
            input.value = 'A typed connection';
            input.focus();
            window.__form = document.querySelector('#edit-dialog .connection-form');
            window.__formInput = input;
            return document.activeElement === input ? '' : 'the name field has no focus';
        }"#
        ),
    )
    .await;
    assert_eq!(opened, "");
    let loads = changes.loads("/fragment/settings");
    changes.bump(ChangeKind::SETTINGS);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("/fragment/settings"),
        loads + 1,
        "an open form holds no region, so the panel loads"
    );
    assert_eq!(live_state(&page, ".settings-panel").await, "current");
    let intact = eval_string(
        &page,
        r#"() => {
            const form = document.querySelector('#edit-dialog .connection-form');
            const input = document.getElementById('conn_name');
            if (form !== window.__form) return 'the form is a new element';
            if (input !== window.__formInput) return 'the name field is a new element';
            if (input.value !== 'A typed connection') return 'the typed name changed: ' + input.value;
            return document.activeElement === input ? '' : 'the name field lost focus';
        }"#,
    )
    .await;
    assert_eq!(intact, "", "the load of the panel leaves the dialog alone");

    start_counting_requests(&page).await;
    assert_eq!(eval_string(&page, CANCEL_DIALOG).await, "");
    assert_eq!(requests_counted(&page).await, 0, "Cancel sends no request");
    assert_eq!(
        eval_string(&page, SUBTAB_ENGAGED.check).await,
        "",
        "the sub-tab is still the one the player chose, after the dialog and the load"
    );
}

#[tokio::test]
async fn run_live_regions_checks() {
    let mut runner = StubRunner::launch().await;
    let outcome = StubActionOutcome::Pending;
    runner
        .run(outcome, check_at_page_start_each_region_loads_exactly_once)
        .await;
    runner
        .run(outcome, check_an_action_asks_at_once_when_its_request_ends)
        .await;
    runner
        .run(
            outcome,
            check_a_moved_number_loads_the_regions_that_list_its_kind,
        )
        .await;
    runner
        .run(outcome, check_an_unchanged_answer_loads_nothing)
        .await;
    runner
        .run(outcome, check_a_new_server_id_loads_every_region)
        .await;
    runner
        .run(outcome, check_a_new_active_game_loads_every_region)
        .await;
    runner
        .run(
            outcome,
            check_a_failed_ask_shows_unreachable_and_a_good_ask_clears_it,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_failed_region_load_is_tried_again_on_the_next_ask,
        )
        .await;
    runner.run(outcome, check_a_hidden_page_does_not_ask).await;
    runner
        .run(outcome, check_a_page_that_becomes_visible_asks_at_once)
        .await;
    runner
        .run(
            outcome,
            check_a_load_skips_the_entry_in_edit_and_shows_the_rest_of_the_log,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_load_in_flight_when_an_edit_began_skips_the_entry_in_edit,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_disclosure_that_lists_its_open_state_stays_open_through_a_load,
        )
        .await;
    runner
        .run(
            outcome,
            check_focus_moves_to_a_surviving_row_when_a_load_removes_the_focused_node,
        )
        .await;
    runner
        .run(outcome, check_a_load_keeps_the_engaged_state_of_each_region)
        .await;
    runner
        .run(
            outcome,
            check_a_region_in_a_hidden_section_waits_and_the_section_loads_it_once,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_save_that_fails_keeps_the_dialog_open_with_its_typed_values,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_control_that_saves_on_change_goes_back_to_its_rendered_value_when_the_save_fails,
        )
        .await;
    runner
        .run(outcome, check_a_load_keeps_an_inline_error_that_is_showing)
        .await;
    runner
        .run(
            outcome,
            check_an_open_rename_dialog_keeps_its_text_and_focus_through_a_load_of_each_kind,
        )
        .await;
    runner
        .run(
            outcome,
            check_saving_the_rename_dialog_closes_it_and_the_games_panel_shows_the_name,
        )
        .await;
    runner
        .run(
            outcome,
            check_the_new_game_selects_keep_the_players_choice_through_a_load,
        )
        .await;
    runner
        .run(
            outcome,
            check_cancelling_the_new_game_dialog_discards_the_choice,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_preset_form_in_the_dialog_survives_a_presets_load_and_cancel_sends_no_request,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_world_form_in_the_dialog_survives_a_game_rename_that_raises_games,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_connection_form_in_the_dialog_survives_a_settings_load_and_keeps_the_sub_tab,
        )
        .await;
    runner.finish().await;
}
