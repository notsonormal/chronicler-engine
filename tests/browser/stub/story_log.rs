//! Stub-browser tests for the story log's poll: the morph that keeps selection, focus, scroll position and the surviving entries. Tagged against `docs/specs/browser_story_log.md`.

use std::time::Duration;

use super::*;
use super::support::{StubRunner, install_story_poll_counter, poll_now, story_poll_count};

/// A state that survives one morph by chance must also survive the next, so
/// the preservation tests run two story-log poll cycles.
async fn poll_story_log_twice(page: &playwright_rs::Page) {
    poll_now(page, "#story-log").await;
    poll_now(page, "#story-log").await;
}

// An innerHTML swap would replace the node and lose the selection, so the poll
// must morph.
// [docs/specs/browser_story_log.md] SCENARIO: 30.11
async fn check_text_selection_survives_the_poll(page: playwright_rs::Page, _stub: StubServer) {
    install_story_poll_counter(&page).await;

    let selected = page
        .evaluate::<(), String>(
            r#"(() => {
                    const text = document.querySelector('.log-entry.narration .text');
                    const range = document.createRange();
                    range.selectNodeContents(text);
                    const selection = window.getSelection();
                    selection.removeAllRanges();
                    selection.addRange(range);
                    window.__selectedNode = text;
                    return selection.toString();
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(!selected.is_empty(), "the test must create a selection");

    poll_story_log_twice(&page).await;
    let polled = wait_for_condition_async(
        Duration::from_secs(8),
        Duration::from_millis(50),
        || async { story_poll_count(&page).await >= 2.0 },
    )
    .await;
    assert!(polled, "the story log must poll at least twice");

    let (still_selected, same_node) = page
        .evaluate::<(), (bool, bool)>(
            r#"(() => {
                    const selection = window.getSelection();
                    const text = document.querySelector('.log-entry.narration .text');
                    return [
                        !!selection && selection.toString().length > 0,
                        text === window.__selectedNode,
                    ];
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        still_selected,
        "a text selection in the log must survive the poll"
    );
    assert!(
        same_node,
        "an idle poll must not replace the entry the selection is in"
    );
}

// An innerHTML swap would drop focus to the body, so the poll must morph.
// [docs/specs/browser_story_log.md] SCENARIO: 30.12
async fn check_entry_focus_survives_the_poll(page: playwright_rs::Page, _stub: StubServer) {
    install_story_poll_counter(&page).await;

    page.locator(".log-entry.narration .retrigger-btn")
        .await
        .focus()
        .await
        .unwrap();
    page.evaluate::<(), ()>(
        "(() => { window.__focused = document.activeElement; })()",
        None,
    )
    .await
    .unwrap();

    poll_story_log_twice(&page).await;
    let polled = wait_for_condition_async(
        Duration::from_secs(8),
        Duration::from_millis(50),
        || async { story_poll_count(&page).await >= 2.0 },
    )
    .await;
    assert!(polled, "the story log must poll at least twice");

    let (focused, same_node) = page
        .evaluate::<(), (bool, bool)>(
            r#"(() => {
                    const btn = document.querySelector('.log-entry.narration .retrigger-btn');
                    return [document.activeElement === btn, btn === window.__focused];
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(focused, "focus inside a log entry must survive the poll");
    assert!(
        same_node,
        "an idle poll must not replace the focused control"
    );
}

// A scroll container needs focus before a keyboard user can arrow-scroll it.
// [docs/specs/browser_story_log.md] SCENARIO: 30.13
async fn check_story_log_is_keyboard_scrollable(page: playwright_rs::Page, _stub: StubServer) {
    let (focusable, label, overflows) = page
        .evaluate::<(), (bool, String, bool)>(
            r#"(() => {
                    const log = document.getElementById('story-log');
                    return [
                        log.getAttribute('tabindex') === '0',
                        log.getAttribute('aria-label') || '',
                        log.scrollHeight > log.clientHeight,
                    ];
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(focusable, "#story-log must be focusable");
    assert!(!label.is_empty(), "#story-log must have an accessible name");
    assert!(
        overflows,
        "the canned log must overflow its container for the scroll check"
    );

    page.locator("#story-log").await.focus().await.unwrap();
    let focused = page
        .evaluate::<(), bool>("(() => document.activeElement.id === 'story-log')()", None)
        .await
        .unwrap();
    assert!(focused, "focusing #story-log must put the keyboard on it");

    // The load swap leaves the log at its bottom (30.23), where ArrowDown
    // cannot scroll further, so the start position is explicit here.
    scroll_story_log_to_top(&page).await;
    let before = page
        .evaluate::<(), f64>(
            "(() => document.getElementById('story-log').scrollTop)()",
            None,
        )
        .await
        .unwrap();
    // A single synthetic key can be consumed while focus settles, so press
    // until the native scroll moves the log.
    let scrolled = wait_for_condition_async(
        Duration::from_secs(3),
        Duration::from_millis(50),
        || async {
            page.keyboard().press("ArrowDown", None).await.unwrap();
            page.evaluate::<(), f64>(
                "(() => document.getElementById('story-log').scrollTop)()",
                None,
            )
            .await
            .unwrap_or(0.0)
                > before
        },
    )
    .await;
    assert!(scrolled, "ArrowDown must scroll the focused log");
}

// The 50-entry cap drops the oldest entry, so the morph's removal matching must
// keep every surviving entry's nodes.
// [docs/specs/browser_story_log.md] SCENARIO: 30.14
async fn check_poll_removal_keeps_the_surviving_entries(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let story = stub.story_log_handle();
    story.set_extra_oldest(true);
    poll_now(&page, "#story-log").await;
    let grew = wait_for_condition_async(
        Duration::from_secs(8),
        Duration::from_millis(50),
        || async {
            page.evaluate::<(), f64>(
                "(() => document.querySelectorAll('#story-log .log-entry').length)()",
                None,
            )
            .await
            .unwrap_or(0.0)
                >= 3.0
        },
    )
    .await;
    assert!(grew, "the poll must render the extra oldest entry");

    page.evaluate::<(), ()>(
        r#"(() => {
                    window.__survivors = Array.from(
                        document.querySelectorAll('#story-log .log-entry'),
                    ).slice(1);
                })()"#,
        None,
    )
    .await
    .unwrap();

    story.set_extra_oldest(false);
    poll_now(&page, "#story-log").await;
    let shrank = wait_for_condition_async(
        Duration::from_secs(8),
        Duration::from_millis(50),
        || async {
            page.evaluate::<(), f64>(
                "(() => document.querySelectorAll('#story-log .log-entry').length)()",
                None,
            )
            .await
            .unwrap_or(0.0)
                <= 2.0
        },
    )
    .await;
    assert!(shrank, "the poll must drop the extra oldest entry");

    let kept = page
        .evaluate::<(), bool>(
            r#"(() => {
                        const now = Array.from(
                            document.querySelectorAll('#story-log .log-entry'),
                        );
                        const before = window.__survivors || [];
                        return now.length === before.length
                            && now.every((el, i) => el === before[i]);
                    })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        kept,
        "a morph must keep the surviving entries' nodes when one is dropped"
    );
}

const STORY_LOG_SCROLL_PROBE: &str = r#"(() => {
    const log = document.getElementById('story-log');
    const entries = log.querySelectorAll('.log-entry');
    const newest = entries[entries.length - 1];
    const view = log.getBoundingClientRect();
    const entry = newest ? newest.getBoundingClientRect() : null;
    return {
        overflows: log.scrollHeight > log.clientHeight,
        bottomGap: log.scrollHeight - log.scrollTop - log.clientHeight,
        scrollTop: log.scrollTop,
        entries: entries.length,
        newestInView: !!entry && entry.top < view.bottom && entry.bottom > view.top,
        newestTailBelowView: !!entry && entry.bottom > view.bottom,
    };
})()"#;

#[derive(serde::Deserialize)]
struct StoryLogScroll {
    overflows: bool,
    #[serde(rename = "bottomGap")]
    bottom_gap: f64,
    #[serde(rename = "scrollTop")]
    scroll_top: f64,
    entries: f64,
    #[serde(rename = "newestInView")]
    newest_in_view: bool,
    #[serde(rename = "newestTailBelowView")]
    newest_tail_below_view: bool,
}

async fn read_story_log_scroll(page: &playwright_rs::Page) -> StoryLogScroll {
    page.evaluate::<(), StoryLogScroll>(STORY_LOG_SCROLL_PROBE, None)
        .await
        .unwrap()
}

/// The follow rule picks up new entries only while the log sits at its bottom.
async fn scroll_story_log_to_bottom(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        "(() => { const log = document.getElementById('story-log'); log.scrollTop = log.scrollHeight; })()",
        None,
    )
    .await
    .unwrap();
}

/// The follow rule leaves a log the player scrolled away from its bottom alone.
async fn scroll_story_log_to_top(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        "(() => { const log = document.getElementById('story-log'); log.scrollTop = 0; })()",
        None,
    )
    .await
    .unwrap();
}

async fn wait_for_story_log_entries(page: &playwright_rs::Page, expected: f64) {
    let rendered = wait_for_condition_async(
        Duration::from_secs(8),
        Duration::from_millis(50),
        || async {
            page.evaluate::<(), f64>(
                "(() => document.querySelectorAll('#story-log .log-entry').length)()",
                None,
            )
            .await
            .unwrap_or(0.0)
                >= expected
        },
    )
    .await;
    assert!(rendered, "the poll must render {expected} entries");
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.20
async fn check_poll_that_appends_an_entry_follows_the_bottom(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let story = stub.story_log_handle();
    scroll_story_log_to_bottom(&page).await;
    let before = read_story_log_scroll(&page).await;
    assert!(
        before.overflows,
        "the canned log must overflow its container for the follow check"
    );
    assert!(
        before.bottom_gap <= 8.0,
        "the test must start with the log at its bottom, gap {} px",
        before.bottom_gap
    );

    story.append_narration("A new dawn breaks over the courtyard.");
    poll_now(&page, "#story-log").await;
    wait_for_story_log_entries(&page, before.entries + 1.0).await;

    let after = read_story_log_scroll(&page).await;
    assert!(
        after.bottom_gap <= 8.0,
        "a poll that appends an entry must scroll the log to its bottom, gap {} px",
        after.bottom_gap
    );
    assert!(
        after.newest_in_view,
        "the appended entry must be in view after the poll"
    );
}

// [docs/specs/browser_story_log.md] SCENARIO: 30.21
async fn check_poll_that_appends_an_entry_leaves_a_scrolled_up_log_alone(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let story = stub.story_log_handle();
    scroll_story_log_to_top(&page).await;
    let before = read_story_log_scroll(&page).await;
    assert!(
        before.overflows,
        "the canned log must overflow its container for the scroll check"
    );
    assert!(
        before.bottom_gap > 8.0,
        "the test must start with the log scrolled away from its bottom"
    );

    story.append_narration("A new dawn breaks over the courtyard.");
    poll_now(&page, "#story-log").await;
    wait_for_story_log_entries(&page, before.entries + 1.0).await;

    let after = read_story_log_scroll(&page).await;
    assert_eq!(
        after.scroll_top, before.scroll_top,
        "a poll must not move a log the player scrolled away from its bottom"
    );
    assert!(
        after.newest_tail_below_view,
        "the newest entry must stay below the fold for a scrolled-up log \
                 (scrollTop {} bottomGap {})",
        after.scroll_top, after.bottom_gap
    );
}

// The load swap is the one the player sees before any poll runs, so its
// follow-the-bottom must land on the first render too.
// [docs/specs/browser_story_log.md] SCENARIO: 30.23
async fn check_first_load_swap_lands_at_the_logs_bottom(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    let loaded = read_story_log_scroll(&page).await;
    assert!(
        loaded.overflows,
        "the canned log must overflow its container for the follow check"
    );

    let followed = wait_for_condition_async(
        Duration::from_secs(2),
        Duration::from_millis(25),
        || async { read_story_log_scroll(&page).await.bottom_gap <= 8.0 },
    )
    .await;
    let after = read_story_log_scroll(&page).await;
    assert!(
        followed,
        "the first load swap must leave the log at its bottom, gap {} px",
        after.bottom_gap
    );
    assert!(
        after.newest_in_view,
        "the newest entry must be in view after the first load swap"
    );
}

#[tokio::test]
async fn run_story_log_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_text_selection_survives_the_poll,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_entry_focus_survives_the_poll,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_story_log_is_keyboard_scrollable,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_poll_removal_keeps_the_surviving_entries,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_poll_that_appends_an_entry_follows_the_bottom,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_poll_that_appends_an_entry_leaves_a_scrolled_up_log_alone,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_first_load_swap_lands_at_the_logs_bottom,
        )
        .await;
    runner.finish().await;
}
