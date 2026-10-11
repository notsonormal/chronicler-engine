//! Stub-browser tests for the LLM Messages panel: the shipped client's keyboard path over a canned row. Tagged against `docs/specs/browser_llm_messages.md`.

// Keyboard operation is the shipped shell's: Enter and Space on the native
// button fire the same click handler a mouse uses. The stub server only serves
// the canned card, so faking the engine changes nothing.
//
// The panel's own 4s poll re-serves the fragment, so a swap can land between
// any two reads. Every assertion polls the state it wants rather than sampling
// once, and reads the live DOM rather than a Playwright element handle that a
// swap has detached.

use std::time::Duration;

use super::*;
use super::support::StubRunner;

/// Open the LLM Messages tab. The stub tier has no htmx swap lifecycle to race,
/// so the tab click needs no settle gate.
async fn open_llm_messages_tab(page: &playwright_rs::Page) {
    page.locator(r#"[data-tab="llm-messages"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(page, ".llm-message-card", Duration::from_millis(2000)).await;
}

/// The card's expanded class and the header's `aria-expanded`, read together so
/// the pair can be asserted as one state.
async fn llm_header_state(page: &playwright_rs::Page) -> (bool, String) {
    page.evaluate::<(), (bool, String)>(
        r#"(() => {
            const card = document.querySelector('.llm-message-card');
            const header = card ? card.querySelector('.llm-message-header') : null;
            return [
                !!(card && card.classList.contains('expanded')),
                header ? (header.getAttribute('aria-expanded') || '') : '',
            ];
        })()"#,
        None,
    )
    .await
    .unwrap()
}

/// Poll until the row reports `expanded` in both places it exposes it.
async fn wait_for_row_state(
    page: &playwright_rs::Page,
    want_expanded: bool,
    want_aria: &str,
) -> bool {
    let want_aria = want_aria.to_string();
    wait_for_condition_async(
        Duration::from_millis(2000),
        Duration::from_millis(20),
        || {
            let want_aria = want_aria.clone();
            async move {
                let (expanded, aria) = llm_header_state(page).await;
                expanded == want_expanded && aria == want_aria
            }
        },
    )
    .await
}

/// Poll until the row's body reaches the wanted visibility.
async fn wait_for_body(page: &playwright_rs::Page, visible: bool) -> bool {
    wait_for_condition_async(
        Duration::from_millis(2000),
        Duration::from_millis(20),
        || async {
            page.locator(".llm-message-body")
                .await
                .is_visible()
                .await
                .unwrap_or(false)
                == visible
        },
    )
    .await
}

/// Stash the current header node, so a later read can prove the panel swap
/// replaced it.
async fn stash_header_node(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"() => { window.__llmHeaderBefore = document.querySelector('.llm-message-header'); }"#,
        None,
    )
    .await
    .unwrap();
}

/// Wait until the panel swap has replaced the stashed header node.
async fn wait_for_header_replacement(page: &playwright_rs::Page) {
    let replaced = wait_for_condition_async(
        Duration::from_millis(2000),
        Duration::from_millis(20),
        || async {
            page.evaluate::<(), bool>(
                r#"() => document.querySelector('.llm-message-header') !== window.__llmHeaderBefore"#,
                None,
            )
            .await
            .unwrap_or(false)
        },
    )
    .await;
    assert!(replaced, "the panel swap never replaced the row");
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.1
async fn check_llm_message_row_toggles_from_the_keyboard(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_llm_messages_tab(&page).await;

    // Tab from the LLM tab button: the first focusable in the panel is the
    // row header, so this proves the header is keyboard-reachable, not just
    // focusable.
    page.locator(r#"[data-tab="llm-messages"]"#)
        .await
        .press("Tab", None)
        .await
        .unwrap();
    let header = page.locator(".llm-message-header").await;
    assert!(
            wait_for_condition_async(
                Duration::from_millis(500),
                Duration::from_millis(20),
                || async {
                    page.evaluate::<(), bool>(
                        r#"() => document.activeElement === document.querySelector('.llm-message-header')"#,
                        None,
                    )
                    .await
                    .unwrap_or(false)
                },
            )
            .await,
            "Tab must reach the row header"
        );
    header.press("Enter", None).await.unwrap();

    assert!(
        wait_for_row_state(&page, true, "true").await,
        "Enter must expand the row and expose the expanded state"
    );
    assert!(
        wait_for_body(&page, true).await,
        "the expanded row must show its body"
    );

    header.press(" ", None).await.unwrap();

    assert!(
        wait_for_row_state(&page, false, "false").await,
        "Space must collapse the row and expose the collapsed state"
    );
    assert!(
        wait_for_body(&page, false).await,
        "the collapsed row must hide its body"
    );
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.2
async fn check_expanded_llm_row_survives_panel_refresh(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_llm_messages_tab(&page).await;

    let header = page.locator(".llm-message-header").await;
    header.focus().await.unwrap();
    header.press("Enter", None).await.unwrap();
    assert!(
        wait_for_row_state(&page, true, "true").await,
        "Enter must expand the row before the refresh"
    );

    stash_header_node(&page).await;
    // Expanding pauses the panel's 4s poll, so drive the same innerHTML
    // swap the poll would make: the panel's own after-settle hook must
    // restore the expanded state onto the fresh row.
    page.evaluate::<(), ()>(
            r#"() => { htmx.ajax("GET", "/fragment/llm-messages", { target: ".llm-messages-panel", swap: "innerHTML" }); }"#,
            None,
        )
        .await
        .unwrap();
    wait_for_header_replacement(&page).await;

    assert!(
        wait_for_row_state(&page, true, "true").await,
        "the refreshed panel must restore the expanded row"
    );
    assert!(
        wait_for_body(&page, true).await,
        "the restored row must show its body"
    );
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.3
async fn check_focused_llm_row_keeps_focus_across_panel_refresh(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_llm_messages_tab(&page).await;

    let header = page.locator(".llm-message-header").await;
    header.focus().await.unwrap();

    stash_header_node(&page).await;
    page.evaluate::<(), ()>(
            r#"() => { htmx.ajax("GET", "/fragment/llm-messages", { target: ".llm-messages-panel", swap: "innerHTML" }); }"#,
            None,
        )
        .await
        .unwrap();
    wait_for_header_replacement(&page).await;

    assert!(
            wait_for_condition_async(
                Duration::from_millis(1000),
                Duration::from_millis(20),
                || async {
                    page.evaluate::<(), bool>(
                        r#"() => document.activeElement === document.querySelector('.llm-message-header')"#,
                        None,
                    )
                    .await
                    .unwrap_or(false)
                },
            )
            .await,
            "the panel refresh must keep focus on the row header"
        );
}

#[tokio::test]
async fn run_llm_messages_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_llm_message_row_toggles_from_the_keyboard,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_expanded_llm_row_survives_panel_refresh,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_focused_llm_row_keeps_focus_across_panel_refresh,
        )
        .await;
    runner.finish().await;
}
