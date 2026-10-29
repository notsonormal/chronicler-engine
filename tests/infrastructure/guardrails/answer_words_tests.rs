//! Detection tests for the answer-words guardrails.

use crate::answer_words::{
    check_request_paths_have_words, check_post_routes_have_words,
    check_words_rows_are_router_routes,
};

const ROWS: [&str; 4] = [
    "/games",
    "/games/:id/rename",
    "/fragment/prompt-presets/new",
    "/prompt-presets/:id/activate",
];

const ROUTER: &str = r#"
    .route("/games", post(games::create))
    .route(
        "/games/:id/rename",
        post(games::rename),
    )
    .route("/fragment/games", get(games::list))
"#;

#[test]
fn test_check_post_routes_have_words_accepts_routes_with_rows() {
    assert!(check_post_routes_have_words("router.rs", ROUTER, &ROWS).is_empty());
}

#[test]
fn test_check_post_routes_have_words_rejects_a_post_route_with_no_row() {
    let without_rename = ["/games"];
    let violations = check_post_routes_have_words("router.rs", ROUTER, &without_rename);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("/games/:id/rename"));
    assert_eq!(
        violations[0].line, 3,
        "the violation names the line of the route"
    );
}

#[test]
fn test_check_post_routes_have_words_ignores_a_get_route() {
    let violations =
        check_post_routes_have_words("router.rs", ROUTER, &["/games", "/games/:id/rename"]);
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn test_check_request_paths_have_words_accepts_a_parameter_a_query_and_a_suffix_placeholder() {
    let template = r#"
        <button hx-post="/games/{{ id }}/rename">Rename</button>
        <button hx-get="/fragment/prompt-presets/new?type=system">Add</button>
        <button hx-post=\"/prompt-presets/{{ preset.id }}/activate{{ button.mode_query }}\">Use</button>
    "#;
    assert!(check_request_paths_have_words("template.rs", template, &ROWS).is_empty());
}

#[test]
fn test_check_request_paths_have_words_rejects_a_path_with_no_row() {
    let template = r#"<button hx-post="/games/{{ id }}/archive">Archive</button>"#;
    let violations = check_request_paths_have_words("template.rs", template, &ROWS);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("/games/{{ id }}/archive"));
}

#[test]
fn test_check_request_paths_have_words_rejects_a_suffix_that_names_another_route() {
    let template =
        r#"<button hx-post="/prompt-presets/{{ id }}/delete{{ query }}">Delete</button>"#;
    let violations = check_request_paths_have_words("template.rs", template, &ROWS);
    assert_eq!(violations.len(), 1, "{violations:?}");
}

#[test]
fn test_check_request_paths_have_words_skips_a_value_that_is_wholly_a_placeholder() {
    let template = r#"<form hx-post="{{ form_action }}"></form>"#;
    assert!(check_request_paths_have_words("template.rs", template, &ROWS).is_empty());
}

#[test]
fn test_check_request_paths_have_words_rejects_a_literal_segment_that_differs() {
    let template = r#"<button hx-get="/fragment/prompt-presets/old">Add</button>"#;
    let violations = check_request_paths_have_words("template.rs", template, &ROWS);
    assert_eq!(violations.len(), 1, "{violations:?}");
}

const WORDS_SOURCE: &str = r#"
    RouteWords::new("/games", "Game", "was not started"),
    RouteWords::new("/games/:key/rename", "Game", "was not renamed"),
"#;

#[test]
fn test_check_words_rows_are_router_routes_accepts_rows_that_the_router_names() {
    let words = r#"RouteWords::new("/games", "Game", "x"), RouteWords::new("/fragment/games", "form", "y")"#;
    assert!(check_words_rows_are_router_routes(ROUTER, "words.rs", words).is_empty());
}

#[test]
fn test_check_words_rows_are_router_routes_rejects_a_row_with_another_parameter_name() {
    let violations = check_words_rows_are_router_routes(ROUTER, "words.rs", WORDS_SOURCE);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("/games/:key/rename"));
    assert_eq!(violations[0].file, "words.rs");
    assert_eq!(
        violations[0].line, 3,
        "the violation names the line of the row"
    );
}

#[test]
fn test_check_post_routes_have_words_finds_a_post_handler_in_a_chain() {
    let router = r#"
    .route("/games", get(games::list).post(games::create))
    .route("/fragment/games", get(games::list))
"#;
    let violations = check_post_routes_have_words("router.rs", router, &[]);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("/games"));
    assert_eq!(violations[0].line, 2);
}
