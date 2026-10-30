//! Answer-words guardrails: every `post` route of the router has a row in the words table, every request path that a template or the shell names matches a row, and no `post` handler returns a plain `Html<String>`.

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

/// The `post` handlers whose result is content and not a request outcome, each with its
/// reason.
pub const HTML_ANSWER_ALLOWED: &[(&str, &str)] = &[
    (
        "test_saved_connection_handler",
        "its result is content that is not stored: a failed Test is a result, not a failed request",
    ),
    (
        "test_form_connection_handler",
        "its result is content that is not stored: a failed Test is a result, not a failed request",
    ),
];

/// A handler that returns `Html<String>` cannot answer an error status, so the answer
/// reader of the page cannot see its failure.
///
/// Not covered: a handler whose parameter list holds a `{` (a destructured
/// `Path(P { id })`), because the signature is cut at the first `{` and the return
/// type is then unread; a return type written with a path, such as
/// `axum::response::Html<String>`, because the type is compared as the exact text
/// `Html<String>`; and a handler that the router names through a closure.
pub fn check_post_handlers_can_answer_an_error(
    router_path: &str,
    router_source: &str,
    handler_files: &[(String, String)],
    allowed: &[(&str, &str)],
) -> Vec<Violation> {
    let post_handler = Regex::new(r"\bpost\(\s*(?:\w+::)*(\w+)\s*\)").expect("post handler");
    let mut offset = 0;
    let mut violations = Vec::new();
    for (index, chunk) in router_source.split(".route(").enumerate() {
        let start = offset;
        offset += chunk.len() + ".route(".len();
        if index == 0 {
            continue;
        }
        let Some(found) = post_handler.captures(chunk) else {
            continue;
        };
        let handler = &found[1];
        if allowed.iter().any(|(name, _)| name == &handler) {
            continue;
        }
        let line = line_of(router_source, start);
        let definition =
            Regex::new(&format!(r"\bfn\s+{handler}\s*\(")).expect("handler definition");
        let signature = handler_files.iter().find_map(|(_, content)| {
            definition.find(content).map(|at| {
                let rest = &content[at.start()..];
                rest[..rest.find('{').unwrap_or(rest.len())].to_string()
            })
        });
        let Some(signature) = signature else {
            violations.push(Violation::error(
                router_path,
                line,
                format!("the `post` handler `{handler}` has no definition under the HTTP adapter, so its return type is unread."),
            ));
            continue;
        };
        let returned = signature
            .rsplit_once("->")
            .map(|(_, returned)| returned.split_whitespace().collect::<String>())
            .unwrap_or_default();
        if returned == "Html<String>" {
            violations.push(Violation::error(
                router_path,
                line,
                format!(
                    "the `post` handler `{handler}` returns `Html<String>`, which cannot answer an error status, so the page cannot see its failure. Return a `Response` that carries the status, or list the handler in `HTML_ANSWER_ALLOWED` with its reason."
                ),
            ));
        }
    }
    violations
}
