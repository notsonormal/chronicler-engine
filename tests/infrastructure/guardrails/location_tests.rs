//! Tests for `location.rs` guardrail.

use std::path::PathBuf;
use crate::location::check_test_file_pairing;

fn temp_src_dir() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let src = dir.path().join("src");
    std::fs::create_dir(&src).expect("create src dir");
    (dir, src)
}

#[test]
fn test_check_test_file_pairing_catches_orphan_next_to_mod_rs() {
    let (_dir, src) = temp_src_dir();
    std::fs::write(src.join("mod.rs"), "").expect("create mod.rs");
    std::fs::write(src.join("foo_tests.rs"), "").expect("create orphan test file");

    let violations = check_test_file_pairing(&src.join("foo_tests.rs").to_string_lossy());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("Orphan test file"));
    assert!(violations[0].message.contains("foo.rs"));
    assert!(violations[0].message.contains("foo/mod.rs"));
}

#[test]
fn test_check_test_file_pairing_allows_matching_rs() {
    let (_dir, src) = temp_src_dir();
    std::fs::write(src.join("foo.rs"), "").expect("create source file");
    std::fs::write(src.join("foo_tests.rs"), "").expect("create test file");

    let violations = check_test_file_pairing(&src.join("foo_tests.rs").to_string_lossy());
    assert!(violations.is_empty());
}

#[test]
fn test_check_test_file_pairing_allows_matching_module_dir() {
    let (_dir, src) = temp_src_dir();
    let module_dir = src.join("foo");
    std::fs::create_dir(&module_dir).expect("create module dir");
    std::fs::write(module_dir.join("mod.rs"), "").expect("create mod.rs");
    std::fs::write(src.join("foo_tests.rs"), "").expect("create test file");

    let violations = check_test_file_pairing(&src.join("foo_tests.rs").to_string_lossy());
    assert!(violations.is_empty());
}
