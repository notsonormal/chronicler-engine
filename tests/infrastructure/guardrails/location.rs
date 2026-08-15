//! Location guardrail tests: ensures `#[test]` / `#[cfg(test)]` units live in the correct directory (e.g., unit tests stay in `src/`, integration tests stay in `tests/`).

use std::path::Path;
use crate::Violation;

/// Rejects unit-test files with the singular `_test.rs` suffix in favor of `_tests.rs`.
pub fn check_test_file_naming(path: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Windows + Unix separators
    if !path.contains("src/") && !path.contains("src\\") {
        return violations;
    }

    let file_name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if !file_name.ends_with(".rs") {
        return violations;
    }

    if file_name.starts_with("test_") {
        return violations;
    }

    if file_name.ends_with("_test.rs") {
        let base = file_name.trim_end_matches("_test.rs");
        violations.push(Violation::error(
            path,
            1,
            format!("Test file uses singular 'test' suffix, expected {base}_tests.rs"),
        ));
        return violations;
    }

    violations
}

/// Requires every `_tests.rs` file in `src/` to have a matching source file or module directory.
pub fn check_test_file_pairing(path: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Windows + Unix separators
    if !path.contains("src/") && !path.contains("src\\") {
        return violations;
    }

    let file_path = Path::new(path);
    let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if !file_name.ends_with("_tests.rs") {
        return violations;
    }

    let base_name = file_name.trim_end_matches("_tests.rs");

    const SPECIAL_FILES: &[&str] = &["integration", "mod", "e2e", "system"];
    if SPECIAL_FILES.contains(&base_name) {
        return violations;
    }

    let source_file_name = format!("{base_name}.rs");
    let parent_dir = file_path.parent().unwrap_or(Path::new(""));
    let expected_source = parent_dir.join(&source_file_name);

    let module_dir = parent_dir.join(base_name);
    let module_mod_rs = module_dir.join("mod.rs");
    let has_module_dir = module_dir.is_dir() && module_mod_rs.exists();

    let has_source_file = expected_source.exists();

    if !has_source_file && !has_module_dir {
        violations.push(Violation::error(
            path,
            1,
            format!("Orphan test file - no matching {base_name}.rs or {base_name}/mod.rs found"),
        ));
    }

    violations
}

/// Combines test-file naming and pairing checks for `src/` test files.
pub fn check_test_file_location(path: &str, _content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Naming
    violations.extend(check_test_file_naming(path));

    // Pairing
    violations.extend(check_test_file_pairing(path));

    violations
}
