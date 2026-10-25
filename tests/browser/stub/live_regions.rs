//! Stub-browser tests for the live regions of the Game tab: the ask loop, the loads it starts, the hold of an edit and the state a load keeps. Tagged against `docs/specs/browser_live_regions.md`.

// The timer of the page does not run in these checks. `chroniclerLive.stopTimer()`
// stops it and waits until no ask or load is in flight. `chroniclerLive.askOnce()`
// then runs one ask and returns when every load of that ask has settled, so each
// check decides when the page asks and what the stub answers.

use chronicler_engine::domain::model::state::generation_status::GenerationFailureKind;

use super::support::{read_banner, StubRunner};
use super::*;

/// A live region of the Game tab: the selector of its element, the path it
/// loads, and the kinds its `data-live` lists.
struct Region {
    selector: &'static str,
    path: &'static str,
    kinds: &'static [&'static str],
}

const REGIONS: [Region; 5] = [
    Region {
        selector: "#header",
        path: "/fragment/header",
        kinds: &["games", "settings", "llm"],
    },
    Region {
        selector: "#story-log",
        path: "/fragment/story-log",
        kinds: &["story", "busy", "worlds"],
    },
    Region {
        selector: ".visual-sidebar",
        path: "/fragment/visual-sidebar",
        kinds: &["story", "worlds"],
    },
    Region {
        selector: "#options-dock",
        path: "/fragment/options-dock",
        kinds: &["options", "busy"],
    },
    Region {
        selector: "#status-display",
        path: "/status/busy",
        kinds: &["busy"],
    },
];

const ALL_KINDS: [&str; 8] = [
    "story", "options", "busy", "games", "worlds", "settings", "presets", "llm",
];

const STORY_LOG: usize = 1;

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

async fn ask_once(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>("() => window.chroniclerLive.askOnce()", None)
        .await
        .expect("the page runs one ask");
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

/// Stops the timer, then asks once so that every region has loaded under the
/// numbers the stub holds now: the first ask loads each region once, because the
/// load that htmx runs at page load carries no record.
async fn start_current(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>("() => window.chroniclerLive.stopTimer()", None)
        .await
        .expect("the page stops its timer");
    ask_once(page).await;
    for region in &REGIONS {
        assert_eq!(
            live_state(page, region.selector).await,
            "current",
            "{} must be current after the first ask",
            region.selector
        );
    }
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
        let expected: Vec<usize> = REGIONS
            .iter()
            .map(|region| usize::from(region.kinds.contains(&kind)))
            .collect();
        assert_eq!(
            moved, expected,
            "loads of [header, story log, sidebar, dock, status] after `{kind}` moved"
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
    for region in &REGIONS {
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
    assert_eq!(loads_since(&changes, &before), vec![1; REGIONS.len()]);
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.4
async fn check_a_new_active_game_loads_every_region(page: playwright_rs::Page, stub: StubServer) {
    let changes = stub.changes_handle();
    start_current(&page).await;

    let before = region_loads(&changes);
    changes.set_game(Some(2));
    ask_once(&page).await;
    assert_eq!(loads_since(&changes, &before), vec![1; REGIONS.len()]);
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
    failure.set_polls_failing(true);
    ask_once(&page).await;
    let (visible, role, text) = read_banner(&page).await;
    assert!(visible, "a failed ask must show the banner");
    assert_eq!(role, "alert");
    assert!(
        text.contains("The engine is unreachable"),
        "the banner must say that the engine is unreachable, got {text:?}"
    );
    assert_eq!(loads_since(&changes, &before), vec![0; REGIONS.len()]);

    failure.set_polls_failing(false);
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
        vec![0, 0, 0, 1, 0],
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

// [docs/specs/browser_live_regions.md] SCENARIO: 44.13
async fn check_a_load_keeps_the_engaged_state_of_each_game_tab_region(
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
}

#[tokio::test]
async fn run_live_regions_checks() {
    let mut runner = StubRunner::launch().await;
    let outcome = StubActionOutcome::Pending;
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
        .run(
            outcome,
            check_a_load_keeps_the_engaged_state_of_each_game_tab_region,
        )
        .await;
    runner.finish().await;
}
