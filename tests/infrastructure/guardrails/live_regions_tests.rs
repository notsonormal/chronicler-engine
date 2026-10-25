//! Tests for the live-region guardrail.

use crate::live_regions::check_live_regions_in_shell;

const GOOD_REGION: &str = r#"<div id="story-log" data-live="story busy worlds" hx-get="/fragment/story-log" hx-trigger="load, refresh" hx-swap="morph:innerHTML"></div>"#;

#[test]
fn test_check_live_regions_in_shell_accepts_a_region_with_the_load_path() {
    assert!(check_live_regions_in_shell("assets/index.html", GOOD_REGION).is_empty());
}

#[test]
fn test_check_live_regions_in_shell_accepts_a_region_that_spans_lines() {
    let content = "<div\n  data-live=\"games\"\n  hx-get=\"/fragment/header\"\n  hx-trigger=\"load, refresh\"\n  hx-swap=\"morph:innerHTML\"\n></div>";
    assert!(check_live_regions_in_shell("assets/index.html", content).is_empty());
}

#[test]
fn test_check_live_regions_in_shell_rejects_an_every_trigger() {
    let content = GOOD_REGION.replace("load, refresh", "load, every 2s, refresh");
    let violations = check_live_regions_in_shell("assets/index.html", &content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("`every` trigger"));
}

#[test]
fn test_check_live_regions_in_shell_rejects_a_missing_refresh_trigger() {
    let content = GOOD_REGION.replace("load, refresh", "load");
    let violations = check_live_regions_in_shell("assets/index.html", &content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("hx-trigger"));
}

#[test]
fn test_check_live_regions_in_shell_rejects_a_swap_that_is_not_morph() {
    let content = GOOD_REGION.replace("morph:innerHTML", "innerHTML");
    let violations = check_live_regions_in_shell("assets/index.html", &content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("morph:innerHTML"));
}

#[test]
fn test_check_live_regions_in_shell_rejects_a_region_with_no_swap() {
    let content = GOOD_REGION.replace(" hx-swap=\"morph:innerHTML\"", "");
    assert_eq!(
        check_live_regions_in_shell("assets/index.html", &content).len(),
        1
    );
}

#[test]
fn test_check_live_regions_in_shell_rejects_an_unknown_kind() {
    let content = GOOD_REGION.replace("worlds", "world");
    let violations = check_live_regions_in_shell("assets/index.html", &content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("`world`"));
}

#[test]
fn test_check_live_regions_in_shell_rejects_an_empty_kind_list() {
    let content = GOOD_REGION.replace("story busy worlds", "");
    let violations = check_live_regions_in_shell("assets/index.html", &content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("no kind"));
}

#[test]
fn test_check_live_regions_in_shell_ignores_an_element_without_data_live() {
    let content = format!(
        "{GOOD_REGION}<div class=\"games-panel\" hx-get=\"/fragment/games\" hx-trigger=\"load, every 5s\"></div>"
    );
    assert!(check_live_regions_in_shell("assets/index.html", &content).is_empty());
}

#[test]
fn test_check_live_regions_in_shell_fails_a_shell_with_no_live_region() {
    let violations = check_live_regions_in_shell("assets/index.html", "<div id=\"x\"></div>");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("guards nothing"));
}
