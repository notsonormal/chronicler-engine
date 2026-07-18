//! Nesting depth guardrail — reports function-body control-flow nesting depth violations (probe only; does not gate the build).

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ExprAsync, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprWhile, File};

use crate::Violation;

pub const MAX_NESTING_DEPTH: usize = 3;

pub fn check_nesting_depth(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let ast: File = match syn::parse_file(content) {
        Ok(ast) => ast,
        Err(_) => return violations,
    };
    let mut visitor = NestingVisitor {
        file_path: path,
        current_depth: 0,
        violations: &mut violations,
        wrapping_label: None,
    };
    visitor.visit_file(&ast);
    violations
}

struct ControlFlowDetector {
    found: bool,
}

impl<'ast> Visit<'ast> for ControlFlowDetector {
    fn visit_expr_if(&mut self, _: &'ast ExprIf) {
        self.found = true;
    }
    fn visit_expr_match(&mut self, _: &'ast ExprMatch) {
        self.found = true;
    }
    fn visit_expr_for_loop(&mut self, _: &'ast ExprForLoop) {
        self.found = true;
    }
    fn visit_expr_while(&mut self, _: &'ast ExprWhile) {
        self.found = true;
    }
    fn visit_expr_loop(&mut self, _: &'ast ExprLoop) {
        self.found = true;
    }

    // Do NOT descend into nested closures/async blocks — each is analysed
    // independently by NestingVisitor, and we don't want one closure's body
    // to be considered control flow for the enclosing closure.
    fn visit_expr_closure(&mut self, _: &'ast ExprClosure) {}
    fn visit_expr_async(&mut self, _: &'ast ExprAsync) {}
}

fn closure_body_has_control_flow(closure: &ExprClosure) -> bool {
    let mut detector = ControlFlowDetector { found: false };
    detector.visit_expr(&closure.body);
    detector.found
}

fn async_block_has_control_flow(node: &ExprAsync) -> bool {
    let mut detector = ControlFlowDetector { found: false };
    detector.visit_block(&node.block);
    detector.found
}

struct NestingVisitor<'a> {
    file_path: &'a str,
    current_depth: usize,
    violations: &'a mut Vec<Violation>,
    /// Label of the innermost enclosing closure/async block whose body
    /// contains control flow (so it bumped the depth). `None` outside any
    /// such wrapper. Used to give context for violations inside wrappers.
    wrapping_label: Option<&'static str>,
}

impl<'a> NestingVisitor<'a> {
    fn enter<F: FnOnce(&mut Self)>(&mut self, line: usize, label: &str, body: F) {
        let new_depth = self.current_depth + 1;
        if new_depth > MAX_NESTING_DEPTH {
            let mut msg = format!(
                "Nesting depth {new_depth} exceeds limit {MAX_NESTING_DEPTH} ({label} at this depth). \
                 Extract a helper or use early return / continue to flatten control flow."
            );
            if let Some(wrap) = self.wrapping_label {
                msg.push_str(&format!(" (inside {wrap} body)"));
            }
            self.violations
                .push(Violation::warn(self.file_path, line, msg));
        }
        self.current_depth = new_depth;
        body(self);
        self.current_depth -= 1;
    }
}

impl<'ast> Visit<'ast> for NestingVisitor<'_> {
    fn visit_expr_if(&mut self, node: &'ast ExprIf) {
        let line = node.if_token.span.start().line;
        self.enter(line, "if", |s| {
            // Default walker dispatches into visit_expr_if for `else if` (ExprIf in else_branch)
            // and into visit_expr_block for `else { ... }` (which does not increment depth).
            syn::visit::visit_expr_if(s, node);
        });
    }

    fn visit_expr_match(&mut self, node: &'ast ExprMatch) {
        let line = node.match_token.span.start().line;
        self.enter(line, "match", |s| {
            // The match itself counts once; arm bodies re-enter visit_expr_* naturally.
            syn::visit::visit_expr_match(s, node);
        });
    }

    fn visit_expr_for_loop(&mut self, node: &'ast ExprForLoop) {
        let line = node.for_token.span.start().line;
        self.enter(line, "for", |s| {
            syn::visit::visit_expr_for_loop(s, node);
        });
    }

    fn visit_expr_while(&mut self, node: &'ast ExprWhile) {
        let line = node.while_token.span.start().line;
        self.enter(line, "while", |s| {
            syn::visit::visit_expr_while(s, node);
        });
    }

    fn visit_expr_loop(&mut self, node: &'ast ExprLoop) {
        let line = node.loop_token.span.start().line;
        self.enter(line, "loop", |s| {
            syn::visit::visit_expr_loop(s, node);
        });
    }

    fn visit_expr_closure(&mut self, node: &'ast ExprClosure) {
        if closure_body_has_control_flow(node) {
            let line = node.span().start().line;
            let prev = self.wrapping_label;
            self.wrapping_label = Some("closure");
            self.enter(line, "closure", |s| {
                syn::visit::visit_expr_closure(s, node);
            });
            self.wrapping_label = prev;
        } else {
            syn::visit::visit_expr_closure(self, node);
        }
    }

    fn visit_expr_async(&mut self, node: &'ast ExprAsync) {
        if async_block_has_control_flow(node) {
            let line = node.async_token.span.start().line;
            let prev = self.wrapping_label;
            self.wrapping_label = Some("async block");
            self.enter(line, "async block", |s| {
                syn::visit::visit_expr_async(s, node);
            });
            self.wrapping_label = prev;
        } else {
            syn::visit::visit_expr_async(self, node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_nesting_depth_three_ifs_no_violation() {
        let src = r#"
fn f() {
    if a {
        if b {
            if c {
                let _ = 1;
            }
        }
    }
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(v.is_empty(), "3-deep chain must not violate, got {v:?}");
    }

    #[test]
    fn check_nesting_depth_four_ifs_one_violation() {
        let src = r#"
fn f() {
    if a {
        if b {
            if c {
                if d {
                    let _ = 1;
                }
            }
        }
    }
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert_eq!(
            v.len(),
            1,
            "4-deep chain must produce 1 violation, got {v:?}"
        );
        assert!(v[0].message.contains("Nesting depth 4"));
    }

    #[test]
    fn check_nesting_depth_else_if_chain_increments_each_level() {
        // if / else if / else if / else if / else if — depths 1, 2, 3, 4, 5.
        // The walker re-enters visit_expr_if for each `else if`, so depths 4 and 5
        // both produce violations. Two violations total.
        let src = r#"
fn f() {
    if a {
    } else if b {
    } else if c {
    } else if d {
    } else if e {
    }
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert_eq!(
            v.len(),
            2,
            "5-level else-if chain must produce 2 violations, got {v:?}"
        );
        assert!(v[0].message.contains("Nesting depth 4"));
        assert!(v[1].message.contains("Nesting depth 5"));
    }

    #[test]
    fn check_nesting_depth_match_arm_bodies_dont_increment_for_match() {
        // The match itself counts as 1; arm-body `if`s each add 1 from depth 1 → 2.
        // Three arm-bodies with `if`s: max depth is 2, no violation.
        let src = r#"
fn f(x: Option<i32>) -> i32 {
    match x {
        Some(0) => { if true { 1 } else { 2 } }
        Some(_) => { if true { 3 } else { 4 } }
        None => { if true { 5 } else { 6 } }
    }
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(
            v.is_empty(),
            "match arms with single inner if must not violate, got {v:?}"
        );
    }

    #[test]
    fn check_nesting_depth_nested_closure_counts_as_level() {
        // function body → closure (depth 1) → if (depth 2). No violation.
        let src = r#"
fn f() {
    let c = || {
        if true {
            let _ = 1;
        }
    };
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(
            v.is_empty(),
            "closure + single if must not violate, got {v:?}"
        );
    }

    #[test]
    fn check_nesting_depth_nested_closure_can_violate() {
        // function body → closure (depth 1) → if (2) → if (3) → if (4). Violation at 4.
        let src = r#"
fn f() {
    let c = || {
        if true {
            if true {
                if true {
                    let _ = 1;
                }
            }
        }
    };
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert_eq!(
            v.len(),
            1,
            "closure + 3-deep if chain must produce 1 violation, got {v:?}"
        );
        assert!(v[0].message.contains("Nesting depth 4"));
        assert!(v[0].message.contains("if"));
    }

    #[test]
    fn check_nesting_depth_function_item_entry_does_not_count() {
        // Five sibling functions, each with one if. None should interact; no violation.
        let src = r#"
fn a() { if true { let _ = 1; } }
fn b() { if true { let _ = 1; } }
fn c() { if true { let _ = 1; } }
fn d() { if true { let _ = 1; } }
fn e() { if true { let _ = 1; } }
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(v.is_empty(), "sibling fns must not stack depth, got {v:?}");
    }

    #[test]
    fn check_nesting_depth_expressions_block_and_try_do_not_count() {
        // `{ ... }` blocks and `?` operator must not count toward depth.
        let src = r#"
fn f(x: Option<i32>) -> i32 {
    let a = { let _y = 1; _y + 1 };
    let _b = x?;
    if true {
        let _ = 1;
    }
    a
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(
            v.is_empty(),
            "blocks and ? must not count toward depth, got {v:?}"
        );
    }

    #[test]
    fn check_nesting_depth_async_block_counts_as_level() {
        // function body → async block (depth 1) → if (depth 2). No violation.
        let src = r#"
fn f() {
    let _f = async {
        if true {
            let _ = 1;
        }
    };
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(
            v.is_empty(),
            "async block + single if must not violate, got {v:?}"
        );
    }

    #[test]
    fn check_nesting_depth_impl_method_starts_at_zero() {
        // A 3-deep if chain inside an impl method must not violate.
        let src = r#"
impl Foo {
    pub fn bar(&self) {
        if a {
            if b {
                if c {
                    let _ = 1;
                }
            }
        }
    }
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(v.is_empty(), "impl method body must reset depth, got {v:?}");
    }

    #[test]
    fn check_nesting_depth_predicate_closure_does_not_bump() {
        // for(1) → predicate-closure(at depth 2, no bump) → inner if chain via outer if at depth 3, not 4.
        // Predicate-only closures should NOT count toward depth.
        let src = r#"
fn f() {
    let xs: Vec<i32> = vec![1, 2, 3];
    for x in xs.iter().filter(|m| **m == 5) {
        if x > &0 {
            if x > &1 {
                let _ = 1;
            }
        }
    }
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(
            v.is_empty(),
            "predicate closure in for-loop must not bump depth, got {v:?}"
        );
    }

    #[test]
    fn check_nesting_depth_closure_with_if_still_bumps() {
        let src = r#"
fn f() {
    let _c = |x: i32| {
        if x > 0 {
            if x > 1 {
                if x > 2 {
                    let _ = 1;
                }
            }
        }
    };
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert_eq!(
            v.len(),
            1,
            "closure with control flow must still bump depth, got {v:?}"
        );
        assert!(v[0].message.contains("closure"));
    }

    #[test]
    fn check_nesting_depth_map_err_in_for_loop_no_violation() {
        // closure(1)→match(2)→for(3)→map_err-closure(skip).
        // With the tightened rule, the predicate `.map_err(|e| ...)` should not bump.
        let src = r#"
fn f(backend: &mut Backend) -> Result<(), ()> {
    let closure = |b: &mut Backend| match b {
        Backend::Sqlite => {
            for row in rows() {
                let _ = row.map_err(|e| format!("err: {e:?}"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    };
    closure(backend)
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(
            v.is_empty(),
            ".map_err closure inside for-loop inside match inside closure must not violate, got {v:?}"
        );
    }

    #[test]
    fn check_nesting_depth_nested_predicate_closure_doesnt_bump_outer() {
        let src = r#"
fn f() {
    let _ = |m: i32| some_iter().filter(|x: &i32| *x == m).count();
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(
            v.is_empty(),
            "outer closure with only nested predicate closure must not bump, got {v:?}"
        );
    }

    #[test]
    fn check_nesting_depth_async_with_no_control_flow_doesnt_bump() {
        let src = r#"
fn f() {
    let _ = async {
        let _ = some_future().await;
    };
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert!(
            v.is_empty(),
            "async block with no control flow must not bump, got {v:?}"
        );
    }

    #[test]
    fn check_nesting_depth_async_with_control_flow_still_bumps() {
        let src = r#"
fn f() {
    let _ = async {
        if true {
            if true {
                if true {
                    let _ = 1;
                }
            }
        }
    };
}
"#;
        let v = check_nesting_depth("test.rs", src);
        assert_eq!(
            v.len(),
            1,
            "async block with control flow must still bump, got {v:?}"
        );
        assert!(v[0].message.contains("async block"));
    }
}
