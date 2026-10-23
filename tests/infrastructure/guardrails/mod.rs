//! Infrastructure test binary root: shared guardrail harness (rule definitions, `Violation` type, file discovery, `check_src_files` / `check_tests_files` runners).

pub mod enums;
pub mod free_fn;
pub mod inherent_impl;
pub mod layers;
pub mod location;
pub mod nesting;
pub mod structure;
pub mod style;
pub mod test_hygiene;
pub mod unit_test_allowlist;
pub mod walker_rows;
pub mod walkers;

pub use inherent_impl::*;
pub use enums::*;
pub use free_fn::*;
pub use layers::*;
pub use nesting::*;
pub use structure::*;
pub use style::*;
pub use test_hygiene::*;
pub use location::*;
pub use unit_test_allowlist::*;
pub use walker_rows::*;
pub use walkers::*;

#[cfg(test)]
mod inherent_impl_tests;

#[cfg(test)]
mod free_fn_tests;

#[cfg(test)]
mod location_tests;

#[cfg(test)]
mod test_hygiene_tests;

#[cfg(test)]
mod structure_tests;

#[cfg(test)]
mod registry_tests;

#[cfg(test)]
mod style_tests;

#[cfg(test)]
mod unit_test_allowlist_tests;

#[cfg(test)]
mod walker_tests;

/// A `src/` or `tests/` file, with the path relative to that tree.
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

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

pub(crate) fn discover_rs_files(root: &str) -> Vec<String> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect()
}

/// The `src/` tree as a snapshot: the walker entry points need every file at once.
pub(crate) fn src_snapshot() -> Vec<SourceFile> {
    tree_snapshot("src")
}

/// Snapshot with paths relative to `tests/`.
pub(crate) fn tests_snapshot() -> Vec<SourceFile> {
    tree_snapshot("tests")
}

fn tree_snapshot(root: &str) -> Vec<SourceFile> {
    let prefix = format!("{root}/");
    discover_rs_files(root)
        .into_iter()
        .map(|full| {
            let content = std::fs::read_to_string(&full).unwrap();
            let path = full.strip_prefix(&prefix).unwrap_or(&full).to_string();
            SourceFile { path, content }
        })
        .collect()
}

pub(crate) fn relative_path(full: &str) -> &str {
    full.strip_prefix("src/").unwrap_or(full)
}

pub(crate) fn assert_violations(violations: &[Violation], rule_name: &str) {
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
    for file in src_snapshot() {
        errors.extend(check(&file.path, &file.content));
    }
    assert_violations(&errors, rule_name);
}

/// The violation a path-limited guardrail reports when its scope matches no file:
/// a stale scope would otherwise pass by checking nothing.
fn dead_scope_violation(rule_name: &str, scope: &str) -> Violation {
    Violation::error(
        "src/",
        0,
        format!(
            "{rule_name}: the scope `{scope}` matches no file under `src/`, so this \
             path-limited guardrail guards nothing"
        ),
    )
}

/// Runs a path-limited guardrail over `src/`, and fails it when its scope matches no
/// file.
fn check_src_files_in_scope(
    rule_name: &str,
    scope: &str,
    mut check: impl FnMut(&str, &str) -> Vec<Violation>,
) {
    let files = src_snapshot();
    let matched = files
        .iter()
        .filter(|file| file.path.starts_with(scope))
        .count();
    let mut errors = Vec::new();
    for file in &files {
        errors.extend(check(&file.path, &file.content));
    }
    if matched == 0 {
        errors.push(dead_scope_violation(rule_name, scope));
    }
    assert_violations(&errors, rule_name);
}

fn check_tests_files(rule_name: &str, mut check: impl FnMut(&str, &str) -> Vec<Violation>) {
    let mut errors = Vec::new();
    for file in tests_snapshot() {
        errors.extend(check(&file.path, &file.content));
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
fn guardrails_template_raw_strings() {
    check_src_files("template raw strings", check_template_raw_strings);
}

#[test]
fn guardrails_form_fields_urlencoded_safe() {
    check_src_files(
        "form fields urlencoded safety",
        check_form_fields_urlencoded_safe,
    );
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
    check_src_files_in_scope(
        "server layer boundary",
        HTTP_ADAPTER_SCOPE,
        check_server_layer_boundaries,
    );
}

#[test]
fn a_dead_scope_names_the_guardrail_and_the_scope() {
    let violation = dead_scope_violation("handler return type", "nowhere/");
    assert!(
        violation.message.contains("handler return type") && violation.message.contains("nowhere/"),
        "{:?}",
        violation.message
    );
}

#[test]
fn guardrails_wiredapp_scope() {
    check_src_files("WiredApp scope", check_wiredapp_scope);
}

#[test]
fn guardrails_test_module_header() {
    check_tests_files("test module header", check_test_module_header);
}

#[test]
fn guardrails_test_file_location() {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        let content = std::fs::read_to_string(&file).unwrap();
        // Preserve "src/" prefix so check_test_file_location recognizes these as src files.
        let rel = file.clone();
        errors.extend(check_test_file_location(&rel, &content));
    }
    assert_violations(&errors, "test file location (src)");
}

#[test]
fn guardrails_no_ignored_tests_src() {
    check_src_files("no ignored tests (src)", check_no_ignored_tests);
}

#[test]
fn guardrails_no_ignored_tests_tests() {
    check_tests_files("no ignored tests (tests)", |path, content| {
        if path.starts_with("llm/") {
            return Vec::new();
        }
        check_no_ignored_tests(path, content)
    });
}

#[test]
fn guardrails_sleep_has_reason_src() {
    check_src_files("sleep has a reason (src)", |path, content| {
        if !path.ends_with("_tests.rs") {
            return Vec::new();
        }
        check_sleep_has_reason(path, content)
    });
}

#[test]
fn guardrails_sleep_has_reason_tests() {
    check_tests_files("sleep has a reason (tests)", |path, content| {
        if path.starts_with("llm/") || path.starts_with("test_utils/") {
            return Vec::new();
        }
        check_sleep_has_reason(path, content)
    });
}

#[test]
fn guardrails_http_storage_leak() {
    check_src_files("HTTP storage leak", check_http_storage_leak);
}

#[test]
fn guardrails_handler_return_type() {
    check_src_files_in_scope(
        "handler return type",
        HTTP_ADAPTER_SCOPE,
        check_handler_return_type,
    );
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
fn guardrails_inherent_impl_locality() {
    let mut files: Vec<(String, String)> = Vec::new();
    for file in discover_rs_files("src") {
        if file.ends_with("_tests.rs") || file.ends_with("_test.rs") || file.ends_with("main.rs") {
            continue;
        }
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file).to_string();
        files.push((rel, content));
    }
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect();
    let errors = check_inherent_impl_locality(&borrowed);
    assert_violations(&errors, "inherent impl locality");
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

#[test]
fn guardrails_browser_interactions_use_htmx_settle() {
    check_tests_files(
        "browser interactions use the htmx settle counter",
        check_browser_interactions_use_htmx_settle,
    );
}

#[test]
fn guardrails_walker_banned_name() {
    let files = src_snapshot();
    let errors = check_banned_name_rows(&files, BANNED_NAME_ROWS);
    assert_violations(&errors, "design-rule walker: banned name");
}

#[test]
fn guardrails_walker_call_site() {
    let files = src_snapshot();
    let errors = check_call_site_rows(&files, CALL_SITE_ROWS);
    assert_violations(&errors, "design-rule walker: call site");
}

#[test]
fn guardrails_walker_type_check() {
    let files = src_snapshot();
    let errors = check_type_check_rows(&files, TYPE_CHECK_ROWS);
    assert_violations(&errors, "design-rule walker: type check");
}

#[test]
fn guardrails_walker_call_count() {
    let files = src_snapshot();
    let errors = check_call_count_rows(&files, CALL_COUNT_ROWS);
    assert_violations(&errors, "design-rule walker: call count");
}

#[test]
fn guardrails_unit_test_allowlist() {
    let files = src_snapshot();
    let errors = check_unit_test_allowlist(&files, LEGACY_UNIT_TEST_FILES, SEAM_CONTRACT_FILES);
    assert_violations(&errors, "legacy unit-test allowlist");
}
