//! Detection tests for the legacy and seam-contract allowlist guardrail: a unit test lives in a listed seam contract, a `domain/model/` file, or a listed legacy file.

use crate::unit_test_allowlist::{check_unit_test_allowlist, is_case_declaration};
use crate::SourceFile;

fn source(path: &str, content: &str) -> SourceFile {
    SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    }
}

const UNIT_TEST_FILE: &str = "application/games/catalogue_tests.rs";
const UNIT_TEST_BODY: &str = "#[test]\nfn test_switch_keeps_the_game() {}\n";
const LISTED: &[&str] = &[UNIT_TEST_FILE];
const CONTRACT_FILE: &str = "adapters/driven/storage/transaction_tests.rs";
const CONTRACT_BODY: &str =
    "// CASE storage-1: a transaction lands whole\n#[test]\nfn test_transaction() {}\n";

#[test]
fn a_listed_legacy_file_passes() {
    let files = [source(UNIT_TEST_FILE, UNIT_TEST_BODY)];
    assert!(check_unit_test_allowlist(&files, LISTED, &[]).is_empty());
}

#[test]
fn an_unlisted_unit_test_file_is_reported() {
    let files = [source(UNIT_TEST_FILE, UNIT_TEST_BODY)];
    let violations = check_unit_test_allowlist(&files, &[], &[]);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("outside the one rule"));
}

#[test]
fn a_listed_seam_contract_passes() {
    let files = [source(CONTRACT_FILE, CONTRACT_BODY)];
    assert!(check_unit_test_allowlist(&files, &[], &[CONTRACT_FILE]).is_empty());
}

#[test]
fn an_unlisted_seam_contract_is_reported() {
    let files = [source(CONTRACT_FILE, CONTRACT_BODY)];
    let violations = check_unit_test_allowlist(&files, &[], &[]);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0]
            .message
            .contains("not on `SEAM_CONTRACT_FILES`")
    );
}

#[test]
fn a_stale_seam_contract_entry_is_reported() {
    let files = [source(UNIT_TEST_FILE, UNIT_TEST_BODY)];
    let violations = check_unit_test_allowlist(&files, LISTED, &[CONTRACT_FILE]);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0]
            .message
            .contains("seam-contract entry names no file")
    );
}

#[test]
fn a_pure_domain_file_passes() {
    let files = [source(
        "domain/model/utils/game_rules_tests.rs",
        "proptest! {\n    #[test]\n    fn prop_rules(x in 0..3u8) {}\n}\n",
    )];
    assert!(check_unit_test_allowlist(&files, &[], &[]).is_empty());
}

#[test]
fn a_test_file_without_a_test_passes() {
    let files = [source(
        "application/games/helper_tests.rs",
        "fn helper() {}\n",
    )];
    assert!(check_unit_test_allowlist(&files, &[], &[]).is_empty());
}

#[test]
fn a_stale_legacy_entry_is_reported() {
    let present = "application/games/other_tests.rs";
    let files = [source(present, UNIT_TEST_BODY)];
    let legacy = [UNIT_TEST_FILE, present];
    let violations = check_unit_test_allowlist(&files, &legacy, &[]);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("names no file"));
}

#[test]
fn the_tag_form_requires_an_id_and_a_description() {
    assert!(is_case_declaration(
        "// CASE storage-1: a transaction lands whole"
    ));
    assert!(is_case_declaration(
        "// CASE storage-2 [failure]: a failing write leaves no row"
    ));
    assert!(!is_case_declaration("// CASE storage-1"));
    assert!(!is_case_declaration("// CASE storage-1:"));
    assert!(!is_case_declaration("// CASE Storage-1: a transaction"));
    assert!(!is_case_declaration("// CASE storage_1: a transaction"));
    assert!(!is_case_declaration("// CASEstorage-1: a transaction"));
}

#[test]
fn the_tag_form_rejects_a_loose_label() {
    assert!(!is_case_declaration(
        "// CASE storage-2 [Failure]: a failing write leaves no row"
    ));
    assert!(!is_case_declaration(
        "// CASE storage-2 [fail ure]: a failing write leaves no row"
    ));
    assert!(!is_case_declaration(
        "// CASE storage-2 [failure: a failing write leaves no row"
    ));
}

#[test]
fn the_old_forms_are_not_declarations() {
    assert!(!is_case_declaration(
        "//! CASE storage-1: a transaction lands whole"
    ));
    assert!(!is_case_declaration("// CASE:storage-1"));
}

#[test]
fn a_loose_case_prefix_does_not_make_a_seam_contract() {
    let loose =
        "// CASE storage one: a transaction lands whole\n\n#[test]\nfn test_transaction() {}\n";
    let files = [source(CONTRACT_FILE, loose)];
    let violations = check_unit_test_allowlist(&files, &[], &[]);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(violations[0].message.contains("outside the one rule"));
}
