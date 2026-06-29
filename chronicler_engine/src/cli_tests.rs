use crate::adapters::driving::cli::{resolve_engine_data_path, scan_worlds};

fn temp_dir(test_name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "chronicler_cli_test_{}_{}",
        test_name,
        std::process::id()
    ))
}

#[test]
fn test_resolve_engine_data_path_returns_path() {
    let path = resolve_engine_data_path();
    assert!(
        !path.as_os_str().is_empty(),
        "Should return a non-empty path"
    );
}

#[test]
fn test_scan_worlds_missing_dir() {
    let tmp = temp_dir("missing_dir");
    let _ = std::fs::create_dir_all(&tmp);
    let result = scan_worlds(&tmp).expect("scan should succeed");
    assert!(
        result.is_empty(),
        "Missing worlds dir should return empty vec"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_scan_worlds_bad_json() {
    let tmp = temp_dir("bad_json");
    let worlds_dir = tmp.join("worlds").join("bad_world");
    std::fs::create_dir_all(&worlds_dir).unwrap();
    std::fs::write(worlds_dir.join("world.json"), "not valid json").unwrap();

    let result = scan_worlds(&tmp).expect("scan should succeed");
    assert!(result.is_empty(), "Bad JSON should be skipped");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_scan_worlds_valid() {
    let tmp = temp_dir("valid");
    let worlds_dir = tmp.join("worlds").join("test_world");
    std::fs::create_dir_all(&worlds_dir).unwrap();
    std::fs::write(
        worlds_dir.join("world.json"),
        r#"{"id":"test_world","name":"Test World","description":"A test world","global_rules":[]}"#,
    )
    .unwrap();

    let result = scan_worlds(&tmp).expect("scan should succeed");
    assert_eq!(result.len(), 1, "Should find one valid world");
    assert_eq!(result[0].0, "test_world");
    assert_eq!(result[0].1, "Test World");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_scan_worlds_mixed() {
    let tmp = temp_dir("mixed");
    let worlds_dir = tmp.join("worlds");

    let valid_dir = worlds_dir.join("valid_world");
    std::fs::create_dir_all(&valid_dir).unwrap();
    std::fs::write(
        valid_dir.join("world.json"),
        r#"{"id":"valid_world","name":"Valid World","description":"A valid world","global_rules":[]}"#,
    )
    .unwrap();

    let bad_dir = worlds_dir.join("bad_world");
    std::fs::create_dir_all(&bad_dir).unwrap();
    std::fs::write(bad_dir.join("world.json"), "bad json").unwrap();

    let no_file_dir = worlds_dir.join("no_file");
    std::fs::create_dir_all(&no_file_dir).unwrap();

    let result = scan_worlds(&tmp).expect("scan should succeed");
    assert_eq!(result.len(), 1, "Should only find valid world");
    assert_eq!(result[0].0, "valid_world");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_list_available_worlds_empty() {
    let tmp = temp_dir("empty");
    std::fs::create_dir_all(&tmp).unwrap();

    let result = scan_worlds(&tmp).expect("scan should succeed");
    assert!(
        result.is_empty(),
        "Empty worlds dir should yield empty list"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}
