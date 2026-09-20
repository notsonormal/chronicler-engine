//! Browser prompt-presets tests: the duplicate → edit → save click chain wiring guard. Tagged against `docs/specs/browser_prompt_presets.md`.

use std::time::Duration;

use super::*;

/// The browser-only question: do the Duplicate click's panel swap, the Edit
/// button's in-place card swap, and the Save form posting the ticked mode flags
/// to the card's own route all work through real clicks. The request chain is
/// covered at HTTP (`tests/http/prompt_presets.rs` SCENARIO 21.27).
///
/// Seeding goes through the engine's own HTTP API (`seed_system_preset`): the
/// layout seeds no non-default preset, and a default card renders View instead
/// of Edit with no `Set Active` buttons at all.
///
/// **Do not stress-loop this test.** The preset id generator is
/// millisecond-based, so a create and a duplicate in the same millisecond
/// collide. One seeded create and one Duplicate click makes that window small
/// but not zero.
// [docs/specs/browser_prompt_presets.md] SCENARIO: 28.1
#[tokio::test]
async fn test_preset_duplicate_edit_save_click_chain() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, port| async move {
            let seed_name = "Wiring Guard Preset";
            seed_system_preset(port, seed_name, "Seed instructions for the preset guard.").await;

            // Reload so the `load`-triggered panel re-renders including the new
            // card; `open_prompt_presets_tab` awaits that reload's settle.
            page.reload(None).await.expect("reload after seeding a preset");
            open_prompt_presets_tab(&page).await;

            let seed_card = preset_card_selector(seed_name);
            wait_until_visible(&page, &seed_card, Duration::from_millis(5000)).await;

            // The whole panel is replaced (hx-target `.prompt-presets-panel`,
            // `outerHTML`).
            let duplicate_selector = format!("{seed_card} button:has-text('Duplicate')");
            click_and_settle(&page, &duplicate_selector, ".prompt-presets-panel").await;

            // The copy is named "<source> (Copy)" and is non-default, so it
            // renders Edit.
            let copy_name = format!("{seed_name} (Copy)");
            let copy_card = preset_card_selector(&copy_name);
            wait_until_visible(&page, &copy_card, Duration::from_millis(5000)).await;

            // Precondition for the post-save assertion: the copy inherits the
            // seed's single mode, so a bug that rendered both buttons anyway
            // would pass the check for the wrong reason.
            let copy_html_before: String = page
                .locator(&copy_card)
                .await
                .inner_html()
                .await
                .unwrap_or_default();
            assert!(
                copy_html_before.contains("Set Active (Novel)")
                    && !copy_html_before.contains("Set Active (IF)"),
                "the duplicated copy must start with only the Novel mode enabled, got: {copy_html_before}"
            );

            // The replacement card is titled "Edit <name>", so `copy_card` no
            // longer matches it — scope the form fields to
            // `.preset-card.edit-form`, of which only one exists at a time.
            open_preset_editor(&page, &copy_card).await;

            // Tick IF: the post-reload assertion proves the save *added* a mode
            // rather than re-rendering one the copy already carried.
            let edit_form = ".preset-card.edit-form";
            page.locator(&format!(r#"{edit_form} input[name="allowed_mode_if"]"#))
                .await
                .set_checked(true, None)
                .await
                .expect("tick the IF mode flag");

            let save_selector = format!("{edit_form} button:has-text('Save')");
            // The Save swap replaces the card in place (`hx-target
            // closest .preset-card`) — await the card, not the panel.
            click_and_settle(&page, &save_selector, ".preset-card").await;

            // Reload before asserting: a fragment edit that only re-rendered
            // client-side would pass an assertion made in the same document.
            page.reload(None).await.expect("reload after saving the preset");
            open_prompt_presets_tab(&page).await;

            wait_until_visible(&page, &copy_card, Duration::from_millis(5000)).await;
            // `inner_html()` runs against the resolved element, so the Playwright
            // `:has()`/`:text-is()` selector is honoured (native `querySelector`
            // cannot parse those pseudo-classes).
            let saved_card_html: String = page
                .locator(&copy_card)
                .await
                .inner_html()
                .await
                .unwrap_or_default();
            assert!(
                saved_card_html.contains("Set Active (Novel)")
                    && saved_card_html.contains("Set Active (IF)"),
                "the saved copy must offer both activation buttons after a reload \
                 (both mode flags persisted), got: {saved_card_html}"
            );
        },
    )
    .await;
}
