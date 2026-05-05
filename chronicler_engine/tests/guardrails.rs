//! [DOC: docs/architecture/guardrails.md]

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, ItemFn, Local};

// ── Severity & Violation Types ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    Error,
    Warning,
}

#[derive(Debug)]
struct Violation {
    file: String,
    line: usize,
    message: String,
    severity: Severity,
}

impl Violation {
    fn error(file: &str, line: usize, message: impl Into<String>) -> Self {
        Self {
            file: file.to_string(),
            line,
            message: message.into(),
            severity: Severity::Error,
        }
    }

    fn warn(file: &str, line: usize, message: impl Into<String>) -> Self {
        Self {
            file: file.to_string(),
            line,
            message: message.into(),
            severity: Severity::Warning,
        }
    }
}

// ── File Discovery ──

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

// ── Import Ordering ──

fn check_import_ordering(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut last_group = 0u8;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("use ") {
            // Blank lines and comments are allowed between use statements
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            // If we hit non-use, non-comment, non-empty line, stop checking imports
            // (imports should be grouped at the top)
            if last_group > 0 {
                break;
            }
            continue;
        }

        let group = if trimmed.starts_with("use std::")
            || trimmed.starts_with("use core::")
            || trimmed.starts_with("use alloc::")
        {
            1
        } else if trimmed.starts_with("use crate::")
            || trimmed.starts_with("use super::")
            || trimmed.starts_with("use self::")
        {
            3
        } else {
            2
        };

        if group < last_group {
            violations.push(Violation::error(
                path,
                line_num + 1,
                format!(
                    "Import ordering violation: group {group} import after group {last_group}. \
                     Expected order: std/core/alloc -> external crates -> crate/super/self"
                ),
            ));
        }
        last_group = group;
    }

    violations
}

// ── "What" Comment Detection ──

/// Set of verb prefixes that indicate a "What" comment.
static WHAT_COMMENT_VERBS: &[&str] = &[
    "loop",
    "check",
    "build",
    "get",
    "set",
    "handle",
    "process",
    "create",
    "add",
    "remove",
    "update",
    "evaluate",
    "apply",
    "call",
    "return",
    "print",
    "log",
    "send",
    "read",
    "write",
    "open",
    "close",
    "start",
    "stop",
    "run",
    "execute",
    "perform",
    "do",
    "make",
    "take",
    "put",
    "go",
    "look",
    "use",
    "find",
    "search",
    "match",
    "compare",
    "sort",
    "filter",
    "map",
    "reduce",
    "collect",
    "clone",
    "copy",
    "move",
    "push",
    "pop",
    "clear",
    "reset",
    "init",
    "parse",
    "convert",
    "transform",
    "validate",
    "compute",
    "calculate",
    "determine",
    "identify",
    "extract",
    "insert",
    "delete",
    "modify",
    "generate",
    "render",
    "format",
    "construct",
    "destroy",
    "load",
    "save",
    "fetch",
    "store",
    "retrieve",
    "update",
    "sync",
    "bind",
    "connect",
    "disconnect",
    "listen",
    "emit",
    "trigger",
    "fire",
    "raise",
    "throw",
    "catch",
    "wrap",
    "unwrap",
    "box",
    "unbox",
    "lock",
    "unlock",
    "wait",
    "notify",
    "signal",
    "poll",
    "spawn",
    "join",
    "detach",
    "resume",
    "pause",
    "suspend",
    "schedule",
    "dispatch",
    "route",
    "forward",
    "redirect",
    "proxy",
    "cache",
    "invalidate",
    "refresh",
    "rebuild",
    "recompute",
    "recalculate",
    "revalidate",
    "repopulate",
    "reinitialize",
];

/// Allow-listed comment prefixes that are NOT "What" comments.
static ALLOWED_COMMENT_PREFIXES: &[&str] = &[
    "[doc:",
    "todo",
    "fixme",
    "safety",
    "workaround",
    "note",
    "hack",
    "bug",
    "review",
    "idea",
    "question",
    "optimize",
    "refactor",
    "deprecated",
    "perf",
    "style",
    "clippy",
    "rustfmt",
    "allow",
    "deny",
    "warn",
    "expect",
    "unwrap",
    "panic",
    "assert",
    "debug",
    "trace",
    "info",
    "log",
    "print",
    "test",
    "mod",
    "doc",
    "use",
    "fn",
    "struct",
    "enum",
    "impl",
    "trait",
    "pub",
    "priv",
    "const",
    "static",
    "let",
    "mut",
    "ref",
    "own",
    "borrow",
    "move",
    "copy",
    "clone",
    "drop",
    "new",
    "default",
    "from",
    "into",
    "as",
    "to",
    "with",
    "self",
    "super",
    "crate",
    "extern",
    "async",
    "await",
    "loop",
    "while",
    "for",
    "if",
    "else",
    "match",
    "return",
    "break",
    "continue",
    "yield",
    "unsafe",
    "where",
    "in",
    "of",
    "on",
    "at",
    "by",
    "without",
    "not",
    "no",
    "all",
    "any",
    "some",
    "none",
    "each",
    "every",
    "when",
    "then",
    "than",
    "as",
    "be",
    "is",
    "are",
    "was",
    "were",
    "been",
    "being",
    "have",
    "has",
    "had",
    "do",
    "does",
    "did",
    "done",
    "will",
    "would",
    "should",
    "could",
    "can",
    "may",
    "might",
    "must",
    "shall",
    "need",
    "needs",
    "needed",
    "want",
    "wants",
    "wanted",
    "like",
    "likes",
    "liked",
];

fn is_what_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("// ") {
        return false;
    }
    // Skip doc comments and doc anchors
    if trimmed.starts_with("///") || trimmed.starts_with("// [DOC:") {
        return false;
    }

    let comment_body = &trimmed[3..]; // after "// "
    let lower = comment_body.to_lowercase();

    // Skip allow-listed prefixes
    for prefix in ALLOWED_COMMENT_PREFIXES {
        if lower.starts_with(prefix) {
            return false;
        }
    }

    // Skip URL/reference comments
    if comment_body.starts_with("http://")
        || comment_body.starts_with("https://")
        || comment_body.starts_with("see ")
        || comment_body.starts_with("ref ")
    {
        return false;
    }

    // Check for generic "This function/module/struct/enum" phrases
    if lower.starts_with("this function")
        || lower.starts_with("this module")
        || lower.starts_with("this struct")
        || lower.starts_with("this enum")
        || lower.starts_with("this trait")
        || lower.starts_with("this impl")
    {
        return true;
    }

    // Check for verb prefixes
    let first_word = lower
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c: char| !c.is_alphabetic());

    for verb in WHAT_COMMENT_VERBS {
        if first_word == *verb {
            return true;
        }
    }

    false
}

struct CfgTestTracker {
    inside_cfg_test: bool,
    brace_depth: i32,
}

impl CfgTestTracker {
    fn new() -> Self {
        Self {
            inside_cfg_test: false,
            brace_depth: 0,
        }
    }

    /// Process a line and return `true` if we are inside a `#[cfg(test)]` block.
    fn process_line(&mut self, line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.starts_with("#[cfg(test)]") {
            self.inside_cfg_test = true;
            self.brace_depth = 0;
            return true;
        }
        if self.inside_cfg_test {
            self.brace_depth += line.matches('{').count() as i32;
            self.brace_depth -= line.matches('}').count() as i32;
            if self.brace_depth <= 0 && trimmed.starts_with('}') {
                self.inside_cfg_test = false;
            }
        }
        self.inside_cfg_test
    }
}

fn check_what_comments(path: &str, content: &str, _in_test_module: &[bool]) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut tracker = CfgTestTracker::new();

    for (line_num, line) in content.lines().enumerate() {
        if tracker.process_line(line) {
            continue;
        }

        if is_what_comment(line) {
            violations.push(Violation::warn(
                path,
                line_num + 1,
                format!(
                    "'What' comment detected: '{}'. \
                     Replace with semantic naming or doc anchor // [DOC: docs/...]",
                    line.trim_start().trim_start_matches("// ").trim()
                ),
            ));
        }
    }

    violations
}

// ── Long Comment Run Detection ──

fn is_countable_comment(line: &str) -> bool {
    if !line.starts_with("//") {
        return false;
    }
    // Exclude visual dividers (4+ slashes)
    if line.starts_with("////") {
        return false;
    }
    // Exclude doc anchors
    if line.starts_with("// [DOC:") {
        return false;
    }
    // Exclude empty comments
    let after_slashes = if line.starts_with("///") || line.starts_with("//!") {
        &line[3..]
    } else {
        &line[2..]
    };
    !after_slashes.trim().is_empty()
}

fn check_long_comment_runs(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut tracker = CfgTestTracker::new();

    let mut run_start: Option<usize> = None;
    let mut run_count = 0;

    for (line_num, line) in content.lines().enumerate() {
        if tracker.process_line(line) {
            run_start = None;
            run_count = 0;
            continue;
        }

        if is_countable_comment(line.trim_start()) {
            if run_start.is_none() {
                run_start = Some(line_num + 1);
            }
            run_count += 1;
            if run_count == 5 {
                violations.push(Violation::warn(
                    path,
                    run_start.unwrap(),
                    "Long comment run: 5+ consecutive comment lines starting here. \
                     Consider replacing with semantic naming or a doc anchor // [DOC: docs/...]",
                ));
            }
        } else {
            run_start = None;
            run_count = 0;
        }
    }

    violations
}

// ── Doc Anchor Requirement ──

struct DocAnchorVisitor<'a> {
    file_path: &'a str,
    content: &'a str,
    violations: &'a mut Vec<Violation>,
}

impl<'ast> Visit<'ast> for DocAnchorVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Only check pub functions
        if !matches!(node.vis, syn::Visibility::Public(_)) {
            return;
        }

        let fn_name = node.sig.ident.to_string();

        // Exempt getters/setters
        if (fn_name.starts_with("get_") || fn_name.starts_with("set_"))
            && stmt_count(&node.block.stmts) <= 3
        {
            return;
        }

        // Exempt trivial functions
        if stmt_count(&node.block.stmts) <= 5 && !contains_control_flow(&node.block.stmts) {
            return;
        }

        // Check if function has doc anchor in attributes
        let has_doc_anchor_attr = node.attrs.iter().any(|attr| {
            let doc_str = quote::quote!(#attr).to_string();
            doc_str.contains("[DOC:")
        });

        if has_doc_anchor_attr {
            return;
        }

        // Check if first statement has doc anchor
        let start = node.sig.ident.span().start().line;
        let end = node.block.brace_token.span.close().start().line;
        let has_anchor_in_body = self
            .content
            .lines()
            .skip(start)
            .take(end.saturating_sub(start))
            .any(|line| line.trim().starts_with("// [DOC:"));

        if has_anchor_in_body {
            return;
        }

        self.violations.push(Violation::warn(
            self.file_path,
            start,
            format!(
                "Public function `{fn_name}` is complex (>5 stmts or has control flow) but lacks a doc anchor. \
                 Add `/// [DOC: docs/path/to/file.md]` or `// [DOC: ...]` inside the function body."
            ),
        ));
    }
}

fn stmt_count(stmts: &[syn::Stmt]) -> usize {
    stmts.len()
}

fn contains_control_flow(stmts: &[syn::Stmt]) -> bool {
    use syn::*;
    for stmt in stmts {
        match stmt {
            Stmt::Local(_) => {}
            Stmt::Item(_) => {}
            Stmt::Expr(expr, _) => {
                if expr_contains_control_flow(expr) {
                    return true;
                }
            }
            Stmt::Macro(_) => {}
        }
    }
    false
}

fn expr_contains_control_flow(expr: &syn::Expr) -> bool {
    use syn::*;
    match expr {
        Expr::If(_) | Expr::Match(_) | Expr::ForLoop(_) | Expr::While(_) | Expr::Loop(_) => true,
        Expr::Try(_) => true, // ? operator
        Expr::Block(b) => contains_control_flow(&b.block.stmts),
        Expr::MethodCall(m) => expr_contains_control_flow(&m.receiver),
        Expr::Call(c) => expr_contains_control_flow(&c.func),
        Expr::Binary(b) => {
            expr_contains_control_flow(&b.left) || expr_contains_control_flow(&b.right)
        }
        Expr::Assign(a) => {
            expr_contains_control_flow(&a.left) || expr_contains_control_flow(&a.right)
        }
        Expr::Field(f) => expr_contains_control_flow(&f.base),
        Expr::Index(i) => expr_contains_control_flow(&i.expr),
        Expr::Tuple(t) => t.elems.iter().any(expr_contains_control_flow),
        Expr::Array(a) => a.elems.iter().any(expr_contains_control_flow),
        Expr::Struct(s) => s.fields.iter().any(|f| expr_contains_control_flow(&f.expr)),
        Expr::Closure(c) => {
            if let syn::Expr::Block(block) = &*c.body {
                contains_control_flow(&block.block.stmts)
            } else {
                expr_contains_control_flow(&c.body)
            }
        }
        _ => false,
    }
}

// ── Single-Letter Variable Detection ──

struct SingleLetterVisitor<'a> {
    file_path: &'a str,
    violations: &'a mut Vec<Violation>,
    fn_stmt_count: usize,
}

impl<'ast> Visit<'ast> for SingleLetterVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let old_count = self.fn_stmt_count;
        self.fn_stmt_count = node.block.stmts.len();
        syn::visit::visit_item_fn(self, node);
        self.fn_stmt_count = old_count;
    }

    fn visit_local(&mut self, node: &'ast Local) {
        if let Some(ident) = get_local_ident(node) {
            let name = ident.to_string();
            if name.len() == 1 && name.chars().next().unwrap().is_ascii_lowercase() {
                // Allow in small functions
                if self.fn_stmt_count <= 10 {
                    return;
                }
                // Allow in for loop headers: for x in ...
                // (detected by checking if parent is ExprForLoop - hard without parent pointers)
                // Simplistic: allow if ident is 'i', 'j', 'k', 'n' (common loop indices)
                if name == "i"
                    || name == "j"
                    || name == "k"
                    || name == "n"
                    || name == "x"
                    || name == "y"
                    || name == "z"
                {
                    return;
                }

                self.violations.push(Violation::warn(
                    self.file_path,
                    ident.span().start().line,
                    format!(
                        "Single-letter variable `{}` in a function with {} statements. \
                         Use a descriptive name (semantic naming convention).",
                        name, self.fn_stmt_count
                    ),
                ));
            }
        }
        syn::visit::visit_local(self, node);
    }
}

fn get_local_ident(local: &Local) -> Option<syn::Ident> {
    match &local.pat {
        syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
        _ => None,
    }
}

// ── mod.rs Purity ──

fn check_mod_purity(path: &str, _content: &str, ast: &File) -> Vec<Violation> {
    let mut violations = Vec::new();

    if !path.ends_with("mod.rs") {
        return violations;
    }

    // Exempt src/server/mod.rs — it contains router setup and shared state
    // that is idiomatic for Axum projects.
    if path.replace('\\', "/").contains("server/mod.rs") {
        return violations;
    }

    for item in &ast.items {
        let (kind, line) = match item {
            syn::Item::Fn(f) => ("function", f.sig.ident.span().start().line),
            syn::Item::Struct(s) => ("struct", s.ident.span().start().line),
            syn::Item::Enum(e) => ("enum", e.ident.span().start().line),
            syn::Item::Impl(i) => ("impl block", i.self_ty.span().start().line),
            syn::Item::Const(c) => ("const", c.ident.span().start().line),
            syn::Item::Static(s) => ("static", s.ident.span().start().line),
            syn::Item::Type(t) => ("type alias", t.ident.span().start().line),
            syn::Item::Trait(t) => ("trait", t.ident.span().start().line),
            // Allowed: Mod, Use, ForeignMod, Verbatim
            _ => continue,
        };

        violations.push(Violation::error(
            path,
            line,
            format!(
                "mod.rs purity violation: `{kind}` definition found in mod.rs. \
                 mod.rs should only contain `pub mod`, `use`, `pub use`, and module docs (`//!`). \
                 Move {kind} definitions to a separate file."
            ),
        ));
    }

    violations
}

// ── Test Functions ──

#[test]
fn guardrails_import_ordering() {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        errors.extend(check_import_ordering(rel, &content));
    }
    assert_violations(&errors, "import ordering");
}

#[test]
fn guardrails_import_ordering_tests() {
    let mut errors = Vec::new();
    for file in discover_rs_files("tests") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = file.strip_prefix("tests/").unwrap_or(&file);
        errors.extend(check_import_ordering(rel, &content));
    }
    assert_violations(&errors, "import ordering (tests)");
}

#[test]
fn guardrails_what_comments() {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        errors.extend(check_what_comments(rel, &content, &[]));
    }
    assert_violations(&errors, "'What' comment");
}

#[test]
fn guardrails_doc_anchors() {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        let ast = syn::parse_file(&content).unwrap();
        let mut visitor = DocAnchorVisitor {
            file_path: rel,
            content: &content,
            violations: &mut errors,
        };
        visitor.visit_file(&ast);
    }
    assert_violations(&errors, "doc anchor");
}

#[test]
fn guardrails_single_letter_vars() {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        let ast = syn::parse_file(&content).unwrap();
        let mut visitor = SingleLetterVisitor {
            file_path: rel,
            violations: &mut errors,
            fn_stmt_count: 0,
        };
        visitor.visit_file(&ast);
    }
    assert_violations(&errors, "single-letter variable");
}

#[test]
fn guardrails_long_comment_runs() {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        errors.extend(check_long_comment_runs(rel, &content));
    }
    assert_violations(&errors, "long comment run");
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

// ── No std::thread in production code ──

fn check_no_std_thread(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Mock backends are allowed to use thread::sleep for test delays.
    if path.contains("mock.rs") {
        return violations;
    }

    let mut tracker = CfgTestTracker::new();

    for (line_num, line) in content.lines().enumerate() {
        if tracker.process_line(line) {
            continue;
        }

        if line.contains("std::thread::spawn") || line.contains("std::thread::sleep") {
            violations.push(Violation::error(
                path,
                line_num + 1,
                format!(
                    "Found {} in production code. Use tokio::task::spawn_blocking instead.",
                    if line.contains("spawn") {
                        "std::thread::spawn"
                    } else {
                        "std::thread::sleep"
                    }
                ),
            ));
        }
    }
    violations
}

#[test]
fn guardrails_no_std_thread() {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        errors.extend(check_no_std_thread(rel, &content));
    }
    assert_violations(&errors, "no-std-thread");
}

// ── Spawn site documentation ──

fn check_spawn_site_docs(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.contains("spawn_blocking") && !trimmed.contains("tokio::spawn") {
            continue;
        }

        // Look back up to 5 lines for a doc anchor
        let start = line_num.saturating_sub(5);
        let has_doc = lines[start..line_num]
            .iter()
            .any(|l| l.trim().starts_with("// [DOC:"));

        if !has_doc {
            violations.push(Violation::warn(
                path,
                line_num + 1,
                format!(
                    "Spawn site `{trimmed}` lacks a doc anchor. \
                     Add `// [DOC: docs/architecture/invariants.md#INV-004]` \
                     or similar within 5 lines above the spawn."
                ),
            ));
        }
    }
    violations
}

#[test]
fn guardrails_spawn_site_docs() {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        errors.extend(check_spawn_site_docs(rel, &content));
    }
    assert_violations(&errors, "spawn-site-docs");
}

// ── File Length ──

fn check_file_length(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let non_blank_count = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    if non_blank_count > 2000 {
        violations.push(Violation::error(
            path,
            1,
            format!(
                "File is too long: {non_blank_count} non-blank lines (max 2000). \
                 Consider splitting into smaller modules."
            ),
        ));
    }

    violations
}

#[test]
fn guardrails_file_length_src() {
    let mut errors = Vec::new();
    for file in discover_rs_files("src") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        errors.extend(check_file_length(rel, &content));
    }
    assert_violations(&errors, "file length (src)");
}

#[test]
fn guardrails_file_length_tests() {
    let mut errors = Vec::new();
    for file in discover_rs_files("tests") {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = file.strip_prefix("tests/").unwrap_or(&file);
        errors.extend(check_file_length(rel, &content));
    }
    assert_violations(&errors, "file length (tests)");
}

// ── Helpers ──

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

impl Violation {
    fn severity_label(&self) -> &'static str {
        match self.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
        }
    }
}
