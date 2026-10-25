//! Live-region guardrails: each element of the dashboard shell that lists data kinds in `data-live` loads through the one load path, and no script or handler writes a region by another path.

use regex::Regex;

use crate::Violation;

/// The names of the numbers of `GET /changes` that a region may list.
const LIVE_KINDS: [&str; 8] = [
    "story", "options", "busy", "games", "worlds", "settings", "presets", "llm",
];

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
        for kind in kinds.iter().filter(|kind| !LIVE_KINDS.contains(kind)) {
            violations.push(Violation::error(
                path,
                line,
                format!(
                    "a live region lists the unknown kind `{kind}`. The kinds are {LIVE_KINDS:?}."
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

/// The names of the region elements of the shell: the `id` (else the first class)
/// of each element with `data-live` or `hx-get`.
pub fn region_names(index_html: &str) -> Vec<String> {
    let start_tag = Regex::new(r"<[A-Za-z][A-Za-z0-9-]*\b([^>]*)>").expect("start tag pattern");
    let attribute = Regex::new(r#"([A-Za-z_:][A-Za-z0-9_:.-]*)(?:\s*=\s*"([^"]*)")?"#)
        .expect("attribute pattern");
    let mut names: Vec<String> = Vec::new();
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
        let name = value_of("id")
            .filter(|id| !id.is_empty())
            .or_else(|| value_of("class").and_then(|classes| classes.split_whitespace().next()));
        if let Some(name) = name {
            if !names.iter().any(|known| known == name) {
                names.push(name.to_string());
            }
        }
    }
    names
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
