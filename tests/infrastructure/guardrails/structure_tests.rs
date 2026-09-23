//! Tests for `structure.rs` guardrail.

use syn::File;
use crate::structure::*;

#[test]
fn test_check_doc_standards_catches_missing_anchor() {
    let violations =
        check_doc_standards("src/narrative/parser.rs", "//! Narrative parser module\n");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("DOC anchor"));
}

#[test]
fn test_check_doc_standards_allows_correct_doc() {
    let violations = check_doc_standards(
        "src/narrative/parser.rs",
        "//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]\n\
         //! Narrative parser module\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_doc_standards_rejects_forbidden_anchor() {
    let violations = check_doc_standards(
        "src/narrative/parser.rs",
        "//! [DOC: docs/plans/architecture.md]\n\
         //! Narrative parser module\n",
    );
    assert_eq!(violations.len(), 2);
    assert!(violations.iter().any(|v| v.message.contains("not allowed")));
    assert!(
        violations
            .iter()
            .any(|v| v.message.contains("must resolve under"))
    );
}

#[test]
fn test_check_doc_standards_test_file_no_anchor_ok() {
    let violations = check_doc_standards(
        "src/narrative/parser_tests.rs",
        "//! Narrative parser tests\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_doc_standards_test_file_rejects_anchor() {
    let violations = check_doc_standards(
        "src/narrative/parser_tests.rs",
        "//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]\n",
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("Test file"));
}

#[test]
fn test_check_mod_purity_catches_fn_in_mod() {
    let src = "fn helper() {}\n";
    let ast = syn::parse_str::<File>(src).expect("parse fixture");
    let violations = check_mod_purity("src/narrative/mod.rs", src, &ast);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("function"));
}

#[test]
fn test_check_mod_purity_allows_use_and_mod_only() {
    let src = "pub mod sub;\nuse std::path::Path;\n";
    let ast = syn::parse_str::<File>(src).expect("parse fixture");
    let violations = check_mod_purity("src/narrative/mod.rs", src, &ast);
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_mod_purity_skips_non_mod_rs() {
    let src = "fn helper() {}\n";
    let ast = syn::parse_str::<File>(src).expect("parse fixture");
    let violations = check_mod_purity("src/narrative/parser.rs", src, &ast);
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_mod_purity_skips_server_mod() {
    let src = "fn helper() {}\n";
    let ast = syn::parse_str::<File>(src).expect("parse fixture");
    let violations = check_mod_purity("src/server/mod.rs", src, &ast);
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_no_legacy_test_context_catches_legacy() {
    let violations = check_no_legacy_test_context(
        "integration/world_smoke.rs",
        "let ctx = make_test_context(&storage);\n",
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("SqliteTestAppBuilder"));
}

#[test]
fn test_check_no_legacy_test_context_allows_sqlite_variant() {
    let violations = check_no_legacy_test_context(
        "integration/world_smoke.rs",
        "let ctx = make_test_context_with_sqlite(&storage);\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_no_legacy_test_context_skips_non_integration() {
    let violations = check_no_legacy_test_context(
        "src/something.rs",
        "let ctx = make_test_context(&storage);\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_no_legacy_test_context_skips_comments() {
    let violations = check_no_legacy_test_context(
        "integration/world_smoke.rs",
        "// let ctx = make_test_context(&storage);\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_empty_rust_file_catches_only_comments() {
    let violations = check_empty_rust_file("src/empty.rs", "// nothing here\n// or here\n");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("File is empty"));
}

#[test]
fn test_check_empty_rust_file_allows_real_code() {
    let violations = check_empty_rust_file("src/foo.rs", "fn x() {}\n");
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_empty_rust_file_catches_completely_empty() {
    let violations = check_empty_rust_file("src/empty.rs", "");
    assert_eq!(violations.len(), 1);
}

#[test]
fn test_check_no_std_thread_all_catches_spawn() {
    let violations =
        check_no_std_thread_all("src/worker.rs", "fn run() { std::thread::spawn(|| {}); }\n");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("std::thread::spawn"));
}

#[test]
fn test_check_no_std_thread_all_catches_sleep() {
    let violations = check_no_std_thread_all(
        "src/worker.rs",
        "fn run() { std::thread::sleep(Duration::from_secs(1)); }\n",
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("std::thread::sleep"));
}

#[test]
fn test_check_no_std_thread_all_skips_mock_rs() {
    let violations = check_no_std_thread_all(
        "src/worker/mock.rs",
        "fn run() { std::thread::spawn(|| {}); }\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_no_std_thread_all_allows_tokio_blocking() {
    let violations = check_no_std_thread_all(
        "src/worker.rs",
        "fn run() { tokio::task::spawn_blocking(|| {}); }\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_no_std_thread_all_skips_cfg_test_block() {
    let violations = check_no_std_thread_all(
        "src/worker.rs",
        "#[cfg(test)]\n\
         mod tests {\n\
             #[test]\n\
             fn smoke() { std::thread::sleep(Duration::from_millis(1)); }\n\
         }\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_no_std_thread_all_skips_tests_rs() {
    let violations = check_no_std_thread_all(
        "src/worker_tests.rs",
        "#[test]\n\
         fn smoke() { std::thread::spawn(|| {}); }\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_file_length_allows_short() {
    let violations = check_file_length("src/foo.rs", "fn x() {}\n");
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_file_length_catches_too_long() {
    let body: String = "fn x() {}\n".repeat(2001);
    let violations = check_file_length("src/big.rs", &body);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("2001"));
    assert!(violations[0].message.contains("max 2000"));
}

#[test]
fn test_check_file_length_boundary_exactly_2000_ok() {
    let body: String = "fn x() {}\n".repeat(2000);
    let violations = check_file_length("src/big.rs", &body);
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_file_length_counts_non_blank_only() {
    // 100 code lines + 500 blank/whitespace lines => only 100 non-blank.
    let mut body = String::new();
    for _ in 0..100 {
        body.push_str("fn x() {}\n");
    }
    for _ in 0..500 {
        body.push_str("   \n");
    }
    let violations = check_file_length("src/foo.rs", &body);
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_test_module_header_catches_missing_header() {
    let violations = check_test_module_header(
        "integration/world_smoke.rs",
        "use crate::setup;\nfn run() {}\n",
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("module header"));
}

#[test]
fn test_check_test_module_header_allows_good_summary() {
    let violations = check_test_module_header(
        "integration/world_smoke.rs",
        "//! Smoke tests for world persistence.\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_test_module_header_skips_test_files() {
    let violations = check_test_module_header("integration/foo_tests.rs", "use crate::setup;\n");
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_test_module_header_rejects_multi_line() {
    let violations = check_test_module_header(
        "integration/world_smoke.rs",
        "//! Line one of summary.\n\
         //! Line two continues here.\n",
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("multi-line"));
}

#[test]
fn test_check_test_module_header_allows_doc_anchor_then_summary() {
    let violations = check_test_module_header(
        "integration/world_smoke.rs",
        "//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]\n\
         //! World persistence smoke tests.\n",
    );
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_test_module_header_rejects_trivial_summary() {
    let violations = check_test_module_header("integration/world_smoke.rs", "//! hi\n");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("trivial"));
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_rejects_raw_click() {
    let content = "with_test_page(a, b, c, |page, _| async move {\n    \
                   page.locator(\"#x\").await.click(None).await;\n});\n";
    let violations = check_browser_interactions_use_htmx_settle("browser/worlds.rs", content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("htmx settle counter"));
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_scans_by_path_not_marker() {
    // The exemption is by path: a tier-3 file that never mentions
    // `with_test_page` (e.g. under a renamed entry-point builder) is still
    // scanned, so the ban cannot silently escape.
    let content = "custom_entry(|page, _| async move {\n    \
                   page.locator(\"#x\").await.click(None).await;\n});\n";
    let violations = check_browser_interactions_use_htmx_settle("browser/worlds.rs", content);
    assert_eq!(violations.len(), 1);
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_rejects_raw_set_checked() {
    // A checkbox with `hx-trigger="change"` would recreate the
    // lost-interaction race, so checkbox toggles are banned like clicks.
    let content = "with_test_page(x, y, z, |page, _| async move {\n    \
                   page.locator(\"#c\").await.set_checked(true, None).await;\n});\n";
    let violations =
        check_browser_interactions_use_htmx_settle("browser/prompt_presets.rs", content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("set_checked("));
    assert!(violations[0].message.contains("settle-guard-exempt"));
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_allows_trailing_marker() {
    // The marker is a trailing comment on the flagged line, with the why
    // inline: the checked element carries no hx attribute, so no settle
    // exists to wait for.
    let content = "with_test_page(x, y, z, |page, _| async move {\n    \
                   page.locator(\"#c\").await.set_checked(true, None).await // settle-guard-exempt: input carries no hx-trigger; form posts via the gated Save click.\n});\n";
    let violations =
        check_browser_interactions_use_htmx_settle("browser/prompt_presets.rs", content);
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_marker_does_not_leak_to_next_line() {
    // A marker suppresses only its own line: a marker-only comment cannot
    // whitewash a banned interaction that follows it.
    let content = "with_test_page(x, y, z, |page, _| async move {\n    \
                   // settle-guard-exempt: (unrelated note)\n    \
                   page.locator(\"#x\").await.click(None).await;\n});\n";
    let violations = check_browser_interactions_use_htmx_settle("browser/worlds.rs", content);
    assert_eq!(violations.len(), 1);
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_check_substring_needs_the_dot() {
    // `.check(` bans the playwright check call, not every name ending in
    // `_check(`: the dot is part of the match.
    let content = "with_test_page(x, y, z, |page, _| async move {\n    \
                   post_action_check(&app, \"/options\").await;\n});\n";
    let violations = check_browser_interactions_use_htmx_settle("browser/worlds.rs", content);
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_rejects_raw_select_option() {
    let content = "with_test_page(x, y, z, |page, _| async move {\n    \
                   page.locator(\"#s\").await.select_option(\"v\", None).await;\n});\n";
    let violations = check_browser_interactions_use_htmx_settle("browser/options.rs", content);
    assert_eq!(violations.len(), 1);
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_rejects_dispatch_event() {
    let content = "with_test_page(x, y, z, |page, _| async move {\n    \
                   document.querySelector('#s').dispatchEvent(new Event('change'));\n});\n";
    let violations = check_browser_interactions_use_htmx_settle("browser/worlds.rs", content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("dispatchEvent("));
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_rejects_request_submit() {
    let content = "with_test_page(x, y, z, |page, _| async move {\n    \
                   document.getElementById('f').requestSubmit();\n});\n";
    let violations = check_browser_interactions_use_htmx_settle("browser/worlds.rs", content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("requestSubmit("));
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_allows_gated_helpers() {
    let content = "with_test_page(x, y, z, |page, _| async move {\n    \
                   click_and_settle(&page, \"#x\", \"#y\").await;\n});\n";
    let violations = check_browser_interactions_use_htmx_settle("browser/worlds.rs", content);
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_exempts_stub_tier() {
    // `with_stub_page` drives a stub with no htmx swap lifecycle: raw clicks
    // are legitimate there. The exemption is by path, not by the absence of a
    // `with_test_page` mention — the content here carries both.
    let content = "with_test_page(a, b, c, |page, _| async move {\n    \
                   page.locator(\"#x\").await.click(None).await;\n});\n";
    let violations =
        check_browser_interactions_use_htmx_settle("browser/stub/dashboard.rs", content);
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_browser_interactions_use_htmx_settle_ignores_comments() {
    let content = "with_test_page(x, y, z, |page, _| async move {\n    \
                   // prefer click_and_settle over .click(\n});\n";
    let violations = check_browser_interactions_use_htmx_settle("browser/worlds.rs", content);
    assert_eq!(violations.len(), 0);
}
