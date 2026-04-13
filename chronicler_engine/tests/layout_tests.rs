//! Layout Tests for Chronicler Engine HTMX Dashboard
//!
//! Run with: cargo test --test layout_tests
//!
//! These tests programmatically validate visual layout issues:
//! - Image sizing and overflow
//! - Element alignment
//! - Responsive behavior
//! - CSS property constraints

mod test_utils;
use test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use playwright_rs::Playwright;

    const TEST_PORT: u16 = 3002;
    const TEST_WORLD: &str = "test";

    #[tokio::test]
    async fn test_image_containers_have_max_size() {
        let _server = TestServer::new(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let image_containers = page.evaluate::<(), serde_json::Value>(
            r#"() => {
                const containers = document.querySelectorAll('.image-container');
                return Array.from(containers).map(c => {
                    const style = window.getComputedStyle(c);
                    const rect = c.getBoundingClientRect();
                    return {
                        className: c.className,
                        maxWidth: style.maxWidth,
                        maxHeight: style.maxHeight,
                        width: rect.width,
                        height: rect.height,
                        overflow: style.overflow,
                        parentWidth: c.parentElement ? c.parentElement.getBoundingClientRect().width : null
                    };
                });
            }"#,
            None,
        ).await.unwrap();

        println!("Image containers: {:?}", image_containers);

        for container in image_containers.as_array().unwrap() {
            let max_width = container["maxWidth"].as_str().unwrap_or("");
            let overflow = container["overflow"].as_str().unwrap_or("");
            let parent_width = container["parentWidth"].as_f64().unwrap_or(0.0);
            let width = container["width"].as_f64().unwrap_or(0.0);

            assert_ne!(
                max_width, "none",
                "Image container should have max-width set, got: none"
            );
            assert_ne!(
                overflow, "visible",
                "Image container should not have overflow:visible (causes overflow issues)"
            );

            if parent_width > 0.0 && width > parent_width {
                panic!(
                    "Image container width ({}) exceeds parent width ({}) - will cause overflow",
                    width, parent_width
                );
            }
        }

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_npc_portraits_alignment() {
        let _server = TestServer::new(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let npc_portraits = page
            .evaluate::<(), serde_json::Value>(
                r#"() => {
                const portraits = document.querySelectorAll('.npc-portrait');
                return Array.from(portraits).map((p, i) => {
                    const rect = p.getBoundingClientRect();
                    const parent = p.parentElement;
                    const parentRect = parent ? parent.getBoundingClientRect() : null;
                    return {
                        index: i,
                        top: rect.top,
                        left: rect.left,
                        width: rect.width,
                        height: rect.height,
                        parentTop: parentRect ? parentRect.top : null,
                        parentLeft: parentRect ? parentRect.left : null,
                        position: window.getComputedStyle(p).position
                    };
                });
            }"#,
                None,
            )
            .await
            .unwrap();

        println!("NPC portraits: {:?}", npc_portraits);

        if let Some(portraits) = npc_portraits.as_array() {
            if portraits.len() > 1 {
                let first_left = portraits[0]["left"].as_f64().unwrap_or(0.0);
                for (i, portrait) in portraits.iter().enumerate().skip(1) {
                    let left = portrait["left"].as_f64().unwrap_or(0.0);
                    let top = portrait["top"].as_f64().unwrap_or(0.0);
                    let first_top = portraits[0]["top"].as_f64().unwrap_or(0.0);

                    assert_eq!(
                        left, first_left,
                        "NPC portrait {} left position ({}) should match first ({})",
                        i, left, first_left
                    );
                    assert_eq!(
                        top, first_top,
                        "NPC portrait {} top position should align with first",
                        i
                    );
                }
            }
        }

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_no_horizontal_overflow() {
        let _server = TestServer::new(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

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

        let elements_with_overflow = page
            .evaluate::<(), serde_json::Value>(
                r#"() => {
                const allElements = document.querySelectorAll('*');
                const overflowIssues = [];
                for (const el of allElements) {
                    const style = window.getComputedStyle(el);
                    if (style.overflow === 'visible' || style.overflowX === 'visible') {
                        const rect = el.getBoundingClientRect();
                        const parent = el.parentElement;
                        if (parent) {
                            const parentRect = parent.getBoundingClientRect();
                            if (rect.width > parentRect.width || rect.right > parentRect.right) {
                                overflowIssues.push({
                                    tag: el.tagName,
                                    class: el.className,
                                    overflow: style.overflow,
                                    width: rect.width,
                                    parentWidth: parentRect.width
                                });
                            }
                        }
                    }
                }
                return overflowIssues;
            }"#,
                None,
            )
            .await
            .unwrap();

        println!("Overflow issues: {:?}", elements_with_overflow);

        if let Some(issues) = elements_with_overflow.as_array() {
            assert!(
                issues.is_empty(),
                "Found {} elements with overflow issues",
                issues.len()
            );
        }

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_element_positioning() {
        let _server = TestServer::new(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let layout = page.evaluate::<(), serde_json::Value>(
            r#"() => {
                const header = document.querySelector('.header');
                const storyLog = document.querySelector('#story-log');
                const actionArea = document.querySelector('.action-area');
                const mainContainer = document.querySelector('.main-container');
                const visualSidebar = document.querySelector('.visual-sidebar');

                const getRect = (el) => el ? el.getBoundingClientRect() : null;
                const getStyle = (el) => el ? window.getComputedStyle(el) : null;

                return {
                    header: getRect(header) && { top: getRect(header).top, left: getRect(header).left, width: getRect(header).width, height: getRect(header).height, display: getStyle(header).display },
                    storyLog: getRect(storyLog) && { top: getRect(storyLog).top, left: getRect(storyLog).left, width: getRect(storyLog).width, height: getRect(storyLog).height, display: getStyle(storyLog).display },
                    actionArea: getRect(actionArea) && { top: getRect(actionArea).top, left: getRect(actionArea).left, width: getRect(actionArea).width, height: getRect(actionArea).height, display: getStyle(actionArea).display },
                    mainContainer: getRect(mainContainer) && { top: getRect(mainContainer).top, height: getRect(mainContainer).height },
                    visualSidebar: getRect(visualSidebar) && { top: getRect(visualSidebar).top, width: getRect(visualSidebar).width }
                };
            }"#,
            None,
        ).await.unwrap();

        println!("Layout: {:?}", layout);

        let header_top = layout["header"]["top"].as_f64().unwrap_or(-1.0);
        let story_log_top = layout["storyLog"]["top"].as_f64().unwrap_or(-1.0);
        let action_area_top = layout["actionArea"]["top"].as_f64().unwrap_or(-1.0);

        assert!(
            story_log_top > header_top,
            "Story log should be below header"
        );
        assert!(
            action_area_top > story_log_top,
            "Action area should be below story log"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_images_have_object_fit() {
        let _server = TestServer::new(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let images = page
            .evaluate::<(), serde_json::Value>(
                r#"() => {
                const imgs = document.querySelectorAll('.image-container img');
                return Array.from(imgs).map(img => {
                    const style = window.getComputedStyle(img);
                    return {
                        src: img.src,
                        objectFit: style.objectFit,
                        width: style.width,
                        height: style.height,
                        maxWidth: style.maxWidth,
                        maxHeight: style.maxHeight
                    };
                });
            }"#,
                None,
            )
            .await
            .unwrap();

        println!("Images: {:?}", images);

        for img in images.as_array().unwrap() {
            let object_fit = img["objectFit"].as_str().unwrap_or("");
            let max_width = img["maxWidth"].as_str().unwrap_or("");

            assert_ne!(object_fit, "", "Image should have object-fit set");
            assert_ne!(max_width, "none", "Image should have max-width constraint");
        }

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_visual_sidebar_stays_within_bounds() {
        let _server = TestServer::new(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let sidebar_bounds = page
            .evaluate::<(), serde_json::Value>(
                r#"() => {
                const sidebar = document.querySelector('.visual-sidebar');
                const container = document.querySelector('.main-container');
                if (!sidebar || !container) return null;

                const sidebarRect = sidebar.getBoundingClientRect();
                const containerRect = container.getBoundingClientRect();
                const style = window.getComputedStyle(sidebar);

                return {
                    sidebarWidth: sidebarRect.width,
                    containerWidth: containerRect.width,
                    sidebarRight: sidebarRect.right,
                    containerRight: containerRect.right,
                    overflow: style.overflow,
                    display: style.display
                };
            }"#,
                None,
            )
            .await
            .unwrap();

        println!("Sidebar bounds: {:?}", sidebar_bounds);

        let sidebar_right = sidebar_bounds["sidebarRight"].as_f64().unwrap_or(0.0);
        let container_right = sidebar_bounds["containerRight"].as_f64().unwrap_or(0.0);

        assert!(
            sidebar_right <= container_right + 1.0,
            "Sidebar ({}) should stay within container ({})",
            sidebar_right,
            container_right
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_npc_portraits_container_flex_layout() {
        let _server = TestServer::new(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let portraits_container = page
            .evaluate::<(), serde_json::Value>(
                r#"() => {
                const container = document.querySelector('.npc-portraits');
                if (!container) return null;

                const style = window.getComputedStyle(container);
                const rect = container.getBoundingClientRect();

                return {
                    display: style.display,
                    flexDirection: style.flexDirection,
                    flexWrap: style.flexWrap,
                    justifyContent: style.justifyContent,
                    alignItems: style.alignItems,
                    width: rect.width,
                    children: container.children.length
                };
            }"#,
                None,
            )
            .await
            .unwrap();

        println!("Portraits container: {:?}", portraits_container);

        let display = portraits_container["display"].as_str().unwrap_or("");

        if !display.is_empty() {
            assert_eq!(
                display, "flex",
                "NPC portraits container should use flex layout for proper alignment"
            );
        }

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_all_images_within_viewport() {
        let _server = TestServer::new(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let images_in_viewport = page
            .evaluate::<(), serde_json::Value>(
                r#"() => {
                const imgs = document.querySelectorAll('.image-container');
                const viewportWidth = window.innerWidth;
                const viewportHeight = window.innerHeight;

                return Array.from(imgs).map(img => {
                    const rect = img.getBoundingClientRect();
                    return {
                        left: rect.left,
                        top: rect.top,
                        right: rect.right,
                        bottom: rect.bottom,
                        width: rect.width,
                        height: rect.height,
                        withinHorizontal: rect.right <= viewportWidth,
                        withinVertical: rect.bottom <= viewportHeight,
                        offScreenLeft: rect.right < 0,
                        offScreenTop: rect.bottom < 0
                    };
                });
            }"#,
                None,
            )
            .await
            .unwrap();

        println!("Images viewport check: {:?}", images_in_viewport);

        for img in images_in_viewport.as_array().unwrap() {
            let within_horizontal = img["withinHorizontal"].as_bool().unwrap_or(false);
            let within_vertical = img["withinVertical"].as_bool().unwrap_or(false);

            assert!(
                within_horizontal,
                "Image should be within horizontal viewport bounds"
            );
            assert!(
                within_vertical,
                "Image should be within vertical viewport bounds"
            );
        }

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_story_log_scrollable() {
        let _server = TestServer::new(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let story_log_scroll = page
            .evaluate::<(), serde_json::Value>(
                r#"() => {
                const el = document.querySelector('#story-log');
                if (!el) return null;

                const style = window.getComputedStyle(el);
                return {
                    overflowY: style.overflowY,
                    overflowX: style.overflowX,
                    maxHeight: style.maxHeight,
                    height: style.height
                };
            }"#,
                None,
            )
            .await
            .unwrap();

        println!("Story log scroll: {:?}", story_log_scroll);

        let overflow_y = story_log_scroll["overflowY"].as_str().unwrap_or("");

        assert!(
            overflow_y == "auto" || overflow_y == "scroll",
            "Story log should have overflow-y set to auto or scroll"
        );

        browser.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_css_box_sizing_consistent() {
        let _server = TestServer::new(TEST_PORT, TEST_WORLD);

        let playwright = Playwright::launch().await.unwrap();
        let browser = playwright.chromium().launch().await.unwrap();
        let page = browser.new_page().await.unwrap();

        page.goto(&format!("http://127.0.0.1:{}", TEST_PORT), None)
            .await
            .unwrap();

        // Wait for initial content to load
        let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

        let box_sizing = page.evaluate::<(), serde_json::Value>(
            r#"() => {
                const elements = ['.header', '#story-log', '.action-area', '.visual-sidebar', '.main-container'];
                return elements.map(sel => {
                    const el = document.querySelector(sel);
                    if (!el) return { selector: sel, boxSizing: null };
                    return {
                        selector: sel,
                        boxSizing: window.getComputedStyle(el).boxSizing
                    };
                });
            }"#,
            None,
        ).await.unwrap();

        println!("Box sizing: {:?}", box_sizing);

        for el in box_sizing.as_array().unwrap() {
            let box_sizing = el["boxSizing"].as_str().unwrap_or("");
            if !box_sizing.is_empty() {
                assert_eq!(
                    box_sizing, "border-box",
                    "Element {} should use border-box for consistent sizing",
                    el["selector"]
                );
            }
        }

        browser.close().await.unwrap();
    }
}
