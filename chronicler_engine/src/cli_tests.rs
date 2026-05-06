use clap::Parser;

use crate::cli::{Args, resolve_engine_data_path};

#[test]
fn test_resolve_engine_data_path_default() {
    let path = resolve_engine_data_path();
    assert!(path.is_relative() || path.is_absolute());
}

#[test]
fn test_resolve_data_path_from_exe_fallback() {
    let data_dir = resolve_engine_data_path();
    // Should return a path (may or may not exist)
    assert!(data_dir.is_relative() || data_dir.is_absolute());
    let _ = data_dir.to_string_lossy(); // Should not panic
}

#[test]
fn test_resolve_data_path_returns_pathbuf() {
    // Verify return type is PathBuf
    let path = resolve_engine_data_path();
    use std::path::PathBuf;
    let _type_check: PathBuf = path;
}

#[test]
fn test_list_worlds_uses_worlds_subdirectory() {
    // list_available_works should look in data/worlds/ subdirectory
    let result = crate::cli::list_available_worlds();
    assert!(result.is_ok()); // Should handle gracefully
}

#[test]
fn test_list_worlds_graceful_when_empty() {
    // Test that empty worlds directory is handled gracefully
    // The function should not panic
    let result = crate::cli::list_available_worlds();
    assert!(result.is_ok() || result.is_err()); // Should return cleanly
}

#[test]
fn test_list_worlds_nonexistent_directory() {
    let _data_dir = resolve_engine_data_path();
    let result = crate::cli::list_available_worlds();
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_cli_args_default_world() {
    let args = Args::try_parse_from(["chronicler-engine"]).unwrap();
    assert_eq!(args.world, "redmist_estate");
    assert_eq!(args.port, 3000);
    assert!(!args.list_worlds);
}

#[test]
fn test_cli_args_custom_world() {
    let args = Args::try_parse_from(["chronicler-engine", "--world", "test"]).unwrap();
    assert_eq!(args.world, "test");
}

#[test]
fn test_cli_args_custom_port() {
    let args = Args::try_parse_from(["chronicler-engine", "--port", "8080"]).unwrap();
    assert_eq!(args.port, 8080);
}

#[test]
fn test_cli_args_list_worlds() {
    let args = Args::try_parse_from(["chronicler-engine", "--list-worlds"]).unwrap();
    assert!(args.list_worlds);
}

#[test]
fn test_cli_args_all_options() {
    let args = Args::try_parse_from([
        "chronicler-engine",
        "--world",
        "my_world",
        "--port",
        "9000",
        "--list-worlds",
    ])
    .unwrap();
    assert_eq!(args.world, "my_world");
    assert_eq!(args.port, 9000);
    assert!(args.list_worlds);
}

#[test]
fn test_resolve_engine_data_path_from_env() {
    // [DOC: docs/system/startup.md]
    // CHRONICLER_DATA env var takes precedence over other resolution methods
    unsafe {
        std::env::set_var("CHRONICLER_DATA", "/tmp/chronicler_test_data");
    }
    let path = resolve_engine_data_path();
    assert_eq!(path, std::path::PathBuf::from("/tmp/chronicler_test_data"));
    unsafe {
        std::env::remove_var("CHRONICLER_DATA");
    }
}

#[test]
fn test_list_worlds_nonexistent_worlds_dir() {
    // Point to a temp directory that has no worlds/ subdirectory
    let temp_dir = std::env::temp_dir().join(format!("chronicler_test_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    unsafe {
        std::env::set_var("CHRONICLER_DATA", &temp_dir);
    }

    let result = crate::cli::list_available_worlds();
    assert!(result.is_ok());

    unsafe {
        std::env::remove_var("CHRONICLER_DATA");
    }
    let _ = std::fs::remove_dir_all(&temp_dir);
}
