//! Stub-browser tests for the Settings panel: the client-side sub-tabs and the shared connection form page. Tagged against `docs/specs/browser_settings.md`.

use std::time::Duration;

use super::*;

async fn open_settings(page: &playwright_rs::Page) {
    page.locator(r#"[data-tab="settings"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(page, "#subtab-connections", Duration::from_secs(5)).await;
}

// [docs/specs/browser_settings.md] SCENARIO: 40.1
#[tokio::test]
async fn test_settings_sub_tabs_keep_their_place_across_a_top_level_tab_change() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        open_settings(&page).await;

        page.locator("#subtab-text-check")
            .await
            .click(None)
            .await
            .unwrap();

        let after_switch: (String, bool, bool) = page
            .evaluate::<(), (String, bool, bool)>(
                r#"(() => {
                    const tab = document.getElementById('subtab-text-check');
                    const textPanel = document.getElementById('settings-text-check');
                    const connectionsPanel = document.getElementById('settings-connections');
                    return [
                        tab.getAttribute('aria-selected'),
                        textPanel.classList.contains('active'),
                        connectionsPanel.classList.contains('active'),
                    ];
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert_eq!(after_switch.0, "true", "the sub-tab must report itself selected");
        assert!(after_switch.1, "the Text Check panel must be visible");
        assert!(
            !after_switch.2,
            "the Connections panel must not be the visible one"
        );

        page.locator(r#"[data-tab="game"]"#)
            .await
            .click(None)
            .await
            .unwrap();
        page.locator(r#"[data-tab="settings"]"#)
            .await
            .click(None)
            .await
            .unwrap();

        let kept: bool = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const tab = document.getElementById('subtab-text-check');
                    return tab.getAttribute('aria-selected') === 'true'
                        && document.getElementById('settings-text-check').classList.contains('active')
                        && !document.getElementById('settings-connections').classList.contains('active');
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            kept,
            "the selected sub-tab must survive a top-level tab change"
        );
    })
    .await;
}

// [docs/specs/browser_settings.md] SCENARIO: 40.2
#[tokio::test]
async fn test_add_and_edit_open_the_shared_connection_form_page() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        open_settings(&page).await;

        page.locator(".connection-add button")
            .await
            .click(None)
            .await
            .unwrap();
        wait_until_visible(&page, ".connection-form-page", Duration::from_secs(5)).await;

        let back_link = page
            .locator(".connection-form-page .back-link")
            .await
            .inner_text()
            .await
            .unwrap_or_default();
        assert!(
            back_link.contains("Connections"),
            "the form page must offer a back link to Connections, got {back_link:?}"
        );

        page.locator(".connection-form-page .back-link")
            .await
            .click(None)
            .await
            .unwrap();
        wait_until_visible(&page, ".connection-list", Duration::from_secs(5)).await;

        let edit_selector = r#".connection-row:has(.connection-name:text-is("openrouter-gpt-4o-mini")) button:has-text('Edit')"#;
        page.locator(edit_selector)
            .await
            .click(None)
            .await
            .unwrap();
        wait_until_visible(&page, ".connection-form-page", Duration::from_secs(5)).await;

        let heading = page
            .locator(".connection-form-page h2")
            .await
            .inner_text()
            .await
            .unwrap_or_default();
        assert!(
            heading.contains("openrouter-gpt-4o-mini"),
            "the Edit form must name the connection, got {heading:?}"
        );

        page.locator(".connection-form-page .back-link")
            .await
            .click(None)
            .await
            .unwrap();
        wait_until_visible(&page, ".connection-list", Duration::from_secs(5)).await;
    })
    .await;
}
