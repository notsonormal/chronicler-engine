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
        "//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]\n\
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
        "//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]\n",
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
        "//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]\n\
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
