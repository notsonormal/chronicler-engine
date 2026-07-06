//! Smoke tests covering uncovered startup branches in `bootstrap::run()`.

use std::sync::atomic::{AtomicU16, Ordering};

use chronicler_engine::adapters::driving::cli::{
    list_available_worlds, resolve_engine_data_path, scan_worlds, Args,
};
use chronicler_engine::bootstrap::run;
use chronicler_engine::error::EngineError;

static NEXT_PORT: AtomicU16 = AtomicU16::new(19001);

fn unique_port() -> u16 {
    // Skip well-known ports; 19001+ range avoids collisions with
    // tests/test_utils/server.rs (which uses 8080-range ports).
    NEXT_PORT.fetch_add(1, Ordering::SeqCst)
}

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
    // Branch (a) exercises the `list_available_worlds()` path directly:
    // succeeds regardless of seed state and either prints "Available worlds"
    // or "No worlds found".
    let result = list_available_worlds();
    assert!(
        result.is_ok(),
        "list_available_worlds must succeed; got: {:?}",
        result.err()
    );
}

#[test]
fn test_run_world_not_found_falls_back_or_errors() {
    // Branch (c): `--world __nonexistent__` enters the None arm of
    // `get_world`. The function then either returns Err (no worlds in db)
    // or falls back to `all_worlds[0]` and proceeds to persona lookup.
    // In the latter case, with a bogus persona it then hits branch (d).
    let port = unique_port();
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
        matches!(err, EngineError::Config(_)),
        "expected EngineError::Config variant for world/persona failure; got: {err:?}"
    );
    let msg = match err {
        EngineError::Config(m) => m,
        other => format!("{other}"),
    };
    let mentions_world = msg.contains("World") || msg.contains("world");
    let mentions_persona = msg.contains("Persona") || msg.contains("persona");
    assert!(
        mentions_world || mentions_persona,
        "expected world-fallback or persona-not-found error, got: {msg}"
    );
}

#[test]
fn test_run_persona_not_found_errors_cleanly() {
    // Branch (d): valid world + bogus persona returns Err with
    // "Persona '...' not found".
    let world_key = match first_available_world_key() {
        Some(k) => k,
        None => {
            panic!(
                "test_run_persona_not_found_errors_cleanly requires at least one seeded world \
                 in data/worlds/; run with `cargo run -- --list-worlds` to verify seed data exists"
            );
        }
    };
    let port = unique_port();
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
        matches!(err, EngineError::Config(_)),
        "expected EngineError::Config variant for persona failure; got: {err:?}"
    );
    let msg = match err {
        EngineError::Config(m) => m,
        other => format!("{other}"),
    };
    assert!(
        msg.contains("Persona '") && msg.contains("not found"),
        "expected persona-not-found error mentioning 'Persona' and 'not found', got: {msg}"
    );
}
