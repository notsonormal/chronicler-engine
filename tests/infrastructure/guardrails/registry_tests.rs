//! Guardrail registry tests: every rule is reachable from `mod.rs` and has a detection test, and every design-rule walker table is registered with a detection case.

// The walker rows are not `check_*` functions, so `walker_tests.rs` runs one
// case table per kind and the tests below keep each table registered and the
// rule numbers unique.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use syn::visit::Visit;
use syn::{Attribute, ItemFn, Visibility};

fn guardrails_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/infrastructure/guardrails")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn rs_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(guardrails_dir())
        .expect("read guardrails dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "rs").unwrap_or(false))
        .collect();
    files.sort();
    files
}

fn is_rule_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name != "mod.rs" && !name.ends_with("_tests.rs")
}

fn defined_rules(content: &str) -> BTreeSet<String> {
    let ast = parsed(content);
    ast.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Fn(function) if matches!(function.vis, Visibility::Public(_)) => {
                let name = function.sig.ident.to_string();
                name.starts_with("check_").then_some(name)
            }
            _ => None,
        })
        .collect()
}

fn parsed(content: &str) -> syn::File {
    syn::parse_file(content).unwrap_or_else(|e| panic!("guardrail source must parse: {e}"))
}

/// A rule registers whether it is passed by value
/// (`check_src_files("…", check_x)`) or called (`check_x(path, content)`); a
/// name in a comment, a string, or a `use` declaration does not.
#[derive(Default)]
struct RuleReferences {
    names: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for RuleReferences {
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if let Some(segment) = node.path.segments.last() {
            self.names.insert(segment.ident.to_string());
        }
        syn::visit::visit_expr_path(self, node);
    }
}

fn referenced_rules(content: &str) -> BTreeSet<String> {
    let mut visitor = RuleReferences::default();
    visitor.visit_file(&parsed(content));
    visitor.names
}

fn calls_rule(content: &str, name: &str) -> bool {
    referenced_rules(content).contains(name)
}

/// The callee ident of every call inside a `#[test]` function body, so a
/// detection test is one that invokes the rule rather than one that names it.
/// Every expression path inside a test body is also collected, so a row table
/// passed to a walker counts as registered.
#[derive(Default)]
struct TestBodies {
    depth: usize,
    called: BTreeSet<String>,
    referenced: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for TestBodies {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if node.attrs.iter().any(is_test_attribute) {
            self.depth += 1;
            syn::visit::visit_item_fn(self, node);
            self.depth -= 1;
        } else {
            syn::visit::visit_item_fn(self, node);
        }
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if self.depth > 0 {
            if let syn::Expr::Path(callee) = node.func.as_ref() {
                if let Some(segment) = callee.path.segments.last() {
                    self.called.insert(segment.ident.to_string());
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if self.depth > 0 {
            if let Some(segment) = node.path.segments.last() {
                self.referenced.insert(segment.ident.to_string());
            }
        }
        syn::visit::visit_expr_path(self, node);
    }
}

fn is_test_attribute(attr: &Attribute) -> bool {
    attr.path().is_ident("test")
}

fn rules_called_in_tests(content: &str, all_names: &BTreeSet<String>) -> BTreeSet<String> {
    let mut visitor = TestBodies::default();
    visitor.visit_file(&parsed(content));
    all_names.intersection(&visitor.called).cloned().collect()
}

fn registered_in_tests(content: &str) -> BTreeSet<String> {
    let mut visitor = TestBodies::default();
    visitor.visit_file(&parsed(content));
    visitor.referenced.union(&visitor.called).cloned().collect()
}

fn all_rule_names(files: &[PathBuf]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for path in files {
        if is_rule_file(path) {
            names.extend(defined_rules(&read(path)));
        }
    }
    names
}

/// Rules reachable from `mod.rs`: directly referenced, or referenced by a rule
/// that is already reachable (composites such as `check_test_file_location`).
fn reachable_rules(files: &[PathBuf], all_names: &BTreeSet<String>) -> BTreeSet<String> {
    let mod_content = read(&guardrails_dir().join("mod.rs"));
    let mut reachable: BTreeSet<String> = all_names
        .intersection(&referenced_rules(&mod_content))
        .cloned()
        .collect();

    let rule_files: Vec<(BTreeSet<String>, BTreeSet<String>)> = files
        .iter()
        .filter(|path| is_rule_file(path))
        .map(|path| {
            let content = read(path);
            (defined_rules(&content), referenced_rules(&content))
        })
        .collect();

    loop {
        let mut added = false;
        for (defines, references) in &rule_files {
            if !defines.iter().any(|name| reachable.contains(name)) {
                continue;
            }
            for name in all_names.intersection(references) {
                if reachable.insert(name.clone()) {
                    added = true;
                }
            }
        }
        if !added {
            break;
        }
    }

    reachable
}

#[test]
fn every_public_check_rule_is_reachable_from_mod_rs() {
    let files = rs_files();
    let all_names = all_rule_names(&files);
    assert!(!all_names.is_empty(), "no guardrail rules found");

    let reachable = reachable_rules(&files, &all_names);
    let unreachable: Vec<&String> = all_names.difference(&reachable).collect();
    assert!(
        unreachable.is_empty(),
        "guardrail rules defined but never reachable from mod.rs: {unreachable:?}"
    );
}

#[test]
fn every_public_check_rule_has_a_detection_test() {
    let files = rs_files();
    let all_names = all_rule_names(&files);
    assert!(!all_names.is_empty(), "no guardrail rules found");

    let mut covered = BTreeSet::new();
    for path in &files {
        covered.extend(rules_called_in_tests(&read(path), &all_names));
    }

    let uncovered: Vec<&String> = all_names.difference(&covered).collect();
    assert!(
        uncovered.is_empty(),
        "guardrail rules with no #[test] that calls them: {uncovered:?}"
    );
}

fn name_set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

#[test]
fn defined_rules_extracts_public_checks_only() {
    let names =
        defined_rules("pub fn check_alpha(a: &str) {}\nfn check_private() {}\npub fn other() {}\n");
    assert_eq!(names, name_set(&["check_alpha"]));
}

#[test]
fn calls_rule_matches_a_rule_cited_as_a_value() {
    assert!(calls_rule(
        "fn register() { check_src_files(\"alpha\", check_alpha); }",
        "check_alpha"
    ));
    assert!(!calls_rule("fn check_alpha_catches() {}", "check_alpha"));
    assert!(!calls_rule(
        "fn register() { let helper_check_alpha = 1; }",
        "check_alpha"
    ));
}

#[test]
fn detection_scan_ignores_names_outside_test_bodies() {
    let names = name_set(&["check_alpha", "check_beta"]);
    let content = "pub fn check_alpha(path: &str, content: &str) -> Vec<Violation> {}\n\
                   // check_beta(path, content) appears only in a comment\n\
                   fn helper() { check_beta(1, 2); }\n\
                   #[test]\n\
                   fn check_alpha_catches() { check_alpha(\"src/x.rs\", \"\"); }\n";
    let called = rules_called_in_tests(content, &names);
    assert!(called.contains("check_alpha"));
    assert!(!called.contains("check_beta"));
}

#[test]
fn detection_scan_ignores_test_name_mention() {
    let names = name_set(&["check_gamma"]);
    let content = "#[test]\nfn check_gamma_catches_violation() { let probe = 1; }\n";
    let called = rules_called_in_tests(content, &names);
    assert!(called.is_empty());
}

#[test]
fn detection_scan_ignores_prefixed_identifier() {
    let names = name_set(&["check_gamma"]);
    let content = "#[test]\nfn probe_case() { helper_check_gamma(1); }\n";
    let called = rules_called_in_tests(content, &names);
    assert!(
        called.is_empty(),
        "a differently-named helper must not count as a detection test"
    );
}

const WALKER_KINDS: &[(&str, &str)] = &[
    ("BANNED_NAME_ROWS", "BANNED_NAME_CASES"),
    ("CALL_SITE_ROWS", "CALL_SITE_CASES"),
    ("TYPE_CHECK_ROWS", "TYPE_CHECK_CASES"),
    ("CALL_COUNT_ROWS", "CALL_COUNT_CASES"),
];

fn all_registered_in_tests() -> BTreeSet<String> {
    let mut registered = registered_in_tests(&read(&guardrails_dir().join("mod.rs")));
    for path in rs_files() {
        registered.extend(registered_in_tests(&read(&path)));
    }
    registered
}

#[test]
fn every_walker_table_is_run_by_a_registered_test() {
    let registered = all_registered_in_tests();
    for (rows, cases) in WALKER_KINDS {
        assert!(
            registered.contains(*rows),
            "row table `{rows}` is never passed to its walker in a #[test] body, \
             so its rows guard nothing"
        );
        assert!(
            registered.contains(*cases),
            "detection case table `{cases}` is never used in a #[test] body, so \
             the walker kind has no detection test"
        );
    }
}

#[test]
fn every_walker_row_has_a_detection_case() {
    let banned_rules: BTreeSet<u8> = crate::walker_tests::BANNED_NAME_CASES
        .iter()
        .map(|case| case.row.rule)
        .collect();
    let call_site_rules: BTreeSet<u8> = crate::walker_tests::CALL_SITE_CASES
        .iter()
        .map(|case| case.row.rule)
        .collect();
    let type_check_rules: BTreeSet<u8> = crate::walker_tests::TYPE_CHECK_CASES
        .iter()
        .map(|case| case.row.rule)
        .collect();
    let call_count_rules: BTreeSet<u8> = crate::walker_tests::CALL_COUNT_CASES
        .iter()
        .map(|case| case.row.rule)
        .collect();

    let checks: [(&str, Vec<u8>, &BTreeSet<u8>); 4] = [
        (
            "BANNED_NAME_ROWS",
            crate::BANNED_NAME_ROWS.iter().map(|row| row.rule).collect(),
            &banned_rules,
        ),
        (
            "CALL_SITE_ROWS",
            crate::CALL_SITE_ROWS.iter().map(|row| row.rule).collect(),
            &call_site_rules,
        ),
        (
            "TYPE_CHECK_ROWS",
            crate::TYPE_CHECK_ROWS.iter().map(|row| row.rule).collect(),
            &type_check_rules,
        ),
        (
            "CALL_COUNT_ROWS",
            crate::CALL_COUNT_ROWS.iter().map(|row| row.rule).collect(),
            &call_count_rules,
        ),
    ];

    for (table, rules, covered) in checks {
        let uncovered: Vec<u8> = rules
            .into_iter()
            .filter(|rule| !covered.contains(rule))
            .collect();
        assert!(
            uncovered.is_empty(),
            "rows in `{table}` have no detection case in `walker_tests.rs`: \
             {uncovered:?} — a new row needs a fixture that proves the walker \
             catches it"
        );
    }
}

#[test]
fn every_walker_rule_number_is_used_once() {
    let mut rules: Vec<u8> = Vec::new();
    rules.extend(crate::BANNED_NAME_ROWS.iter().map(|row| row.rule));
    rules.extend(crate::CALL_SITE_ROWS.iter().map(|row| row.rule));
    rules.extend(crate::TYPE_CHECK_ROWS.iter().map(|row| row.rule));
    rules.extend(crate::CALL_COUNT_ROWS.iter().map(|row| row.rule));

    let mut seen: BTreeSet<u8> = BTreeSet::new();
    let duplicates: Vec<u8> = rules
        .into_iter()
        .filter(|rule| !seen.insert(*rule))
        .collect();
    assert!(
        duplicates.is_empty(),
        "a design rule has one row, in one table; these rule numbers have more: \
         {duplicates:?}"
    );
}
