//! Live-region guardrails: each element of the dashboard shell that lists data kinds in `data-live` loads through the one load path, and no script or handler writes a region by another path.

use chronicler_engine::domain::model::change::ChangeKind;
use regex::Regex;

use crate::Violation;

/// The names of the numbers of `GET /changes` that a region may list.
fn live_kinds() -> [&'static str; 8] {
    ChangeKind::ALL.map(ChangeKind::token)
}

/// Checks the dashboard shell, `assets/index.html`. Every element with `hx-get` is a
/// live region: it has a `data-live` attribute. Each element with a `data-live`
/// attribute must have exactly `hx-trigger="refresh"` and `hx-swap="morph:innerHTML"`,
/// no `every` trigger, and only known kinds. An element with a timer loads on its own
/// schedule instead of on a change, and a swap other than morph destroys the state
/// the load should keep. The shell must hold at least one live region, so a rename
/// of the attribute cannot leave the rule guarding nothing.
pub fn check_live_regions_in_shell(path: &str, content: &str) -> Vec<Violation> {
    let start_tag = Regex::new(r"<[A-Za-z][A-Za-z0-9-]*\b([^>]*)>").expect("start tag pattern");
    let attribute = Regex::new(r#"([A-Za-z_:][A-Za-z0-9_:.-]*)(?:\s*=\s*"([^"]*)")?"#)
        .expect("attribute pattern");
    let mut violations = Vec::new();
    let mut regions = 0;

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
        let line = content[..whole.start()].matches('\n').count() + 1;
        let Some(live) = value_of("data-live") else {
            if value_of("hx-get").is_some() {
                violations.push(Violation::error(
                    path,
                    line,
                    "an element with `hx-get` has no `data-live`. Every element that loads a fragment is a live region: it lists the kinds of data it shows.",
                ));
            }
            continue;
        };
        regions += 1;
        let trigger = value_of("hx-trigger").unwrap_or("");
        if trigger
            .split(',')
            .any(|spec| spec.trim().starts_with("every"))
        {
            violations.push(Violation::error(
                path,
                line,
                format!("a live region has an `every` trigger (`{trigger}`). It loads when a number of its kinds changes, not on a timer."),
            ));
        } else if trigger != "refresh" {
            violations.push(Violation::error(
                path,
                line,
                format!("a live region has `hx-trigger=\"{trigger}\"`. It must be `hx-trigger=\"refresh\"`: only the ask loop starts a load, so every load has a record."),
            ));
        }
        let swap = value_of("hx-swap").unwrap_or("");
        if swap != "morph:innerHTML" {
            violations.push(Violation::error(
                path,
                line,
                format!("a live region has `hx-swap=\"{swap}\"`. It must be `hx-swap=\"morph:innerHTML\"`, so a load keeps what it does not change."),
            ));
        }
        let kinds: Vec<&str> = live.split_whitespace().collect();
        if kinds.is_empty() {
            violations.push(Violation::error(
                path,
                line,
                "a live region lists no kind in `data-live`.",
            ));
        }
        for kind in kinds.iter().filter(|kind| !live_kinds().contains(kind)) {
            violations.push(Violation::error(
                path,
                line,
                format!(
                    "a live region lists the unknown kind `{kind}`. The kinds are {:?}.",
                    live_kinds()
                ),
            ));
        }
    }

    if regions == 0 {
        violations.push(Violation::error(
            path,
            1,
            "the shell has no element with `data-live`, so this guardrail guards nothing.",
        ));
    }
    violations
}

/// One region element of the shell: an element with `data-live` or `hx-get`.
pub struct ShellRegion {
    /// The `id`, else the first class.
    pub name: String,
    /// `#id`, else `.class`: the selector the tests use.
    pub selector: String,
    /// The `hx-get` path.
    pub path: String,
    /// The tokens of `data-live`.
    pub kinds: Vec<String>,
}

/// The region elements of the shell, each name once.
pub fn regions_of_shell(index_html: &str) -> Vec<ShellRegion> {
    let start_tag = Regex::new(r"<[A-Za-z][A-Za-z0-9-]*\b([^>]*)>").expect("start tag pattern");
    let attribute = Regex::new(r#"([A-Za-z_:][A-Za-z0-9_:.-]*)(?:\s*=\s*"([^"]*)")?"#)
        .expect("attribute pattern");
    let mut regions: Vec<ShellRegion> = Vec::new();
    for tag in start_tag.captures_iter(index_html) {
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
        if value_of("data-live").is_none() && value_of("hx-get").is_none() {
            continue;
        }
        let id = value_of("id").filter(|id| !id.is_empty());
        let class = value_of("class").and_then(|classes| classes.split_whitespace().next());
        let (name, selector) = match (id, class) {
            (Some(id), _) => (id, format!("#{id}")),
            (None, Some(class)) => (class, format!(".{class}")),
            (None, None) => continue,
        };
        if regions.iter().any(|known| known.name == name) {
            continue;
        }
        regions.push(ShellRegion {
            name: name.to_string(),
            selector,
            path: value_of("hx-get").unwrap_or("").to_string(),
            kinds: value_of("data-live")
                .unwrap_or("")
                .split_whitespace()
                .map(str::to_string)
                .collect(),
        });
    }
    regions
}

/// The names of the region elements of the shell: the `id` (else the first class)
/// of each element with `data-live` or `hx-get`.
pub fn region_names(index_html: &str) -> Vec<String> {
    regions_of_shell(index_html)
        .into_iter()
        .map(|region| region.name)
        .collect()
}

/// One row of a table that lists the live regions of the shell by hand.
pub struct ListedRegion {
    pub selector: String,
    pub path: String,
    pub kinds: Vec<String>,
}

/// Checks a hand-written table of regions against the regions of the shell: each
/// region of the shell with `data-live` has a row with the same selector, `hx-get`
/// path and kinds, and each row names such a region. A page test that loops over the
/// table then covers every region the page has.
pub fn check_region_table_matches_shell(
    path: &str,
    index_html: &str,
    table: &[ListedRegion],
) -> Vec<Violation> {
    let sorted = |kinds: &[String]| {
        let mut kinds = kinds.to_vec();
        kinds.sort();
        kinds
    };
    let shell: Vec<ShellRegion> = regions_of_shell(index_html)
        .into_iter()
        .filter(|region| !region.kinds.is_empty())
        .collect();
    let mut violations = Vec::new();
    for region in &shell {
        let Some(row) = table.iter().find(|row| row.selector == region.selector) else {
            violations.push(Violation::error(
                path,
                1,
                format!(
                    "the shell has the live region `{}` and the table has no row for it.",
                    region.selector
                ),
            ));
            continue;
        };
        if row.path != region.path {
            violations.push(Violation::error(
                path,
                1,
                format!(
                    "the row of `{}` loads `{}`, and the shell loads `{}`.",
                    region.selector, row.path, region.path
                ),
            ));
        }
        if sorted(&row.kinds) != sorted(&region.kinds) {
            violations.push(Violation::error(
                path,
                1,
                format!(
                    "the row of `{}` lists the kinds {:?}, and the shell lists {:?}.",
                    region.selector, row.kinds, region.kinds
                ),
            ));
        }
    }
    for row in table {
        if !shell.iter().any(|region| region.selector == row.selector) {
            violations.push(Violation::error(
                path,
                1,
                format!(
                    "the table has a row for `{}`, and the shell has no such live region.",
                    row.selector
                ),
            ));
        }
    }
    violations
}

/// True when `text` holds a quoted string that names the region: an id, a class or
/// a selector that contains the name as a whole word.
fn names_a_region(text: &str, regions: &[String]) -> bool {
    let quoted = Regex::new(r#""([^"]*)"|'([^']*)'|`([^`]*)`"#).expect("quoted string pattern");
    quoted.captures_iter(text).any(|capture| {
        let literal = capture
            .iter()
            .skip(1)
            .flatten()
            .next()
            .map(|literal| literal.as_str())
            .unwrap_or("");
        regions.iter().any(|region| {
            let whole_word = Regex::new(&format!(
                r"(^|[^A-Za-z0-9_-]){}($|[^A-Za-z0-9_-])",
                regex::escape(region)
            ))
            .expect("region name pattern");
            whole_word.is_match(literal)
        })
    })
}

fn line_of(content: &str, offset: usize) -> usize {
    content[..offset].matches('\n').count() + 1
}

/// Checks one `assets/dashboard-*.js` script. A script must not write the markup of
/// a region element: a region shows what its load renders, and the only change a
/// script makes to it is to ask for that load. The check finds:
/// - `X.innerHTML = …`, `X.outerHTML = …` and `X.insertAdjacentHTML(…)`, where `X`
///   is a chain that names a region (`document.getElementById("story-log")`) or an
///   identifier that the same file declares from such a chain;
/// - `htmx.ajax(…)` whose call names a region, which swaps into it.
///
/// An element that reaches a helper as a parameter is not resolved, so a helper
/// that writes markup takes the element from a caller that does not name a region.
pub fn check_scripts_write_no_region(
    path: &str,
    script: &str,
    regions: &[String],
) -> Vec<Violation> {
    let write = Regex::new(
        r"([\w$]+(?:\([^)]*\))?(?:\.[\w$]+(?:\([^)]*\))?)*)\.(?:(?:innerHTML|outerHTML)\s*=(?:[^=]|$)|insertAdjacentHTML\()",
    )
    .expect("markup write pattern");
    let declaration =
        Regex::new(r"(?:const|let|var)\s+([\w$]+)\s*=\s*([^;]+);").expect("declaration pattern");
    let ajax = Regex::new(r"htmx\.ajax\(([^;]*)").expect("htmx.ajax pattern");
    let mut violations = Vec::new();

    for capture in write.captures_iter(script) {
        let whole = capture.get(0).expect("whole match");
        let chain = &capture[1];
        let names_region = if chain.contains('(') {
            names_a_region(chain, regions)
        } else {
            let root = chain.split('.').next().unwrap_or(chain);
            declaration
                .captures_iter(script)
                .filter(|decl| &decl[1] == root)
                .any(|decl| names_a_region(&decl[2], regions))
        };
        if names_region {
            violations.push(Violation::error(
                path,
                line_of(script, whole.start()),
                format!("`{chain}` is a region element, and a script writes its markup. A region shows what its own load renders: ask for the load instead."),
            ));
        }
    }
    for capture in ajax.captures_iter(script) {
        let whole = capture.get(0).expect("whole match");
        if names_a_region(&capture[1], regions) {
            violations.push(Violation::error(
                path,
                line_of(script, whole.start()),
                "a script swaps into a region with `htmx.ajax`. A region loads through its own `refresh` trigger.",
            ));
        }
    }
    violations
}

/// Checks one `assets/dashboard-*.js` script other than `dashboard-live.js`. A
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
/// and never steers the answer: no `HX-Retarget` header may name a region element,
/// and no handler may answer `HX-Refresh`. A region shows what its own load renders,
/// and a new active Game or a changed number loads it, so no page reload is needed.
pub fn check_handler_headers(path: &str, source: &str, regions: &[String]) -> Vec<Violation> {
    let retarget = Regex::new(r#"HX-Retarget"([^;]*)"#).expect("retarget pattern");
    let refresh = Regex::new(r#"(?i)"HX-Refresh""#).expect("refresh pattern");
    let mut violations: Vec<Violation> = retarget
        .captures_iter(source)
        .filter(|capture| names_a_region(&capture[1], regions))
        .map(|capture| {
            Violation::error(
                path,
                line_of(source, capture.get(0).expect("whole match").start()),
                "a handler answers `HX-Retarget` into a region element. A region shows what its own load renders.",
            )
        })
        .collect();
    violations.extend(refresh.find_iter(source).map(|found| {
        Violation::error(
            path,
            line_of(source, found.start()),
            "a handler answers `HX-Refresh`. The change numbers load the regions that show the data: a page reload is never needed.",
        )
    }));
    violations
}

/// Checks one template source: a file of the HTTP adapter that renders markup, or
/// the shell. An action changes only the element it acts on, so:
/// - no `hx-target` names a region element (`#story-log`, `.games-panel`, …);
/// - no `outerHTML` swap replaces a region element: an element that swaps
///   `outerHTML` and names a region as its target, or has no target and is itself a
///   region.
///
/// The check reads each start tag, including the ones in a Rust string with escaped
/// quotes.
pub fn check_template_targets(path: &str, content: &str, regions: &[String]) -> Vec<Violation> {
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
        let target_is_region =
            target.is_some_and(|value| names_a_region(&format!("\"{value}\""), regions));
        if target_is_region {
            violations.push(Violation::error(
                path,
                line,
                format!(
                    "`hx-target=\"{}\"` names a region element. An action changes only the element it acts on, or its own editor slot: the region loads the data.",
                    target.unwrap_or("")
                ),
            ));
        }
        let swaps_outer = value_of("hx-swap").is_some_and(|swap| swap.contains("outerHTML"));
        if swaps_outer && target.is_none() {
            let own_name = value_of("id").filter(|id| !id.is_empty()).or_else(|| {
                value_of("class").and_then(|classes| classes.split_whitespace().next())
            });
            if own_name.is_some_and(|name| regions.iter().any(|region| region == name)) {
                violations.push(Violation::error(
                    path,
                    line,
                    "a region element swaps `outerHTML` of itself. A region is never replaced: it loads with `morph:innerHTML`.",
                ));
            }
        }
    }
    violations
}

/// Checks one template source for out-of-band swaps. An element that shows data
/// belongs to one region and loads with it, so no answer carries a second element
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
                "a template has `hx-swap-oob`. No answer writes a second element out of band: each element that shows data belongs to one region and loads with it.",
            )
        })
        .collect()
}

/// A template file of the HTTP adapter that still renders a draft control without
/// a form marker, with the reason, and the stage of the plan that removes the row.
/// A row with no stage is a field-builder file: its literals hold no `<form`, and
/// the screen sweep checks where its fields render. The list only shrinks: a row
/// whose file renders no such control any more fails the guardrail.
pub struct DraftControlException {
    pub path: &'static str,
    pub reason: &'static str,
    pub removed_in_stage: Option<u8>,
}

pub const DRAFT_CONTROL_EXCEPTIONS: &[DraftControlException] = &[
    DraftControlException {
        path: "adapters/driving/http/games/templates/games.rs",
        reason: "the Rename and New Game forms are still inside the Games panel",
        removed_in_stage: Some(3),
    },
    DraftControlException {
        path: "adapters/driving/http/prompt_presets/templates/prompt_presets.rs",
        reason: "the preset Add forms are still inside the Prompt Presets panel",
        removed_in_stage: Some(3),
    },
    DraftControlException {
        path: "adapters/driving/http/builders/presets.rs",
        reason: "the preset Edit form still replaces its card inside the Prompt Presets panel",
        removed_in_stage: Some(3),
    },
    DraftControlException {
        path: "adapters/driving/http/worlds/templates/worlds.rs",
        reason: "the World editor is still inside the Worlds panel",
        removed_in_stage: Some(3),
    },
    DraftControlException {
        path: "adapters/driving/http/builders/forms.rs",
        reason: "the field builder: its literals hold no form, and the screen sweep checks where the fields render",
        removed_in_stage: None,
    },
];

/// The markers that say a literal renders outside every region: a form of the edit
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
/// region replaces what it renders, so a literal that holds a draft control passes
/// only if the same literal carries `data-dialog-form` (the edit dialog is outside
/// every region) or `data-shell-form` (a form of the shell).
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
                "a draft control is in a literal without `data-dialog-form` or `data-shell-form`. A load of a region would erase the draft: the form belongs in the edit dialog.",
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
/// and a permanent row (no stage) names a file whose literals hold no `<form`.
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
        if row.removed_in_stage.is_none() {
            let holds_form = sources().any(|(_, source)| {
                raw_string_literals(source)
                    .iter()
                    .any(|(_, literal)| literal.contains("<form"))
            });
            if holds_form {
                violations.push(Violation::error(
                    row.path,
                    1,
                    "a permanent exception names a file whose literals hold a `<form`: only a field builder may stay without a form marker",
                ));
            }
        }
    }
    violations
}
