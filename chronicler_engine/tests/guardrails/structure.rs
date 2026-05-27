use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, ItemFn};

use crate::Violation;

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

pub fn check_doc_anchors(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let ast = syn::parse_file(content).unwrap();
    let mut visitor = DocAnchorVisitor {
        file_path: path,
        content,
        violations: &mut violations,
    };
    visitor.visit_file(&ast);
    violations
}

// ── mod.rs Purity ──

pub fn check_mod_purity(path: &str, _content: &str, ast: &File) -> Vec<Violation> {
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

// ── No std::thread in production code ──

fn check_no_std_thread(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Mock backends are allowed to use thread::sleep for test delays.
    if path.contains("mock.rs") {
        return violations;
    }

    let mut tracker = crate::style::CfgTestTracker::new();

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

pub fn check_no_std_thread_all(path: &str, content: &str) -> Vec<Violation> {
    check_no_std_thread(path, content)
}

// ── Spawn site documentation ──

pub fn check_spawn_site_docs(path: &str, content: &str) -> Vec<Violation> {
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

// ── File Length ──

pub fn check_file_length(path: &str, content: &str) -> Vec<Violation> {
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

// ── One Table Per Storage Module (ADR-019) ──

pub fn check_one_table_per_storage(path: &str, content: &str) -> Vec<Violation> {
    // Only check actual storage modules in src/storage/
    let normalized = path.replace('\\', "/");
    if !normalized.starts_with("storage/") || !normalized.ends_with("_storage.rs") {
        return Vec::new();
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut tables = std::collections::HashSet::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") {
            continue;
        }

        // INSERT ... INTO tablename
        if trimmed.contains("INSERT ") && trimmed.contains(" INTO ") {
            if let Some(after) = trimmed.split(" INTO ").nth(1) {
                if let Some(word) = after.split_whitespace().next() {
                    let table = sanitize_table_name(word);
                    if !table.is_empty() && !is_sql_keyword(&table) {
                        tables.insert(table);
                    }
                }
            }
        }

        // UPDATE [OR ...] tablename SET
        if trimmed.contains("UPDATE ") && trimmed.contains(" SET") {
            if let Some(after) = trimmed.split("UPDATE ").nth(1) {
                if let Some(word) = after.split_whitespace().next() {
                    let table = sanitize_table_name(word);
                    if !table.is_empty() && !is_sql_keyword(&table) {
                        tables.insert(table);
                    }
                }
            }
        }

        // DELETE FROM tablename
        if let Some(table) = extract_sql_table(trimmed, "DELETE FROM ") {
            tables.insert(table);
        }

        // CREATE TABLE [IF NOT EXISTS] tablename
        if trimmed.contains("CREATE TABLE ") {
            let after = trimmed.split("CREATE TABLE ").nth(1).unwrap_or("");
            let after = after.strip_prefix("IF NOT EXISTS ").unwrap_or(after);
            if let Some(word) = after.split_whitespace().next() {
                let table = sanitize_table_name(word);
                if !table.is_empty() && !is_sql_keyword(&table) {
                    tables.insert(table);
                }
            }
        }

        // DROP TABLE [IF EXISTS] tablename
        if trimmed.contains("DROP TABLE ") {
            let after = trimmed.split("DROP TABLE ").nth(1).unwrap_or("");
            let after = after.strip_prefix("IF EXISTS ").unwrap_or(after);
            if let Some(word) = after.split_whitespace().next() {
                let table = sanitize_table_name(word);
                if !table.is_empty() && !is_sql_keyword(&table) {
                    tables.insert(table);
                }
            }
        }

        // ALTER TABLE tablename
        if let Some(table) = extract_sql_table(trimmed, "ALTER TABLE ") {
            tables.insert(table);
        }

        // JOIN tablename
        if let Some(table) = extract_sql_table(trimmed, "JOIN ") {
            if !is_sql_keyword(&table) {
                tables.insert(table);
            }
        }

        // FROM tablename (only in SELECT context)
        if trimmed.contains("FROM ") {
            let in_select = (i.saturating_sub(5)..=i).any(|j| {
                lines.get(j).map_or(false, |l| l.contains("SELECT"))
            });
            if in_select {
                if let Some(table) = extract_sql_table(trimmed, "FROM ") {
                    if !table.starts_with("sqlite_") && !is_sql_keyword(&table) {
                        tables.insert(table);
                    }
                }
            }
        }
    }

    // Filter out temporary migration tables
    let mut permanent_tables: Vec<String> = tables
        .into_iter()
        .filter(|t| !t.ends_with("_new"))
        .collect();

    let mut violations = Vec::new();
    if permanent_tables.len() > 1 {
        permanent_tables.sort();
        violations.push(Violation::error(
            path,
            1,
            format!(
                "Storage module references multiple tables: {}. \
                 ADR-019 requires exactly one table per storage module.",
                permanent_tables.join(", ")
            ),
        ));
    }

    violations
}

fn extract_sql_table(line: &str, keyword: &str) -> Option<String> {
    if !line.contains(keyword) {
        return None;
    }
    let after = line.split(keyword).nth(1)?;
    let word = after.split_whitespace().next()?;
    let table = sanitize_table_name(word);
    if table.is_empty() || is_sql_keyword(&table) {
        return None;
    }
    Some(table)
}

fn sanitize_table_name(raw: &str) -> String {
    raw.trim_matches(|c: char| c == '(' || c == ',' || c == ')' || c == '"' || c == ';' || c == '`')
        .to_lowercase()
}

fn is_sql_keyword(word: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "set", "where", "or", "and", "not", "null", "default", "primary", "key",
        "foreign", "references", "unique", "check", "autoincrement", "collate",
        "asc", "desc", "limit", "offset", "union", "intersect", "except",
        "all", "distinct", "as", "on", "using", "indexed", "notindexed",
        "inner", "outer", "left", "right", "full", "cross", "natural",
        "group", "order", "by", "having", "values", "select", "insert",
        "delete", "update", "create", "drop", "alter", "begin", "commit",
        "rollback", "savepoint", "release", "pragma", "vacuum", "analyze",
        "explain", "with", "recursive", "case", "when", "then", "else", "end",
        "cast", "exists", "in", "between", "like", "glob", "regexp", "match",
        "escape", "is", "isnull", "notnull", "each", "row", "of", "to",
        "do", "nothing", "conflict", "abort", "fail", "ignore", "replace",
    ];
    KEYWORDS.contains(&word)
}
