//! Test hygiene guardrails: no `#[ignore]`d test outside `tests/llm/`, and no bare sleep in a test without a comment that gives its reason.

use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Meta, Token};

use crate::Violation;

/// Rejects every `#[ignore]`d fn, also through `cfg_attr`; the caller exempts `tests/llm/`.
/// An ignored test anywhere else hides a failure from the gate.
pub fn check_no_ignored_tests(path: &str, content: &str) -> Vec<Violation> {
    let ast = match syn::parse_file(content) {
        Ok(ast) => ast,
        Err(error) => return vec![parse_violation(path, "no ignored tests", error)],
    };
    let mut visitor = IgnoredTests::default();
    visitor.visit_file(&ast);
    visitor
        .found
        .into_iter()
        .map(|(name, line)| {
            Violation::error(
                path,
                line,
                format!(
                    "`{name}` is `#[ignore]`d. Fix the test or delete it; only the \
                     real-provider suite in `tests/llm/` ignores tests."
                ),
            )
        })
        .collect()
}

/// Rejects a `sleep(..)` call in a test that has no `//` comment on the line above it.
/// The caller picks the test files and exempts the wait helpers in `tests/test_utils/`.
pub fn check_sleep_has_reason(path: &str, content: &str) -> Vec<Violation> {
    let ast = match syn::parse_file(content) {
        Ok(ast) => ast,
        Err(error) => return vec![parse_violation(path, "sleep has a reason", error)],
    };
    let mut visitor = SleepCalls::default();
    visitor.visit_file(&ast);
    let lines: Vec<&str> = content.lines().collect();
    visitor
        .lines
        .into_iter()
        .filter(|&line| !has_comment_above(&lines, line))
        .map(|line| {
            Violation::error(
                path,
                line,
                "Bare sleep in a test. Wait on the condition with a helper in \
                 `tests/test_utils/` (`wait.rs`, `browser.rs`, `server.rs`, \
                 `htmx_settle.rs`), or put a `//` comment with the reason on the line above.",
            )
        })
        .collect()
}

fn parse_violation(path: &str, rule: &str, error: syn::Error) -> Violation {
    Violation::error(
        path,
        1,
        format!("the `{rule}` guardrail could not parse this file: {error}"),
    )
}

/// `line` is 1-based; the line above it must be a `//` comment.
fn has_comment_above(lines: &[&str], line: usize) -> bool {
    line >= 2
        && lines
            .get(line - 2)
            .is_some_and(|above| above.trim_start().starts_with("//"))
}

#[derive(Default)]
struct IgnoredTests {
    found: Vec<(String, usize)>,
}

impl<'ast> Visit<'ast> for IgnoredTests {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if let Some(attr) = node.attrs.iter().find(|attr| ignores_test(attr)) {
            self.found
                .push((node.sig.ident.to_string(), attr.span().start().line));
        }
        syn::visit::visit_item_fn(self, node);
    }
}

/// `#[ignore]`, `#[ignore = ".."]`, or `#[cfg_attr(<predicate>, .., ignore, ..)]`.
fn ignores_test(attr: &Attribute) -> bool {
    if attr.path().is_ident("ignore") {
        return true;
    }
    if !attr.path().is_ident("cfg_attr") {
        return false;
    }
    attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .map(|metas| {
            metas
                .iter()
                .skip(1)
                .any(|meta| meta.path().is_ident("ignore"))
        })
        .unwrap_or(false)
}

#[derive(Default)]
struct SleepCalls {
    lines: Vec<usize>,
}

impl<'ast> Visit<'ast> for SleepCalls {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(callee) = node.func.as_ref() {
            if callee
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "sleep")
            {
                self.lines.push(node.span().start().line);
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}
