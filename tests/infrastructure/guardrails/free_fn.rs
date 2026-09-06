//! Free fn location guardrail: top-level free fns must live in a folder named `mappers`, `utils`, `builders`, `test_support`, `bootstrap`, or `handlers`.

use std::path::Path;

use crate::Violation;

/// Allowlisted immediate-parent folder names for top-level free fns.
const ALLOWED_FREE_FN_FOLDERS: &[&str] = &[
    "mappers",
    "utils",
    "builders",
    "test_support",
    "bootstrap",
    "handlers",
];

/// Composition-root entry points that are filename-exempt regardless of folder.
const PATH_EXEMPT_FILES: &[&str] = &["main.rs"];

// TODO: This is a grandfathered list of files. Do not any add any others to this list.
//  The plan is to eventually get rid of them, only keeping `main.rs`.
const PATH_EXEMPT_PATHS: &[&str] = &[
    "application/generation/slot.rs",
    "application/pipeline/spawn.rs",
    "application/prompting/prompt_merge.rs",
    "application/prompting/sanitize.rs",
    "application/prompting/token_budget.rs",
];

/// Restricts top-level free functions to allowlisted category folders or exempt paths.
pub fn check_free_fn_location(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Test files are exempt — unit tests under src/ follow the `*_tests.rs` convention.
    if path.ends_with("_tests.rs") {
        return violations;
    }

    let file_name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if PATH_EXEMPT_FILES.contains(&file_name)
        || PATH_EXEMPT_PATHS.contains(&path.trim_start_matches("src/"))
    {
        return violations;
    }

    // Immediate-parent rule: the file's parent directory must be in the allowlist.
    // Ancestor match is not enough — resists deep-nesting gaming.
    let parent_dir = Path::new(path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if ALLOWED_FREE_FN_FOLDERS.contains(&parent_dir) {
        return violations;
    }

    // Parse with syn and flag every top-level free fn (any visibility, sync or async).
    let Ok(ast) = syn::parse_file(content) else {
        return violations; // let cargo's own compile step surface syntax errors
    };
    let allowed = ALLOWED_FREE_FN_FOLDERS.join(", ");
    for item in ast.items {
        if let syn::Item::Fn(f) = item {
            let line = f.sig.fn_token.span.start().line;
            violations.push(Violation::error(
                path,
                line,
                format!(
                    "free fn `{}` outside allowed category folder — relocate to method or move to a folder named {allowed}/",
                    f.sig.ident
                ),
            ));
        }
    }
    violations
}
