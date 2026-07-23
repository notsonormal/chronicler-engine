---
diataxis: reference
title: Integration Test Standards
---

## Pattern 1 — SQLite-backed app + builder handoff

**Purpose.** Test an application service method end-to-end against a real `:memory:` SQLite database. The service is the system under test; state persists through snapshots and messages the way it does in production. The same `SqliteTestAppBuilder` is the basis for tests that need a *swap-mid-scenario* (calling `TestAppBuilder::from_base(&app, new_service)` to change the `GameService` while keeping the storage).

**The standard.**

```rust
#[test]
fn test_<scenario>_<expected_outcome>() {
    let app = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)                 // .backends / .mock_backend / .separate_backends
        .build_service()
        .unwrap();

    execute_action_impl(&app, "<input>".to_string());

    let state = app.latest_state();                    // via `PipelineHelpers` in tests/helpers/pipeline_helpers
    assert_eq!(state.narrative.history().len(), <expected>);
}
```

For tests that need a different provider between actions, the canonical handoff uses `TestAppBuilder::from_base(&app, Arc::new(service))` to swap the `GameService` while keeping the storage:

```rust
let failing_app = SqliteTestAppBuilder::default_test()
    .separate_backends(|| MockBackend::default().with_fail(), MockBackend::default)
    .message(msg).build_service().unwrap();

execute_action_impl(&failing_app, "look".to_string());
let after_fail = failing_app.latest_state();
assert!(
    after_fail
        .narrative
        .input_buffer
        .status
        .error_message()
        .is_some(),
    "failing backend should leave the input buffer in an error state"
);

let working_app = TestAppBuilder::from_base(
    &failing_app,
    Arc::new(GameService::with_backends(...)),
);
retry_last_response_impl(&working_app);
```

**Exemplar.** `tests/integration/application/action_pipeline/actions.rs` (8 tests, named `test_pipeline_*`, one behaviour per test, `execute_action_impl` invocation, `app.latest_state()` snapshot assertion). `tests/integration/flow/retry_main.rs` (10 tests, the canonical handoff form — every test uses `TestAppBuilder::from_base(&app, …)` to swap services between actions).

**Drift notes.**

- `tests/integration/application/lifecycle.rs` builds through `TestAppBuilder::with_data(...).storage(...).skip_seeding(true)` instead of `SqliteTestAppBuilder`. The file's own docstring flags this as "cross-cutting over `src/application/`… kept here for simplicity until the suite grows enough to split per-module". This is acceptable — lifecycle tests assert `create_game` / `switch_game` / `delete_game` paths on storage-without-service, which `SqliteTestAppBuilder` would over-wire. Future refactor: extract a third pattern (Lifecycle via `TestAppBuilder` + storage-only) once the suite grows.
- `tests/integration/application/application_service.rs` mixes both styles within the same file — the five sync tests use `TestAppBuilder` + `working_service()`; the two async tests use `SqliteTestAppBuilder::with_data(...).game_service_fn(...)`. Future refactor: separate the sync and async tests into two files, each one pure-style.
- `tests/integration/flow/{retry_event,retry_main,sequence}.rs` open-code `state.narrative.history.clear()` then call `.state_mut(move |s| *s = state)`. The `state_mut` form is intended for "state has no dedicated builder method", but these files use it for the bulk of fixture setup. Future refactor: extend `TestDataBuilder` for the missing constructors.
- `src/bootstrap/llm_factory_tests.rs::mock_backend_recorder_persists_forensics_to_storage` drives `LlmCallRecorder::complete(...)` through `get_llm_recorder_for` and asserts the message landed in real `Storage`. The regression it catches: prod factory wires `SaveLlmMessageFn` to a no-op closure instead of `Storage::save_llm_message` — silent fallback would leave `storage.list_latest_llm_messages(...)` empty.

## Pattern 2 — Real `TestServer` lifecycle + port allocation

**Purpose.** Spawn the actual `chronicler_engine` binary as a child process on a dynamically-allocated port, drive it with Playwright or HTTP requests, tear it down on test exit. The integration tier's only true end-to-end process shape.

**The standard.**

```rust
const CONFIG_PATH: &str = "tests/test_config.json";

#[tokio::test]
async fn test_<scenario>() {
    with_test_page(
        CONFIG_PATH, "test_world", "test_player",
        |page, port| async move {
            wait_for_status_ready(&page).await;
            send_action(&page, "look").await;
            wait_for_log_entries(&page, 1).await;
            assert!(element_exists(&page, "#story-log .log-entry").await);
        },
    ).await;
}
```

Where `with_test_page` (`tests/test_utils/browser.rs`) does:

1. `get_config_port(CONFIG_PATH)` → port from 3010–3050 with file-lock under `/tmp/chronicler_test_ports/port_<N>.lock`.
2. `TestServer::new_with_mock(port, world, persona)` → spawns the real `chronicler_engine` binary via `start_server_with_env(port, world, persona, /* use_mock= */ true)` (which writes a temp Mock-connections JSON and passes `--settings-path` to the binary). The mock form is the right default; use `TestServer::new(port, …)` (no `with_mock`) only when the test needs real env-var configurations.
3. `launch_chrome()` → honours `HEADED=1` (sets `headless = Some(false)`) and `SLOW_MO=<ms>` env vars so the same test runs headless in CI and visibly in local dev.
4. `goto_with_connection_check(&page, port)` → fails loud on `ERR_CONNECTION_REFUSED` rather than timing out silently.
5. Passes `(page, port)` to the test closure.

The `TestServer::Drop` impl calls `self.child.kill()` (i.e., `std::process::Child::kill`, which sends SIGKILL on Unix), then `self.child.wait()`, releases the port lock, deletes the SQLite DB file, and removes the temp settings directory. Tests do not need to do any of this manually. The `libc::kill(SIGTERM)` path lives in `terminate_pid` (`tests/test_utils/server.rs:54-66`), called by `kill_existing_server` (`:76-83`), not by `Drop` — those are separate functions invoked when launching a new server on a port that already has a stale registry entry.

**Exemplar.** `tests/browser/structure.rs::test_page_loads` (13 tests covering DOM structure, scrollbar, overflow). Uses `with_test_page` exclusively; no inline `sleep`.

**Drift notes.**

- `tests/browser/editing.rs::test_delete_removes_message` rolls its own polling loop (40 × 100ms) instead of using `wait_for_element_children` / `wait_for_log_entries`. The closed form is in the same file but unused for this test. Future refactor: convert to `wait_for_log_entries(&page, expected_count)`.
- `tests/llm/flow_llm_tests.rs` does NOT use `with_test_page` — it calls `TestServer::new(port, TEST_WORLD, TEST_PERSONA)` directly (no mock). This is correct (LLM tests need real env-var configuration, not the auto-injected Mock), but it bypasses `tests/test_config.json::test_specific.flow_llm_tests.backend == "real"` (which is read by `TestServer::from_config` and is currently a no-op). Future refactor: switch to `TestServer::from_config(...)` so the JSON file actually drives the LLM suite's backend choice.
- `tests/http/server_impl_wiring.rs` is the only HTTP file that *actually* binds a `tokio::net::TcpListener` via `server_impl::run_server_with_config(..., ServerConfig { port: 0, bind_attempts: Some(1) })`. It does NOT use a port from 3010–3050. This file exists to test port-binding edge cases (already-bound port → error) and is not part of the standard lifecycle.
- `tests/integration/adapters/driven/llm/llm_client.rs` is the only file that spawns a hand-rolled `TcpListener` in a `thread::spawn` to act as a mock HTTP server for `call_chat_completions`. It binds `127.0.0.1:0` (OS-assigned ephemeral port) — drift from the 3010–3050 convention, but justified because the test exercises the HTTP layer, not the engine binary.

## Pattern 3 — Storage-direct round-trip

**Purpose.** Exercise a `Storage` method directly against a real SQLite (or in-memory) backend, with no `GameService` and no `DefaultApplicationService`. The persistence layer is the system under test.

**The standard.**

```rust
#[test]
fn test_<method>_<expected_outcome>() {
    let storage = create_test_storage(1);                  // from tests/helpers/fixtures.rs
    let entity = <make test entity>;

    let id = storage.<method>(&entity).unwrap();
    assert_eq!(storage.<load_method>().unwrap().len(), <expected>);
}
```

For tests that only need the in-memory variant (faster, no SQLite startup):

```rust
let storage = Storage::new_in_memory();
// same exercise, same assertions
```

**Exemplar.** `tests/integration/storage/preset_storage.rs` (28 tests, the largest single file in the corpus; one concern — CRUD on `PromptPreset`; uses `Storage::new_in_memory` exclusively; the SQLite variant is exercised by the unit tier's Pattern 2 storage backend pair rather than this file).

**Drift notes.**

- `tests/integration/storage/message_storage.rs` mixes SQLite and in-memory storage within the same file but as **separate test groups**, not parity pairs (drift from unit-tier Pattern 2 which demands character-for-character parity on every method). The doc comment at the top of the file says "Integration tests for `Message` persistence" — parity is intended but not strict at the integration tier. Future refactor: pick one tier (in-memory for speed OR parity for thoroughness) and document the choice.
- `tests/integration/storage/world_storage.rs` splits 16 in-memory tests at the top from 3 SQLite tests at the bottom. The 3 sqlite tests target the seed path specifically (the in-memory seed is identical to the unit tier's; the SQLite seed is what the prod binary sees). This split is intentional; do not collapse.
- `tests/integration/storage/snapshot_storage.rs::test_row_to_snapshot_bad_json` bypasses the `Storage` API to insert a malformed row directly via `pool.conn().execute("INSERT …", rusqlite::params![…])`. This is the only test in the corpus that drops to raw `rusqlite` to provoke an error. Future refactor: lift this into a `Storage::insert_raw_row_for_test` helper so the drift is contained.
- `tests/integration/storage/preset_storage.rs::test_list_presets_ordered_by_updated_at_desc` uses `std::thread::sleep(15ms)` between saves, with an in-source comment explaining "intentional and unavoidable" (SQLite timestamps have ms precision; ordering tests need distinct timestamps). Acceptable.
- `tests/integration/storage/llm_message_storage.rs` uses `create_test_storage(1)` (real SQLite in `:memory:`) for all 5 tests — the canonical Pattern 3 form.

## Pattern 4 — HTTP one-shot via `tower::ServiceExt::oneshot`

**Purpose.** Test an axum HTTP handler end-to-end by dispatching a single `Request` against a `TestAppBuilder`-built router. There is **no listening port** — the handler runs once, the response is the assertion. This is the dominant HTTP test shape (~10 files).

**The standard.**

```rust
#[tokio::test]
async fn test_<handler>_<scenario>() {
    let _guard = SettingsTestGuard::new();                  // only when mutating global settings
    let app = TestAppBuilder::default_app();                 // or .default_test() with seeded data

    let req = Request::builder()
        .uri("/fragment/<endpoint>")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(response.status().is_success());

    let body = axum::body::to_bytes(response.into_body(), 16_384).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("<expected substring>"));
}
```

For tests that need a custom storage with seeded data, install it before `build()`:

```rust
let storage = Arc::new(Storage::new_in_memory().with_failure(
    "save_snapshot", TestOverride::internal("..."),
));
let app = TestAppBuilder::default_test().storage(storage).build();
```

**Exemplar.** `tests/http/debug.rs::test_debug_state_endpoint_includes_all_documented_fields` enumerates the 13 expected fields in the JSON response and asserts each one is present. The exhaustive assertion is the canonical shape; do not assert "non-empty body" when the load-bearing contract is "all 13 fields are present".

**Drift notes.**

- `tests/http/fragment.rs::test_action_confirm_empty_command` is the only HTTP file that pulls `DefaultApplicationService` *out* of the app to observe internal pipeline state — uses `let (app, service) = TestAppBuilder::default_test().build_with_service();` and then `service.load_or_fresh()` to verify the pipeline finalises. Acceptable: this is the only HTTP test where the pipeline-state observation matters more than the response body.
- `tests/http/test_helpers.rs` defines one helper function: `fetch_body(app, uri) -> String`. It panics on non-2xx. Use it for the GET/empty-body case rather than rolling the `oneshot` + `to_bytes` boilerplate inline.
- `tests/http/worlds_fragment_handlers.rs`'s file docstring says "Unit tests for worlds_fragment handlers" but the file goes through `TestAppBuilder` (integration-shaped). Naming-vs-shape drift; candidate for a future rename to `worlds_fragment_integration_tests.rs` or similar.
- `tests/http/endpoints/text_check.rs::test_async_action_saves_input_to_story_log_with_sqlite` creates an on-disk SQLite DB at `/tmp/chronicler_component_test_<pid>/test.db` and never cleans it up — drift from `TestServer::Drop`'s cleanup discipline. Future refactor: wrap the storage in a `Drop` guard or use a `tempfile::TempDir`.
- `tests/http/actions.rs` (8 tests) is entirely failure-injection shaped — every test wraps the storage with `with_failure(...)`. See Pattern 5 for the canonical failure-injection form; Pattern 4 here is the HTTP layer that Pattern 5 installs onto.

## Pattern 5 — Failure-injection via `Storage::with_failure` / `Storage::with_test_failures`

**Purpose.** Test that the storage layer handles storage failures correctly (a `load` that returns a SQL error, a `save` that fails because the file is unwritable, etc.) and that those failures propagate correctly through the application or HTTP layer above. The failure is the system under test.

**The standard (single-test, inline form):**

```rust
#[tokio::test]
async fn test_<scenario>_storage_failure() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "<storage_method_name>", TestOverride::internal("<reason>"),
    ));
    let app = TestAppBuilder::default_test().storage(storage).build();

    let response = app.oneshot(<request for the path that triggers the failure>).await.unwrap();
    assert!(response.status().is_server_error());           // or whatever the documented failure shape is
}
```

**The standard (handle form, when you need to fail-only-sometimes):**

```rust
let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
failing_storage.seed_world(&world, &map)?;
handle.set("load_latest_snapshot", TestOverride::internal("..."));
<exercise the path that triggers the failure>
handle.clear("load_latest_snapshot");
<exercise the path that succeeds afterwards>
```

The two `TestOverride` variants (`internal` and `config`) map to two different `EngineError` arms downstream. Use `internal` for unexpected runtime errors (DB corruption, write failure) and `config` for input-validation failures.

**Exemplar.** `tests/http/actions.rs` (the entire file is failure-shape coverage of the `/action`, `/action/confirm`, and `/action/check` endpoints — each test flips a different storage method). `tests/integration/flow/arrival_persistence.rs::test_arrival_persistence_fails_loudly_when_save_message_returns_error` (the inline-handle form).

**Drift notes.**

- The same `TestOverride::internal("<reason>")` reason strings are reused across tests (`"simulated load failure"`, `"simulated save failure"`). These are load-bearing tokens, not arbitrary text — they appear in error traces that humans triage.
- `tests/integration/flow/arrival_persistence.rs::test_arrival_persistence_fails_loudly_when_save_message_returns_error` mixes both forms: inline `handle.set(...)` followed by post-assertion `handle.clear(...)` rather than rebuilding storage. Both forms are valid; pick one and stay consistent.
- Failure-injection tests at this tier **mirror** unit-tier Cross-cutting pattern A (`TestOverride` for storage failures) but exercise a different surface: HTTP responses rather than direct method returns. The unit doc's `TestOverride` examples are tightly scoped; integration tier widens the assertion to status codes and rendered fragments.

## Pattern 6 — Bootstrap `run(Args)` direct invocation

**Purpose.** Exercise the prod `bootstrap::run(Args)` entry-point end-to-end (the same code path the production CLI uses), assert the error path or the success path, and observe the SQLite DB file the binary opened on its own. This is the only tier where integration tests can reach the CLI startup path.

**The standard.**

```rust
#[test]
fn test_<branch>() {
    let port = get_available_port(3010, 3050).expect("port allocation failed for run_branches test");
    cleanup_db_for_port(port);
    let args = Args {
        world: "__nonexistent_world__".to_string(),
        persona: "__nonexistent_persona__".to_string(),
        list_worlds: false,
        port,
        settings_path: None,
    };
    let result = run(args);
    assert!(matches!(result, Err(EngineError::PersonaNotFound(key)) if key == "..."));
}
```

The `cleanup_db_for_port(port)` helper is **load-bearing**: `bootstrap::run` opens `<exe_parent>/chronicler_<port>.db` plus SQLite WAL/SHM sidecars. Stale files left by previous test runs cause migrations to re-apply `ALTER TABLE` statements to an already-migrated schema, surfacing as "duplicate column name: persona_key" or transient "disk I/O error" when WAL files from concurrent runs collide.

**Exemplar.** `tests/integration/bootstrap/run_branches.rs::test_run_persona_not_found_after_world_fallback` (the canonical form) and `test_list_available_worlds_lists_seeded_worlds` (the success-path form).

**Drift notes.**

- This is the **only** file that calls `bootstrap::run(Args)` directly rather than going through `TestServer`. The other startup-touched path (`tests/http/server_impl_wiring.rs`) reaches a different entry-point (`server_impl::run_server_with_config`) and binds a `TcpListener` rather than running the CLI.
- The file hard-codes `3010..=3050` rather than reading `tests/test_config.json::port_range`. Drift from the rest of the corpus. Acceptable: the file's only purpose is to reach a startup error path; the port range is the engine's own allocated range, not a per-test config knob. Future refactor: lift the port-range constant into a shared `tests/test_utils::ports::RANGE` if a second file needs it.

## Pattern 7 — Arch-lint rule self-tests

**Purpose.** Test the arch-lint rule functions themselves — paired positive and negative tests that feed synthetic source strings through `check_<rule>()` and assert the violation set is what the rule should produce. The tests live in `tests/infrastructure/guardrails/` and run against `arch-lint.toml`'s rule definitions.

**The standard.**

```rust
#[test]
fn test_<rule>_catches_violation() {
    let v = check_<rule>(
        "<path>",
        "<source string that should violate the rule>",
    );
    assert_eq!(v.len(), 1);
    assert!(v[0].message.contains("<expected message fragment>"));
}

#[test]
fn test_<rule>_allows_correct() {
    let v = check_<rule>("<path>", "<source that should NOT violate>");
    assert!(v.is_empty());
}
```

The paired positive/negative shape is invariant across the rule-self-tests. Each test exercises one rule function with one synthetic input and asserts the violation count and the violation message (or its absence).

**Exemplar.** `tests/infrastructure/guardrails/layers.rs::test_check_application_storage_direct_*` (4 paired tests for the application-vs-storage layer-boundary rule: catches-violation, skips-tests-files, skips-non-application, skips-comments). `tests/infrastructure/guardrails/enums.rs::check_enum_variant_docs_*` (5 paired tests for the variant-docs rule: trivial-marker-skips-check, flags-missing-variant-docs, accepts-documented-variants, skips-empty-enum, flags-trivial-with-variant-docs).

**Drift notes.**

- `tests/infrastructure/guardrails/{location,structure,style}.rs` contain **no** `#[test]` of their own. They define only `check_<rule>(...)` pure functions; the tests live in `tests/infrastructure/guardrails/mod.rs` (which iterates over the live `src/` + `tests/` trees and exercises every rule). Future refactor: move the rule-execution tests out of `mod.rs` and into per-rule files so the rule tests are co-located with the rule definitions; lift them from `mod.rs` to per-rule files.
- `tests/infrastructure/guardrails/style.rs` defines `CfgTestTracker` so the comment-run and no-std-thread rules can ignore `#[cfg(test)]` blocks. The `CfgTestTracker` is the only "test tooling inside a rule file" in the corpus; document why it exists.
- `tests/infrastructure/architecture.rs::arch_lint::check!()` emits many `#[test]`s at expansion time (the count is not visible at the Rust source level — the `#[test]` marker is generated by the macro). When this doc says "X tests in `tests/infrastructure/architecture.rs`", that count is macro-expanded, not source-level; if you read the file you see `arch_lint::check!();` on a single line and many `#[test]`s at compile time. Reference citations in this doc use the source-level count of one.
- `tests/infrastructure/invariant_contract.rs` is structurally distinct from Pattern 7 — it carries runtime-invariant regression tests (INV-001…INV-007 + invariant P4) using `SqliteTestAppBuilder` and one `proptest!` block. It lives in the `infrastructure` binary (alongside `architecture.rs` and `guardrails/`), but its content is closer to a Pattern-1 integration test with concurrency invariants. Don't fold it into Pattern 7 just because of binary location.

## Cross-cutting patterns

### Cross-cutting 1 — `SettingsTestGuard` for settings mutations

Every HTTP test that mutates `AppSettings` (via `connections/add`, `settings`, `prompt-presets`, etc.) starts with `let _guard = SettingsTestGuard::new();`. This is load-bearing because `AppSettings` lives in a global static and parallel HTTP test runs would race. The guard is a `Mutex<()>` with poisoning-recovery (`unwrap_or_else(|e| e.into_inner())`) so a panic in one test does not deadlock subsequent ones.

The guard is the near-universal idiom for HTTP tests; assume any HTTP test that mutates settings needs it. Read-only HTTP tests (e.g., `GET /fragment/<x>` against default settings) do not need it but use it anyway as a defensive blanket convention. If a test unnecessarily acquires the guard, that's acceptable drift; if a test mutates settings without it, that's a real bug.

**Where.** `tests/test_utils/settings_guard.rs`. Used by every test in `tests/http/connections.rs`, every test in `tests/http/prompt_presets.rs`, 5 of 6 tests in `tests/integration/model/settings.rs`, etc.

### Cross-cutting 2 — File-locked port allocation on 3010–3050

`tests/test_utils/server.rs::get_available_port(min, max)` allocates ports from `tests/test_config.json::port_range` (default 3010–3050) via POSIX `OpenOptions::create_new` on `/tmp/chronicler_test_ports/port_<N>.lock`. This is atomic on POSIX; the test claims the port by creating the lock file and writing its PID to it. On contention, the helper GCs stale locks (PIDs no longer alive) and retries with exponential backoff (50ms → 500ms, 20 attempts).

The port range and lock-file path are **deliberately outside the cargo workspace** (`/tmp` rather than `target/`) so concurrent `cargo build` invocations can't see stale locks. When refactoring the helper, do not move the lock path into `target/`.

**Where.** All browser tests (`with_test_page` → `get_config_port(CONFIG_PATH)`), `tests/integration/bootstrap/run_branches.rs` (`get_available_port(3010, 3050)` directly), and `tests/llm/flow_llm_tests.rs` (`get_config_port(CONFIG_PATH)`). Tests that bind `0.0.0.0:0` directly (e.g., `tests/integration/adapters/driven/llm/llm_client.rs`, `tests/http/server_impl_wiring.rs`) opt out of the 3010–3050 range by design.

### Cross-cutting 3 — Mock-backend auto-injection via `--settings-path`

`TestServer::start(port, world, persona, use_mock)` writes a temp JSON file at `/tmp/chronicler_test_settings_<pid>_<port>/settings.json` with two Mock connections (`openrouter-gpt-4o-mini`, `openrouter-euryale`) when `use_mock = true`, then passes `--settings-path` to the engine binary. The `Drop` impl removes the directory. This is what `TestServer::new_with_mock` defaults to and what `with_test_page` uses for browser tests.

Tests that need real env-var configurations (e.g., `OPENROUTER_API_KEY`) call `TestServer::new(port, ...)` instead — the binary loads its default settings (which the env var populates). The Mock auto-injection is gated by the boolean; the JSON-fallback is opt-out, not opt-in.

**Where.** All `with_test_page` browser tests, all `with_real_llm` LLM tests, and any HTTP test that uses `TestServer::from_config(...)`. Tests that use `tower::ServiceExt::oneshot` (Pattern 4) bypass this entirely because the router runs in-process and the engine binary never starts — they construct the app directly with `TestAppBuilder`.

### Cross-cutting 4 — `HEADED` / `SLOW_MO` env-var overrides for Playwright

`tests/test_utils/browser.rs::launch_chrome()` reads two env vars before constructing Playwright launch options:

- `HEADED=1` → `options.headless = Some(false)` (run Playwright in headed mode, surfacing the browser window for interactive debugging).
- `SLOW_MO=<ms>` → `options.slow_mo = Some(<ms>)` (introduce a pause between Playwright steps).

Both default off. Run `HEADED=1 SLOW_MO=500 cargo nextest run --test browser <name>` to debug a single browser test interactively; default to headless in CI.

This convention exists **only** for the browser binary. The LLM binary does not override Playwright launch — when an LLM test runs through `with_test_page` (if it did), it would inherit the same `HEADED` / `SLOW_MO` discipline. Today it uses `TestServer::new` directly, not `with_test_page`, so it bypasses this — see Pattern 2's drift note.

### Cross-cutting 5 — `capture_failure_state` diagnostic dump

`tests/test_utils/browser.rs::capture_failure_state(page, name)` writes two diagnostic artifacts on demand:

- `chronicler_engine/tmp/screenshots/<epoch>_<sanitized_name>.png` (browser screenshot at the moment of failure).
- `chronicler_engine/tmp/test_diagnostics/<name>.html` (the DOM dump, full HTML at the moment of failure).

It is called **only** by the timed-out `wait_for_*` helpers in `tests/test_utils/wait.rs` (`wait_for_element_children`, `wait_for_element_text`, `wait_for_element_exists`, `wait_for_element_not_exists`, `wait_for_non_loading_value`, `wait_for_status_ready`, `wait_for_status_ready_or_error`). Tests do not intentionally produce diagnostics on success — diagnostics are a failure artefact, not a control artefact.

The convention: **never swallow `capture_failure_state`'s panic**. If a `wait_for_*` helper times out, it panics with the diagnostic already written to disk; the test runner's failure report includes the path so a developer can `cat` the HTML or `xdg-open` the PNG. Removing the panic (e.g., returning `Err` instead) would suppress the diagnostic dump and break the convention.

There is no equivalent for sync polling in the integration suite (no screenshots for sync-test failures). Sync tests use `cargo nextest`'s built-in failure reporting and the `--retries` / `--failure-output` flags.

**Drift note.** Six helpers in `tests/test_utils/wait.rs` are defined but never called from outside the file: `wait_for_element_class`, `wait_for_element_text`, `wait_for_more_messages`, `wait_for_location_change`, `wait_for_story_log_change`, `wait_for_condition_sync`. (`wait_for_element_text` does reference `capture_failure_state` from `:174` and is therefore reachable from itself, but no test invokes it directly.) Refactor target: either delete them, mark them `#[allow(dead_code)]` explicitly, or wire them up where they replace longer polling loops. This is the only dead-code drift in the smart-waiting layer.

### Cross-cutting 6 — `SqliteTestAppBuilder` over `TestAppBuilder` for snapshot assertions

When you need to assert state after `execute_action_impl`, the canonical builder is `SqliteTestAppBuilder` (defined in `tests/helpers/sqlite_test_app_builder.rs`). It builds a full `DefaultApplicationService` backed by in-memory SQLite, persists snapshots + messages the way production does, and exposes `app.storage()`, `app.cancel_token()`, `app.is_generating()`. The alternative — `TestAppBuilder::default_test().skip_seeding(true)` — only goes through `:memory:` SQLite storage but skips the production service wiring, which is enough for tests that call `app_service.create_game(...)` directly but **not** for tests that observe generation phases or assert through the snapshot path.

The decision rule:

- Service method under test that mutates state the test will then read? → `SqliteTestAppBuilder`.
- Service method under test on a pre-seeded state (read or write)? → `TestAppBuilder` (faster; no SQLite startup).
- `Storage` method under test (no service at all)? → `create_test_storage(...)` or `Storage::new_in_memory()` (Pattern 3).

This decision rule appears across `tests/integration/application/*.rs` and is the single most-violated convention in the corpus — see Pattern 1's drift notes.

### Cross-cutting 7 — `MockBackend` factory form vs closure form

Two patterns coexist for the same `MockBackend`; the choice matters when the test wants to attach a configuration (`with_fail`, `with_empty_response`, `with_prompt_responses`, `with_narrations`, `with_trigger_narration_fail`, `with_trigger_delay`) to a specific backend:

- **Function pointer form:** `MockBackend::default` — passed directly to `.backends(MockBackend::default)`. Used when no per-backend configuration is needed. Faster, no `move` capture needed.
- **Closure form:** `|| MockBackend::default()` (often chained: `|| MockBackend::default().with_fail()`) — wrapped in builder methods that expect `Fn() -> MockBackend + 'static`. Used when per-backend configuration is needed. The factory is invoked twice (once for narrator, once for quantifier), so one `MockBackend` instance cannot be reused — the closure must yield a fresh instance each call.

The factory is invoked twice because `SqliteTestAppBuilder` constructs one `MockBackend` for the narrator and one for the quantifier. This is non-obvious; if you bind `MockBackend` to a `let` and pass the variable, the test will panic with a "consumed twice" error. Pass a factory.

**Where.** Every test in `tests/integration/application/*.rs`, every test in `tests/integration/flow/*.rs`, and `tests/infrastructure/invariant_contract.rs::test_p4_*`.

### Cross-cutting 8 — `#[ignore]` for real-LLM tests + `--llm-only` invocation

Two tests in `tests/llm/flow_llm_tests.rs` (`test_chained_actions_through_real_llm`, `test_verify_llm_messages_logged`) are `#[ignore = "slow: requires OPENROUTER_API_KEY"]` AND short-circuit at runtime if the env var is unset:

```rust
if !has_llm_api_key() {
    eprintln!("Skipping: OPENROUTER_API_KEY not set");
    return;
}
```

Defense-in-depth: the `#[ignore]` keeps the tests out of `cargo test` runs; the runtime check keeps them short-circuiting under `cargo nextest run --test llm --run-ignored` where `--ignore` is honoured. Both tests are skipped when the env var is unset, so the LLM binary runs in ~1s in CI without the key and ~30s locally with it.

**Where this is wired.** `tests/llm/` (only file in the binary). Invoked via:

- `cargo nextest run --test llm` (default: ignored, ~1s)
- `cargo nextest run --test llm --run-ignored` (with key: real LLM, ~30s)
- `python build.py --llm-only` (the build.py convenience invocation that aggregates LLM tests across the tree; see `tests/AGENTS.md`).

The `--llm-only` invocation is the canonical one for the integration-tier LLM tests. When modifying `src/narrative/` or LLM-parsing code, run `--llm-only` once locally with a valid `OPENROUTER_API_KEY` to confirm the tests still pass — CI does not exercise the ignored tests.

## Document References

- [`unit_test_standards.md`](unit_test_standards.md) — unit-test standards (storage backend pair, LLM provider trait, LLM recorder, fragment renderer, concurrency invariant, proptest!; cross-tier alignment).
- Legacy `docs/reference/testing.md` (migration target: `reference/testing.md`) — testing policy in terse form: what's required, builder-selection rule, XSS regression checks list (must never be deleted without replacement), critical test categories (XSS, Continue, scroll, overflow).
- `tests/AGENTS.md` — engine-side test-infrastructure policy; test-mirror convention, structure overview, `[binary] ↔ fixture-weight` mapping.
- `chronicler_engine/docs/adr/0028-test-module-header.md` (ADR-028) — the `check_test_module_header` guardrail; enforces single-line `//! <summary>` on every `*_tests.rs` file (mirrored for integration via Pattern 7's Cross-cutting 10 in `unit_test_standards.md`; the integration tier inherits the same rule).
- `tests/test_utils/server.rs` — port allocation, `TestServer` lifecycle, `SERVER_MANAGED` PID registry.
- `tests/test_utils/wait.rs` — smart-waiting helpers (`wait_for_llm_idle`, `wait_for_status_ready`, `wait_for_element_children`, etc.).
- `tests/helpers/sqlite_test_app_builder.rs` — `SqliteTestAppBuilder` (Pattern 1) and `TestAppBuilder::from_base(...)` handoff.
- `tests/helpers/pipeline_helpers.rs` — extension trait `PipelineHelpers` (`latest_state`, `wait_for_generation_complete`) and `wait_for_condition` (sync).
- `tests/helpers/fixtures.rs` — extension trait `TestWorldFixture` for storage-backed test-world seeding.
- `tests/test_utils/browser.rs` — Playwright setup, `with_test_page`, `capture_failure_state`, `HEADED` / `SLOW_MO` env-var conventions.
- `chronicler_engine/scripts/tests/test_validate_docs_diataxis.py` — 14 fixture tests enforcing front-matter, mode vocabulary, and required H2 sections (`## Overview` + `## Document References`) for Reference docs.
- `chronicler_engine/docs-diataxis/explanation/diataxis.md` — Diátaxis primer (Reference mode's place in the four-kind taxonomy).
- `chronicler_engine/docs-diataxis/AGENTS.md` — writing conventions for the docs-diataxis tree (seam-identifier vs mechanics-leak discipline, Reference-defers-to-source rule, no code-indexer docs, no negative explaining).
