//! [DOC: docs/architecture/guardrails.md]
//! Custom architecture guardrails using syn AST analysis.
//!
//! These tests enforce coding standards that clippy and arch-lint cannot catch:
//! - Import ordering (std -> external -> crate)
//! - "What" comment detection
//! - Doc anchor requirements on complex functions
//! - mod.rs purity
//! - Single-letter variable naming

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

fn discover_src_files() -> Vec<String> {
    walkdir::WalkDir::new("src")
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
    "resume",
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
    "for",
    "from",
    "with",
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

fn check_what_comments(path: &str, content: &str, _in_test_module: &[bool]) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut inside_cfg_test = false;
    let mut brace_depth = 0;

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Track cfg(test) module depth roughly
        if trimmed.starts_with("#[cfg(test)]") {
            inside_cfg_test = true;
        }
        for c in trimmed.chars() {
            if c == '{' {
                brace_depth += 1;
            } else if c == '}' {
                brace_depth -= 1;
                if brace_depth == 0 && inside_cfg_test {
                    inside_cfg_test = false;
                }
            }
        }

        // Skip comments inside test modules
        if inside_cfg_test {
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
                "Public function `{}` is complex (>5 stmts or has control flow) but lacks a doc anchor. \
                 Add `/// [DOC: docs/path/to/file.md]` or `// [DOC: ...]` inside the function body.",
                fn_name
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
                "mod.rs purity violation: `{}` definition found in mod.rs. \
                 mod.rs should only contain `pub mod`, `use`, `pub use`, and module docs (`//!`). \
                 Move {kind} definitions to a separate file.",
                kind
            ),
        ));
    }

    violations
}

// ── Test Functions ──

#[test]
fn guardrails_import_ordering() {
    let mut errors = Vec::new();
    for file in discover_src_files() {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        errors.extend(check_import_ordering(rel, &content));
    }
    assert_violations(&errors, "import ordering");
}

#[test]
fn guardrails_what_comments() {
    let mut warnings = Vec::new();
    for file in discover_src_files() {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        warnings.extend(check_what_comments(rel, &content, &[]));
    }
    print_warnings(&warnings, "'What' comment");
    // Currently warn-only due to existing violations.
    // Flip to assert_empty when baseline is clean.
}

#[test]
fn guardrails_doc_anchors() {
    let mut warnings = Vec::new();
    for file in discover_src_files() {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        let ast = syn::parse_file(&content).unwrap();
        let mut visitor = DocAnchorVisitor {
            file_path: rel,
            content: &content,
            violations: &mut warnings,
        };
        visitor.visit_file(&ast);
    }
    print_warnings(&warnings, "doc anchor");
    // Currently warn-only due to existing violations.
    // Flip to assert_empty when baseline is clean.
}

#[test]
fn guardrails_single_letter_vars() {
    let mut warnings = Vec::new();
    for file in discover_src_files() {
        let content = std::fs::read_to_string(&file).unwrap();
        let rel = relative_path(&file);
        let ast = syn::parse_file(&content).unwrap();
        let mut visitor = SingleLetterVisitor {
            file_path: rel,
            violations: &mut warnings,
            fn_stmt_count: 0,
        };
        visitor.visit_file(&ast);
    }
    print_warnings(&warnings, "single-letter variable");
    // Currently warn-only due to existing violations.
    // Flip to assert_empty when baseline is clean.
}

#[test]
fn guardrails_mod_purity() {
    let mut errors = Vec::new();
    for file in discover_src_files() {
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

fn print_warnings(violations: &[Violation], rule_name: &str) {
    if !violations.is_empty() {
        eprintln!(
            "\n=== {} warnings: {} violation(s) ===",
            rule_name,
            violations.len()
        );
        for v in violations {
            eprintln!("  {}:{} - {}", v.file, v.line, v.message);
        }
        eprintln!("=== End {} warnings ===\n", rule_name);
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
