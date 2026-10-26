//! Tests for the screen guardrail.

use crate::screen::{
    check_dialog_fragments_carry_the_title, check_draft_control_exceptions_are_used,
    check_draft_controls_only_in_dialog_forms, check_handler_headers, check_no_out_of_band_swaps,
    check_part_table_matches_shell, check_part_write_exceptions_are_used, check_parts_in_shell,
    check_scripts_listen_no_part_swaps, check_scripts_use_action_fetch,
    check_scripts_write_no_part, check_shell_parts_match_the_server,
    check_stub_answers_the_parts_of_the_server, check_template_targets, part_element_names,
    DraftControlException, ListedPart, PartWriteException,
};

const GOOD_PART: &str = r#"<div class="tab-content" id="game-tab"><div id="story-log" data-part="story-log"></div></div>"#;

#[test]
fn test_check_parts_in_shell_accepts_a_part_that_only_the_screen_load_writes() {
    assert!(check_parts_in_shell("assets/index.html", GOOD_PART).is_empty());
}

#[test]
fn test_check_parts_in_shell_accepts_a_part_that_spans_lines() {
    let content =
        "<div class=\"tab-content\"><div\n  data-part=\"header\"\n  id=\"header\"\n></div></div>";
    assert!(check_parts_in_shell("assets/index.html", content).is_empty());
}

#[test]
fn test_check_parts_in_shell_rejects_each_attribute_that_loads_a_part_on_its_own() {
    for attribute in [
        "hx-get=\"/fragment/story-log\"",
        "hx-trigger=\"refresh\"",
        "hx-swap=\"morph:innerHTML\"",
    ] {
        let content = GOOD_PART.replace(
            "data-part=\"story-log\"",
            &format!("data-part=\"story-log\" {attribute}"),
        );
        let violations = check_parts_in_shell("assets/index.html", &content);
        assert_eq!(violations.len(), 1, "{attribute}: {violations:?}");
        assert!(
            violations[0]
                .message
                .contains("Only the screen load writes a part")
        );
    }
}

#[test]
fn test_check_parts_in_shell_rejects_the_attributes_of_the_table_of_kinds() {
    for attribute in ["data-live=\"story\"", "data-live-state=\"current\""] {
        let content = GOOD_PART.replace(
            "data-part=\"story-log\"",
            &format!("data-part=\"story-log\" {attribute}"),
        );
        let violations = check_parts_in_shell("assets/index.html", &content);
        assert_eq!(violations.len(), 1, "{attribute}: {violations:?}");
        assert!(violations[0].message.contains("no table of kinds"));
    }
}

#[test]
fn test_check_parts_in_shell_rejects_a_section_that_holds_no_part() {
    let content =
        format!("{GOOD_PART}<div class=\"tab-content\" id=\"empty-tab\"><p>text</p></div>");
    let violations = check_parts_in_shell("assets/index.html", &content);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("holds no part"));
}

#[test]
fn test_check_parts_in_shell_ignores_an_element_that_loads_nothing() {
    let content = format!(
        "{GOOD_PART}<form id=\"command-form\" hx-post=\"/action/check\" hx-target=\"#action-preview\"></form>"
    );
    assert!(check_parts_in_shell("assets/index.html", &content).is_empty());
}

#[test]
fn test_check_parts_in_shell_fails_a_shell_with_no_part() {
    let violations = check_parts_in_shell("assets/index.html", "<div id=\"x\"></div>");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("guards nothing"));
}

const TWO_PARTS: &str = r#"<div id="story-log" data-part="story-log"></div>
<aside class="visual-sidebar side" data-part="visual-sidebar"></aside>"#;

#[test]
fn test_check_shell_parts_match_the_server_accepts_the_same_names() {
    assert!(
        check_shell_parts_match_the_server(
            "assets/index.html",
            TWO_PARTS,
            &["story-log", "visual-sidebar"]
        )
        .is_empty()
    );
}

#[test]
fn test_check_shell_parts_match_the_server_rejects_a_part_with_no_renderer_and_a_renderer_with_no_part()
 {
    let violations =
        check_shell_parts_match_the_server("assets/index.html", TWO_PARTS, &["story-log", "games"]);
    assert_eq!(violations.len(), 2, "{violations:?}");
    assert!(violations[0].message.contains("`visual-sidebar`"));
    assert!(violations[1].message.contains("`games`"));
}

#[test]
fn test_check_shell_parts_match_the_server_rejects_a_name_that_appears_twice() {
    let twice = format!("{TWO_PARTS}<div id=\"story-log-again\" data-part=\"story-log\"></div>");
    let violations = check_shell_parts_match_the_server(
        "assets/index.html",
        &twice,
        &["story-log", "visual-sidebar"],
    );
    assert!(
        violations
            .iter()
            .all(|v| v.message.contains("more than once")),
        "{violations:?}"
    );
    assert_eq!(violations.len(), 2);
}

const SHELL: &str = r#"
<div class="header" id="header" data-part="header"></div>
<div class="visual-sidebar" data-part="visual-sidebar"></div>
<div class="games-panel" data-part="games"></div>
<div id="failure-banner-degraded" data-part="failure-banner"></div>
<div id="action-preview" class="action-preview"></div>
"#;

fn parts() -> Vec<String> {
    part_element_names(SHELL)
}

#[test]
fn test_part_element_names_take_the_id_else_the_first_class_of_each_part() {
    let names = parts();
    for expected in [
        "header",
        "visual-sidebar",
        "games-panel",
        "failure-banner-degraded",
        "games",
        "failure-banner",
    ] {
        assert!(
            names.iter().any(|name| name == expected),
            "{expected} in {names:?}"
        );
    }
    assert!(
        !names.iter().any(|name| name == "action-preview"),
        "an element that loads nothing is not a part: {names:?}"
    );
}

#[test]
fn test_scripts_write_no_part_rejects_a_write_through_a_selector() {
    let script = r#"document.getElementById("header").innerHTML = html;"#;
    let violations = check_scripts_write_no_part("assets/dashboard-x.js", script, &parts());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("part element"));
}

#[test]
fn test_scripts_write_no_part_rejects_a_write_through_a_declared_identifier() {
    let script = "const area = document.querySelector(\"#games-tab > .games-panel\");\nfunction paint() {\n  area.innerHTML = '<p>x</p>';\n}";
    let violations = check_scripts_write_no_part("assets/dashboard-x.js", script, &parts());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].line, 3);
}

#[test]
fn test_scripts_write_no_part_rejects_an_outer_html_and_an_insert() {
    let script = "document.querySelector('.visual-sidebar').outerHTML = a;\ndocument.getElementById(\"header\").insertAdjacentHTML(\"beforeend\", b);";
    assert_eq!(
        check_scripts_write_no_part("assets/dashboard-x.js", script, &parts()).len(),
        2
    );
}

#[test]
fn test_scripts_write_no_part_rejects_an_htmx_ajax_into_a_part() {
    let script =
        r##"htmx.ajax("GET", "/fragment/story-log", { target: "#header", swap: "innerHTML" });"##;
    let violations = check_scripts_write_no_part("assets/dashboard-x.js", script, &parts());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("htmx.ajax"));
}

#[test]
fn test_scripts_write_no_part_rejects_a_write_through_a_data_part_name() {
    let script = r#"document.getElementById("games").innerHTML = html;"#;
    assert_eq!(
        check_scripts_write_no_part("assets/dashboard-x.js", script, &parts()).len(),
        1
    );
}

#[test]
fn test_scripts_write_no_part_rejects_a_write_through_a_data_part_selector() {
    for selector in [
        r#"document.querySelector('[data-part="games"]')"#,
        r#"document.querySelector("[data-part]")"#,
    ] {
        let script = format!("{selector}.innerHTML = html;");
        assert_eq!(
            check_scripts_write_no_part("assets/dashboard-x.js", &script, &parts()).len(),
            1,
            "{selector}"
        );
    }
}

#[test]
fn test_scripts_write_no_part_rejects_every_content_write_of_a_part() {
    for write in [
        "replaceChildren()",
        "appendChild(node)",
        "append(node)",
        "prepend(node)",
        "insertBefore(node, null)",
        "replaceWith(node)",
        "insertAdjacentHTML('beforeend', html)",
        "textContent = text",
        "innerText = text",
        "innerHTML = html",
        "outerHTML = html",
    ] {
        let script = format!("document.getElementById(\"header\").{write};");
        assert_eq!(
            check_scripts_write_no_part("assets/dashboard-x.js", &script, &parts()).len(),
            1,
            "{write}"
        );
    }
}

#[test]
fn test_scripts_write_no_part_follows_a_selector_constant_to_the_part_that_it_lists() {
    let script = "const CONTAINERS = \"form, .row, [data-part]\";\nfunction put(elt, slot) {\n  const container = elt.closest(CONTAINERS);\n  container.appendChild(slot);\n}";
    let violations = check_scripts_write_no_part("assets/dashboard-x.js", script, &parts());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].line, 4);
}

#[test]
fn test_scripts_write_no_part_accepts_a_selector_constant_that_lists_no_part() {
    let script = "const CONTAINERS = \"form, .row\";\nfunction put(elt, slot) {\n  const container = elt.closest(CONTAINERS);\n  container.appendChild(slot);\n}";
    assert!(check_scripts_write_no_part("assets/dashboard-x.js", script, &parts()).is_empty());
}

#[test]
fn test_part_write_exceptions_flag_a_script_that_writes_no_part() {
    let exceptions = [PartWriteException {
        path: "assets/dashboard-x.js",
        reason: "a reason",
    }];
    let writer = (
        "assets/dashboard-x.js".to_string(),
        r#"document.getElementById("header").replaceChildren();"#.to_string(),
    );
    let quiet = (
        "assets/dashboard-x.js".to_string(),
        "const a = 1;".to_string(),
    );
    assert!(check_part_write_exceptions_are_used(&[writer], &parts(), &exceptions).is_empty());
    assert_eq!(
        check_part_write_exceptions_are_used(&[quiet], &parts(), &exceptions).len(),
        1
    );
}

#[test]
fn test_scripts_write_no_part_accepts_a_write_to_an_element_that_is_not_a_part() {
    let script = "const preview = document.getElementById(\"action-preview\");\npreview.innerHTML = \"\";\nconst own = document.getElementById(\"action-status\");\nown.innerHTML = x;\nconst shown = degraded.innerHTML.trim() !== \"\";";
    assert!(check_scripts_write_no_part("assets/dashboard-x.js", script, &parts()).is_empty());
}

#[test]
fn test_scripts_write_no_part_accepts_a_comparison_and_a_read() {
    let script = "const region = document.getElementById(\"header\");\nif (region.innerHTML === \"\") {}\nconst html = region.innerHTML;";
    assert!(check_scripts_write_no_part("assets/dashboard-x.js", script, &parts()).is_empty());
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
fn test_handlers_retarget_no_part_rejects_a_retarget_into_a_part() {
    let source =
        r##"response.headers_mut().insert("HX-Retarget", HeaderValue::from_static("#header"));"##;
    let violations = check_handler_headers("src/x.rs", source, &parts());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("HX-Retarget"));
}

#[test]
fn test_handlers_retarget_no_part_accepts_a_retarget_to_another_element() {
    let source = r##"response.headers_mut().insert("HX-Retarget", HeaderValue::from_static("#action-preview"));"##;
    assert!(check_handler_headers("src/x.rs", source, &parts()).is_empty());
}

#[test]
fn test_handler_headers_rejects_a_refresh_header() {
    let source = r#"Response::builder().header("HX-Refresh", "true").body(Body::empty())"#;
    let violations = check_handler_headers("src/x.rs", source, &parts());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("HX-Refresh"));
}

#[test]
fn test_handler_headers_accepts_a_comment_free_handler_without_refresh() {
    let source = r#"ok("")"#;
    assert!(check_handler_headers("src/x.rs", source, &parts()).is_empty());
}

#[test]
fn test_templates_target_no_part_rejects_an_hx_target_that_names_a_part() {
    for target in [
        "#header",
        ".games-panel",
        "closest .games-panel",
        "#failure-banner-degraded",
    ] {
        let source = format!(
            r##"<button hx-post="/x" hx-target="{target}" hx-swap="innerHTML">Go</button>"##
        );
        let violations = check_template_targets("src/x.rs", &source, &parts());
        assert_eq!(violations.len(), 1, "{target}");
        assert!(violations[0].message.contains("names a part element"));
    }
}

#[test]
fn test_templates_target_no_part_reads_a_target_in_an_escaped_rust_string() {
    let source =
        r##"format!("<select hx-post=\"/x\" hx-target=\"#header\" hx-swap=\"innerHTML\">")"##;
    assert_eq!(
        check_template_targets("src/x.rs", source, &parts()).len(),
        1
    );
}

#[test]
fn test_templates_target_no_part_rejects_an_outer_html_swap_of_a_part() {
    let own = r#"<div class="games-panel" hx-get="/fragment/games" hx-swap="outerHTML"></div>"#;
    let violations = check_template_targets("src/x.rs", own, &parts());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("outerHTML"));

    let by_target = r##"<button hx-post="/x" hx-target="#header" hx-swap="outerHTML">Go</button>"##;
    assert_eq!(
        check_template_targets("src/x.rs", by_target, &parts()).len(),
        1,
        "one violation: the target names a part"
    );
}

#[test]
fn test_templates_target_no_part_accepts_targets_that_are_no_part() {
    let source = r##"<button hx-get="/a" hx-target="#world-editor" hx-swap="innerHTML">A</button>
<button hx-get="/b" hx-target="closest .preset-card" hx-swap="outerHTML">B</button>
<button hx-post="/c" hx-target="next .connection-test-slot" hx-swap="innerHTML">C</button>
<form hx-post="/d" hx-swap="none"></form>
<div class="header-bar"></div>
<div id="preset-card-x" hx-swap="outerHTML"></div>"##;
    assert!(check_template_targets("src/x.rs", source, &parts()).is_empty());
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

fn row(selector: &str, name: &str) -> ListedPart {
    ListedPart {
        selector: selector.to_string(),
        name: name.to_string(),
    }
}

fn good_table() -> Vec<ListedPart> {
    vec![
        row("#story-log", "story-log"),
        row(".visual-sidebar", "visual-sidebar"),
    ]
}

#[test]
fn test_check_part_table_matches_shell_accepts_a_table_that_lists_every_part() {
    assert!(
        check_part_table_matches_shell("assets/index.html", TWO_PARTS, &good_table()).is_empty()
    );
}

#[test]
fn test_check_part_table_matches_shell_rejects_a_part_the_table_lacks() {
    let mut table = good_table();
    table.pop();
    let violations = check_part_table_matches_shell("assets/index.html", TWO_PARTS, &table);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains(".visual-sidebar"));
}

#[test]
fn test_check_part_table_matches_shell_rejects_a_row_with_no_part() {
    let mut table = good_table();
    table.push(row("#gone", "games"));
    let violations = check_part_table_matches_shell("assets/index.html", TWO_PARTS, &table);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("#gone"));
}

#[test]
fn test_check_part_table_matches_shell_rejects_another_name() {
    let table = vec![
        row("#story-log", "story-log"),
        row(".visual-sidebar", "sidebar"),
    ];
    let violations = check_part_table_matches_shell("assets/index.html", TWO_PARTS, &table);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("names the part `sidebar`"));
}

const PART_WITH_A_TEST_FORM: &str = r##"
#[template(source = r#"<div class="settings-panel">
    <form hx-post="/connections/test" hx-swap="none">
        <input type="text" name="conn_model" value="">
        <button type="submit">Test</button>
    </form>
</div>"#)]
struct SettingsTemplate;
"##;

#[test]
fn test_check_draft_controls_rejects_a_test_form_inside_a_part() {
    let violations = check_draft_controls_only_in_dialog_forms(
        "adapters/driving/http/settings/templates/settings.rs",
        PART_WITH_A_TEST_FORM,
    );
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("data-dialog-form"));
    assert_eq!(violations[0].line, 4);
}

#[test]
fn test_check_draft_controls_accepts_a_form_in_a_dialog_fragment() {
    let source = PART_WITH_A_TEST_FORM.replace(
        r#"<div class="settings-panel">"#,
        r#"<div class="connection-form" data-dialog-form>"#,
    );
    assert!(check_draft_controls_only_in_dialog_forms("a.rs", &source).is_empty());
}

#[test]
fn test_check_draft_controls_accepts_controls_that_save_on_change() {
    let source = r##"r#"<select name="a" hx-post="/x" hx-trigger="change"><option></option></select>
<form hx-post="/y" hx-trigger="change"><input type="checkbox" name="b"><select name="c"></select></form>"#"##;
    assert!(check_draft_controls_only_in_dialog_forms("a.rs", source).is_empty());
}

#[test]
fn test_check_draft_controls_ignores_hidden_readonly_disabled_and_button_controls() {
    let source = r##"r#"<form hx-post="/y"><input type="hidden" name="a"><input type="text" readonly><textarea disabled></textarea><input type="submit"></form>"#"##;
    assert!(check_draft_controls_only_in_dialog_forms("a.rs", source).is_empty());
}

#[test]
fn test_check_draft_controls_judges_a_control_after_an_autosave_form_by_its_own_form() {
    let source = r##"r#"<form hx-post="/y" hx-trigger="change"><select name="c"></select></form><form hx-post="/z"><input type="text" name="d"></form>"#"##;
    assert_eq!(
        check_draft_controls_only_in_dialog_forms("a.rs", source).len(),
        1
    );
}

#[test]
fn test_check_draft_control_exceptions_flags_a_stale_row() {
    let exceptions = [DraftControlException {
        path: "a.rs",
        reason: "a field builder",
    }];
    let clean = vec![(
        "a.rs".to_string(),
        r##"r#"<div data-dialog-form><input type="text"></div>"#"##.to_string(),
    )];
    let still_needed = vec![(
        "a.rs".to_string(),
        r##"r#"<div class="form-group"><textarea name="a"></textarea></div>"#"##.to_string(),
    )];

    assert_eq!(
        check_draft_control_exceptions_are_used(&clean, &exceptions).len(),
        1
    );
    assert!(check_draft_control_exceptions_are_used(&still_needed, &exceptions).is_empty());
}

#[test]
fn test_check_draft_controls_accepts_a_form_of_the_shell() {
    let source =
        r##"r#"<div class="preview" data-shell-form><textarea name="command"></textarea></div>"#"##;
    assert!(check_draft_controls_only_in_dialog_forms("a.rs", source).is_empty());
}

#[test]
fn test_check_draft_controls_judges_the_literal_not_its_first_tag() {
    // The marker sits on an inner tag of a different literal, so the first
    // literal holds a draft control with no marker of its own.
    let source = r##"r#"<div class="a"><input type="text" name="x"></div>"#; r#"<div data-dialog-form></div>"#"##;
    assert_eq!(
        check_draft_controls_only_in_dialog_forms("a.rs", source).len(),
        1
    );
}

#[test]
fn test_check_draft_controls_counts_a_conditionally_disabled_control() {
    for tag in [
        r#"<textarea name="a" {% if busy %}disabled{% endif %}></textarea>"#,
        r#"<input type="text" name="a"{{ disabled(busy) }}>"#,
        r#"<select name="a" {% if busy %} disabled{% endif %}></select>"#,
    ] {
        let source = format!("r#\"{tag}\"#");
        assert_eq!(
            check_draft_controls_only_in_dialog_forms("a.rs", &source).len(),
            1,
            "{tag}"
        );
    }
}

#[test]
fn test_check_draft_controls_accepts_an_unconditionally_disabled_or_readonly_control() {
    for tag in [
        r#"<textarea rows="3" disabled>x</textarea>"#,
        r#"<input type="text" readonly value="x">"#,
        r#"<textarea readonly></textarea>"#,
    ] {
        let source = format!("r#\"{tag}\"#");
        assert!(
            check_draft_controls_only_in_dialog_forms("a.rs", &source).is_empty(),
            "{tag}"
        );
    }
}

#[test]
fn test_check_dialog_fragments_carry_the_title() {
    let titled = r##"r#"<div data-dialog-form><h2 id="edit-dialog-title">Add</h2></div>"#"##;
    let untitled = r##"r#"<div data-dialog-form><h2>Add</h2></div>"#"##;
    assert!(check_dialog_fragments_carry_the_title("a.rs", titled).is_empty());
    let violations = check_dialog_fragments_carry_the_title("a.rs", untitled);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("edit-dialog-title"));
}

#[test]
fn test_check_draft_control_exceptions_flags_a_row_whose_file_holds_a_form() {
    let permanent = [DraftControlException {
        path: "a.rs",
        reason: "a field builder",
    }];
    let builder = vec![(
        "a.rs".to_string(),
        r##"r#"<div class="form-group"><textarea name="a"></textarea></div>"#"##.to_string(),
    )];
    let with_form = vec![(
        "a.rs".to_string(),
        r##"r#"<form hx-post="/x"><textarea name="a"></textarea></form>"#"##.to_string(),
    )];

    assert!(check_draft_control_exceptions_are_used(&builder, &permanent).is_empty());
    let violations = check_draft_control_exceptions_are_used(&with_form, &permanent);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("field builder"));
}

#[test]
fn test_scripts_listen_no_part_swaps_rejects_an_after_swap_listener_that_names_a_part() {
    let script = "document.body.addEventListener(\"htmx:afterSwap\", function (evt) {\n  if (evt.detail.target.id === \"header\") announce();\n});";
    let violations = check_scripts_listen_no_part_swaps("assets/dashboard-x.js", script, &parts());
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("screen:loaded"));
}

#[test]
fn test_scripts_listen_no_part_swaps_rejects_before_swap_and_after_settle() {
    for event in ["htmx:beforeSwap", "htmx:afterSettle"] {
        let script = format!(
            "document.body.addEventListener(\"{event}\", function (evt) {{\n  if (evt.detail.target.id === \"header\") run();\n}});"
        );
        assert_eq!(
            check_scripts_listen_no_part_swaps("assets/dashboard-x.js", &script, &parts()).len(),
            1,
            "{event}"
        );
    }
}

#[test]
fn test_scripts_listen_no_part_swaps_accepts_a_swap_listener_that_names_no_part() {
    let script = "document.body.addEventListener(\"htmx:afterSwap\", function (evt) {\n  if (evt.detail.target !== editDialogBody) return;\n});";
    assert!(
        check_scripts_listen_no_part_swaps("assets/dashboard-x.js", script, &parts()).is_empty()
    );
}

#[test]
fn test_scripts_listen_no_part_swaps_accepts_the_events_of_the_screen_loop() {
    let script = "document.addEventListener(\"screen:loaded\", function (evt) {\n  if (evt.detail.parts.includes(\"story-log\")) announce();\n});";
    assert!(
        check_scripts_listen_no_part_swaps("assets/dashboard-x.js", script, &parts()).is_empty()
    );
}

const STUB: &str = "async fn screen_part_html(state: &Arc<StubState>, name: &str) -> Option<String> {\n    Some(match name {\n        \"header\" => header_html(),\n        \"games\" => games_fragment(state()).await.0,\n        _ => return None,\n    })\n}\n\nasync fn other() {\n    match x {\n        \"extra\" => 1,\n    }\n}\n";

#[test]
fn test_check_stub_answers_the_parts_of_the_server_accepts_the_same_names() {
    assert!(
        check_stub_answers_the_parts_of_the_server("stub.rs", STUB, &["header", "games"])
            .is_empty()
    );
}

#[test]
fn test_check_stub_answers_the_parts_of_the_server_rejects_a_part_on_one_side_only() {
    let violations =
        check_stub_answers_the_parts_of_the_server("stub.rs", STUB, &["header", "worlds"]);
    assert_eq!(violations.len(), 2, "{violations:?}");
    assert!(violations.iter().any(|v| v.message.contains("`games`")));
    assert!(violations.iter().any(|v| v.message.contains("`worlds`")));
}

#[test]
fn test_check_stub_answers_the_parts_of_the_server_fails_a_stub_without_the_function() {
    assert_eq!(
        check_stub_answers_the_parts_of_the_server("stub.rs", "fn other() {}", &["header"]).len(),
        1
    );
}
