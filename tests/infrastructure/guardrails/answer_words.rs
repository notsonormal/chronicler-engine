//! Answer-words guardrails: every `post` route of the router has a row in the words table, and every request path that a template or the shell names matches a row.

use regex::Regex;

use crate::Violation;

fn line_of(content: &str, offset: usize) -> usize {
    content[..offset].matches('\n').count() + 1
}

/// A route without a row would answer a failure with no sentence of what was not done.
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
                    "the `post` route `{route}` has no row in `ANSWER_WORDS`. A new route adds its row in the same commit."
                ),
            ));
        }
    }
    violations
}

/// The layer finds a row by the matched route, written as the router writes it: a row with
/// another parameter name, or for a route that is gone, is never found.
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
                    "the row `{}` of `ANSWER_WORDS` is no route of the router, so the layer never finds it.",
                    &found[1]
                ),
            )
        })
        .collect()
}

/// `None` for a segment that is wholly a placeholder, which stands for any parameter.
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

/// A value that is wholly a placeholder names its route in a Rust string, so only the
/// `post` route check covers it.
pub fn check_request_paths_have_words(path: &str, content: &str, rows: &[&str]) -> Vec<Violation> {
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
                        "the `hx-{}` path `{value}` matches no row of `ANSWER_WORDS`. A control that sends a request needs the words of its failure.",
                        &found[1]
                    ),
                )
            })
        })
        .collect()
}
