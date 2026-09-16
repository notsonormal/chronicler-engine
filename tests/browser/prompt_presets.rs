//! Browser prompt-presets tests: allowed-modes editor roundtrip. Tagged against `docs/specs/browser_prompt_presets.md`.

use std::time::Duration;

use playwright_rs::expect;

use super::*;

// [docs/specs/browser_prompt_presets.md] SCENARIO: 28.1
#[tokio::test]
async fn test_preset_editor_mode_flags_roundtrip() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            page.locator(r#".tab[data-tab="prompt-presets"]"#)
                .await
                .click(None)
                .await
                .unwrap();
            // All seeded presets are defaults (View-only), so the duplicated
            // copy is the only card with an Edit button.
            wait_for_element_children(
                &page,
                r#"#prompt-presets-tab button[hx-post$="/duplicate"]"#,
                1,
            )
            .await;
            page.locator(r#"#prompt-presets-tab button[hx-post$="/duplicate"]"#)
                .await
                .first()
                .click(None)
                .await
                .unwrap();
            wait_for_element_children(&page, r#"#prompt-presets-tab button[hx-get$="/edit"]"#, 1)
                .await;
            page.locator(r#"#prompt-presets-tab button[hx-get$="/edit"]"#)
                .await
                .first()
                .click(None)
                .await
                .unwrap();
            wait_until_visible(&page, ".preset-card.edit-form", Duration::from_millis(1000)).await;

            let name = page
                .locator(r#".preset-card.edit-form input[name="name"]"#)
                .await
                .input_value(None)
                .await
                .unwrap_or_default();
            assert!(!name.is_empty(), "edit form renders the copy's name");

            let novel_box = page
                .locator(r#".preset-card.edit-form input[name="allowed_mode_novel"]"#)
                .await;
            assert!(
                novel_box.is_checked().await.unwrap_or(false),
                "copy of the Novel-only default checks the novel flag"
            );
            let if_box = page
                .locator(r#".preset-card.edit-form input[name="allowed_mode_if"]"#)
                .await;
            assert!(
                !if_box.is_checked().await.unwrap_or(true),
                "copy of the Novel-only default leaves the IF flag unchecked"
            );

            if_box.check(None).await.unwrap();
            page.locator(r#".preset-card.edit-form button[type="submit"]"#)
                .await
                .click(None)
                .await
                .unwrap();

            // The update swaps the card in place; the edit form disappears.
            let edit_gone = page.locator(".preset-card.edit-form").await;
            if let Err(e) = expect(edit_gone)
                .with_timeout(std::time::Duration::from_secs(5))
                .to_be_hidden()
                .await
            {
                panic!("saving should swap the edit form away: {e}");
            }

            let saved = page
                .evaluate::<String, bool>(
                    r#"(name) => {
                    const cards = [...document.querySelectorAll('.preset-card')];
                    const card = cards.find((c) => {
                        const title = c.querySelector('.card-title');
                        return title && title.textContent.includes(name);
                    });
                    if (!card) return false;
                    return !!card.querySelector(
                        'button[hx-post$="activate?mode=interactive_fiction"]'
                    );
                }"#,
                    Some(&name),
                )
                .await
                .unwrap_or(false);
            assert!(
                saved,
                "saved card should offer Set Active (IF) after enabling the IF flag"
            );
        },
    )
    .await;
}
