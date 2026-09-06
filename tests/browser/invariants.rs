//! Rendering invariants (named exemption in STRATEGY.md): no spec link, test code is the definition. CSS computed styles, layout measurements, text-wrap behavior — only a real browser can observe these. Nine checks share one server+browser (no server-state mutation); each runs on a fresh page via `run_subtest` with panic isolation and a per-check timing summary.

use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

use futures_util::future::FutureExt;
use playwright_rs::{Browser, Page, Viewport};

use super::*;

struct SubtestReport {
    name: &'static str,
    duration: Duration,
    passed: bool,
}

/// Run one invariant check on a fresh page of the shared browser.
///
/// A panic in `check` is caught so the remaining checks still run; the failure
/// is recorded and reported at the end. The page is closed afterward.
async fn run_subtest<Fut>(
    browser: &Browser,
    port: u16,
    name: &'static str,
    check: impl FnOnce(Page) -> Fut,
) -> SubtestReport
where
    Fut: std::future::Future<Output = ()>,
{
    let start = Instant::now();
    let page = browser.new_page().await.unwrap();
    goto_with_connection_check(&page, port)
        .await
        .expect("Failed to connect to server");
    let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

    // Clone for the check; the original closes the page afterward.
    let result = AssertUnwindSafe(check(page.clone())).catch_unwind().await;
    let _ = page.close().await;

    SubtestReport {
        name,
        duration: start.elapsed(),
        passed: result.is_ok(),
    }
}

fn print_summary(reports: &[SubtestReport]) {
    eprintln!("--- Invariant subtests ({}) ---", reports.len());
    for r in reports {
        let status = if r.passed { "OK" } else { "FAIL" };
        eprintln!(
            "  {:>7.3}s  [{status}]  {}",
            r.duration.as_secs_f64(),
            r.name
        );
    }
    let total: f64 = reports.iter().map(|r| r.duration.as_secs_f64()).sum();
    eprintln!("  {total:>7.3}s  total (shared server+browser)");
}

#[tokio::test]
async fn test_invariants() {
    let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
    let _server = TestServer::new_with_mock(port, TEST_WORLD, TEST_PERSONA).await;
    let (_playwright, browser) = launch_chrome().await;

    let reports = vec![
        run_subtest(
            &browser,
            port,
            "story_log_scrollable",
            check_story_log_scrollable,
        )
        .await,
        run_subtest(
            &browser,
            port,
            "no_horizontal_overflow",
            check_no_horizontal_overflow,
        )
        .await,
        run_subtest(
            &browser,
            port,
            "log_entry_text_wraps_within_bubble",
            check_log_entry_text_wraps_within_bubble,
        )
        .await,
        run_subtest(
            &browser,
            port,
            "element_positioning",
            check_element_positioning,
        )
        .await,
        run_subtest(
            &browser,
            port,
            "npc_portraits_horizontal_layout",
            check_npc_portraits_horizontal_layout,
        )
        .await,
        run_subtest(
            &browser,
            port,
            "npc_portraits_fixed_width",
            check_npc_portraits_fixed_width,
        )
        .await,
        run_subtest(
            &browser,
            port,
            "edit_textarea_matches_original_height",
            check_edit_textarea_matches_original_height,
        )
        .await,
        run_subtest(
            &browser,
            port,
            "responsive_layout_under_768px",
            check_responsive_layout_under_768px,
        )
        .await,
        run_subtest(
            &browser,
            port,
            "root_design_tokens",
            check_root_design_tokens,
        )
        .await,
    ];

    print_summary(&reports);

    let failed: Vec<&str> = reports
        .iter()
        .filter(|r| !r.passed)
        .map(|r| r.name)
        .collect();
    if !failed.is_empty() {
        panic!(
            "{} invariant subtest(s) failed: {}",
            failed.len(),
            failed.join(", ")
        );
    }
}

async fn check_story_log_scrollable(page: Page) {
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

async fn check_no_horizontal_overflow(page: Page) {
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

/// Regression test for text overflowing log-entry bubbles.
async fn check_log_entry_text_wraps_within_bubble(page: Page) {
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

async fn check_element_positioning(page: Page) {
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

async fn check_npc_portraits_horizontal_layout(page: Page) {
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

async fn check_npc_portraits_fixed_width(page: Page) {
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

async fn check_edit_textarea_matches_original_height(page: Page) {
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

    wait_for_element_exists(&page, "#edit-textarea", 10).await;

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

    assert!(
        textarea_height >= original_height,
        "Textarea height ({textarea_height}) should not be smaller than original text height ({original_height})"
    );
    assert!(
        textarea_height <= original_height * 2.0 + 20.0,
        "Textarea height ({textarea_height}) should not be drastically larger than original text height ({original_height})"
    );
}

/// Responsive layout invariant: `styles.css` declares `@media (max-width: 768px)`
/// which flips `.main-container` to `flex-direction: column` (desktop is the
/// default `row`). No other test exercises the responsive rules; this one proves
/// the @media machinery is wired by reading the computed style at a narrow width.
async fn check_responsive_layout_under_768px(page: Page) {
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

/// Design-token invariant: `:root` declares the core custom-property tokens
/// used by the UI.
async fn check_root_design_tokens(page: Page) {
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
