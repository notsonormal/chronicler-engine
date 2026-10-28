//! Answer-words guardrails: every action route of the router has a row in the words table, and every request path that a template or the shell names matches a row.

use regex::Regex;

use crate::Violation;

fn line_of(content: &str, offset: usize) -> usize {
    content[..offset].matches('\n').count() + 1
}

/// Fails for a `post` route of the router that has no row in the words table.
///
/// A route without a row would answer a failure with no sentence of what was not done.
/// The check finds a `post` handler also in a chain such as `get(a).post(b)`.
pub fn check_post_routes_have_words(path: &str, source: &str, rows: &[&str]) -> Vec<Violation> {
    let post_handler = Regex::new(r"\bpost\(").expect("post handler");
    let mut offset = 0;
    let mut violations = Vec::new();
    for (index, chunk) in source.split(".route(").enumerate() {
        let start = offset;
        offset += chunk.len() + ".route(".len();
        if index == 0 || !post_handler.is_match(chunk) {
            continue;
        }
        let Some(route) = chunk.split('"').nth(1) else {
            continue;
        };
        if !rows.contains(&route) {
            violations.push(Violation::error(
                path,
                line_of(source, start),
                format!(
                    "the `post` route `{route}` has no row in `REFUSAL_WORDS`. A new action adds its row in the same commit."
                ),
            ));
        }
    }
    violations
}

/// Fails for a row of the words table that is no route of the router.
///
/// The layer finds a row by the matched route, written as the router writes it. A row with
/// another parameter name, or for a route that is gone, is never found. The violation names
/// the line of the row in the words source.
pub fn check_words_rows_are_router_routes(
    router_source: &str,
    words_path: &str,
    words_source: &str,
) -> Vec<Violation> {
    let any_route = Regex::new(r#"\.route\(\s*"([^"]+)""#).expect("route");
    let row = Regex::new(r#"RouteWords::new\("([^"]+)""#).expect("row");
    let routes: Vec<&str> = any_route
        .captures_iter(router_source)
        .map(|route| route.get(1).expect("route path").as_str())
        .collect();
    row.captures_iter(words_source)
        .filter(|found| !routes.contains(&&found[1]))
        .map(|found| {
            Violation::error(
                words_path,
                line_of(words_source, found.get(0).expect("whole match").start()),
                format!(
                    "the row `{}` of `REFUSAL_WORDS` is no route of the router, so the layer never finds it.",
                    &found[1]
                ),
            )
        })
        .collect()
}

/// A path segment as the matcher compares it: `None` for a segment that is wholly a
/// `{{ … }}` placeholder, which stands for any parameter, and the literal text with
/// each placeholder dropped for any other segment.
fn comparable_segment(segment: &str, placeholder: &Regex) -> Option<String> {
    let literal = placeholder.replace_all(segment, "");
    (!literal.is_empty() || segment.is_empty()).then(|| literal.into_owned())
}

fn path_matches_row(path: &str, row: &str, placeholder: &Regex) -> bool {
    let wanted: Vec<&str> = row.split('/').collect();
    let actual: Vec<&str> = path.split('/').collect();
    wanted.len() == actual.len()
        && wanted.iter().zip(&actual).all(|(wanted, actual)| {
            match comparable_segment(actual, placeholder) {
                None => wanted.starts_with(':'),
                Some(literal) => wanted.starts_with(':') || *wanted == literal,
            }
        })
}

/// Fails for an `hx-get` or `hx-post` path of a template or the shell that matches no row.
///
/// A `{{ … }}` in a path counts as a parameter, and a query string is dropped. A value that
/// is wholly a placeholder names its route in a Rust string, so only the `post` route check
/// covers it.
pub fn check_action_paths_have_words(path: &str, content: &str, rows: &[&str]) -> Vec<Violation> {
    let attribute = Regex::new(r#"hx-(get|post)=\\?"([^"\\]*)\\?""#).expect("hx attribute");
    let placeholder = Regex::new(r"\{\{[^}]*\}\}").expect("placeholder");
    attribute
        .captures_iter(content)
        .filter_map(|found| {
            let value = found[2].split('?').next().unwrap_or_default().to_string();
            let named = value.starts_with('/');
            let unmatched = !rows
                .iter()
                .any(|row| path_matches_row(&value, row, &placeholder));
            (named && unmatched).then(|| {
                Violation::error(
                    path,
                    line_of(content, found.get(0).expect("whole match").start()),
                    format!(
                        "the `hx-{}` path `{value}` matches no row of `REFUSAL_WORDS`. A control that sends a request needs the words of its failure.",
                        &found[1]
                    ),
                )
            })
        })
        .collect()
}
