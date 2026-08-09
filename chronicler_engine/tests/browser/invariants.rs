//! Rendering invariants (named exemption in STRATEGY.md): no spec link, test code is the definition. CSS computed styles, layout measurements, text-wrap behavior — only a real browser can observe these.

use playwright_rs::Viewport;

use super::*;

#[tokio::test]
async fn test_story_log_scrollable() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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
        },
    )
    .await;
}

#[tokio::test]
async fn test_no_horizontal_overflow() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
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
    })
    .await;
}

/// Regression test for text overflowing log-entry bubbles.
#[tokio::test]
async fn test_log_entry_text_wraps_within_bubble() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
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
    })
    .await;
}

#[tokio::test]
async fn test_element_positioning() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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
        },
    )
    .await;
}

#[tokio::test]
async fn test_npc_portraits_horizontal_layout() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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
        },
    )
    .await;
}

#[tokio::test]
async fn test_npc_portraits_fixed_width() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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
        },
    )
    .await;
}

#[tokio::test]
async fn test_edit_textarea_matches_original_height() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
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
    })
    .await;
}

/// Responsive layout invariant: `styles.css` declares `@media (max-width: 768px)`
/// which flips `.main-container` to `flex-direction: column` (desktop is the
/// default `row`). No other test exercises the responsive rules; this one proves
/// the @media machinery is wired by reading the computed style at a narrow width.
// ponytail: one breakpoint (< 768px) is the minimum that proves the @media rule
// applies; the second breakpoint (< 480px) is cosmetic header-wrap detail — add
// only if this test stops proving the responsive machinery is wired.
#[tokio::test]
async fn test_responsive_layout_under_768px() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
        page.set_viewport_size(Viewport {
            width: 500,
            height: 800,
        })
        .await
        .unwrap();

        // Poll the computed style until the @media rule applies (reflow is
        // async at the new viewport) or timeout — avoids a blind sleep.
        // ponytail: one breakpoint (< 768px) is the minimum that proves the @media rule
        // applies; the second breakpoint (< 480px) is cosmetic header-wrap detail — add
        // only if this test stops proving the responsive machinery is wired.
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
    })
    .await;
}

/// Design-token invariant: `:root` declares the core custom-property tokens
/// used by the UI. This replaces the HTTP-level CSS-content checks from the
/// deleted `tests/integration/model/css.rs`.
#[tokio::test]
async fn test_root_design_tokens() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
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
    })
    .await;
}
