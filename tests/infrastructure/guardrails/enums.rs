//! Enum variant doc guardrail: every enum variant must carry `///` doc, OR the enum must be marked `/// [TRIVIAL_ENUM]` with all variants bare.

use syn::visit::Visit;
use syn::{File, ItemEnum};

use crate::Violation;

const TRIVIAL_MARKER: &str = "[TRIVIAL_ENUM]";

fn has_trivial_marker(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("doc") {
            return false;
        }
        match attr.meta.require_name_value() {
            Ok(expr) => match &expr.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => s.value().contains(TRIVIAL_MARKER),
                _ => false,
            },
            Err(_) => false,
        }
    })
}

fn has_doc(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("doc"))
}

struct EnumVisitor<'a> {
    file_path: &'a str,
    violations: &'a mut Vec<Violation>,
}

impl<'a> Visit<'a> for EnumVisitor<'a> {
    fn visit_item_enum(&mut self, node: &'a ItemEnum) {
        let trivial = has_trivial_marker(&node.attrs);
        for variant in &node.variants {
            let documented = has_doc(&variant.attrs);
            match (trivial, documented) {
                (false, false) => self.violations.push(Violation::error(
                    self.file_path,
                    variant.ident.span().start().line,
                    format!(
                        "Enum variant `{}::{}` lacks a `///` doc comment. \
                         Either document the variant or mark the enum with `/// [TRIVIAL_ENUM]` \
                         directly above the `enum` declaration if variants are self-documenting.",
                        node.ident, variant.ident
                    ),
                )),
                (true, true) => self.violations.push(Violation::error(
                    self.file_path,
                    variant.ident.span().start().line,
                    format!(
                        "Enum `{}` is marked `/// [TRIVIAL_ENUM]` but variant `{}` carries a `///` doc \
                         — remove either the marker or all variant docs.",
                        node.ident, variant.ident
                    ),
                )),
                _ => {}
            }
        }
    }
}

pub fn check_enum_variant_docs(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let ast: File = match syn::parse_file(content) {
        Ok(ast) => ast,
        Err(_) => return violations,
    };
    let mut visitor = EnumVisitor {
        file_path: path,
        violations: &mut violations,
    };
    visitor.visit_file(&ast);
    violations
}

#[test]
fn check_enum_variant_docs_trivial_marker_skips_check() {
    let src = r#"
/// [TRIVIAL_ENUM]
enum Direction { North, South, East }
"#;
    let v = check_enum_variant_docs("test.rs", src);
    assert!(v.is_empty(), "expected no violations, got {v:?}");
}

#[test]
fn check_enum_variant_docs_flags_missing_variant_docs() {
    let src = r#"
enum PhaseError { Cancelled, Failed }
"#;
    let v = check_enum_variant_docs("test.rs", src);
    assert_eq!(v.len(), 2, "expected 2 violations, got {v:?}");
}

#[test]
fn check_enum_variant_docs_accepts_documented_variants() {
    let src = r#"
enum PhaseError {
    /// Generation cancelled by user.
    Cancelled,
    /// Narrator LLM call failed.
    Failed,
}
"#;
    let v = check_enum_variant_docs("test.rs", src);
    assert!(v.is_empty(), "expected no violations, got {v:?}");
}

#[test]
fn check_enum_variant_docs_skips_empty_enum() {
    let src = r#"
enum Never {}
"#;
    let v = check_enum_variant_docs("test.rs", src);
    assert!(v.is_empty());
}

#[test]
fn check_enum_variant_docs_flags_trivial_with_variant_docs() {
    let src = r#"
/// [TRIVIAL_ENUM]
enum Color {
    /// Red hue.
    Red,
    Blue,
}
"#;
    let v = check_enum_variant_docs("test.rs", src);
    assert_eq!(v.len(), 1, "expected 1 conflict violation, got {v:?}");
}
