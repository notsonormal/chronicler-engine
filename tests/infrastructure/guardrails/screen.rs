//! Screen guardrails: each element of the dashboard shell that holds a part of the screen is written by the one screen script, and no script or handler writes a part by another path.

use regex::Regex;

use crate::Violation;

fn shell_tags(content: &str) -> Vec<(usize, Vec<(String, String)>)> {
    let start_tag = Regex::new(r"<[A-Za-z][A-Za-z0-9-]*\b([^>]*)>").expect("start tag pattern");
    let attribute = Regex::new(r#"([A-Za-z_:][A-Za-z0-9_:.-]*)(?:\s*=\s*"([^"]*)")?"#)
        .expect("attribute pattern");
    start_tag
        .captures_iter(content)
        .map(|tag| {
            let whole = tag.get(0).expect("whole match");
            let attributes = attribute
                .captures_iter(&tag[1])
                .map(|attr| {
                    (
                        attr[1].to_string(),
                        attr.get(2)
                            .map(|value| value.as_str())
                            .unwrap_or("")
                            .to_string(),
                    )
                })
                .collect();
            (
                content[..whole.start()].matches('\n').count() + 1,
                attributes,
            )
        })
        .collect()
}

fn value_in<'a>(attributes: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(attr_name, _)| attr_name == name)
        .map(|(_, value)| value.as_str())
}

/// Checks the dashboard shell, `assets/index.html`. A part element has a `data-part`
/// attribute and is written by the screen load only: it has no `hx-get`, `hx-trigger`
/// or `hx-swap`. No element has `data-live` or `data-live-state`: the browser keeps no
/// table of kinds. Each `.tab-content` section holds at least one part, and the shell
/// holds at least one part, so a rename of the attribute cannot leave the rule guarding
/// nothing.
pub fn check_parts_in_shell(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut parts = 0;
    for (line, attributes) in shell_tags(content) {
        for gone in ["data-live", "data-live-state"] {
            if value_in(&attributes, gone).is_some() {
                violations.push(Violation::error(
                    path,
                    line,
                    format!("an element has `{gone}`. The browser has no table of kinds: the screen loads as one."),
                ));
            }
        }
        if value_in(&attributes, "data-part").is_none() {
            continue;
        }
        parts += 1;
        for forbidden in ["hx-get", "hx-trigger", "hx-swap"] {
            if value_in(&attributes, forbidden).is_some() {
                violations.push(Violation::error(
                    path,
                    line,
                    format!(
                        "a part element has `{forbidden}`. Only the screen load writes a part."
                    ),
                ));
            }
        }
    }
    if parts == 0 {
        violations.push(Violation::error(
            path,
            1,
            "the shell has no element with `data-part`, so this guardrail guards nothing.",
        ));
    }
    let section =
        Regex::new(r#"<div[^>]*class="tab-content[^"]*"[^>]*>"#).expect("section pattern");
    let starts: Vec<usize> = section
        .find_iter(content)
        .map(|found| found.start())
        .collect();
    for (index, start) in starts.iter().enumerate() {
        let end = starts.get(index + 1).copied().unwrap_or(content.len());
        if !content[*start..end].contains("data-part=") {
            violations.push(Violation::error(
                path,
                content[..*start].matches('\n').count() + 1,
                "a `.tab-content` section holds no part, so its tab would show nothing.",
            ));
        }
    }
    violations
}

/// One part element of the shell: an element with `data-part`.
pub struct ShellPart {
    /// The value of `data-part`.
    pub name: String,
    /// `#id`, else `.class`: the selector the tests use.
    pub selector: String,
}

/// The part elements of the shell, in document order.
pub fn parts_of_shell(index_html: &str) -> Vec<ShellPart> {
    shell_tags(index_html)
        .into_iter()
        .filter_map(|(_, attributes)| {
            let name = value_in(&attributes, "data-part")?.to_string();
            let id = value_in(&attributes, "id").filter(|id| !id.is_empty());
            let class = value_in(&attributes, "class")
                .and_then(|classes| classes.split_whitespace().next());
            let selector = match (id, class) {
                (Some(id), _) => format!("#{id}"),
                (None, Some(class)) => format!(".{class}"),
                (None, None) => return None,
            };
            Some(ShellPart { name, selector })
        })
        .collect()
}

/// The names under which a script can reach a part element: the `id` (else the
/// first class) of each element with `data-part`, and the `data-part` value itself,
/// which a script uses in `[data-part="settings"]`.
pub fn part_element_names(index_html: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for part in parts_of_shell(index_html) {
        for name in [part.selector[1..].to_string(), part.name] {
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

/// Checks the part names of the shell against the names that the server renders
/// (`SCREEN_PARTS`): each part of the shell names a renderer, each renderer has a
/// part in the shell, and each name appears once.
pub fn check_shell_parts_match_the_server(
    path: &str,
    index_html: &str,
    server_parts: &[&str],
) -> Vec<Violation> {
    let shell = parts_of_shell(index_html);
    let mut violations = Vec::new();
    for part in &shell {
        if !server_parts.contains(&part.name.as_str()) {
            violations.push(Violation::error(
                path,
                1,
                format!(
                    "the shell has the part `{}` and the server has no renderer for it.",
                    part.name
                ),
            ));
        }
        if shell.iter().filter(|other| other.name == part.name).count() > 1 {
            violations.push(Violation::error(
                path,
                1,
                format!("the shell names the part `{}` more than once.", part.name),
            ));
        }
    }
    for name in server_parts {
        if !shell.iter().any(|part| part.name == *name) {
            violations.push(Violation::error(
                path,
                1,
                format!(
                    "the server renders the part `{name}` and the shell has no element for it."
                ),
            ));
        }
    }
    violations
}

/// Checks the source of the stub server against the names that the server renders
/// (`SCREEN_PARTS`). The stub answers `GET /screen` with its own match on the part
/// name, in `screen_part_html`: each arm names a part of the server, and each part of
/// the server has an arm.
pub fn check_stub_answers_the_parts_of_the_server(
    path: &str,
    stub_source: &str,
    server_parts: &[&str],
) -> Vec<Violation> {
    let Some(start) = stub_source.find("async fn screen_part_html") else {
        return vec![Violation::error(
            path,
            1,
            "the stub has no `screen_part_html`, so this guardrail guards nothing.",
        )];
    };
    let body = &stub_source[start..];
    let body = &body[..body.find("\n}\n").unwrap_or(body.len())];
    let arm = Regex::new(r#"(?m)^\s*"([a-z-]+)"\s*=>"#).expect("match arm pattern");
    let answered: Vec<&str> = arm
        .captures_iter(body)
        .map(|capture| capture.get(1).expect("part name").as_str())
        .collect();
    let mut violations = Vec::new();
    for name in &answered {
        if !server_parts.contains(name) {
            violations.push(Violation::error(
                path,
                line_of(stub_source, start),
                format!("the stub answers the part `{name}` and the server renders no such part."),
            ));
        }
    }
    for name in server_parts {
        if !answered.contains(name) {
            violations.push(Violation::error(
                path,
                line_of(stub_source, start),
                format!("the server renders the part `{name}` and the stub does not answer it."),
            ));
        }
    }
    violations
}

/// One row of a table that lists the parts of the shell by hand.
pub struct ListedPart {
    pub selector: String,
    pub name: String,
}

/// Checks a hand-written table of parts against the parts of the shell: each part of
/// the shell has a row with the same selector and name, and each row names such a
/// part. A page test that loops over the table then covers every part the page has.
pub fn check_part_table_matches_shell(
    path: &str,
    index_html: &str,
    table: &[ListedPart],
) -> Vec<Violation> {
    let shell = parts_of_shell(index_html);
    let mut violations = Vec::new();
    for part in &shell {
        match table.iter().find(|row| row.selector == part.selector) {
            None => violations.push(Violation::error(
                path,
                1,
                format!(
                    "the shell has the part `{}` and the table has no row for it.",
                    part.selector
                ),
            )),
            Some(row) if row.name != part.name => violations.push(Violation::error(
                path,
                1,
                format!(
                    "the row of `{}` names the part `{}`, and the shell names `{}`.",
                    part.selector, row.name, part.name
                ),
            )),
            Some(_) => {}
        }
    }
    for row in table {
        if !shell.iter().any(|part| part.selector == row.selector) {
            violations.push(Violation::error(
                path,
                1,
                format!(
                    "the table has a row for `{}`, and the shell has no such part.",
                    row.selector
                ),
            ));
        }
    }
    violations
}

/// True when `text` holds a quoted string that names a part: an id, a class, a
/// `data-part` value or a selector that contains the name as a whole word, or a
/// selector on the attribute itself (`[data-part]`, `[data-part="games"]`), which
/// names every part.
fn names_a_part(text: &str, parts: &[String]) -> bool {
    let quoted = Regex::new(r#""([^"]*)"|'([^']*)'|`([^`]*)`"#).expect("quoted string pattern");
    quoted.captures_iter(text).any(|capture| {
        let literal = capture
            .iter()
            .skip(1)
            .flatten()
            .next()
            .map(|literal| literal.as_str())
            .unwrap_or("");
        literal.contains("[data-part")
            || parts.iter().any(|part| {
                let whole_word = Regex::new(&format!(
                    r"(^|[^A-Za-z0-9_-]){}($|[^A-Za-z0-9_-])",
                    regex::escape(part)
                ))
                .expect("part name pattern");
                whole_word.is_match(literal)
            })
    })
}

/// True when `expression` names a part, or uses an identifier that the script
/// declares (`const`, `let`, `var`) from an expression that does, up to three
/// declarations deep: `const root = el.closest(CONTAINERS)` reaches a part when
/// `CONTAINERS` is a selector that lists `[data-part]`.
fn resolves_to_part(expression: &str, script: &str, parts: &[String], depth: usize) -> bool {
    if names_a_part(expression, parts) {
        return true;
    }
    if depth == 0 {
        return false;
    }
    let identifier = Regex::new(r"[A-Za-z_$][\w$]*").expect("identifier pattern");
    let declaration =
        Regex::new(r"(?:const|let|var)\s+([\w$]+)\s*=\s*([^;]+);").expect("declaration pattern");
    identifier.find_iter(expression).any(|used| {
        declaration
            .captures_iter(script)
            .filter(|decl| &decl[1] == used.as_str())
            .any(|decl| resolves_to_part(&decl[2], script, parts, depth - 1))
    })
}

fn line_of(content: &str, offset: usize) -> usize {
    content[..offset].matches('\n').count() + 1
}

/// Checks one `assets/dashboard-*.js` script other than `dashboard-screen.js`, the
/// one script that writes a part (the morph, and the clear on leave). A script must
/// not write the content of a part element: a part shows what the screen load
/// renders, and the only change a script makes to it is to ask for that load. The
/// check finds:
/// - `X.innerHTML = …`, `X.outerHTML = …`, `X.textContent = …`, `X.innerText = …`,
///   and the calls `X.insertAdjacentHTML(…)`, `X.replaceChildren(…)`,
///   `X.appendChild(…)`, `X.append(…)`, `X.prepend(…)`, `X.insertBefore(…)`,
///   `X.replaceWith(…)`, where `X` is a chain that names a part, or an identifier
///   that the same file declares from an expression that does (a selector chain, a
///   `[data-part]` selector, a constant that lists one);
/// - `htmx.ajax(…)` whose call names a part, which swaps into it.
///
/// An element that reaches a helper as a parameter is not resolved, so a helper
/// that writes takes the element from a caller that does not name a part.
pub fn check_scripts_write_no_part(path: &str, script: &str, parts: &[String]) -> Vec<Violation> {
    let write = Regex::new(
        r"([\w$]+(?:\([^)]*\))?(?:\.[\w$]+(?:\([^)]*\))?)*)\.(?:(?:innerHTML|outerHTML|textContent|innerText)\s*=(?:[^=]|$)|(?:insertAdjacentHTML|replaceChildren|appendChild|append|prepend|insertBefore|replaceWith)\()",
    )
    .expect("content write pattern");
    let ajax = Regex::new(r"htmx\.ajax\(([^;]*)").expect("htmx.ajax pattern");
    let mut violations = Vec::new();

    for capture in write.captures_iter(script) {
        let whole = capture.get(0).expect("whole match");
        let chain = &capture[1];
        let names_part = if chain.contains('(') {
            resolves_to_part(chain, script, parts, 3)
        } else {
            let root = chain.split('.').next().unwrap_or(chain);
            resolves_to_part(root, script, parts, 3)
        };
        if names_part {
            violations.push(Violation::error(
                path,
                line_of(script, whole.start()),
                format!("`{chain}` is a part element, and a script writes its content. The screen load writes a part: ask for the load instead."),
            ));
        }
    }
    for capture in ajax.captures_iter(script) {
        let whole = capture.get(0).expect("whole match");
        if names_a_part(&capture[1], parts) {
            violations.push(Violation::error(
                path,
                line_of(script, whole.start()),
                "a script swaps into a part with `htmx.ajax`. A part is written by the screen load only.",
            ));
        }
    }
    violations
}

/// A script that writes into a part on purpose, with the reason. A row whose script
/// no longer writes a part fails the guardrail.
pub struct PartWriteException {
    pub path: &'static str,
    pub reason: &'static str,
}

/// Checks the exceptions of the part-write rule against the scripts they name: a
/// row whose script has no part write is stale.
pub fn check_part_write_exceptions_are_used(
    scripts: &[(String, String)],
    parts: &[String],
    exceptions: &[PartWriteException],
) -> Vec<Violation> {
    exceptions
        .iter()
        .filter(|row| {
            !scripts
                .iter()
                .filter(|(path, _)| path.ends_with(row.path))
                .any(|(path, script)| !check_scripts_write_no_part(path, script, parts).is_empty())
        })
        .map(|row| {
            Violation::error(
                row.path,
                1,
                format!(
                    "the exception `{}` is stale: the script writes no part. Remove the row.",
                    row.reason
                ),
            )
        })
        .collect()
}

/// Checks one `assets/dashboard-*.js` script other than `dashboard-screen.js`. A
/// script sends an action with `actionFetch`, which asks for the changes when the
/// request ends, so the result reaches the screen without waiting for the timer.
pub fn check_scripts_use_action_fetch(path: &str, script: &str) -> Vec<Violation> {
    let raw_fetch = Regex::new(r"(^|[^\w$.])fetch\(").expect("fetch pattern");
    script
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim_start().starts_with("//"))
        .filter(|(_, line)| raw_fetch.is_match(line))
        .map(|(index, _)| {
            Violation::error(
                path,
                index + 1,
                "a script calls `fetch(` directly. Send an action with `actionFetch(`, so the page asks for the changes when the request ends.",
            )
        })
        .collect()
}

/// Checks one handler file of the HTTP adapter. A handler answers its own element
/// and never steers the answer: no `HX-Retarget` header may name a part element,
/// and no handler may answer `HX-Refresh`. A part shows what the screen load renders,
/// and a new active Game or a changed number loads it, so no page reload is needed.
pub fn check_handler_headers(path: &str, source: &str, parts: &[String]) -> Vec<Violation> {
    let retarget = Regex::new(r#"HX-Retarget"([^;]*)"#).expect("retarget pattern");
    let refresh = Regex::new(r#"(?i)"HX-Refresh""#).expect("refresh pattern");
    let mut violations: Vec<Violation> = retarget
        .captures_iter(source)
        .filter(|capture| names_a_part(&capture[1], parts))
        .map(|capture| {
            Violation::error(
                path,
                line_of(source, capture.get(0).expect("whole match").start()),
                "a handler answers `HX-Retarget` into a part element. The screen load writes a part.",
            )
        })
        .collect();
    violations.extend(refresh.find_iter(source).map(|found| {
        Violation::error(
            path,
            line_of(source, found.start()),
            "a handler answers `HX-Refresh`. The revision loads the screen: a page reload is never needed.",
        )
    }));
    violations
}

/// Checks one template source: a file of the HTTP adapter that renders markup, or
/// the shell. An action changes only the element it acts on, so:
/// - no `hx-target` names a part element (`#story-log`, `.games-panel`, …);
/// - no `outerHTML` swap replaces a part element: an element that swaps
///   `outerHTML` and names a part as its target, or has no target and is itself a
///   part.
///
/// The check reads each start tag, including the ones in a Rust string with escaped
/// quotes.
pub fn check_template_targets(path: &str, content: &str, parts: &[String]) -> Vec<Violation> {
    let start_tag = Regex::new(r"<[A-Za-z][A-Za-z0-9-]*\b([^>]*)>").expect("start tag pattern");
    let attribute = Regex::new(r#"([A-Za-z_:][A-Za-z0-9_:.-]*)(?:\s*=\s*\\?"([^"\\]*)\\?")?"#)
        .expect("attribute pattern");
    let mut violations = Vec::new();

    for tag in start_tag.captures_iter(content) {
        let whole = tag.get(0).expect("whole match");
        let attributes: Vec<(String, String)> = attribute
            .captures_iter(&tag[1])
            .map(|attr| {
                (
                    attr[1].to_string(),
                    attr.get(2)
                        .map(|value| value.as_str())
                        .unwrap_or("")
                        .to_string(),
                )
            })
            .collect();
        let value_of = |name: &str| {
            attributes
                .iter()
                .find(|(attr_name, _)| attr_name == name)
                .map(|(_, value)| value.as_str())
        };
        let line = line_of(content, whole.start());
        let target = value_of("hx-target");
        let target_is_part =
            target.is_some_and(|value| names_a_part(&format!("\"{value}\""), parts));
        if target_is_part {
            violations.push(Violation::error(
                path,
                line,
                format!(
                    "`hx-target=\"{}\"` names a part element. An action changes only the element it acts on, or its own editor slot: the screen load writes the data.",
                    target.unwrap_or("")
                ),
            ));
        }
        let swaps_outer = value_of("hx-swap").is_some_and(|swap| swap.contains("outerHTML"));
        if swaps_outer && target.is_none() {
            let own_name = value_of("id").filter(|id| !id.is_empty()).or_else(|| {
                value_of("class").and_then(|classes| classes.split_whitespace().next())
            });
            if own_name.is_some_and(|name| parts.iter().any(|part| part == name)) {
                violations.push(Violation::error(
                    path,
                    line,
                    "a part element swaps `outerHTML` of itself. A part is never replaced: the screen load morphs it.",
                ));
            }
        }
    }
    violations
}

/// Checks one template source for out-of-band swaps. An element that shows data
/// belongs to one part and loads with it, so no answer carries a second element
/// to swap somewhere else: no `hx-swap-oob`.
pub fn check_no_out_of_band_swaps(path: &str, content: &str) -> Vec<Violation> {
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim_start().starts_with("//"))
        .filter(|(_, line)| line.contains("hx-swap-oob"))
        .map(|(index, _)| {
            Violation::error(
                path,
                index + 1,
                "a template has `hx-swap-oob`. No answer writes a second element out of band: each element that shows data belongs to one part and loads with it.",
            )
        })
        .collect()
}

/// A field-builder file of the HTTP adapter that renders a draft control without
/// a form marker, with the reason. Its literals hold no `<form`, and the screen
/// sweep checks where its fields render. A row whose file renders no such control
/// any more fails the guardrail.
pub struct DraftControlException {
    pub path: &'static str,
    pub reason: &'static str,
}

pub const DRAFT_CONTROL_EXCEPTIONS: &[DraftControlException] = &[DraftControlException {
    path: "adapters/driving/http/builders/forms.rs",
    reason: "the field builder: its literals hold no form, and the screen sweep checks where the fields render",
}];

/// The markers that say a literal renders outside every part: a form of the edit
/// dialog, and a form of the shell (the text-check preview).
const DRAFT_FORM_MARKERS: [&str; 2] = ["data-dialog-form", "data-shell-form"];

/// The raw string literals (`r"…"`, `r#"…"#`, …) of a Rust source, with the byte
/// offset of each one's content. The markup of the HTTP adapter sits in raw strings.
fn raw_string_literals(source: &str) -> Vec<(usize, &str)> {
    let bytes = source.as_bytes();
    let mut literals = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let starts_raw = bytes[index] == b'r'
            && (index == 0
                || !(bytes[index - 1].is_ascii_alphanumeric() || bytes[index - 1] == b'_'));
        if !starts_raw {
            index += 1;
            continue;
        }
        let mut cursor = index + 1;
        while cursor < bytes.len() && bytes[cursor] == b'#' {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'"' {
            index += 1;
            continue;
        }
        let hashes = cursor - index - 1;
        let content_start = cursor + 1;
        let closing = format!("\"{}", "#".repeat(hashes));
        match source[content_start..].find(&closing) {
            Some(length) => {
                literals.push((
                    content_start,
                    &source[content_start..content_start + length],
                ));
                index = content_start + length + closing.len();
            }
            None => break,
        }
    }
    literals
}

/// The tag with its template syntax cut out. A conditional attribute such as
/// `{% if busy %} disabled{% endif %}` or `{{ disabled(busy) }}` is no attribute
/// that holds for every render, so it must not count as one.
fn tag_without_template_syntax(tag: &str) -> String {
    let conditional =
        Regex::new(r"(?s)\{%-?\s*if\b.*?\{%-?\s*endif\s*-?%\}").expect("conditional pattern");
    let expression = Regex::new(r"(?s)\{\{.*?\}\}|\{%.*?%\}|\{#.*?#\}").expect("template pattern");
    let without_conditionals = conditional.replace_all(tag, "");
    expression
        .replace_all(&without_conditionals, "")
        .into_owned()
}

/// Checks one source file of the HTTP adapter. A draft control is a form control
/// whose value waits for a Save: an `input`, `textarea` or `select` that is not
/// hidden, a button, or always read-only or disabled, and that does not save on
/// change (its own `hx-trigger="change"`, or the one of its form). A load of a
/// part replaces what it renders, so a literal that holds a draft control passes
/// only if the same literal carries `data-dialog-form` (the edit dialog is outside
/// every part) or `data-shell-form` (a form of the shell).
pub fn check_draft_controls_only_in_dialog_forms(path: &str, source: &str) -> Vec<Violation> {
    let token = Regex::new(r"(?s)<form\b[^>]*>|</form>|<(?:input|textarea|select)\b[^>]*>")
        .expect("form control pattern");
    let always = Regex::new(r"\s(?:disabled|readonly)(?:[\s=/>]|$)").expect("always pattern");
    let mut violations = Vec::new();

    for (offset, literal) in raw_string_literals(source) {
        if DRAFT_FORM_MARKERS
            .iter()
            .any(|marker| literal.contains(marker))
        {
            continue;
        }
        let mut in_autosave_form = false;
        for found in token.find_iter(literal) {
            let tag = tag_without_template_syntax(found.as_str());
            if tag.starts_with("<form") {
                in_autosave_form = tag.contains(r#"hx-trigger="change""#);
                continue;
            }
            if tag == "</form>" {
                in_autosave_form = false;
                continue;
            }
            let saves_on_change = in_autosave_form || tag.contains(r#"hx-trigger="change""#);
            let holds_no_draft = [r#"type="hidden""#, r#"type="button""#, r#"type="submit""#]
                .iter()
                .any(|marker| tag.contains(marker))
                || always.is_match(&tag);
            if saves_on_change || holds_no_draft {
                continue;
            }
            violations.push(Violation::error(
                path,
                line_of(source, offset + found.start()),
                "a draft control is in a literal without `data-dialog-form` or `data-shell-form`. A load of a part would erase the draft: the form belongs in the edit dialog.",
            ));
        }
    }
    violations
}

/// Checks that a literal of a dialog fragment carries the title that names the
/// dialog: the edit dialog takes its accessible name from `edit-dialog-title`.
pub fn check_dialog_fragments_carry_the_title(path: &str, source: &str) -> Vec<Violation> {
    raw_string_literals(source)
        .into_iter()
        .filter(|(_, literal)| literal.contains("data-dialog-form"))
        .filter(|(_, literal)| !literal.contains(r#"id="edit-dialog-title""#))
        .map(|(offset, _)| {
            Violation::error(
                path,
                line_of(source, offset),
                "a literal with `data-dialog-form` has no element with `id=\"edit-dialog-title\"`: the dialog would have no name",
            )
        })
        .collect()
}

/// Checks the exceptions of the draft-control rule against the files they name: a
/// row whose file no longer renders a draft control outside a form marker is stale,
/// and a row names a file whose literals hold no `<form`.
pub fn check_draft_control_exceptions_are_used(
    files: &[(String, String)],
    exceptions: &[DraftControlException],
) -> Vec<Violation> {
    let mut violations = Vec::new();
    for row in exceptions {
        let sources = || files.iter().filter(|(path, _)| path == row.path);
        let needed = sources().any(|(path, source)| {
            !check_draft_controls_only_in_dialog_forms(path, source).is_empty()
        });
        if !needed {
            violations.push(Violation::error(
                row.path,
                1,
                format!(
                    "the exception `{}` is stale: the file has no draft control outside a form marker. Remove the row.",
                    row.reason
                ),
            ));
        }
        let holds_form = sources().any(|(_, source)| {
            raw_string_literals(source)
                .iter()
                .any(|(_, literal)| literal.contains("<form"))
        });
        if holds_form {
            violations.push(Violation::error(
                row.path,
                1,
                "an exception names a file whose literals hold a `<form`: only a field builder may stay without a form marker",
            ));
        }
    }
    violations
}

/// Checks one `assets/dashboard-*.js` script. A direct morph writes each part, so an
/// htmx swap event of a part never fires: a listener for `htmx:beforeSwap`,
/// `htmx:afterSwap` or `htmx:afterSettle` that names a part would never run. The
/// listener moves to an event of the screen loop (`screen:beforemorph`,
/// `screen:loaded`). The handler is read up to the next top-level statement.
pub fn check_scripts_listen_no_part_swaps(
    path: &str,
    script: &str,
    parts: &[String],
) -> Vec<Violation> {
    let listener =
        Regex::new(r#"addEventListener\(\s*"htmx:(?:beforeSwap|afterSwap|afterSettle)""#)
            .expect("swap listener pattern");
    let mut violations = Vec::new();
    for found in listener.find_iter(script) {
        let rest = &script[found.start()..];
        let end = rest.find("\n});").map(|at| at + 4).unwrap_or(rest.len());
        if names_a_part(&rest[..end], parts) {
            violations.push(Violation::error(
                path,
                line_of(script, found.start()),
                "a script listens for the htmx swap of a part. A direct morph fires no such event: listen for `screen:beforemorph` or `screen:loaded`.",
            ));
        }
    }
    violations
}
