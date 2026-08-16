//! Inherent impl locality guardrail: every inherent impl must live in the type's defining file or a folder named after the type.

use std::collections::HashMap;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ItemEnum, ItemImpl, ItemStruct, ItemUnion, Type};

use crate::Violation;

/// Checks all files in `src/` for inherent impls that violate the module-per-type rule.
///
/// An inherent impl is allowed only in the type's defining file or inside a folder named after the type.
/// `files` is a list of `(relative_path, content)` pairs. Paths are relative to `src/`.
pub fn check_inherent_impl_locality(files: &[(&str, &str)]) -> Vec<Violation> {
    let mut definitions: HashMap<String, String> = HashMap::new();

    for (path, content) in files {
        let ast = syn::parse_file(content).unwrap();
        let mut collector = TypeDefCollector::new(path.to_string());
        collector.visit_file(&ast);
        for (name, def_path) in collector.defs {
            definitions.entry(name).or_insert(def_path);
        }
    }

    let mut violations = Vec::new();
    for (path, content) in files {
        let ast = syn::parse_file(content).unwrap();
        let mut collector = ImplCollector::new(path, &definitions, &mut violations);
        collector.visit_file(&ast);
    }
    violations
}

struct TypeDefCollector {
    path: String,
    defs: Vec<(String, String)>,
}

impl TypeDefCollector {
    fn new(path: String) -> Self {
        Self {
            path,
            defs: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for TypeDefCollector {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        self.defs.push((node.ident.to_string(), self.path.clone()));
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        self.defs.push((node.ident.to_string(), self.path.clone()));
    }

    fn visit_item_union(&mut self, node: &'ast ItemUnion) {
        self.defs.push((node.ident.to_string(), self.path.clone()));
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if is_cfg_test(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }
}

struct ImplCollector<'a> {
    path: &'a str,
    definitions: &'a HashMap<String, String>,
    violations: &'a mut Vec<Violation>,
}

impl<'a> ImplCollector<'a> {
    fn new(
        path: &'a str,
        definitions: &'a HashMap<String, String>,
        violations: &'a mut Vec<Violation>,
    ) -> Self {
        Self {
            path,
            definitions,
            violations,
        }
    }

    fn path_ends_with_type_folder(&self, type_name: &str) -> bool {
        let snake = to_snake_case(type_name);
        self.path
            .rsplit_once('/')
            .map(|(dir, _)| dir == snake || dir.ends_with(&format!("/{snake}")))
            .unwrap_or(false)
    }
}

impl<'ast, 'a> Visit<'ast> for ImplCollector<'a> {
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        if node.trait_.is_some() {
            return;
        }
        let Some(type_name) = simple_type_name(&node.self_ty) else {
            return;
        };
        let Some(def_path) = self.definitions.get(&type_name) else {
            return;
        };

        if self.path == def_path || self.path_ends_with_type_folder(&type_name) {
            return;
        }

        let line = node.self_ty.span().start().line;
        self.violations.push(Violation::error(
            self.path,
            line,
            format!(
                "inherent impl for `{}` is here but the type is defined in `{}` (not in a `/{}/` folder)",
                type_name,
                def_path,
                to_snake_case(&type_name)
            ),
        ));
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if is_cfg_test(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }
}

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("cfg") {
            return false;
        }
        let mut is_test = false;
        let _ = attr.parse_nested_meta(|meta| {
            if cfg_predicate_includes_test(&meta) {
                is_test = true;
            }
            Ok(())
        });
        is_test
    })
}

/// Whether a `cfg` predicate makes a module test-only: `test` directly, or any
/// `all(...)`/`any(...)` conjunction containing `test`. `not(test)` and
/// `feature = "test"` are correctly rejected — the module is not test-gated.
fn cfg_predicate_includes_test(meta: &syn::meta::ParseNestedMeta) -> bool {
    if meta.path.is_ident("test") {
        return true;
    }
    if meta.path.is_ident("all") || meta.path.is_ident("any") {
        let mut found = false;
        let _ = meta.parse_nested_meta(|inner| {
            if cfg_predicate_includes_test(&inner) {
                found = true;
            }
            Ok(())
        });
        return found;
    }
    false
}

fn simple_type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        Type::Reference(reference) => simple_type_name(&reference.elem),
        Type::Paren(parenthesized) => simple_type_name(&parenthesized.elem),
        Type::Group(group) => simple_type_name(&group.elem),
        Type::Slice(slice) => simple_type_name(&slice.elem),
        Type::Array(array) => simple_type_name(&array.elem),
        Type::Ptr(pointer) => simple_type_name(&pointer.elem),
        _ => None,
    }
}

pub(crate) fn to_snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut result = String::with_capacity(name.len() + 4);
    for (i, c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev_lower = i > 0 && chars[i - 1].is_lowercase();
            let next_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
            let prev_upper = i > 0 && chars[i - 1].is_uppercase();
            if prev_lower || (prev_upper && next_lower) {
                result.push('_');
            }
            for lower in c.to_lowercase() {
                result.push(lower);
            }
        } else {
            result.push(*c);
        }
    }
    result
}
