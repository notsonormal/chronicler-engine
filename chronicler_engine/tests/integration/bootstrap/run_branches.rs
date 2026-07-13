//! Smoke tests covering uncovered startup branches in `bootstrap::run()`.

use chronicler_engine::adapters::driving::cli::{
    list_available_worlds, resolve_engine_data_path, scan_worlds, Args,
};
use chronicler_engine::bootstrap::run;
use chronicler_engine::error::EngineError;
use crate::test_utils::server::get_available_port;

fn cleanup_db_for_port(port: u16) {
    // `bootstrap::run` opens `<exe_parent>/chronicler_{port}.db` plus SQLite
    // WAL/SHM sidecars. Remove any stale files left by previous test runs so
    // migrations run against a fresh DB rather than re-applying ALTER TABLE
    // statements to an already-migrated schema (which surfaces as
    // "duplicate column name: persona_key" or transient "disk I/O error"
    // when WAL files from concurrent runs collide).
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(dir) = exe.parent() else { return };
    let base = format!("chronicler_{port}.db");
    let _ = std::fs::remove_file(dir.join(&base));
    let _ = std::fs::remove_file(dir.join(format!("{base}-wal")));
    let _ = std::fs::remove_file(dir.join(format!("{base}-shm")));
    let _ = std::fs::remove_file(dir.join(format!("{base}-journal")));
}

fn first_available_world_key() -> Option<String> {
    let worlds = scan_worlds(&resolve_engine_data_path()).ok()?;
    worlds.first().map(|(id, _)| id.clone())
}

#[test]
fn test_list_available_worlds_lists_seeded_worlds() {
    let result = list_available_worlds();
    assert!(
        result.is_ok(),
        "list_available_worlds must succeed; got: {:?}",
        result.err()
    );
}

#[test]
fn test_run_persona_not_found_after_world_fallback() {
    let port =
        get_available_port(3010, 3050).expect("port allocation failed for run_branches test");
    cleanup_db_for_port(port);
    let args = Args {
        world: "__nonexistent_world__".to_string(),
        persona: "__nonexistent_persona__".to_string(),
        list_worlds: false,
        port,
        settings_path: None,
    };
    let result = run(args);
    assert!(
        result.is_err(),
        "bogus world+persona should not succeed; got Ok"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(
            &err,
            EngineError::PersonaNotFound(key) if key == "__nonexistent_persona__"
        ),
        "expected EngineError::PersonaNotFound(\"__nonexistent_persona__\"), got: {err:?}"
    );
}

#[test]
fn test_run_persona_not_found_errors_cleanly() {
    let world_key = match first_available_world_key() {
        Some(k) => k,
        None => {
            panic!(
                "test_run_persona_not_found_errors_cleanly requires at least one seeded world \
                 in data/worlds/; run with `cargo run -- --list-worlds` to verify seed data exists"
            );
        }
    };
    let port =
        get_available_port(3010, 3050).expect("port allocation failed for run_branches test");
    cleanup_db_for_port(port);
    let args = Args {
        world: world_key,
        persona: "__nonexistent_persona__".to_string(),
        list_worlds: false,
        port,
        settings_path: None,
    };
    let result = run(args);
    assert!(result.is_err(), "bogus persona must error; got Ok");
    let err = result.unwrap_err();
    assert!(
        matches!(
            &err,
            EngineError::PersonaNotFound(key) if key == "__nonexistent_persona__"
        ),
        "expected EngineError::PersonaNotFound(\"__nonexistent_persona__\"), got: {err:?}"
    );
    let msg = err.to_string();
    assert!(
        msg.contains("Persona not found: __nonexistent_persona__"),
        "expected canonical display, got: {msg}"
    );
}
