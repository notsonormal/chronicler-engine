use syn::spanned::Spanned;
use syn::File;

use crate::Violation;

const MODULE_DOC_EXEMPTIONS: &[&str] = &[
    // Crate root and binary entry (purely structural)
    "lib.rs",
    "main.rs",
    // Test infrastructure (internal, no public API)
    "test_support/",
];

fn is_module_doc_exempt(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    MODULE_DOC_EXEMPTIONS
        .iter()
        .any(|exempt| normalized.contains(exempt))
}

fn points_to_system_md(doc_path: &str) -> bool {
    doc_path == "docs/architecture/system.md" || doc_path.ends_with("/architecture/system.md")
}

const SYSTEM_MD_EXEMPT: &[&str] = &[
    "model/",   // model tier IS architecture
    "storage/", // storage tier IS architecture
    "error.rs", // error taxonomy IS architecture
];

fn is_system_md_exempt(path: &str) -> bool {
    let normalized = path.replace("\\", "/");
    SYSTEM_MD_EXEMPT
        .iter()
        .any(|exempt| normalized.contains(exempt))
}

fn expected_doc_target(path: &str) -> Option<&'static str> {
    if path.starts_with("application/") {
        Some("docs/system/game_flow.md")
    } else if path == "engine/mod.rs" || path == "engine/logic.rs" {
        Some("docs/system/navigation.md")
    } else if path == "engine/trigger_eval.rs" {
        Some("docs/system/triggers.md")
    } else if path == "engine/state_diagnostics.rs" {
        Some("docs/architecture/invariants.md")
    } else if path.starts_with("engine/") {
        Some("docs/system/game_flow.md")
    } else if path == "model/character.rs" {
        Some("docs/system/character_state.md")
    } else if path == "model/trigger.rs" {
        Some("docs/system/triggers.md")
    } else if path == "model/agent.rs" {
        Some("docs/system/agent_system.md")
    } else if path.starts_with("model/llm")
        || path == "model/llm_backend.rs"
        || path == "model/llm_message.rs"
    {
        Some("docs/system/llm_processing.md")
    } else if path.starts_with("model/") {
        None // model tier IS architecture
    } else if path.starts_with("narrative/llm") || path.starts_with("narrative/llm_client") {
        Some("docs/system/llm_processing.md")
    } else if path.starts_with("narrative/prompt") {
        Some("docs/system/prompt_system.md")
    } else if path.starts_with("narrative/agents") {
        Some("docs/system/agent_system.md")
    } else if path.starts_with("narrative/text_check") {
        Some("docs/system/text_check.md")
    } else if path == "narrative/mod.rs" || path.starts_with("narrative/") {
        Some("docs/system/narration_engine.md")
    } else if path.starts_with("server/") {
        Some("docs/system/dashboard.md")
    } else if path.starts_with("bootstrap/") {
        Some("docs/system/startup.md")
    } else {
        None
    }
}

fn extract_doc_anchor_path(line: &str) -> Option<&str> {
    let start = line.find('[')? + 1;
    let end = line.find(']')?;
    line[start..end]
        .trim()
        .strip_prefix("DOC:")
        .map(|s| s.trim())
}

pub fn check_doc_standards(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    if path.ends_with("_tests.rs") || path.ends_with("_test.rs") {
        return violations;
    }

    if is_module_doc_exempt(path) {
        return violations;
    }

    let lines: Vec<&str> = content.lines().collect();

    // Check line 1: DOC anchor
    if lines.is_empty() || !lines[0].trim().starts_with("//! [DOC:") {
        violations.push(Violation::warn(
            path,
            1,
            format!("Module `{path}` lacks a module-level DOC anchor. Add `//! [DOC: docs/path/to/file.md]` at the top of the file."),
        ));
        return violations;
    }

    let anchor_path = extract_doc_anchor_path(lines[0]);

    if let Some(anchor) = anchor_path {
        if points_to_system_md(anchor) && !is_system_md_exempt(path) {
            violations.push(Violation::warn(
                path,
                1,
                format!("Module `{path}` points to `system.md` but should point to a domain-specific doc. Files outside model/storage tiers must use specific docs (e.g., `game_flow.md`, `navigation.md`)."),
            ));
        }
    }

    // Check line 2: module summary (must exist, must be //! but not another [DOC:])
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
                "Integration tests must use make_test_context_with_sqlite() for consistent SQLite testing.".to_string(),
            ));
        }
    }
    violations
}

fn check_no_std_thread(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

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
