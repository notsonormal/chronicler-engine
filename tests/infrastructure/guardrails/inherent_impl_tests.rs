//! Tests for the inherent impl locality guardrail.

use crate::inherent_impl::*;

#[test]
fn test_same_file_is_clean() {
    let files = vec![("foo.rs", "struct Foo;\nimpl Foo {}\n")];
    let violations = check_inherent_impl_locality(&files);
    assert!(
        violations.is_empty(),
        "expected no violations, got {violations:?}"
    );
}

#[test]
fn test_split_in_named_folder_is_clean() {
    let files = vec![("foo.rs", "struct Foo;"), ("foo/bar.rs", "impl Foo {}")];
    let violations = check_inherent_impl_locality(&files);
    assert!(
        violations.is_empty(),
        "expected no violations, got {violations:?}"
    );
}

#[test]
fn test_split_in_wrong_folder_is_violation() {
    let files = vec![("foo.rs", "struct Foo;"), ("bar/baz.rs", "impl Foo {}")];
    let violations = check_inherent_impl_locality(&files);
    assert_eq!(
        violations.len(),
        1,
        "expected one violation, got {violations:?}"
    );
    assert!(violations[0].message.contains("Foo"));
}

#[test]
fn test_cross_folder_violation() {
    let files = vec![
        ("domain/foo.rs", "struct Foo;"),
        ("app/foo_impl.rs", "impl Foo {}"),
    ];
    let violations = check_inherent_impl_locality(&files);
    assert_eq!(
        violations.len(),
        1,
        "expected one violation, got {violations:?}"
    );
}

#[test]
fn test_trait_impl_is_ignored() {
    let files = vec![
        ("foo.rs", "struct Foo;"),
        ("bar/baz.rs", "trait Bar {}\nimpl Bar for Foo {}"),
    ];
    let violations = check_inherent_impl_locality(&files);
    assert!(
        violations.is_empty(),
        "expected no violations, got {violations:?}"
    );
}

#[test]
fn test_generic_self_type_is_normalized() {
    let files = vec![
        ("foo.rs", "struct Foo<T>(T);"),
        ("foo/bar.rs", "impl<'a> Foo<&'a str> {}"),
    ];
    let violations = check_inherent_impl_locality(&files);
    assert!(
        violations.is_empty(),
        "expected no violations, got {violations:?}"
    );
}

#[test]
fn test_cfg_test_block_is_skipped() {
    let files = vec![
        ("bar.rs", "struct Foo;"),
        ("foo.rs", "#[cfg(test)]\nmod tests {\n    impl Foo {}\n}\n"),
    ];
    let violations = check_inherent_impl_locality(&files);
    assert!(
        violations.is_empty(),
        "cfg(test) mod must be skipped, got {violations:?}"
    );
}

#[test]
fn test_cfg_all_test_block_is_skipped() {
    let files = vec![
        ("bar.rs", "struct Foo;"),
        (
            "foo.rs",
            "#[cfg(all(test, feature = \"x\"))]\nmod tests {\n    impl Foo {}\n}\n",
        ),
    ];
    let violations = check_inherent_impl_locality(&files);
    assert!(
        violations.is_empty(),
        "compound cfg(all(test, ...)) must be skipped, got {violations:?}"
    );
}

#[test]
fn test_cfg_any_test_block_is_skipped() {
    let files = vec![
        ("bar.rs", "struct Foo;"),
        (
            "foo.rs",
            "#[cfg(any(test, feature = \"x\"))]\nmod tests {\n    impl Foo {}\n}\n",
        ),
    ];
    let violations = check_inherent_impl_locality(&files);
    assert!(
        violations.is_empty(),
        "compound cfg(any(test, ...)) must be skipped, got {violations:?}"
    );
}

#[test]
fn test_cfg_feature_test_is_not_skipped() {
    let files = vec![
        ("bar.rs", "struct Foo;"),
        (
            "foo.rs",
            "#[cfg(feature = \"test\")]\nmod helper {\n    impl Foo {}\n}\n",
        ),
    ];
    let violations = check_inherent_impl_locality(&files);
    assert_eq!(
        violations.len(),
        1,
        "cfg(feature = \"test\") must NOT be skipped, got {violations:?}"
    );
}

#[test]
fn test_to_snake_case_simple() {
    assert_eq!(to_snake_case("Foo"), "foo");
    assert_eq!(to_snake_case("ActionPipeline"), "action_pipeline");
    assert_eq!(to_snake_case("DbPool"), "db_pool");
}

#[test]
fn test_to_snake_case_acronym_run() {
    // The `prev_upper && next_lower` rule inserts `_` only at the last
    // uppercase of an acronym run, so the run stays together.
    assert_eq!(to_snake_case("XMLHttpRequest"), "xml_http_request");
    assert_eq!(to_snake_case("APIKey"), "api_key");
    assert_eq!(to_snake_case("LLMMessage"), "llm_message");
}

#[test]
fn test_to_snake_case_edge_shapes() {
    assert_eq!(to_snake_case("A"), "a");
    assert_eq!(to_snake_case("ABC"), "abc");
}

#[test]
fn test_to_snake_case_digit_breaks_acronym_chain() {
    // A digit is neither upper nor lower case, so it breaks the acronym
    // detection: the uppercase after the digit gets no leading `_`.
    // No digit-containing type names exist in `src/` today; this locks in
    // current behaviour so a future addition surfaces the gap explicitly.
    assert_eq!(to_snake_case("Foo2Bar"), "foo2bar");
    assert_eq!(to_snake_case("OAuth2Token"), "o_auth2token");
}
