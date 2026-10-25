//! Tests for the live-region guardrail.

use crate::live_regions::{
    check_handler_headers, check_live_regions_in_shell, check_no_out_of_band_swaps,
    check_scripts_use_action_fetch, check_scripts_write_no_region, check_template_targets,
    region_names,
};

const GOOD_REGION: &str = r#"<div id="story-log" data-live="story busy worlds" hx-get="/fragment/story-log" hx-trigger="refresh" hx-swap="morph:innerHTML"></div>"#;

#[test]
fn test_check_live_regions_in_shell_accepts_a_region_with_the_load_path() {
    assert!(check_live_regions_in_shell("assets/index.html", GOOD_REGION).is_empty());
}

#[test]
fn test_check_live_regions_in_shell_accepts_a_region_that_spans_lines() {
    let content = "<div\n  data-live=\"games\"\n  hx-get=\"/fragment/header\"\n  hx-trigger=\"refresh\"\n  hx-swap=\"morph:innerHTML\"\n></div>";
    assert!(check_live_regions_in_shell("assets/index.html", content).is_empty());
}

#[test]
fn test_check_live_regions_in_shell_rejects_an_every_trigger() {
    let content = GOOD_REGION.replace("\"refresh\"", "\"every 2s, refresh\"");
    let violations = check_live_regions_in_shell("assets/index.html", &content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("`every` trigger"));
}

#[test]
fn test_check_live_regions_in_shell_rejects_a_missing_refresh_trigger() {
    let content = GOOD_REGION.replace("\"refresh\"", "\"load\"");
    let violations = check_live_regions_in_shell("assets/index.html", &content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("hx-trigger"));
}

#[test]
fn test_check_live_regions_in_shell_rejects_a_load_trigger_beside_refresh() {
    let content = GOOD_REGION.replace("\"refresh\"", "\"load, refresh\"");
    let violations = check_live_regions_in_shell("assets/index.html", &content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("hx-trigger=\"refresh\""));
}

#[test]
fn test_check_live_regions_in_shell_rejects_a_region_with_no_trigger() {
    let content = GOOD_REGION.replace(" hx-trigger=\"refresh\"", "");
    assert_eq!(
        check_live_regions_in_shell("assets/index.html", &content).len(),
        1
    );
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
fn test_check_live_regions_in_shell_rejects_an_hx_get_element_without_data_live() {
    let content = format!(
        "{GOOD_REGION}<div class=\"games-panel\" hx-get=\"/fragment/games\" hx-trigger=\"load\"></div>"
    );
    let violations = check_live_regions_in_shell("assets/index.html", &content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("no `data-live`"));
}

#[test]
fn test_check_live_regions_in_shell_ignores_an_element_that_loads_nothing() {
    let content = format!(
        "{GOOD_REGION}<form id=\"command-form\" hx-post=\"/action/check\" hx-target=\"#action-preview\"></form>"
    );
    assert!(check_live_regions_in_shell("assets/index.html", &content).is_empty());
}

#[test]
fn test_check_live_regions_in_shell_fails_a_shell_with_no_live_region() {
    let violations = check_live_regions_in_shell("assets/index.html", "<div id=\"x\"></div>");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("guards nothing"));
}

const SHELL: &str = r#"
<div class="header" id="header" data-live="games" hx-get="/fragment/header" hx-trigger="refresh" hx-swap="morph:innerHTML"></div>
<div class="visual-sidebar" data-live="story worlds" hx-get="/fragment/visual-sidebar" hx-trigger="refresh" hx-swap="morph:innerHTML"></div>
<div class="games-panel" data-live="games worlds presets" hx-get="/fragment/games" hx-trigger="refresh" hx-swap="morph:innerHTML"></div>
<div id="failure-banner-degraded" data-live="settings llm" hx-get="/fragment/failure-banner" hx-trigger="refresh" hx-swap="morph:innerHTML"></div>
<div id="action-preview" class="action-preview"></div>
"#;

fn regions() -> Vec<String> {
    region_names(SHELL)
}

#[test]
fn test_region_names_take_the_id_else_the_first_class_of_each_region() {
    let names = regions();
    for expected in [
        "header",
        "visual-sidebar",
        "games-panel",
        "failure-banner-degraded",
    ] {
        assert!(
            names.iter().any(|name| name == expected),
            "{expected} in {names:?}"
        );
    }
    assert!(
        !names.iter().any(|name| name == "action-preview"),
        "an element that loads nothing is not a region: {names:?}"
    );
}

#[test]
fn test_scripts_write_no_region_rejects_a_write_through_a_selector() {
    let script = r#"document.getElementById("header").innerHTML = html;"#;
    let violations = check_scripts_write_no_region("assets/dashboard-x.js", script, &regions());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("region element"));
}

#[test]
fn test_scripts_write_no_region_rejects_a_write_through_a_declared_identifier() {
    let script = "const area = document.querySelector(\"#games-tab > .games-panel\");\nfunction paint() {\n  area.innerHTML = '<p>x</p>';\n}";
    let violations = check_scripts_write_no_region("assets/dashboard-x.js", script, &regions());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].line, 3);
}

#[test]
fn test_scripts_write_no_region_rejects_an_outer_html_and_an_insert() {
    let script = "document.querySelector('.visual-sidebar').outerHTML = a;\ndocument.getElementById(\"header\").insertAdjacentHTML(\"beforeend\", b);";
    assert_eq!(
        check_scripts_write_no_region("assets/dashboard-x.js", script, &regions()).len(),
        2
    );
}

#[test]
fn test_scripts_write_no_region_rejects_an_htmx_ajax_into_a_region() {
    let script =
        r##"htmx.ajax("GET", "/fragment/story-log", { target: "#header", swap: "innerHTML" });"##;
    let violations = check_scripts_write_no_region("assets/dashboard-x.js", script, &regions());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("htmx.ajax"));
}

#[test]
fn test_scripts_write_no_region_accepts_a_write_to_an_element_that_is_not_a_region() {
    let script = "const preview = document.getElementById(\"action-preview\");\npreview.innerHTML = \"\";\nconst own = document.getElementById(\"action-status\");\nown.innerHTML = x;\nconst shown = degraded.innerHTML.trim() !== \"\";";
    assert!(check_scripts_write_no_region("assets/dashboard-x.js", script, &regions()).is_empty());
}

#[test]
fn test_scripts_write_no_region_accepts_a_comparison_and_a_read() {
    let script = "const region = document.getElementById(\"header\");\nif (region.innerHTML === \"\") {}\nconst html = region.innerHTML;";
    assert!(check_scripts_write_no_region("assets/dashboard-x.js", script, &regions()).is_empty());
}

#[test]
fn test_scripts_use_action_fetch_rejects_a_raw_fetch() {
    let script = "function save() {\n  fetch(\"/history/1\", { method: \"POST\" });\n}";
    let violations = check_scripts_use_action_fetch("assets/dashboard-x.js", script);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].line, 2);
}

#[test]
fn test_scripts_use_action_fetch_accepts_the_wrapper_and_a_comment() {
    let script = "// A raw fetch(url) answers nothing to the page.\nactionFetch(\"/history/1\", { method: \"POST\" });\nwindow.fetchCount += 1;";
    assert!(check_scripts_use_action_fetch("assets/dashboard-x.js", script).is_empty());
}

#[test]
fn test_handlers_retarget_no_region_rejects_a_retarget_into_a_region() {
    let source =
        r##"response.headers_mut().insert("HX-Retarget", HeaderValue::from_static("#header"));"##;
    let violations = check_handler_headers("src/x.rs", source, &regions());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("HX-Retarget"));
}

#[test]
fn test_handlers_retarget_no_region_accepts_a_retarget_to_another_element() {
    let source = r##"response.headers_mut().insert("HX-Retarget", HeaderValue::from_static("#action-preview"));"##;
    assert!(check_handler_headers("src/x.rs", source, &regions()).is_empty());
}

#[test]
fn test_handler_headers_rejects_a_refresh_header() {
    let source = r#"Response::builder().header("HX-Refresh", "true").body(Body::empty())"#;
    let violations = check_handler_headers("src/x.rs", source, &regions());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("HX-Refresh"));
}

#[test]
fn test_handler_headers_accepts_a_comment_free_handler_without_refresh() {
    let source = r#"ok("")"#;
    assert!(check_handler_headers("src/x.rs", source, &regions()).is_empty());
}

#[test]
fn test_templates_target_no_region_rejects_an_hx_target_that_names_a_region() {
    for target in [
        "#header",
        ".games-panel",
        "closest .games-panel",
        "#failure-banner-degraded",
    ] {
        let source = format!(
            r##"<button hx-post="/x" hx-target="{target}" hx-swap="innerHTML">Go</button>"##
        );
        let violations = check_template_targets("src/x.rs", &source, &regions());
        assert_eq!(violations.len(), 1, "{target}");
        assert!(violations[0].message.contains("names a region element"));
    }
}

#[test]
fn test_templates_target_no_region_reads_a_target_in_an_escaped_rust_string() {
    let source =
        r##"format!("<select hx-post=\"/x\" hx-target=\"#header\" hx-swap=\"innerHTML\">")"##;
    assert_eq!(
        check_template_targets("src/x.rs", source, &regions()).len(),
        1
    );
}

#[test]
fn test_templates_target_no_region_rejects_an_outer_html_swap_of_a_region() {
    let own = r#"<div class="games-panel" hx-get="/fragment/games" hx-swap="outerHTML"></div>"#;
    let violations = check_template_targets("src/x.rs", own, &regions());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("outerHTML"));

    let by_target = r##"<button hx-post="/x" hx-target="#header" hx-swap="outerHTML">Go</button>"##;
    assert_eq!(
        check_template_targets("src/x.rs", by_target, &regions()).len(),
        1,
        "one violation: the target names a region"
    );
}

#[test]
fn test_templates_target_no_region_accepts_targets_that_are_no_region() {
    let source = r##"<button hx-get="/a" hx-target="#world-editor" hx-swap="innerHTML">A</button>
<button hx-get="/b" hx-target="closest .preset-card" hx-swap="outerHTML">B</button>
<button hx-post="/c" hx-target="next .connection-test-slot" hx-swap="innerHTML">C</button>
<form hx-post="/d" hx-swap="none"></form>
<div class="header-bar"></div>
<div id="preset-card-x" hx-swap="outerHTML"></div>"##;
    assert!(check_template_targets("src/x.rs", source, &regions()).is_empty());
}

#[test]
fn test_templates_have_no_out_of_band_swaps_rejects_the_attribute() {
    let source = "let a = 1;\nformat!(\"<span id=\\\"x\\\" hx-swap-oob=\\\"true\\\">\")\n";
    let violations = check_no_out_of_band_swaps("src/x.rs", source);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].line, 2);
}

#[test]
fn test_templates_have_no_out_of_band_swaps_accepts_a_comment_and_clean_markup() {
    let source = "// The header no longer answers hx-swap-oob swaps.\n<div hx-swap=\"none\"></div>";
    assert!(check_no_out_of_band_swaps("src/x.rs", source).is_empty());
}
