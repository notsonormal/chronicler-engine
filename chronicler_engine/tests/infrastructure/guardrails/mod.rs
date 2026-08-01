//! Infrastructure test binary root: shared guardrail harness (rule definitions, `Violation` type, file discovery, `check_src_files` / `check_tests_files` runners).

pub mod enums;
pub mod free_fn;
pub mod layers;
pub mod location;
pub mod nesting;
pub mod structure;
pub mod style;

pub use enums::*;
pub use free_fn::*;
pub use layers::*;
pub use nesting::*;
pub use structure::{check_no_legacy_test_context, *};
pub use style::*;
pub use location::*;

#[cfg(test)]
mod free_fn_tests;

#[cfg(test)]
mod structure_tests;


// TODO: These types and functions should be moved to other folders

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug)]
pub struct Violation {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub severity: Severity,
}

impl Violation {
    pub fn error(file: &str, line: usize, message: impl Into<String>) -> Self {
        Self {
            file: file.to_string(),
            line,
            message: message.into(),
            severity: Severity::Error,
        }
    }

    pub fn warn(file: &str, line: usize, message: impl Into<String>) -> Self {
        Self {
            file: file.to_string(),
            line,
            message: message.into(),
            severity: Severity::Warning,
        }
    }

    pub fn severity_label(&self) -> &'static str {
        match self.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
        }
    }
}

fn discover_rs_files(root: &str) -> Vec<String> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect()
}

fn relative_path(full: &str) -> &str {
    full.strip_prefix("src/").unwrap_or(full)
}

fn assert_violations(violations: &[Violation], rule_name: &str) {
    if !violations.is_empty() {
        for v in violations {
            eprintln!(
                "{}:{} [{}] {}",
                v.file,
                v.line,
                v.severity_label(),
                v.message
            );
        }
        panic!(
            "{} violation(s) found for {}. See output above.",
            violations.len(),
            rule_name
        );
    }
}

fn check_src_files(rule_name: &str, mut check: impl FnMut(&str, &str) -> Vec<Violation>) {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        errors.extend(check(rel, &content));
    }
    assert_violations(&errors, rule_name);
}

fn check_tests_files(rule_name: &str, mut check: impl FnMut(&str, &str) -> Vec<Violation>) {
    let mut errors = Vec::new();
    for file in discover_rs_files("tests") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = file.strip_prefix("tests/").unwrap_or(&file);
        errors.extend(check(rel, &content));
    }
    assert_violations(&errors, rule_name);
}

#[test]
fn guardrails_import_ordering() {
    check_src_files("import ordering", check_import_ordering);
}

#[test]
fn guardrails_import_ordering_tests() {
    check_tests_files("import ordering (tests)", check_import_ordering);
}

#[test]
fn guardrails_doc_standards() {
    check_src_files("doc standards", check_doc_standards);
}

#[test]
fn guardrails_doc_standards_tests() {
    let mut errors = Vec::new();
    for file in discover_rs_files("tests") {
        let content = std::fs::read_to_string(&file).unwrap();
        // Preserve "tests/" prefix so check_doc_standards recognizes these as test files.
        let rel = file.clone();
        errors.extend(check_doc_standards(&rel, &content));
    }
    assert_violations(&errors, "doc standards (tests)");
}

#[test]
fn guardrails_single_letter_vars() {
    check_src_files("single-letter variable", check_single_letter_vars);
}

#[test]
fn guardrails_separator_comments() {
    check_src_files("separator comment", check_separator_comments);
}
#[test]
fn guardrails_long_comment_runs() {
    check_src_files("long comment run", check_long_comment_runs);
}

#[test]
fn guardrails_mod_purity() {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        if !file.ends_with("mod.rs") {
            continue;
        }
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        let ast = syn::parse_file(&content).unwrap();
        errors.extend(check_mod_purity(rel, &content, &ast));
    }
    assert_violations(&errors, "mod.rs purity");
}

#[test]
fn guardrails_no_std_thread() {
    check_src_files("no-std-thread", check_no_std_thread_all);
}

#[test]
fn guardrails_file_length_src() {
    check_src_files("file length (src)", check_file_length);
}

#[test]
fn guardrails_file_length_tests() {
    check_tests_files("file length (tests)", check_file_length);
}

#[test]
fn guardrails_server_layer_boundaries() {
    check_src_files("server layer boundary", check_server_layer_boundaries);
}

#[test]
fn guardrails_wiredapp_scope() {
    check_src_files("WiredApp scope", check_wiredapp_scope);
}

#[test]
fn guardrails_test_layer_boundaries() {
    check_tests_files("test layer boundary", check_test_layer_boundaries);
}
#[test]
fn guardrails_no_legacy_test_context() {
    check_tests_files("legacy test context", check_no_legacy_test_context);
}

#[test]
fn guardrails_test_module_header() {
    check_tests_files("test module header", check_test_module_header);
}

#[test]
fn guardrails_test_file_location() {
    check_src_files("test file location (src)", check_test_file_location);
}

#[test]
fn guardrails_messages_swipes_separation() {
    check_src_files(
        "messages/swipes separation",
        check_messages_swipes_separation,
    );
}

#[test]
fn guardrails_handler_return_type() {
    check_src_files("handler return type", check_handler_return_type);
}

#[test]
fn guardrails_enum_variant_docs() {
    check_src_files("enum variant docs", check_enum_variant_docs);
}

#[test]
fn guardrails_enum_variant_docs_tests() {
    check_tests_files("enum variant docs (tests)", check_enum_variant_docs);
}

#[test]
fn guardrails_nesting_depth_src() {
    check_src_files("nesting depth (src)", check_nesting_depth);
}

#[test]
fn guardrails_free_fn_location() {
    check_src_files("free fn location", check_free_fn_location);
}

#[test]
fn guardrails_empty_rust_files_src() {
    check_src_files("empty rust file", check_empty_rust_file);
}

#[test]
fn guardrails_empty_rust_files_tests() {
    check_tests_files("empty rust file (tests)", check_empty_rust_file);
}
