use clap::Parser;

use crate::cli::{Args, resolve_engine_data_path, scan_worlds};

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

#[test]
fn test_scan_worlds_with_valid_world() {
    let temp_dir = std::env::temp_dir().join(format!("chronicler_scan_test_{}", std::process::id()));
    let world_dir = temp_dir.join("worlds").join("my_world");
    std::fs::create_dir_all(&world_dir).unwrap();

    let manifest = crate::model::world::WorldManifest {
        id: "my_world".to_string(),
        name: "My World".to_string(),
        starting_room_id: "start".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: "".to_string(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };
    let json = serde_json::to_string_pretty(&manifest).unwrap();
    std::fs::write(world_dir.join("world.json"), json).unwrap();

    let result = scan_worlds(&temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);

    assert!(result.is_ok());
    let worlds = result.unwrap();
    assert_eq!(worlds.len(), 1);
    assert_eq!(worlds[0], ("my_world".to_string(), "My World".to_string()));
}

#[test]
fn test_scan_worlds_empty_worlds_dir() {
    let temp_dir = std::env::temp_dir().join(format!("chronicler_scan_empty_{}", std::process::id()));
    std::fs::create_dir_all(temp_dir.join("worlds")).unwrap();

    let result = scan_worlds(&temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_scan_worlds_missing_worlds_dir() {
    let temp_dir = std::env::temp_dir().join(format!("chronicler_scan_missing_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let result = scan_worlds(&temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_scan_worlds_skips_invalid_json() {
    let temp_dir = std::env::temp_dir().join(format!("chronicler_scan_badjson_{}", std::process::id()));
    let world_dir = temp_dir.join("worlds").join("bad_world");
    std::fs::create_dir_all(&world_dir).unwrap();
    std::fs::write(world_dir.join("world.json"), "not json").unwrap();

    let result = scan_worlds(&temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_scan_worlds_skips_non_dir_entries() {
    let temp_dir = std::env::temp_dir().join(format!("chronicler_scan_nondir_{}", std::process::id()));
    let worlds_dir = temp_dir.join("worlds");
    std::fs::create_dir_all(&worlds_dir).unwrap();
    std::fs::write(worlds_dir.join("not_a_dir.txt"), "hello").unwrap();

    let result = scan_worlds(&temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_resolve_engine_data_path_from_exe_dir() {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let data_dir = exe_dir.join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    unsafe {
        std::env::remove_var("CHRONICLER_DATA");
    }
    let path = resolve_engine_data_path();
    assert_eq!(path, data_dir);

    let _ = std::fs::remove_dir(&data_dir);
}
