//! Structure guardrail tests: doc-anchor standards, mod.rs purity, no-std-thread, file length, and the new test module-header rule.

use syn::spanned::Spanned;
use syn::File;

use crate::Violation;

const MODULE_DOC_EXEMPTIONS: &[&str] = &["lib.rs", "main.rs", "test_support/"];

fn is_module_doc_exempt(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    MODULE_DOC_EXEMPTIONS
        .iter()
        .any(|exempt| normalized.contains(exempt))
}

// TODO: These system.md files no longer exists so specific guardrails (related to system.md)
//  need to be removed or at least updated
fn points_to_system_md(doc_path: &str) -> bool {
    doc_path == "docs/architecture/system.md" || doc_path.ends_with("/architecture/system.md")
}

const SYSTEM_MD_EXEMPT: &[&str] = &[
    "model/",    // model tier IS architecture
    "storage/",  // storage tier IS architecture
    "domain/",   // domain tier IS architecture
    "adapters/", // adapters tier IS architecture
    "error.rs",  // error taxonomy IS architecture
];

fn is_system_md_exempt(path: &str) -> bool {
    let normalized = path.replace("\\", "/");
    SYSTEM_MD_EXEMPT
        .iter()
        .any(|exempt| normalized.contains(exempt))
}

fn extract_doc_anchor_path(line: &str) -> Option<&str> {
    let start = line.find('[')? + 1;
    let end = line.find(']')?;
    line[start..end]
        .trim()
        .strip_prefix("DOC:")
        .map(|s| s.trim())
}

fn is_forbidden_doc_path(doc_path: &str) -> bool {
    doc_path == "docs/plans" || doc_path.starts_with("docs/plans/")
}

fn is_test_file(path: &str) -> bool {
    path.ends_with("_tests.rs")
        || path.ends_with("_test.rs")
        || path.starts_with("tests/")
        || path.starts_with("tests\\")
}

/// Enforces module-level doc-anchor standards on production files and rejects DOC anchors in test files.
pub fn check_doc_standards(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if is_test_file(path) {
        if let Some(first_line) = content.lines().next() {
            if first_line.trim().starts_with("//! [DOC:") {
                violations.push(Violation::error(
                    path,
                    1,
                    format!(
                        "Test file `{path}` has a DOC anchor. Test files must use a plain `//! <summary>` line only."
                    ),
                ));
            }
        }
        return violations;
    }

    if is_module_doc_exempt(path) {
        return violations;
    }

    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || !lines[0].trim().starts_with("//! [DOC:") {
        violations.push(Violation::warn(
            path,
            1,
            format!("Module `{path}` lacks a module-level DOC anchor. Add `//! [DOC: docs/diataxis/reference/<area>/<name>.md]` at the top of the file."),
        ));
        return violations;
    }

    let anchor_path = extract_doc_anchor_path(lines[0]);

    if let Some(anchor) = anchor_path {
        if !anchor.starts_with("docs/diataxis/reference/") || anchor.contains("/../") {
            violations.push(Violation::error(
                path,
                1,
                format!("Module `{path}` DOC anchor `{anchor}` must resolve under `docs/diataxis/reference/`."),
            ));
        }

        if is_forbidden_doc_path(anchor) {
            violations.push(Violation::error(
                path,
                1,
                format!(
                    "Module `{path}` DOC anchor `{anchor}` is not allowed. Anchors must point to domain docs, not plans (`docs/plans/*`)."
                ),
            ));
        }

        if points_to_system_md(anchor) && !is_system_md_exempt(path) {
            violations.push(Violation::warn(
                path,
                1,
                format!("Module `{path}` points to `system.md` but should point to a domain-specific doc. Files outside model/storage tiers must use specific docs (e.g., `game_flow.md`, `navigation.md`)."),
            ));
        }
    }

    if lines.len() < 2 {
        violations.push(Violation::warn(
            path,
            2,
            format!("Module `{path}` lacks a module summary. Add a `//!` summary line after the DOC anchor (e.g., `//! Character sheet data structures`)."),
        ));
        return violations;
    }

    let line2 = lines[1];
    if !line2.starts_with("//!") {
        violations.push(Violation::warn(
            path,
            2,
            format!("Module `{path}` lacks a module summary on line 2. Add a `//!` summary line after the DOC anchor."),
        ));
    } else if line2.trim().starts_with("//! [DOC:") {
        violations.push(Violation::warn(
            path,
            2,
            format!("Module `{path}` has a double DOC anchor. Line 2 must be a module summary, not another anchor."),
        ));
    } else if line2.trim() == "//!" {
        violations.push(Violation::warn(
            path,
            2,
            format!("Module `{path}` has an empty summary. Line 2 must contain meaningful module description."),
        ));
    }

    violations
}

// TODO: IS this actually catching all problems? Or is catching products in
//  just `src` and not `tests/`
/// Enforces mod.rs purity: only module declarations, imports, and module docs are allowed.
pub fn check_mod_purity(path: &str, _content: &str, ast: &File) -> Vec<Violation> {
    let mut violations = Vec::new();

    if !path.ends_with("mod.rs") {
        return violations;
    }

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
            _ => continue, // Allowed: Mod, Use, ForeignMod, Verbatim
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

/// Rejects legacy test-context helpers in integration tests.
pub fn check_no_legacy_test_context(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if !path.starts_with("integration/") {
        return violations;
    }

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        if line.contains("make_test_context(") && !line.contains("make_test_context_with_sqlite(") {
            violations.push(Violation::error(
                path,
                line_num + 1,
                "Integration tests must use SqliteTestAppBuilder for consistent SQLite testing; make_test_app_with_* helpers deleted.".to_string(),
            ));
        }
    }
    violations
}

fn check_no_std_thread(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if path.contains("mock.rs") || path.ends_with("_tests.rs") {
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

/// Flags `.rs` files that contain only comments and blank lines.
/// A line is "code" if, after trimming, it is non-empty and does not start
/// with `//`, `//!`, `/*`, `*`, or `*/`. Belt-and-suspenders with `syn`: also
/// require the file parses cleanly with zero items, so a file that opens a
/// block comment and never closes it (which would fool the text scan) is not
/// wrongly flagged.
pub fn check_empty_rust_file(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    let has_code_line = content.lines().any(|line| {
        let trimmed = line.trim_start();
        !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("/*")
            && !trimmed.starts_with("*")
    });

    if has_code_line {
        return violations;
    }

    // No code lines by text scan. Confirm with syn: an empty file with a
    // dangling block comment could parse to a stream of `None` tokens (Err)
    // rather than a clean empty file; in that case it's a real parse error
    // and another guardrail will surface it.
    let Ok(ast) = syn::parse_file(content) else {
        return violations;
    };
    if ast.items.is_empty() {
        violations.push(Violation::error(
            path,
            1,
            "File is empty: only comments and blank lines. Delete it or add real code.".to_string(),
        ));
    }

    violations
}

/// Exposes the internal no-std-thread check for use outside the standard walker.
pub fn check_no_std_thread_all(path: &str, content: &str) -> Vec<Violation> {
    check_no_std_thread(path, content)
}

/// Enforces a maximum of 2000 non-blank lines per file.
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

/// Test files must have a single-line `//!` summary on the first non-blank line.
///
/// Accepted shapes:
///   - `//! <summary>` on line 1 (no DOC anchor).
///   - `//! [DOC: <path>]` on line 1 + `//! <summary>` on line 2.
///
/// Multi-line summary blocks and continuation lines beyond the single summary
/// line are rejected.
/// Enforces a single-line `//!` module summary at the start of each test file.
pub fn check_test_module_header(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if path.ends_with("_test.rs") || path.ends_with("_tests.rs") {
        return violations;
    }

    let lines: Vec<&str> = content.lines().collect();

    let first_idx = lines.iter().position(|ln| !ln.trim().is_empty());
    let Some(first_idx) = first_idx else {
        violations.push(Violation::warn(
            path,
            1,
            format!("Test file `{path}` is empty. Add a `//! <summary>` header."),
        ));
        return violations;
    };

    let line1 = lines[first_idx];
    let l1_text = line1.strip_prefix("//!").unwrap_or("").trim();

    if !line1.trim_start().starts_with("//!") {
        violations.push(Violation::warn(
            path,
            first_idx + 1,
            format!(
                "Test file `{path}` lacks a `//!` module header on line {}. \
                 Add `//! <summary>` describing what this file tests.",
                first_idx + 1
            ),
        ));
        return violations;
    }

    let summary_line_idx = if l1_text.starts_with("[DOC:") {
        let Some(next_non_blank) = lines
            .iter()
            .enumerate()
            .skip(first_idx + 1)
            .find(|(_, ln)| !ln.trim().is_empty())
            .map(|(i, _)| i)
        else {
            violations.push(Violation::warn(
                path,
                first_idx + 2,
                format!("Test file `{path}` has a DOC anchor but no `//!` summary line after it."),
            ));
            return violations;
        };

        let next_line = lines[next_non_blank];
        let next_text = next_line.strip_prefix("//!").unwrap_or("").trim();

        if !next_line.trim_start().starts_with("//!") {
            violations.push(Violation::warn(
                path,
                next_non_blank + 1,
                format!("Test file `{path}` has a DOC anchor but no `//!` summary line after it."),
            ));
            return violations;
        }

        if next_text.is_empty() || next_text.len() < 6 {
            violations.push(Violation::warn(
                path,
                next_non_blank + 1,
                format!(
                    "Test file `{path}` has an empty or trivial summary. \
                     Describe what this file tests in plain language."
                ),
            ));
            return violations;
        }

        next_non_blank
    } else {
        if l1_text.is_empty() || l1_text.len() < 6 {
            violations.push(Violation::warn(
                path,
                first_idx + 1,
                format!(
                    "Test file `{path}` has an empty or trivial summary. \
                     Describe what this file tests in plain language."
                ),
            ));
            return violations;
        }
        first_idx
    };

    let next_line = lines.get(summary_line_idx + 1);
    if let Some(line) = next_line {
        if !line.trim().is_empty() && line.trim_start().starts_with("//!") {
            violations.push(Violation::warn(
                path,
                summary_line_idx + 2,
                format!(
                    "Test file `{path}` has a multi-line `//!` summary block. \
                     Multi-line summaries are not allowed; condense to a single line."
                ),
            ));
        }
    }

    violations
}
