//! Guardrail registry tests: every rule is reachable from `mod.rs` and has a detection test.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, ItemFn};

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

fn is_comment_or_use(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("//")
        || trimmed.starts_with("*")
        || trimmed.starts_with("use ")
        || trimmed.starts_with("pub use ")
}

/// `pub fn check_<name>(` definitions in `content`.
fn defined_rules(content: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in content.lines() {
        let Some(rest) = line.trim_start().strip_prefix("pub fn check_") else {
            continue;
        };
        if let Some(idx) = rest.find('(') {
            names.insert(format!("check_{}", &rest[..idx]));
        }
    }
    names
}

/// True when `content` references `name` as a whole identifier on a line that
/// is neither a comment, a `use`, nor the rule's own definition line. The rule
/// may be referenced as a value (`check_src_files("…", check_x)`) or called
/// (`check_x(path, content)`).
fn calls_rule(content: &str, name: &str) -> bool {
    let definition = format!("pub fn {name}(");
    content.lines().any(|line| {
        !is_comment_or_use(line)
            && !line.trim_start().starts_with(&definition)
            && references_name(line, name)
    })
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// True when `name` appears in `line` with identifier boundaries on both sides.
fn references_name(line: &str, name: &str) -> bool {
    bounded_match(line, name, |after| after.is_none_or(|b| !is_ident_byte(b)))
}

/// True when `line` calls `name`: identifier boundary before it, `(` after it.
fn calls_name(line: &str, name: &str) -> bool {
    bounded_match(line, name, |after| after == Some(b'('))
}

/// Scan `line` for `name` preceded by a non-identifier byte, accepting when the
/// byte that follows satisfies `accept_after`.
fn bounded_match(line: &str, name: &str, accept_after: impl Fn(Option<u8>) -> bool) -> bool {
    let bytes = line.as_bytes();
    let mut offset = 0;
    while let Some(idx) = line[offset..].find(name) {
        let start = offset + idx;
        let end = start + name.len();
        let boundary_before = start == 0 || !is_ident_byte(bytes[start - 1]);
        if boundary_before && accept_after(bytes.get(end).copied()) {
            return true;
        }
        offset = start + 1;
    }
    false
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

/// Rules reachable from `mod.rs`: directly registered, or called by a rule that
/// is already reachable (composites such as `check_test_file_location`).
fn reachable_rules(files: &[PathBuf], all_names: &BTreeSet<String>) -> BTreeSet<String> {
    let mod_content = read(&guardrails_dir().join("mod.rs"));
    let mut reachable: BTreeSet<String> = all_names
        .iter()
        .filter(|name| calls_rule(&mod_content, name))
        .cloned()
        .collect();

    let rule_contents: Vec<String> = files
        .iter()
        .filter(|path| is_rule_file(path))
        .map(|path| read(path))
        .collect();

    loop {
        let mut added = false;
        for content in &rule_contents {
            let defines = defined_rules(content);
            if !defines.iter().any(|name| reachable.contains(name)) {
                continue;
            }
            for name in all_names {
                if !reachable.contains(name) && calls_rule(content, name) {
                    reachable.insert(name.clone());
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

struct TestSpans {
    spans: Vec<(usize, usize)>,
}

impl<'ast> Visit<'ast> for TestSpans {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if node.attrs.iter().any(is_test_attribute) {
            let span = node.block.span();
            self.spans.push((span.start().line, span.end().line));
        }
        syn::visit::visit_item_fn(self, node);
    }
}

fn is_test_attribute(attr: &Attribute) -> bool {
    attr.path().is_ident("test")
}

/// Rule names called inside a `#[test]` function body in `content`.
fn rules_called_in_tests(content: &str, all_names: &BTreeSet<String>) -> BTreeSet<String> {
    let Ok(ast) = syn::parse_file(content) else {
        return BTreeSet::new();
    };
    let mut visitor = TestSpans { spans: Vec::new() };
    visitor.visit_file(&ast);

    let lines: Vec<&str> = content.lines().collect();
    let mut called = BTreeSet::new();
    for (start, end) in visitor.spans {
        for line in lines.iter().take(end).skip(start.saturating_sub(1)) {
            if is_comment_or_use(line) {
                continue;
            }
            for name in all_names {
                if calls_name(line, name) {
                    called.insert(name.clone());
                }
            }
        }
    }
    called
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
fn references_name_requires_identifier_boundaries() {
    assert!(references_name(
        "check_src_files(\"alpha\", check_alpha);",
        "check_alpha"
    ));
    assert!(!references_name(
        "fn check_alpha_catches() {}",
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
