use std::path::Path;
use crate::Violation;

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
            format!(
                "Test file uses singular 'test' suffix, expected {}_tests.rs",
                base
            ),
        ));
        return violations;
    }

    violations
}

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


    let source_file_name = format!("{}.rs", base_name);
    let parent_dir = file_path.parent().unwrap_or(Path::new(""));
    let expected_source = parent_dir.join(&source_file_name);


    let module_dir = parent_dir.join(base_name);
    let module_mod_rs = module_dir.join("mod.rs");
    let has_module_dir = module_dir.is_dir() && module_mod_rs.exists();


    let has_source_file = expected_source.exists();

    if !has_source_file {
        if has_module_dir {
            // Orphan: test outside module dir
            violations.push(Violation::error(
                path,
                1,
                format!(
                    "Test file for module '{}' is outside module directory. Move {} to {}/",
                    base_name, file_name, base_name
                ),
            ));
        } else {
            violations.push(Violation::error(
                path,
                1,
                format!(
                    "Orphan test file - no matching {} or {}/mod.rs found",
                    file_name, base_name
                ),
            ));
        }
    }

    violations
}

pub fn check_test_file_location(path: &str, _content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Naming
    violations.extend(check_test_file_naming(path));

    // Pairing
    violations.extend(check_test_file_pairing(path));

    violations
}
