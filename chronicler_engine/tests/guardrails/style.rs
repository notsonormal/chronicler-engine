use syn::visit::Visit;
use syn::{ItemFn, Local};

use crate::Violation;

// ── Import Ordering ──

pub fn check_import_ordering(path: &str, content: &str) -> Vec<Violation> {
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

pub struct CfgTestTracker {
    inside_cfg_test: bool,
    brace_depth: i32,
}

impl CfgTestTracker {
    pub fn new() -> Self {
        Self {
            inside_cfg_test: false,
            brace_depth: 0,
        }
    }
}

impl Default for CfgTestTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CfgTestTracker {
    /// Process a line and return `true` if we are inside a `#[cfg(test)]` block.
    pub fn process_line(&mut self, line: &str) -> bool {
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

pub fn check_long_comment_runs(path: &str, content: &str) -> Vec<Violation> {
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

pub fn check_single_letter_vars(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let ast = syn::parse_file(content).unwrap();
    let mut visitor = SingleLetterVisitor {
        file_path: path,
        violations: &mut violations,
        fn_stmt_count: 0,
    };
    visitor.visit_file(&ast);
    violations
}
