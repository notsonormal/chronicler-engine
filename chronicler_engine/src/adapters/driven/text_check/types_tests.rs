#[test]
fn test_issue_kind_display() {
    use crate::application::ports::text_checker::IssueKind;

    assert_eq!(format!("{}", IssueKind::Spelling), "spell");
    assert_eq!(format!("{}", IssueKind::Grammar), "grammar");
    assert_eq!(format!("{}", IssueKind::Capitalization), "capitalization");
    assert_eq!(format!("{}", IssueKind::Style), "style");
    assert_eq!(format!("{}", IssueKind::Formatting), "formatting");
    assert_eq!(format!("{}", IssueKind::Other), "other");
}
