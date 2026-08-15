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
    let files = vec![(
        "foo.rs",
        "struct Foo;\n#[cfg(test)]\nmod tests {\n    impl Foo {}\n}\n",
    )];
    let violations = check_inherent_impl_locality(&files);
    assert!(
        violations.is_empty(),
        "expected no violations, got {violations:?}"
    );
}
