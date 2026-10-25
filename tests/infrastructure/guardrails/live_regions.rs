//! Live-region guardrail: each element of the dashboard shell that lists data kinds in `data-live` loads through the one load path.

use regex::Regex;

use crate::Violation;

/// The names of the numbers of `GET /changes` that a region may list.
const LIVE_KINDS: [&str; 8] = [
    "story", "options", "busy", "games", "worlds", "settings", "presets", "llm",
];

/// Checks the dashboard shell, `assets/index.html`. Each element with a `data-live`
/// attribute must have `hx-trigger="load, refresh"` and `hx-swap="morph:innerHTML"`,
/// no `every` trigger, and only known kinds. An element with a timer loads on its own
/// schedule instead of on a change, and a swap other than morph destroys the state
/// the load should keep. The shell must hold at least one live region, so a rename
/// of the attribute cannot leave the rule guarding nothing. Stage 3 of the refresh
/// plan extends the rule to every element with `hx-get`.
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
        let Some(live) = value_of("data-live") else {
            continue;
        };
        regions += 1;
        let line = content[..whole.start()].matches('\n').count() + 1;
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
        } else if trigger != "load, refresh" {
            violations.push(Violation::error(
                path,
                line,
                format!("a live region has `hx-trigger=\"{trigger}\"`. It must be `hx-trigger=\"load, refresh\"`."),
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
