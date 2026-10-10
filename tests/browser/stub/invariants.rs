//! Rendering invariants (declared exemption in the spec-coverage validator): no spec link, test code is the definition. CSS computed styles, layout measurements, text-wrap behavior — only a real browser can observe these. Twelve checks share one browser through the shared runner; each runs on a fresh page against its own stub server, with panic isolation and a per-check timing summary.

use std::time::Duration;

use playwright_rs::{EmulateMediaOptions, ForcedColors, Page, Viewport};

use super::support::StubRunner;
use super::*;

#[tokio::test]
async fn run_invariants() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(StubActionOutcome::Pending, check_story_log_scrollable)
        .await;
    runner
        .run(StubActionOutcome::Pending, check_no_horizontal_overflow)
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_log_entry_text_wraps_within_bubble,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_command_input_width_is_stable,
        )
        .await;
    runner
        .run(StubActionOutcome::Pending, check_element_positioning)
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_npc_portraits_horizontal_layout,
        )
        .await;
    runner
        .run(StubActionOutcome::Pending, check_npc_portraits_fixed_width)
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_edit_textarea_matches_original_height,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_responsive_layout_under_768px,
        )
        .await;
    runner
        .run(StubActionOutcome::Pending, check_root_design_tokens)
        .await;
    runner
        .run(StubActionOutcome::Pending, check_forced_colors_focus_ring)
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_action_area_makes_room_for_the_inline_error,
        )
        .await;
    runner.finish().await;
}

async fn check_story_log_scrollable(page: Page, _stub: StubServer) {
    let overflow_y: String = page
        .evaluate::<(), String>(
            "(() => {
                    const el = document.querySelector('#story-log');
                    return window.getComputedStyle(el).overflowY;
                })()",
            None,
        )
        .await
        .unwrap();

    assert!(
        overflow_y == "auto" || overflow_y == "scroll",
        "Story log should be scrollable"
    );
}

async fn check_no_horizontal_overflow(page: Page, _stub: StubServer) {
    let has_overflow = page
        .evaluate::<(), bool>(
            r#"() => {
                    const body = document.body;
                    const html = document.documentElement;
                    return body.scrollWidth > html.clientWidth || body.clientWidth > html.clientWidth;
                }"#,
            None,
        )
        .await
        .unwrap();

    assert!(!has_overflow, "Page should not have horizontal overflow");
}

async fn check_log_entry_text_wraps_within_bubble(page: Page, _stub: StubServer) {
    let overflows: bool = page
        .evaluate::<(), bool>(
            r#"() => {
                    const storyLog = document.querySelector('#story-log');
                    if (!storyLog) return false;

                    const entry = document.createElement('div');
                    entry.className = 'log-entry narration';
                    entry.innerHTML = '<span class="timestamp">10:43</span>' +
                        '<span class="text"><pre><code>The air at the gates was thick with scent. ' +
                        'The heavy,-ironic scent of history pressed against stone walls. ' +
                        'AVeryLongUnbrokenWordThatWouldNormallyOverflowTheContainerBoundsIfWrappingIsBroken ' +
                        'He ignored the distance in her eyes and the shadows that clung to the threshold.</code></pre></span>';
                    storyLog.appendChild(entry);

                    void entry.offsetHeight;

                    return entry.scrollWidth > entry.clientWidth;
                }"#,
            None,
        )
        .await
        .unwrap();

    assert!(
        !overflows,
        "Log entry with <pre><code> content should not overflow horizontally"
    );
}

/// Mid-turn the label grows to "Generating…": the input keeps its width and the
/// label stays inside the button.
async fn check_command_input_width_is_stable(page: Page, _stub: StubServer) {
    let widths: (f64, f64, f64, f64) = page
        .evaluate::<(), (f64, f64, f64, f64)>(
            r#"() => {
                const input = document.querySelector('#command-form input[name="command"]');
                const button = document.querySelector('#command-form button[type="submit"]');
                const idle = input.getBoundingClientRect().width;
                applyActionState('generating');
                const generating = input.getBoundingClientRect().width;
                const clipped = button.scrollWidth - button.clientWidth;
                applyActionState('idle');
                return [idle, generating, clipped, button.getBoundingClientRect().width];
            }"#,
            None,
        )
        .await
        .unwrap();

    assert!(
        (widths.0 - widths.1).abs() < 0.5,
        "the command input must not resize while the button reads Generating… \
         (idle {} against generating {})",
        widths.0,
        widths.1
    );
    assert!(
        widths.2 <= 0.5,
        "the Generating… label must fit the button (overflows by {}px, button {}px)",
        widths.2,
        widths.3
    );
}

async fn check_element_positioning(page: Page, _stub: StubServer) {
    let header_top = page
        .evaluate::<(), f64>(
            "document.querySelector('.header')?.getBoundingClientRect().top || -1",
            None,
        )
        .await
        .unwrap();
    let story_log_top = page
        .evaluate::<(), f64>(
            "document.querySelector('#story-log')?.getBoundingClientRect().top || -1",
            None,
        )
        .await
        .unwrap();
    let action_area_top = page
        .evaluate::<(), f64>(
            "document.querySelector('.action-area')?.getBoundingClientRect().top || -1",
            None,
        )
        .await
        .unwrap();

    assert!(
        story_log_top > header_top,
        "Story log should be below header"
    );
    assert!(
        action_area_top > story_log_top,
        "Action area should be below story log"
    );
}

async fn check_npc_portraits_horizontal_layout(page: Page, _stub: StubServer) {
    let flex_wrap: String = page
        .evaluate::<(), String>(
            r#"(() => {
                    const el = document.querySelector('.npc-portraits');
                    if (!el) return 'no-element';
                    return window.getComputedStyle(el).flexWrap;
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        flex_wrap, "nowrap",
        "NPC portraits should have flex-wrap: nowrap"
    );

    let overflow_x: String = page
        .evaluate::<(), String>(
            r#"(() => {
                    const el = document.querySelector('.npc-portraits');
                    if (!el) return 'no-element';
                    return window.getComputedStyle(el).overflowX;
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        overflow_x, "auto",
        "NPC portraits should have overflow-x: auto"
    );
}

async fn check_npc_portraits_fixed_width(page: Page, _stub: StubServer) {
    let width: f64 = page
        .evaluate::<(), f64>(
            r#"(() => {
                    const el = document.querySelector('.image-container.npc-portrait img');
                    if (!el) return 0;
                    const rect = el.getBoundingClientRect();
                    return rect.width;
                })()"#,
            None,
        )
        .await
        .unwrap();

    assert!(
        width > 50.0 && width < 120.0,
        "NPC portrait should have fixed width around 80px, got {width}"
    );
}

/// Mirrors the `max-height: 50vh` cap `assets/styles.css` puts on `#edit-textarea`.
const AUTO_GROW_VIEWPORT_HEIGHT_RATIO: f64 = 0.5;

async fn check_edit_textarea_matches_original_height(page: Page, _stub: StubServer) {
    let original_height: f64 = page
        .evaluate::<(), f64>(
            r#"(() => {
                    const text = document.querySelector('.log-entry .text');
                    if (!text) return -1;
                    const rect = text.getBoundingClientRect();
                    return rect.height;
                })()"#,
            None,
        )
        .await
        .unwrap();

    assert!(
        original_height > 0.0,
        "Original text should have a valid height"
    );

    page.evaluate::<(), bool>(
        r#"(() => {
                const entry = document.querySelector('.log-entry');
                const btn = entry?.querySelector('.edit-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
        None,
    )
    .await
    .unwrap();

    wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;

    let textarea_height: f64 = page
        .evaluate::<(), f64>(
            r#"(() => {
                    const textarea = document.querySelector('#edit-textarea');
                    if (!textarea) return -1;
                    void textarea.offsetHeight;
                    const rect = textarea.getBoundingClientRect();
                    return rect.height;
                })()"#,
            None,
        )
        .await
        .unwrap();

    assert!(textarea_height > 0.0, "Textarea should have a valid height");

    let viewport_height: f64 = page
        .evaluate::<(), f64>("window.innerHeight", None)
        .await
        .unwrap();
    let expected_height = original_height.min(viewport_height * AUTO_GROW_VIEWPORT_HEIGHT_RATIO);

    assert!(
        textarea_height >= expected_height,
        "Textarea height ({textarea_height}) should not be smaller than the entry text height ({expected_height})"
    );
    assert!(
        textarea_height <= expected_height * 2.0 + 20.0,
        "Textarea height ({textarea_height}) should not be drastically larger than the entry text height ({expected_height})"
    );
}

async fn check_responsive_layout_under_768px(page: Page, _stub: StubServer) {
    page.set_viewport_size(Viewport {
        width: 500,
        height: 800,
    })
    .await
    .unwrap();

    // Poll until the @media rule applies — reflow is async at the new viewport.
    let flex_direction = wait_for_condition_async(
        std::time::Duration::from_secs(2),
        std::time::Duration::from_millis(50),
        || async {
            page.evaluate::<(), String>(
                r#"(() => {
                        const el = document.querySelector('.main-container');
                        if (!el) return '';
                        return window.getComputedStyle(el).flexDirection;
                    })()"#,
                None,
            )
            .await
            .unwrap_or_default()
                == "column"
        },
    )
    .await;

    assert!(
        flex_direction,
        "At <768px viewport, .main-container should switch to flex-direction: column (responsive @media rule)"
    );
}

async fn check_root_design_tokens(page: Page, _stub: StubServer) {
    let covered: usize = page
        .evaluate::<(), usize>(
            r#"(() => {
                    const root = window.getComputedStyle(document.documentElement);
                    const prefixes = [
                        '--color-bg-',
                        '--color-text-',
                        '--color-accent-',
                        '--color-button-',
                        '--color-log-',
                        '--font-',
                    ];
                    return prefixes.filter(p => {
                        const vars = Array.from(document.styleSheets)
                            .flatMap(s => {
                                try {
                                    return Array.from(s.cssRules);
                                } catch (_) {
                                    return [];
                                }
                            })
                            .filter(r => r.type === CSSRule.STYLE_RULE && r.selectorText === ':root')
                            .flatMap(r => Array.from(r.style))
                            .filter(name => name.startsWith(p));
                        return vars.length > 0 || root.getPropertyValue(p + '-primary') !== '';
                    }).length;
                })()"#,
            None,
        )
        .await
        .unwrap();

    assert!(
        covered >= 5,
        "CSS :root should define variables in at least 5 of 6 core areas, only {covered}/6 found"
    );
}

/// The forced-colors rule in `assets/styles.css` restores the focus ring that
/// the field's `outline: none` reset removes.
async fn check_forced_colors_focus_ring(page: Page, _stub: StubServer) {
    page.emulate_media(Some(
        EmulateMediaOptions::builder()
            .forced_colors(ForcedColors::Active)
            .build(),
    ))
    .await
    .expect("Failed to emulate forced colors");

    // `:focus-visible` matches a keyboard focus, not a programmatic one, so the
    // input has to be reached with real key input. The tab bound is loose: a
    // control added to the shipped story-log template shifts `#command-input`.
    let mut reached = false;
    for _ in 0..30 {
        page.keyboard().press("Tab", None).await.unwrap();
        let active: String = page
            .evaluate::<(), String>("() => document.activeElement.id", None)
            .await
            .unwrap_or_default();
        if active == "command-input" {
            reached = true;
            break;
        }
    }
    assert!(reached, "Tab must reach #command-input");

    let ring: bool = page
        .evaluate::<(), bool>(
            r#"() => {
                    const input = document.getElementById('command-input');
                    const s = window.getComputedStyle(input);
                    return s.outlineStyle !== 'none' && parseFloat(s.outlineWidth) > 0;
                }"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        ring,
        "forced colors must restore a visible focus ring on the command input"
    );
}

/// The client's own inline-error renderer fills the form's slot, so the action
/// area must grow to hold it: the command row stays below the story log and the
/// slot stays above the bottom of the viewport.
async fn check_action_area_makes_room_for_the_inline_error(page: Page, _stub: StubServer) {
    let (resting_height, grown_height, command_row_overlap, slot_below_viewport): (
        f64,
        f64,
        f64,
        f64,
    ) = page
        .evaluate::<(), (f64, f64, f64, f64)>(
            r#"() => {
                    const area = document.getElementById('action-area');
                    const input = document.getElementById('command-input');
                    const log = document.getElementById('story-log');
                    const slot = document.querySelector('#command-form [data-error-slot]');
                    if (!area || !input || !log || !slot) return [0, 0, 0, 0];
                    const resting = area.getBoundingClientRect().height;
                    renderInlineError(
                        slot,
                        'The engine is unreachable.',
                        'No response from the server.',
                    );
                    void slot.offsetHeight;
                    return [
                        resting,
                        area.getBoundingClientRect().height,
                        log.getBoundingClientRect().bottom - input.getBoundingClientRect().top,
                        slot.getBoundingClientRect().bottom - window.innerHeight,
                    ];
                }"#,
            None,
        )
        .await
        .unwrap();

    assert!(
        grown_height > resting_height,
        "the action area must grow to hold the inline error, got {grown_height} \
         against a resting {resting_height}"
    );
    assert!(
        command_row_overlap <= 0.0,
        "the command row must stay below the story log (overlap {command_row_overlap}px)"
    );
    assert!(
        slot_below_viewport <= 0.0,
        "the inline error must be fully visible (slot bottom {slot_below_viewport}px past the viewport)"
    );
}
