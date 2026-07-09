# Fix run_branches.rs port allocation race

Pre-existing flaky bootstrap tests (SQLite file-system race). Not caused by A7 changes — A7 doesn't touch migration logic or test infra for `bootstrap::run()`.

## Summary

`tests/integration/bootstrap/run_branches.rs` uses a per-process `static NEXT_PORT: AtomicU16 = 19001` to allocate a port for each test. When nextest runs multiple test binaries in parallel, every process starts at 19001 and races on `<exe_parent>/chronicler_19001.db` (plus WAL/SHM sidecars), producing "disk I/O error" during migration. Switch to the existing cross-process port allocator `test_utils::server::get_available_port(3010, 3050)` (driven by `tests/test_config.json`) so each parallel test binary gets a unique port → unique DB filename.

Verification surface: 339 test sites use `:memory:`; only these 3 tests in `run_branches.rs` touch file-based DBs (because they exercise the real `bootstrap::run()` path which opens `<exe_parent>/chronicler_{port}.db` per `src/bootstrap/run.rs:63`). A7 work confirmed unrelated — touched `application_service.rs`, `game_service.rs`, `context.rs`, test files. None touch `bootstrap/run.rs` or `run_branches.rs`.

## Key Changes

- `tests/integration/bootstrap/run_branches.rs` — replace `static NEXT_PORT` + `unique_port()` with `get_available_port` call. Keep `cleanup_db_for_port` as defensive belt-and-braces.
- No changes to `src/`, no changes to `bootstrap::run`, no changes to `DbPool`.

## Implementation

### Phase 1: Switch run_branches.rs to shared port allocator

- [ ] #### Task 1.1: Use cross-process port allocator (0.5 SP)
  - Remove `static NEXT_PORT: AtomicU16` and `fn unique_port()` from `tests/integration/bootstrap/run_branches.rs`.
  - At each call site (currently lines 61, 105), replace `let port = unique_port();` with `let port = chronicler_engine::test_utils::server::get_available_port(3010, 3050).expect("port allocation failed for run_branches test");`.
  - Keep `cleanup_db_for_port(port)` call unchanged — defends against same-binary re-runs (e.g. `--retries 3`) leaving WAL/SHM sidecars behind.
  - **Validate (all four):**
    1. `cargo nextest run -p chronicler_engine --test integration bootstrap::` green 5× consecutive runs.
    2. `cargo nextest run -p chronicler_engine --test integration -j 4` 3× runs with no `disk I/O error` from bootstrap tests.
    3. `cargo nextest run --retries 3` (same-binary re-run path) green.
    4. Full `python build.py` from `cd chronicler_engine` — 1262 tests pass + 2 skipped (regression check on A7 work).

## Test Plan

- 5× consecutive single-process runs of `run_branches.rs` — all 3 tests pass each time.
- 3× parallel runs (`-j 4`) — no `disk I/O error`, no `chronicler_*.db` collision in `target/debug/`.
- `cargo nextest run --retries 3` (the scenario `cleanup_db_for_port` defends against) green.
- Full `python build.py` — 1262 tests pass + 2 skipped (regression check on A7 work).

## Per Task Validation Steps

- **Task 1.1**: all 4 validation scenarios above green.

## Assumptions

- `3010-3050` range has ≥ 3 free slots even when `test_utils/server.rs`, browser tests, and llm flow tests run concurrently. 40 slots; current usage small. If CI saturates, error becomes "No available ports in range 3010-3050" — clearer than silent flake.
- Lock file cleanup: per-process exit leaves 3 lock files in `temp_dir/chronicler_test_ports/`. Next run either reaps dead PIDs (via `is_process_alive` in `tests/test_utils/server.rs:218-225`) or skips those ports. 37 free slots remain — adequate.
- `bootstrap::run` keeps file-based DB behavior. Tests must exercise the real boot path; refactoring to accept injectable `DbPool` is out of scope (5+ SP, not justified here).
- A7 work does not touch `run.rs::prepare_data` or `run_branches.rs` — confirmed pre-existing flake, not a regression.
- TCP port not bound by these tests (they fail in `prepare_data` before `start_server`). `get_available_port` does `bind + drop` which is fine — the port value is only used as the DB filename key.

## Out of Scope (NOT addressed)

- Refactoring `bootstrap::run` to accept injectable `DbPool` — would eliminate the file-based race entirely; ~5 SP, touches production code, not justified here.
- Changing the `3010-3050` port range.
- Fixing flaky tests other than the 3 in `run_branches.rs`.
- Patching `pi-plan-mode` extension to auto-persist plan on mode disable via user message (separate follow-up — extension silently drops plan if ready menu is dismissed without selection).
