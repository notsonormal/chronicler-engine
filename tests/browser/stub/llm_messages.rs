//! Stub-browser tests for the LLM Messages panel: the shipped client's keyboard path over a canned row. Tagged against `docs/specs/browser_llm_messages.md`.

// Keyboard operation is the shipped shell's: Enter and Space on the native
// button fire the same click handler a mouse uses. The stub server only serves
// the canned card, so faking the engine changes nothing.
//
// The panel loads when the `llm` number moves. The page's timer stops in the
// checks that move it, so each load is one the check asked for. Every assertion
// polls the state it wants rather than sampling once, and reads the live DOM
// rather than a Playwright element handle.

use std::time::Duration;

use super::support::{ask_page_once, stop_page_timer, StubRunner};
use super::*;

/// Open the LLM Messages tab. The section loads its region when it opens, so the
/// wait is for the card the load renders.
async fn open_llm_messages_tab(page: &playwright_rs::Page) {
    page.locator(r#"[data-tab="llm-messages"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(page, ".llm-message-card", Duration::from_millis(2000)).await;
}

async fn llm_header_state(page: &playwright_rs::Page) -> (bool, String) {
    page.evaluate::<(), (bool, String)>(
        r#"(() => {
            const card = document.querySelector('.llm-message-card');
            const header = card ? card.querySelector('.llm-message-header') : null;
            const body = card ? card.querySelector('.llm-message-body') : null;
            return [
                !!(body && !body.hidden),
                header ? (header.getAttribute('aria-expanded') || '') : '',
            ];
        })()"#,
        None,
    )
    .await
    .unwrap()
}

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

async fn stash_nodes(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"() => {
            window.__llmHeaderBefore = document.querySelector('.llm-message-header');
            window.__llmDetailsBefore = Array.from(
                document.querySelectorAll('.llm-message-prompt-details'),
            );
        }"#,
        None,
    )
    .await
    .unwrap();
}

/// The number of `llm` moves and the page asks: the panel loads once.
async fn load_the_panel(page: &playwright_rs::Page, stub: &StubServer) {
    let changes = stub.changes_handle();
    let before = changes.loads("/fragment/llm-messages");
    changes.bump("llm");
    ask_page_once(page).await;
    assert_eq!(
        changes.loads("/fragment/llm-messages"),
        before + 1,
        "a moved `llm` number must load the panel once"
    );
}

async fn header_is_the_same_node(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        r#"() => document.querySelector('.llm-message-header') === window.__llmHeaderBefore"#,
        None,
    )
    .await
    .unwrap()
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
async fn check_expanded_llm_row_survives_panel_load(page: playwright_rs::Page, stub: StubServer) {
    open_llm_messages_tab(&page).await;
    stop_page_timer(&page).await;

    let header = page.locator(".llm-message-header").await;
    header.focus().await.unwrap();
    header.press("Enter", None).await.unwrap();
    assert!(
        wait_for_row_state(&page, true, "true").await,
        "Enter must expand the row before the load"
    );

    stash_nodes(&page).await;
    load_the_panel(&page, &stub).await;

    assert!(
        header_is_the_same_node(&page).await,
        "the load must keep the row's header node"
    );
    assert!(
        wait_for_row_state(&page, true, "true").await,
        "the loaded panel must keep the row expanded"
    );
    assert!(
        wait_for_body(&page, true).await,
        "the kept row must show its body"
    );
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.3
async fn check_focused_llm_row_keeps_focus_across_panel_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    open_llm_messages_tab(&page).await;
    stop_page_timer(&page).await;

    let header = page.locator(".llm-message-header").await;
    header.focus().await.unwrap();

    stash_nodes(&page).await;
    load_the_panel(&page, &stub).await;

    assert!(
        header_is_the_same_node(&page).await,
        "the load must keep the row's header node"
    );
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
            "the panel load must keep focus on the row header"
        );
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.4
async fn check_a_new_llm_message_appears_after_any_number_of_expands_and_collapses(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    open_llm_messages_tab(&page).await;
    let header = page.locator(".llm-message-header").await;
    for _ in 0..2 {
        header.press("Enter", None).await.unwrap();
        assert!(wait_for_row_state(&page, true, "true").await, "expanded");
        header.press("Enter", None).await.unwrap();
        assert!(wait_for_row_state(&page, false, "false").await, "collapsed");
    }

    // The page's own timer loads the panel, with nothing paused by the expands.
    stub.llm_handle().record_message();
    let appeared = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(50),
        || async {
            page.query_selector_all(".llm-message-card")
                .await
                .map(|cards| cards.len() == 2)
                .unwrap_or(false)
        },
    )
    .await;
    assert!(
        appeared,
        "a new LLM message must appear in the panel with no reload of the page"
    );
    assert!(
        wait_for_row_state(&page, false, "false").await,
        "the card the client collapsed must still be collapsed"
    );
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.5
async fn check_a_prompt_disclosure_the_client_changed_survives_a_panel_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    open_llm_messages_tab(&page).await;
    stop_page_timer(&page).await;

    let header = page.locator(".llm-message-header").await;
    header.press("Enter", None).await.unwrap();
    assert!(wait_for_row_state(&page, true, "true").await, "expanded");

    // The engine renders the Response disclosure open and the System one closed.
    page.evaluate::<(), ()>(
        r#"() => {
            const [system, user, response] = document.querySelectorAll('.llm-message-prompt-details');
            response.open = false;
            system.open = true;
        }"#,
        None,
    )
    .await
    .unwrap();
    stash_nodes(&page).await;
    load_the_panel(&page, &stub).await;

    let state = page
        .evaluate::<(), String>(
            r#"() => {
                const now = Array.from(document.querySelectorAll('.llm-message-prompt-details'));
                const before = window.__llmDetailsBefore;
                if (now.length !== before.length || now.some((node, i) => node !== before[i])) {
                    return 'a disclosure is a new element';
                }
                const [system, user, response] = now;
                if (!system.open) return 'the System disclosure closed';
                if (response.open) return 'the Response disclosure reopened';
                return '';
            }"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        state, "",
        "the disclosures must stay as the client left them"
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
            check_expanded_llm_row_survives_panel_load,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_focused_llm_row_keeps_focus_across_panel_load,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_new_llm_message_appears_after_any_number_of_expands_and_collapses,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_prompt_disclosure_the_client_changed_survives_a_panel_load,
        )
        .await;
    runner.finish().await;
}
