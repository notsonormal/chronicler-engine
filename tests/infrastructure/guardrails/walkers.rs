//! Table-driven design-rule walkers: the four kinds, their row types, and the checks that run one row table.

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::{SourceFile, Violation};

/// Where a row's reason is written, named in every violation so a reader can
/// find the rule without opening the walker.
const ROW_TABLE: &str = "tests/infrastructure/guardrails/walker_rows.rs";

pub enum BannedName {
    /// A `static` item declaration; a `&'static str` lifetime is not an item.
    StaticItem,
    /// A name that starts with this prefix, in a type, a `use` tree, or an
    /// expression path.
    NamePrefix(&'static str),
    /// A name that holds this part, which covers a family of names that share one
    /// word (an `AnalysisAgent`, an `OptionsAgent`, an `AgentRunner`).
    NameContains(&'static str),
}

pub struct BannedNameRow {
    /// The design-rule number this row enforces.
    pub rule: u8,
    /// A path prefix relative to `src/`.
    pub scope: &'static str,
    pub banned: &'static [BannedName],
}

pub struct CallSiteRow {
    /// The design-rule number this row enforces.
    pub rule: u8,
    /// A path prefix relative to `src/`.
    pub scope: &'static str,
    /// The associated callees (`Type::method`) or bare method or free-function
    /// names (`hold`) this row guards: a rule about one type may name several
    /// of its methods.
    ///
    /// `Type::method` matches a path call (`Type::method(..)`) and a method
    /// call whose receiver ends in an identifier or field named the
    /// snake_case of `Type` (`active_game.get()`, `self.active_game.set(..)`).
    /// The method-call half needs no type information, so it relies on that
    /// naming: a receiver stored under another name is not matched.
    pub callees: &'static [&'static str],
    /// What may call it, relative to `src/`: an entry ending in `/` allows
    /// every file under that folder, any other entry allows one exact file.
    /// Empty allows no file.
    pub allowed: &'static [&'static str],
}

pub enum TypeCheckPosition {
    /// Struct fields, and the fields of struct variants.
    Fields,
    /// Function and method parameters.
    Parameters,
}

pub struct TypeCheckRow {
    /// The design-rule number this row enforces.
    pub rule: u8,
    /// A path prefix relative to `src/`.
    pub scope: &'static str,
    /// Banned types, matched on the type's last path segment.
    pub banned_types: &'static [&'static str],
    pub position: TypeCheckPosition,
}

pub struct CallCountRow {
    /// The design-rule number this row enforces.
    pub rule: u8,
    /// A path prefix relative to `src/`.
    pub scope: &'static str,
    /// Narrows the count to this file inside the scope.
    pub file: Option<&'static str>,
    /// An associated path suffix, or a method or free-function name.
    pub callee: &'static str,
    pub max_calls: usize,
}

fn row_violation(file: &str, line: usize, kind: &str, rule: u8, item: &str) -> Violation {
    Violation::error(
        file,
        line,
        format!(
            "{kind} violation of design rule {rule}: {item}, in `{file}`; the \
             rule's reason is on its row in `{ROW_TABLE}`"
        ),
    )
}

fn dead_path_violation(rule: u8, kind: &str, scope: &str, file: Option<&str>) -> Violation {
    let target = match file {
        Some(file) => format!("`{file}`"),
        None => format!("no file under `{scope}`"),
    };
    Violation::error(
        "src/",
        0,
        format!(
            "the {kind} walker row of design rule {rule} matches {target}: the row \
             guards no file, so point it at the code it guards or remove it"
        ),
    )
}

fn parse_violation(file: &SourceFile, error: syn::Error) -> Violation {
    Violation::error(
        &file.path,
        1,
        format!("the design-rule walkers could not parse this file: {error}"),
    )
}

fn in_scope(path: &str, scope: &str) -> bool {
    path.starts_with(scope)
}

/// Whether `path` is selected by an allowlist entry: a folder entry (ending in
/// `/`) covers everything under it, any other entry names one file.
fn allowed(path: &str, allowed: &[&str]) -> bool {
    allowed.iter().any(|entry| match entry.strip_suffix('/') {
        Some(_) => in_scope(path, entry),
        None => *entry == path,
    })
}

/// Runs one row over the files it selects, and reports the row when it selects none.
fn scan_row(
    files: &[SourceFile],
    kind: &str,
    rule: u8,
    scope: &str,
    file: Option<&str>,
    skip: impl Fn(&SourceFile) -> bool,
    on_file: impl Fn(&SourceFile, &syn::File, &mut Vec<Violation>),
) -> Vec<Violation> {
    let selected: Vec<&SourceFile> = match file {
        Some(only) => files
            .iter()
            .filter(|candidate| candidate.path == only)
            .collect(),
        None => files
            .iter()
            .filter(|candidate| in_scope(&candidate.path, scope))
            .collect(),
    };
    if selected.is_empty() {
        return vec![dead_path_violation(rule, kind, scope, file)];
    }

    let mut violations = Vec::new();
    for source in selected {
        if skip(source) {
            continue;
        }
        match syn::parse_file(&source.content) {
            Ok(ast) => on_file(source, &ast, &mut violations),
            Err(error) => violations.push(parse_violation(source, error)),
        }
    }
    violations
}

/// Runs the banned-name table: each row's scope must not name a banned name.
pub fn check_banned_name_rows(files: &[SourceFile], rows: &[BannedNameRow]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for row in rows {
        violations.extend(scan_row(
            files,
            "banned name",
            row.rule,
            row.scope,
            None,
            |_| false,
            |file, ast, out| {
                BannedNameVisitor {
                    path: &file.path,
                    row,
                    violations: out,
                }
                .visit_file(ast);
            },
        ));
    }
    violations
}

struct BannedNameVisitor<'a> {
    path: &'a str,
    row: &'a BannedNameRow,
    violations: &'a mut Vec<Violation>,
}

impl BannedNameVisitor<'_> {
    fn report(&mut self, ident: &syn::Ident, what: &str) {
        let item = format!("{what} `{ident}`");
        let violation = row_violation(
            self.path,
            ident.span().start().line,
            "banned name",
            self.row.rule,
            &item,
        );
        self.violations.push(violation);
    }

    fn check_names(&mut self, ident: &syn::Ident, what: &str) {
        let name = ident.to_string();
        let banned = self.row.banned.iter().any(|banned| match banned {
            BannedName::NamePrefix(prefix) => name.starts_with(prefix),
            BannedName::NameContains(part) => name.contains(part),
            BannedName::StaticItem => false,
        });
        if banned {
            self.report(ident, what);
        }
    }
}

impl<'ast> Visit<'ast> for BannedNameVisitor<'_> {
    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        let bans_static = self
            .row
            .banned
            .iter()
            .any(|banned| matches!(banned, BannedName::StaticItem));
        if bans_static {
            let ident = &node.ident;
            self.report(ident, "static item");
        }
        syn::visit::visit_item_static(self, node);
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if let Some(segment) = node.path.segments.last() {
            let ident = &segment.ident;
            self.check_names(ident, "type");
        }
        syn::visit::visit_type_path(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        for segment in &node.path.segments {
            let ident = &segment.ident;
            self.check_names(ident, "path");
        }
        syn::visit::visit_expr_path(self, node);
    }

    /// A trait bound, as in `Box<dyn AnalysisAgent>`: the trait's own path is not a
    /// type path, so the type and import walks do not reach it.
    fn visit_trait_bound(&mut self, node: &'ast syn::TraitBound) {
        if let Some(segment) = node.path.segments.last() {
            let ident = &segment.ident;
            self.check_names(ident, "trait bound");
        }
        syn::visit::visit_trait_bound(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let mut idents = Vec::new();
        use_tree_idents(&node.tree, &mut idents);
        for ident in idents {
            self.check_names(&ident, "import");
        }
        syn::visit::visit_item_use(self, node);
    }
}

fn use_tree_idents(tree: &syn::UseTree, found: &mut Vec<syn::Ident>) {
    match tree {
        syn::UseTree::Path(path) => {
            found.push(path.ident.clone());
            use_tree_idents(&path.tree, found);
        }
        syn::UseTree::Name(name) => found.push(name.ident.clone()),
        syn::UseTree::Rename(rename) => found.push(rename.ident.clone()),
        syn::UseTree::Group(group) => {
            for item in &group.items {
                use_tree_idents(item, found);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

/// Whether a callee path ends with the row's `::`-joined callee.
fn path_matches_callee(path: &syn::Path, callee: &str) -> bool {
    let parts: Vec<&str> = callee.split("::").collect();
    let actual: Vec<String> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    if actual.len() < parts.len() {
        return false;
    }
    actual[actual.len() - parts.len()..]
        .iter()
        .zip(parts)
        .all(|(found, want)| found == want)
}

/// The type of an associated callee: the segment before the last one.
fn associated_type(callee: &str) -> Option<&str> {
    let parts: Vec<&str> = callee.split("::").collect();
    parts.len().checked_sub(2).map(|index| parts[index])
}

/// The callee's method or function name: the last segment.
fn callee_name(callee: &str) -> &str {
    let parts: Vec<&str> = callee.split("::").collect();
    parts[parts.len() - 1]
}

/// How a type name appears when it is stored in a variable or a field.
fn snake_case(name: &str) -> String {
    let mut rendered = String::new();
    for (index, character) in name.chars().enumerate() {
        if character.is_uppercase() {
            if index > 0 {
                rendered.push('_');
            }
            rendered.extend(character.to_lowercase());
        } else {
            rendered.push(character);
        }
    }
    rendered
}

/// Whether a method call's receiver ends in an identifier or field named after `type_name`.
fn receiver_names_type(receiver: &syn::Expr, type_name: &str) -> bool {
    let expected = snake_case(type_name);
    match receiver {
        syn::Expr::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident == expected)
            .unwrap_or(false),
        syn::Expr::Field(field) => match &field.member {
            syn::Member::Named(name) => name == &expected,
            syn::Member::Unnamed(_) => false,
        },
        syn::Expr::Paren(paren) => receiver_names_type(&paren.expr, type_name),
        syn::Expr::Reference(reference) => receiver_names_type(&reference.expr, type_name),
        _ => false,
    }
}

struct CallCollector<'a> {
    callee: &'a str,
    hits: Vec<usize>,
}

impl CallCollector<'_> {
    fn new(callee: &str) -> CallCollector<'_> {
        CallCollector {
            callee,
            hits: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for CallCollector<'_> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(callee) = node.func.as_ref() {
            if path_matches_callee(&callee.path, self.callee) {
                self.hits.push(node.func.span().start().line);
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let callee = self.callee;
        let matches = match associated_type(callee) {
            Some(type_name) => {
                node.method == callee_name(callee) && receiver_names_type(&node.receiver, type_name)
            }
            None => node.method == callee_name(callee),
        };
        if matches {
            self.hits.push(node.method.span().start().line);
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Runs the call-site table: only what a row allows may call its callees.
pub fn check_call_site_rows(files: &[SourceFile], rows: &[CallSiteRow]) -> Vec<Violation> {
    let mut violations: Vec<Violation> = Vec::new();
    for row in rows {
        for callee in row.callees {
            violations.extend(scan_row(
                files,
                "call site",
                row.rule,
                row.scope,
                None,
                |file| allowed(&file.path, row.allowed),
                |file, ast, out| {
                    let mut collector = CallCollector::new(callee);
                    collector.visit_file(ast);
                    for line in collector.hits {
                        let item = format!("call to `{callee}`");
                        out.push(row_violation(
                            &file.path,
                            line,
                            "call site",
                            row.rule,
                            &item,
                        ));
                    }
                },
            ));
        }
    }
    violations
}

/// Runs the type-check table: no listed field or parameter holds a banned type.
pub fn check_type_check_rows(files: &[SourceFile], rows: &[TypeCheckRow]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for row in rows {
        violations.extend(scan_row(
            files,
            "type check",
            row.rule,
            row.scope,
            None,
            |_| false,
            |file, ast, out| {
                BannedTypeVisitor {
                    path: &file.path,
                    row,
                    violations: out,
                }
                .visit_file(ast);
            },
        ));
    }
    violations
}

struct BannedTypeVisitor<'a> {
    path: &'a str,
    row: &'a TypeCheckRow,
    violations: &'a mut Vec<Violation>,
}

impl BannedTypeVisitor<'_> {
    fn check_type(&mut self, ty: &syn::Type, position: &str) {
        let mut finder = BannedTypeFinder {
            banned: self.row.banned_types,
            hits: Vec::new(),
        };
        finder.visit_type(ty);
        for (line, name) in finder.hits {
            let item = format!("{position} type `{name}`");
            self.violations.push(row_violation(
                self.path,
                line,
                "type check",
                self.row.rule,
                &item,
            ));
        }
    }
}

impl<'ast> Visit<'ast> for BannedTypeVisitor<'_> {
    fn visit_field(&mut self, node: &'ast syn::Field) {
        if matches!(self.row.position, TypeCheckPosition::Fields) {
            self.check_type(&node.ty, "field");
        }
        syn::visit::visit_field(self, node);
    }

    fn visit_fn_arg(&mut self, node: &'ast syn::FnArg) {
        if matches!(self.row.position, TypeCheckPosition::Parameters) {
            if let syn::FnArg::Typed(typed) = node {
                self.check_type(&typed.ty, "parameter");
            }
        }
        syn::visit::visit_fn_arg(self, node);
    }
}

struct BannedTypeFinder<'a> {
    banned: &'a [&'a str],
    hits: Vec<(usize, String)>,
}

impl<'ast> Visit<'ast> for BannedTypeFinder<'_> {
    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if let Some(segment) = node.path.segments.last() {
            let name = segment.ident.to_string();
            if self.banned.contains(&name.as_str()) {
                self.hits.push((segment.ident.span().start().line, name));
            }
        }
        syn::visit::visit_type_path(self, node);
    }
}

/// Runs the call-count table: no function calls a row's callee more than the row allows.
pub fn check_call_count_rows(files: &[SourceFile], rows: &[CallCountRow]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for row in rows {
        violations.extend(scan_row(
            files,
            "call count",
            row.rule,
            row.scope,
            row.file,
            |_| false,
            |file, ast, out| {
                for (name, line, calls) in function_call_counts(ast, row.callee) {
                    if calls <= row.max_calls {
                        continue;
                    }
                    let item = format!(
                        "function `{name}` calls `{}` {calls} times (max {})",
                        row.callee, row.max_calls
                    );
                    out.push(row_violation(
                        &file.path,
                        line,
                        "call count",
                        row.rule,
                        &item,
                    ));
                }
            },
        ));
    }
    violations
}

/// Every function definition in `ast`, with the number of matching calls in its body.
fn function_call_counts(ast: &syn::File, callee: &str) -> Vec<(String, usize, usize)> {
    let mut counts = Vec::new();
    for item in &ast.items {
        match item {
            syn::Item::Fn(function) => {
                counts.push(count_in_block(
                    &function.sig.ident.to_string(),
                    function.sig.ident.span().start().line,
                    &function.block,
                    callee,
                ));
            }
            syn::Item::Impl(impl_block) => {
                for impl_item in &impl_block.items {
                    if let syn::ImplItem::Fn(function) = impl_item {
                        counts.push(count_in_block(
                            &function.sig.ident.to_string(),
                            function.sig.ident.span().start().line,
                            &function.block,
                            callee,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    counts
}

fn count_in_block(
    name: &str,
    line: usize,
    block: &syn::Block,
    callee: &str,
) -> (String, usize, usize) {
    let mut collector = CallCollector::new(callee);
    collector.visit_block(block);
    (name.to_string(), line, collector.hits.len())
}
