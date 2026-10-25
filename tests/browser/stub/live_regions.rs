//! Stub-browser tests for the live regions of the Game tab: the ask loop, the loads it starts, the hold of an edit and the state a load keeps. Tagged against `docs/specs/browser_live_regions.md`.

// The timer of the page does not run in these checks. `chroniclerLive.stopTimer()`
// stops it and waits until no ask or load is in flight. `chroniclerLive.askOnce()`
// then runs one ask and returns when every load of that ask has settled, so each
// check decides when the page asks and what the stub answers.

use std::time::Duration;

use chronicler_engine::domain::model::state::generation_status::GenerationFailureKind;

use super::support::{ask_once, read_banner, stop_page_timer, StubRunner};
use super::*;

/// A live region: the selector of its element, the path it loads, the kinds its
/// `data-live` lists, and the `data-tab` of its section (empty for the header,
/// which is outside every section).
struct Region {
    selector: &'static str,
    path: &'static str,
    kinds: &'static [&'static str],
    section: &'static str,
}

impl Region {
    /// The checks start on the Game tab: the header and the Game tab regions are
    /// visible, and the regions of the other sections are hidden.
    fn starts_visible(&self) -> bool {
        self.section.is_empty() || self.section == "game"
    }
}

const REGIONS: [Region; 11] = [
    Region {
        selector: "#header",
        path: "/fragment/header",
        kinds: &["games"],
        section: "",
    },
    Region {
        selector: "#story-log",
        path: "/fragment/story-log",
        kinds: &["story", "busy", "worlds"],
        section: "game",
    },
    Region {
        selector: ".visual-sidebar",
        path: "/fragment/visual-sidebar",
        kinds: &["story", "worlds"],
        section: "game",
    },
    Region {
        selector: "#options-dock",
        path: "/fragment/options-dock",
        kinds: &["options", "busy"],
        section: "game",
    },
    Region {
        selector: "#status-display",
        path: "/status/busy",
        kinds: &["busy"],
        section: "game",
    },
    Region {
        selector: ".games-panel",
        path: "/fragment/games",
        kinds: &["games", "worlds", "presets"],
        section: "games",
    },
    Region {
        selector: ".llm-messages-panel",
        path: "/fragment/llm-messages",
        kinds: &["llm"],
        section: "llm-messages",
    },
    Region {
        selector: "#failure-banner-degraded",
        path: "/fragment/failure-banner",
        kinds: &["settings", "llm"],
        section: "",
    },
    Region {
        selector: ".settings-panel",
        path: "/fragment/settings",
        kinds: &["settings", "llm"],
        section: "settings",
    },
    Region {
        selector: ".prompt-presets-panel",
        path: "/fragment/prompt-presets",
        kinds: &["presets", "settings", "games"],
        section: "prompt-presets",
    },
    Region {
        selector: ".worlds-panel",
        path: "/fragment/worlds",
        kinds: &["worlds", "games"],
        section: "worlds",
    },
];

const ALL_KINDS: [&str; 8] = [
    "story", "options", "busy", "games", "worlds", "settings", "presets", "llm",
];

const STORY_LOG: usize = 1;
const GAMES_PANEL: usize = 5;
const LLM_PANEL: usize = 6;
const FAILURE_BANNER: usize = 7;
const SETTINGS_PANEL: usize = 8;
const PRESETS_PANEL: usize = 9;
const WORLDS_PANEL: usize = 10;

/// The loads a moved `kind` starts on the Game tab: each visible region that lists it.
fn expected_loads(kind: &str) -> Vec<usize> {
    REGIONS
        .iter()
        .map(|region| usize::from(region.starts_visible() && region.kinds.contains(&kind)))
        .collect()
}

/// One load of each region that is visible on the Game tab.
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

/// Stops the timer, then asks once so that every region is current under the
/// numbers the stub holds now. The first ask of the page already loaded each
/// region once, so this ask loads none.
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

    for kind in ALL_KINDS {
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
    changes.bump("options");
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
        expected_loads("options"),
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
async fn check_an_edit_holds_its_region_which_loads_once_when_the_edit_ends(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    eval_string(&page, OPEN_EDIT).await;
    let before = region_loads(&changes);
    for _ in 0..3 {
        changes.bump("story");
        ask_once(&page).await;
    }
    assert_eq!(
        loads_since(&changes, &before)[STORY_LOG],
        0,
        "a held region must not load"
    );
    assert_eq!(live_state(&page, "#story-log").await, "held");
    assert_eq!(eval_string(&page, EDIT_IS_INTACT).await, "");

    page.evaluate::<(), ()>("() => cancelEdit()", None)
        .await
        .unwrap();
    settle(&page).await;
    assert_eq!(
        loads_since(&changes, &before)[STORY_LOG],
        1,
        "the region must load once when the edit ends"
    );
    assert_eq!(live_state(&page, "#story-log").await, "current");
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.10
async fn check_a_load_in_flight_when_an_edit_began_does_not_replace_the_edit(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    changes.set_story_log_delay_ms(400);
    changes.bump("story");
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
    assert_eq!(live_state(&page, "#story-log").await, "held");
    assert_eq!(eval_string(&page, EDIT_IS_INTACT).await, "");
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
    changes.bump("options");
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
    let held = engaged.end.is_some();
    let problem = eval_string(page, engaged.engage).await;
    assert_eq!(problem, "", "{}: engaging", region.selector);

    let before = changes.loads(region.path);
    for kind in region.kinds {
        changes.bump(kind);
        ask_once(page).await;
        let problem = eval_string(page, engaged.check).await;
        assert_eq!(problem, "", "{} after `{kind}` moved", region.selector);
    }
    let loaded = changes.loads(region.path) - before;
    if let Some(end) = engaged.end {
        assert_eq!(
            loaded, 0,
            "{} is held, so it must not load",
            region.selector
        );
        assert_eq!(live_state(page, region.selector).await, "held");
        eval_string(page, end).await;
        settle(page).await;
        assert_eq!(
            changes.loads(region.path) - before,
            1,
            "{} must load once when its edits end",
            region.selector
        );
    } else {
        assert_eq!(
            loaded,
            region.kinds.len(),
            "{} must load once for each kind it lists",
            region.selector
        );
    }
    assert!(!held || live_state(page, region.selector).await == "current");
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

const CONNECTION_EDITOR_ENGAGED: Engaged = Engaged {
    region: SETTINGS_PANEL,
    engage: concat!(
        "async () => {",
        after_settle!(),
        r#"
        await afterSettle(() =>
            document.querySelector('#connection-openrouter-gpt-4o-mini button[hx-get$="/edit"]').click());
        const input = document.getElementById('conn_name');
        if (!input) return 'the editor did not open';
        input.value = 'A typed connection';
        input.focus();
        window.__editor = document.getElementById('connection-form-page');
        window.__editorInput = input;
        return document.activeElement === input ? '' : 'the name field has no focus';
    }"#
    ),
    check: r#"() => {
        const editor = document.getElementById('connection-form-page');
        const input = document.getElementById('conn_name');
        if (editor !== window.__editor) return 'the editor is a new element';
        if (!editor.hasAttribute('data-work')) return 'the editor lost its hold';
        if (input !== window.__editorInput) return 'the name field is a new element';
        if (input.value !== 'A typed connection') return 'the typed name changed: ' + input.value;
        return document.activeElement === input ? '' : 'the name field lost focus';
    }"#,
    end: Some(
        r#"async () => {
        document.querySelector('#connection-editor .back-link').click();
        await window.chroniclerLive.settle();
        return '';
    }"#,
    ),
};

const PRESET_EDIT_ENGAGED: Engaged = Engaged {
    region: PRESETS_PANEL,
    engage: OPEN_PRESET_EDIT,
    check: PRESET_EDIT_IS_INTACT,
    end: Some(END_PRESET_EDIT),
};

const WORLD_EDITOR_ENGAGED: Engaged = Engaged {
    region: WORLDS_PANEL,
    engage: OPEN_WORLD_EDITOR,
    check: WORLD_EDITOR_IS_INTACT,
    end: Some(END_WORLD_EDITOR),
};

const OPEN_PRESET_EDIT: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('button[hx-get="/fragment/prompt-presets/custom_ref/edit"]').click());
    const input = document.getElementById('edit-name-custom_ref');
    if (!input) return 'the editor did not open';
    input.value = 'A typed preset name';
    input.focus();
    window.__presetEditor = document.getElementById('preset-card-custom_ref');
    window.__presetInput = input;
    return document.activeElement === input ? '' : 'the name field has no focus';
}"#
);

const PRESET_EDIT_IS_INTACT: &str = r#"() => {
    const card = document.getElementById('preset-card-custom_ref');
    const input = document.getElementById('edit-name-custom_ref');
    if (card !== window.__presetEditor) return 'the card in edit is a new element';
    if (!card.classList.contains('edit-form')) return 'the card left its edit';
    if (input !== window.__presetInput) return 'the name field is a new element';
    if (input.value !== 'A typed preset name') return 'the typed name changed: ' + input.value;
    return document.activeElement === input ? '' : 'the name field lost focus';
}"#;

/// The edit after a click on another card: the click moved focus to that card's
/// control, and the edit keeps everything else.
const PRESET_EDIT_IS_INTACT_AFTER_A_CLICK_ELSEWHERE: &str = r#"() => {
    const card = document.getElementById('preset-card-custom_ref');
    const input = document.getElementById('edit-name-custom_ref');
    if (card !== window.__presetEditor) return 'the card in edit is a new element';
    if (!card.classList.contains('edit-form')) return 'the card left its edit';
    if (input !== window.__presetInput) return 'the name field is a new element';
    return input.value === 'A typed preset name' ? '' : 'the typed name changed: ' + input.value;
}"#;

/// Cancel replaces the card with its list card. The edit ends when htmx has
/// settled that replacement, and the page asks when that request ends.
const END_PRESET_EDIT: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('#preset-card-custom_ref .form-actions button[type=button]').click());
    await window.chroniclerLive.settle();
    return '';
}"#
);

const OPEN_WORLD_EDITOR: &str = concat!(
    "async () => {",
    after_settle!(),
    r#"
    await afterSettle(() =>
        document.querySelector('#world-test button[hx-get$="/edit"]').click());
    const input = document.querySelector('#world-editor input[name="name"]');
    if (!input) return 'the editor did not open';
    input.value = 'A typed world name';
    input.focus();
    window.__worldEditor = document.getElementById('world-form-container');
    window.__worldInput = input;
    return document.activeElement === input ? '' : 'the name field has no focus';
}"#
);

const WORLD_EDITOR_IS_INTACT: &str = r#"() => {
    const editor = document.getElementById('world-form-container');
    const input = document.querySelector('#world-editor input[name="name"]');
    if (editor !== window.__worldEditor) return 'the editor is a new element';
    if (!editor.hasAttribute('data-work')) return 'the editor lost its hold';
    if (input !== window.__worldInput) return 'the name field is a new element';
    if (input.value !== 'A typed world name') return 'the typed name changed: ' + input.value;
    return document.activeElement === input ? '' : 'the name field lost focus';
}"#;

const END_WORLD_EDITOR: &str = r#"async () => {
    document.querySelector('#world-editor button[data-close-editor].btn-cyan').click();
    await window.chroniclerLive.settle();
    return '';
}"#;

/// The scripts of one engaged state are built from the scripts of the single
/// scenarios, so each state is defined once.
fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

fn games_engaged() -> Engaged {
    Engaged {
        region: GAMES_PANEL,
        engage: leak(format!(
            "() => {{ const problem = ({OPEN_RENAME})(); return problem || ({CHOOSE_NEW_GAME})(); }}"
        )),
        check: leak(format!(
            "() => ({RENAME_IS_INTACT})() || ({NEW_GAME_IS_INTACT})()"
        )),
        end: Some(leak(format!(
            "async () => {{ await ({REVERT_NEW_GAME})(); return await ({SAVE_RENAME})(); }}"
        ))),
    }
}

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

    open_section(&page, "games", ".games-panel").await;
    assert_engaged_state_survives(&page, &changes, &games_engaged()).await;

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
    assert_engaged_state_survives(&page, &changes, &CONNECTION_EDITOR_ENGAGED).await;

    open_section(&page, "prompt-presets", ".prompt-presets-panel").await;
    assert_engaged_state_survives(&page, &changes, &PRESET_EDIT_ENGAGED).await;

    open_section(&page, "worlds", ".worlds-panel").await;
    assert_engaged_state_survives(&page, &changes, &WORLD_EDITOR_ENGAGED).await;
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

const OPEN_RENAME: &str = r#"() => {
    const details = document.querySelector('#game-row-2 .game-rename');
    details.open = true;
    const input = details.querySelector('input[name="display_name"]');
    input.value = 'A typed name';
    input.focus();
    window.__renameDetails = details;
    window.__renameInput = input;
    return document.activeElement === input ? '' : 'the name field has no focus';
}"#;

const RENAME_IS_INTACT: &str = r#"() => {
    const details = document.querySelector('#game-row-2 .game-rename');
    const input = details ? details.querySelector('input[name="display_name"]') : null;
    if (details !== window.__renameDetails) return 'the rename form is a new element';
    if (!details.open) return 'the rename form closed';
    if (input !== window.__renameInput) return 'the name field is a new element';
    if (input.value !== 'A typed name') return 'the typed name changed: ' + input.value;
    if (document.activeElement !== input) return 'the name field lost focus';
    return '';
}"#;

/// Presses Save in the rename form of the saved Game, and returns once the
/// request has ended and every ask and load that followed has settled. The wait
/// for a macrotask lets the page's own `afterRequest` listener run first: it sits
/// on the body, and this listener sits on the form.
const SAVE_RENAME: &str = r#"async () => {
    const form = document.querySelector('#game-row-2 .game-rename form');
    const ended = new Promise((resolve) =>
        form.addEventListener('htmx:afterRequest', resolve, { once: true }));
    form.querySelector('button[type=submit]').click();
    await ended;
    await new Promise((resolve) => setTimeout(resolve, 0));
    await window.chroniclerLive.settle();
    return '';
}"#;

const CHOOSE_NEW_GAME: &str = r#"() => {
    const world = document.getElementById('new-game-world');
    const persona = document.getElementById('new-game-persona');
    world.value = 'test';
    persona.value = 'test_player';
    window.__newGameWorld = world;
    window.__newGamePersona = persona;
    return world.value === 'test' && persona.value === 'test_player' ? '' : 'the choice did not take';
}"#;

const NEW_GAME_IS_INTACT: &str = r#"() => {
    const world = document.getElementById('new-game-world');
    const persona = document.getElementById('new-game-persona');
    if (world !== window.__newGameWorld) return 'the World select is a new element';
    if (persona !== window.__newGamePersona) return 'the Persona select is a new element';
    if (world.value !== 'test') return 'the World choice changed: ' + world.value;
    if (persona.value !== 'test_player') return 'the Persona choice changed: ' + persona.value;
    return '';
}"#;

const REVERT_NEW_GAME: &str = r#"async () => {
    document.getElementById('new-game-world').selectedIndex = 0;
    document.getElementById('new-game-persona').selectedIndex = 0;
    await window.chroniclerLive.askOnce();
    return '';
}"#;

/// Bumps each kind of the Games panel, one at a time, and asks after each: the
/// panel waits, because the player works in it.
async fn assert_games_panel_holds_through_each_kind(
    page: &playwright_rs::Page,
    changes: &StubChangesHandle,
    check: &str,
) {
    let before = changes.loads("/fragment/games");
    for kind in REGIONS[GAMES_PANEL].kinds {
        changes.bump(kind);
        ask_once(page).await;
        assert_eq!(eval_string(page, check).await, "", "after `{kind}` moved");
    }
    assert_eq!(
        changes.loads("/fragment/games"),
        before,
        "a held panel must not load"
    );
    assert_eq!(live_state(page, ".games-panel").await, "held");
}

// [docs/specs/browser_games.md] SCENARIO: 27.4
async fn check_an_open_rename_form_keeps_its_text_and_focus_through_a_load_of_each_kind(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;
    assert_eq!(live_state(&page, ".games-panel").await, "current");

    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    assert_games_panel_holds_through_each_kind(&page, &changes, RENAME_IS_INTACT).await;
}

// [docs/specs/browser_games.md] SCENARIO: 27.5
async fn check_ending_the_rename_edit_loads_the_games_panel_once(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    assert_games_panel_holds_through_each_kind(&page, &changes, RENAME_IS_INTACT).await;

    let before = changes.loads("/fragment/games");
    assert_eq!(eval_string(&page, SAVE_RENAME).await, "");
    assert_eq!(
        changes.loads("/fragment/games"),
        before + 1,
        "ending the edit must load the panel exactly once"
    );
    assert_eq!(live_state(&page, ".games-panel").await, "current");
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

    assert_eq!(eval_string(&page, CHOOSE_NEW_GAME).await, "");
    let before = changes.loads("/fragment/games");
    for kind in ["games", "worlds"] {
        changes.bump(kind);
        ask_once(&page).await;
        assert_eq!(
            eval_string(&page, NEW_GAME_IS_INTACT).await,
            "",
            "after `{kind}` moved"
        );
    }
    assert_eq!(
        changes.loads("/fragment/games"),
        before,
        "a held panel must not load"
    );
    assert_eq!(live_state(&page, ".games-panel").await, "held");
}

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
    changes.bump("games");
    ask_once(&page).await;
    assert_eq!(changes.loads("/fragment/header"), header + 1);
    assert_eq!(changes.loads("/fragment/games"), 0, "a hidden region waits");
    assert_eq!(live_state(&page, ".games-panel").await, "stale");

    changes.bump("llm");
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
    changes.bump("story");
    ask_once(&page).await;
    assert_eq!(changes.loads("/fragment/story-log"), story);
    assert_eq!(live_state(&page, "#story-log").await, "stale");
    open_section(&page, "game", "#story-log").await;
    assert_eq!(changes.loads("/fragment/story-log"), story + 1);
    assert_eq!(live_state(&page, "#story-log").await, "current");
}

const WATCH_RENAME_SAVE: &str = r#"() => {
    const details = document.querySelector('#game-row-2 .game-rename');
    const input = details.querySelector('input[name="display_name"]');
    window.__afterSave = null;
    // Registered after the page's own listener on the body, so it reads what
    // the end-of-edit rule left, before any load of the panel arrives.
    document.body.addEventListener('htmx:afterRequest', (evt) => {
        if (!evt.detail.elt || evt.detail.elt.tagName !== 'FORM') return;
        window.__afterSave = {
            changed: input.value !== input.defaultValue,
            open: details.open,
        };
    });
    return '';
}"#;

// [docs/specs/browser_live_regions.md] SCENARIO: 44.17
async fn check_a_successful_save_ends_the_edit_and_a_marked_disclosure_closes(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    eval_string(&page, WATCH_RENAME_SAVE).await;
    changes.bump("worlds");
    ask_once(&page).await;
    assert_eq!(live_state(&page, ".games-panel").await, "held");

    let before = changes.loads("/fragment/games");
    assert_eq!(eval_string(&page, SAVE_RENAME).await, "");
    let after_save = eval_string(
        &page,
        "() => { const s = window.__afterSave; return s ? JSON.stringify(s) : 'the save never ended'; }",
    )
    .await;
    assert_eq!(
        after_save, r#"{"changed":false,"open":false}"#,
        "when the save ended, the name was no longer a changed field and the disclosure was closed"
    );
    assert_eq!(changes.loads("/fragment/games"), before + 1);
    assert_eq!(live_state(&page, ".games-panel").await, "current");
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

// [docs/specs/browser_live_regions.md] SCENARIO: 44.18
async fn check_a_failed_save_keeps_the_edit_and_a_failed_auto_save_goes_back(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    stub.games_handle().set_rename_failing(true);
    assert_eq!(eval_string(&page, OPEN_RENAME).await, "");
    changes.bump("worlds");
    ask_once(&page).await;
    assert_eq!(eval_string(&page, SAVE_RENAME).await, "");
    let (shown, line, raw) =
        read_error_disclosure(&page, "#game-row-2 .game-rename form [data-error-slot]").await;
    assert!(shown, "a failed save must report in the form's own slot");
    assert!(line.contains("action failed"), "got {line:?}");
    assert!(raw.contains("Stub rename failure"), "got {raw:?}");
    assert_eq!(
        eval_string(&page, RENAME_IS_INTACT).await,
        "",
        "the form stays open and keeps the typed name"
    );
    ask_once(&page).await;
    assert_eq!(live_state(&page, ".games-panel").await, "held");

    // The save of a select that fails goes back to the rendered value.
    stub.games_handle().set_rename_failing(false);
    eval_string(
        &page,
        "() => { const input = window.__renameInput; input.value = input.defaultValue; return ''; }",
    )
    .await;
    assert_eq!(
        eval_string(&page, TYPE_TENSE).await,
        "past",
        "the Tense select must show the rendered value again"
    );
    let (shown, line, _) =
        read_error_disclosure(&page, "#game-posture-controls [data-error-slot]").await;
    assert!(shown && line.contains("action failed"), "got {line:?}");
    ask_once(&page).await;
    assert_eq!(
        live_state(&page, ".games-panel").await,
        "current",
        "a select that went back is no changed field, so it holds nothing"
    );
}

const PRESS_SAVE_UNCHANGED: &str = r#"async () => {
    const details = document.querySelector('#game-row-2 .game-rename');
    details.open = true;
    const form = details.querySelector('form');
    const ended = new Promise((resolve) =>
        form.addEventListener('htmx:afterRequest', resolve, { once: true }));
    form.querySelector('button[type=submit]').click();
    await ended;
    await new Promise((resolve) => setTimeout(resolve, 0));
    await window.chroniclerLive.settle();
    return '';
}"#;

// [docs/specs/browser_live_regions.md] SCENARIO: 44.19
async fn check_a_load_keeps_an_inline_error_that_is_showing(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "games", ".games-panel").await;

    let slot = "#game-row-2 .game-rename form [data-error-slot]";
    stub.games_handle().set_rename_failing(true);
    assert_eq!(eval_string(&page, PRESS_SAVE_UNCHANGED).await, "");
    let (shown, _, raw) = read_error_disclosure(&page, slot).await;
    assert!(shown, "the failed save must report in the form's slot");

    let before = changes.loads("/fragment/games");
    changes.bump("games");
    ask_once(&page).await;
    assert_eq!(
        changes.loads("/fragment/games"),
        before + 1,
        "the unchanged form holds nothing, so the panel loads"
    );
    let (still_shown, _, still_raw) = read_error_disclosure(&page, slot).await;
    assert!(still_shown, "a load must keep the error that is showing");
    assert_eq!(still_raw, raw);

    stub.games_handle().set_rename_failing(false);
    assert_eq!(eval_string(&page, PRESS_SAVE_UNCHANGED).await, "");
    let (shown, _, _) = read_error_disclosure(&page, slot).await;
    assert!(!shown, "a save that goes through clears the error");
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.20
async fn check_a_card_in_edit_survives_the_activate_of_another_card(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "prompt-presets", ".prompt-presets-panel").await;

    assert_eq!(eval_string(&page, OPEN_PRESET_EDIT).await, "");
    let loads = changes.loads("/fragment/prompt-presets");
    let asks = changes.asks();
    page.locator(r#"button[hx-post="/prompt-presets/custom_two/activate?mode=novel"]"#)
        .await
        .click(None)
        .await
        .expect("press Set Active on the other card");
    wait_for_asks(&changes, asks + 1).await;
    settle(&page).await;

    assert_eq!(
        eval_string(&page, PRESET_EDIT_IS_INTACT_AFTER_A_CLICK_ELSEWHERE).await,
        "",
        "the other card's Activate must not destroy the card in edit"
    );
    assert_eq!(live_state(&page, ".prompt-presets-panel").await, "held");
    assert_eq!(
        changes.loads("/fragment/prompt-presets"),
        loads,
        "a held panel must not load"
    );

    assert_eq!(eval_string(&page, END_PRESET_EDIT).await, "");
    assert_eq!(
        changes.loads("/fragment/prompt-presets"),
        loads + 1,
        "ending the edit must load the panel once"
    );
    assert_eq!(live_state(&page, ".prompt-presets-panel").await, "current");
    let badge = page
        .locator("#preset-card-custom_two .badge.primary")
        .await
        .inner_text()
        .await
        .unwrap();
    assert!(
        badge.contains("Active"),
        "the load shows the Activate that waited: {badge:?}"
    );
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.21
async fn check_a_world_editor_survives_a_game_rename_that_raises_games(
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
        "a Game rename must not destroy the open World editor"
    );
    assert_eq!(live_state(&page, ".worlds-panel").await, "held");
    assert_eq!(changes.loads("/fragment/worlds"), loads);

    assert_eq!(eval_string(&page, END_WORLD_EDITOR).await, "");
    assert_eq!(
        changes.loads("/fragment/worlds"),
        loads + 1,
        "closing the editor must load the panel once"
    );
    assert_eq!(live_state(&page, ".worlds-panel").await, "current");
    assert_eq!(
        page.query_selector_all("#world-editor > *")
            .await
            .unwrap()
            .len(),
        0,
        "the load renders the editor slot empty"
    );
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.22
async fn check_a_connection_editor_keeps_the_sub_tab_and_cancel_loads_the_panel_once(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    open_section(&page, "settings", ".settings-panel").await;

    assert_eq!(eval_string(&page, SUBTAB_ENGAGED.engage).await, "");
    assert_eq!(
        eval_string(&page, CONNECTION_EDITOR_ENGAGED.engage).await,
        ""
    );
    let loads = changes.loads("/fragment/settings");
    let requests = changes.loads("/fragment/connections/openrouter-gpt-4o-mini/edit");
    changes.bump("settings");
    ask_once(&page).await;
    assert_eq!(live_state(&page, ".settings-panel").await, "held");
    assert_eq!(changes.loads("/fragment/settings"), loads);

    assert_eq!(
        eval_string(&page, CONNECTION_EDITOR_ENGAGED.end.unwrap()).await,
        ""
    );
    assert_eq!(
        changes.loads("/fragment/connections/openrouter-gpt-4o-mini/edit"),
        requests,
        "Cancel sends no request"
    );
    assert_eq!(
        changes.loads("/fragment/settings"),
        loads + 1,
        "Cancel ends the edit, and the panel loads once"
    );
    assert_eq!(
        eval_string(&page, SUBTAB_ENGAGED.check).await,
        "",
        "the sub-tab is still the one the player chose, after the editor and the load"
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
            check_an_edit_holds_its_region_which_loads_once_when_the_edit_ends,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_load_in_flight_when_an_edit_began_does_not_replace_the_edit,
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
            check_a_successful_save_ends_the_edit_and_a_marked_disclosure_closes,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_failed_save_keeps_the_edit_and_a_failed_auto_save_goes_back,
        )
        .await;
    runner
        .run(outcome, check_a_load_keeps_an_inline_error_that_is_showing)
        .await;
    runner
        .run(
            outcome,
            check_an_open_rename_form_keeps_its_text_and_focus_through_a_load_of_each_kind,
        )
        .await;
    runner
        .run(
            outcome,
            check_ending_the_rename_edit_loads_the_games_panel_once,
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
            check_a_card_in_edit_survives_the_activate_of_another_card,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_world_editor_survives_a_game_rename_that_raises_games,
        )
        .await;
    runner
        .run(
            outcome,
            check_a_connection_editor_keeps_the_sub_tab_and_cancel_loads_the_panel_once,
        )
        .await;
    runner.finish().await;
}
