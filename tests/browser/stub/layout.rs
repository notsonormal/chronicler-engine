//! Layout checks (declared exemption in the spec-coverage validator): no spec link, test code is the definition. CSS computed styles, layout measurements, text-wrap behavior — only a real browser can observe these. Twelve checks share one browser through the shared runner. Each runs on a fresh page against its own stub server, with panic isolation and a per-check timing summary.

use std::time::Duration;

use chronicler_engine::domain::model::state::generation_status::Step;
use playwright_rs::{EmulateMediaOptions, ForcedColors, Page, Viewport};

use super::support::StubRunner;
use super::*;

#[tokio::test]
async fn run_layout() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(StubActionOutcome::Pending, check_story_log_scrollable)
        .await;
    runner
        .run(StubActionOutcome::Pending, check_no_horizontal_overflow)
        .await;
    runner
        .run_without_convergence(
            "the check appends its own entry to the story log by script",
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
            check_character_portraits_horizontal_layout,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_character_portraits_fixed_width,
        )
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
    runner
        .run_without_convergence(
            "the check writes its own markup into the status display by script",
            StubActionOutcome::Pending,
            check_a_long_status_sentence_clamps_with_a_visible_cut,
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

async fn check_command_input_width_is_stable(page: Page, stub: StubServer) {
    const MEASURE: &str = r#"() => {
        const input = document.querySelector('#command-form input[name="command"]');
        const button = document.getElementById('submit-btn');
        return [
            input.getBoundingClientRect().width,
            button.scrollWidth - button.clientWidth,
            button.getBoundingClientRect().width,
            button.textContent.trim(),
        ];
    }"#;
    wait_for_screen_current(&page).await;
    let idle: (f64, f64, f64, String) = page
        .evaluate::<(), (f64, f64, f64, String)>(MEASURE, None)
        .await
        .unwrap();
    assert_eq!(idle.3, "Send", "an idle Game labels the button Send");

    stub.status_handle().set(StubStatus::Step(Step::Narrating));
    ask_once(&page).await;
    let generating: (f64, f64, f64, String) = page
        .evaluate::<(), (f64, f64, f64, String)>(MEASURE, None)
        .await
        .unwrap();
    assert!(
        generating.3.contains("Generating"),
        "a Job that runs makes the server render the generating label, got {:?}",
        generating.3
    );

    assert!(
        (idle.0 - generating.0).abs() < 0.5,
        "the command input must not resize while the button reads Generating… \
         (idle {} against generating {})",
        idle.0,
        generating.0
    );
    assert!(
        generating.1 <= 0.5,
        "the Generating… label must fit the button (overflows by {}px, button {}px)",
        generating.1,
        generating.2
    );

    stub.status_handle().set(StubStatus::Idle);
    ask_once(&page).await;
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

async fn check_character_portraits_horizontal_layout(page: Page, _stub: StubServer) {
    let flex_wrap: String = page
        .evaluate::<(), String>(
            r#"(() => {
                    const el = document.querySelector('.character-portraits');
                    if (!el) return 'no-element';
                    return window.getComputedStyle(el).flexWrap;
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        flex_wrap, "nowrap",
        "Character portraits should have flex-wrap: nowrap"
    );

    let overflow_x: String = page
        .evaluate::<(), String>(
            r#"(() => {
                    const el = document.querySelector('.character-portraits');
                    if (!el) return 'no-element';
                    return window.getComputedStyle(el).overflowX;
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        overflow_x, "auto",
        "Character portraits should have overflow-x: auto"
    );
}

async fn check_character_portraits_fixed_width(page: Page, _stub: StubServer) {
    let width: f64 = page
        .evaluate::<(), f64>(
            r#"(() => {
                    const el = document.querySelector('.image-container.character-portrait img');
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
        "Character portrait should have fixed width around 80px, got {width}"
    );
}

#[derive(serde::Deserialize)]
struct EditTextareaBox {
    height: f64,
    #[serde(rename = "maxHeight")]
    max_height: String,
}

async fn read_edit_textarea_box(page: &Page) -> EditTextareaBox {
    page.evaluate::<(), EditTextareaBox>(
        r#"(() => {
                const textarea = document.querySelector('#edit-textarea');
                if (!textarea) return { height: -1, maxHeight: '' };
                void textarea.offsetHeight;
                return {
                    height: textarea.getBoundingClientRect().height,
                    maxHeight: getComputedStyle(textarea).maxHeight,
                };
            })()"#,
        None,
    )
    .await
    .unwrap()
}

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

    let textarea = read_edit_textarea_box(&page).await;
    assert!(textarea.height > 0.0, "Textarea should have a valid height");

    let cap = textarea
        .max_height
        .strip_suffix("px")
        .and_then(|value| value.parse::<f64>().ok())
        .expect("`#edit-textarea` must cap its height in px");
    let expected_height = original_height.min(cap);

    assert!(
        textarea.height >= expected_height,
        "Textarea height ({}) should not be smaller than the entry text height ({expected_height})",
        textarea.height
    );
    assert!(
        textarea.height <= expected_height * 2.0 + 20.0,
        "Textarea height ({}) should not be drastically larger than the entry text height ({expected_height})",
        textarea.height
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

/// A clamped sentence's cut has to be visible — a silently hidden tail reads as
/// the end of the sentence — and the Details control beside it must stay laid out
/// and focusable.
async fn check_a_long_status_sentence_clamps_with_a_visible_cut(page: Page, _stub: StubServer) {
    let (
        line_clamp,
        text_align,
        box_height,
        lines_needed,
        line_height,
        toggle_width,
        toggle_height,
        toggle_off_column,
        toggle_focused,
    ): (String, String, f64, f64, f64, f64, f64, f64, bool) = page
        .evaluate::<(), (String, String, f64, f64, f64, f64, f64, f64, bool)>(
            r#"() => {
                    const display = document.getElementById('status-display');
                    if (!display) return ['', '', 0, 0, 0, 0, 0, 0, false];
                    display.innerHTML =
                        '<div class="error-disclosure">' +
                        '<span class="error-disclosure-message">' +
                        '<span class="error-disclosure-text">' +
                        'The language model could not be reached, and the retry after the ' +
                        'connection came back did not answer either, so the turn you sent was ' +
                        'never narrated and nothing in this game has moved on from it.' +
                        '</span></span>' +
                        '<button type="button" class="error-details-toggle" ' +
                        'aria-expanded="false" aria-controls="layout-status-popover">' +
                        'Details</button>' +
                        '<div class="error-detail-popover" id="layout-status-popover" hidden>' +
                        '</div></div>';
                    const text = display.querySelector('.error-disclosure-text');
                    const toggle = display.querySelector('.error-details-toggle');
                    if (!text || !toggle) return ['', '', 0, 0, 0, 0, 0, 0, false];
                    const style = getComputedStyle(text);
                    // The clamp fragments the text it cuts, so the height the
                    // sentence really needs is measured off a clone of it.
                    const unclamped = document.createElement('div');
                    unclamped.style.cssText =
                        'position:absolute;left:-9999px;top:0;white-space:normal;' +
                        'font-size:' + style.fontSize + ';font-family:' + style.fontFamily +
                        ';line-height:' + style.lineHeight + ';width:' + text.clientWidth +
                        'px;';
                    unclamped.textContent = text.textContent;
                    document.body.appendChild(unclamped);
                    const needed = unclamped.getBoundingClientRect().height;
                    unclamped.remove();
                    toggle.focus();
                    return [
                        style.webkitLineClamp || style.getPropertyValue('-webkit-line-clamp'),
                        style.textAlign,
                        text.getBoundingClientRect().height,
                        needed,
                        parseFloat(style.lineHeight),
                        toggle.getBoundingClientRect().width,
                        toggle.getBoundingClientRect().height,
                        toggle.getBoundingClientRect().right -
                            display.getBoundingClientRect().right,
                        document.activeElement === toggle,
                    ];
                }"#,
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        line_clamp, "2",
        "the status column must clamp a long sentence to two lines"
    );
    assert_eq!(
        text_align, "left",
        "the clamp draws its ellipsis only for left-aligned text, so the failure line \
         must not inherit the column's own right alignment"
    );
    assert!(
        box_height <= 2.0 * line_height + 1.0,
        "the clamp must draw two lines and no more ({box_height}px against {line_height}px a line)"
    );
    assert!(
        lines_needed > box_height + 1.0,
        "the sentence must need more lines than the clamp draws for this check to mean \
         anything ({lines_needed}px of text in {box_height}px)"
    );
    assert!(
        toggle_width > 0.0 && toggle_height > 0.0,
        "the clamp must not collapse the Details control ({toggle_width}x{toggle_height})"
    );
    assert!(
        toggle_off_column <= 2.0,
        "the Details control must stay inside the status column (off by {toggle_off_column}px)"
    );
    assert!(
        toggle_focused,
        "the Details control must stay focusable beside the clamped sentence"
    );
}

/// The client's own inline-error renderer fills the form's slot, so the action
/// area must grow to hold it.
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
