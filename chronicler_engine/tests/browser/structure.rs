use super::*;

#[tokio::test]
async fn test_page_loads() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let title = page.title().await.unwrap();
            assert_eq!(title, "Chronicler Engine");

            assert!(
                element_exists(&page, ".header").await,
                "Header should exist"
            );
            assert!(
                element_exists(&page, "#story-log").await,
                "Story log should exist"
            );
            assert!(
                element_exists(&page, ".action-area").await,
                "Action area should exist"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_header_displays_game_title() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let title: String = page
                .evaluate::<(), String>(
                    "document.querySelector('.game-title')?.innerText || ''",
                    None,
                )
                .await
                .unwrap();

            assert_eq!(
                title, "Chronicler Engine",
                "Header should display game title"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_connection_status_indicator() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            assert!(
                element_exists(&page, "#connection-status").await,
                "Connection status indicator should exist"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_action_area_elements() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            assert!(
                element_exists(&page, "#command-form input").await,
                "Input field should exist"
            );
            assert!(
                element_exists(&page, "#command-form button").await,
                "Submit button should exist"
            );
        },
    )
    .await;
}

/// Input field must not have HTML5 `required` validation.
#[tokio::test]
async fn test_input_no_required_attribute() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let has_required: bool = page
                .evaluate::<(), bool>(
                    "document.querySelector('#command-form input')?.hasAttribute('required')",
                    None,
                )
                .await
                .unwrap();
            assert!(
                !has_required,
                "Input should NOT have required attribute (empty input triggers continuation)"
            );
        },
    )
    .await;
}

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
async fn test_visual_sidebar_exists() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            assert!(
                element_exists(&page, ".visual-sidebar").await,
                "Visual sidebar should exist"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_form_stays_static_after_submission() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let form_id_before: String = page
                .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
                .await
                .unwrap();

            page.evaluate::<(), ()>(
                "(() => { 
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'look';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()",
                None,
            )
            .await
            .unwrap();

            let _ = wait_for_element_children(&page, "#story-log .log-entry", 2).await;

            let form_id_after: String = page
                .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
                .await
                .unwrap();

            assert_eq!(
                form_id_before, form_id_after,
                "Form should stay in DOM (static shell)"
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
