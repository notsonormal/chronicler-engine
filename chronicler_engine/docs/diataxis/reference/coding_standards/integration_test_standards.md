---
diataxis: reference
title: Integration Test Standards
---

## Pattern 1 — SQLite-backed app + builder handoff

**Purpose.** Test an application collaborator method end-to-end against a real `:memory:` SQLite database. The service is the system under test; state persists through snapshots and messages the way it does in production. The same `SqliteTestAppBuilder` is the basis for tests that need a *swap-mid-scenario* — a different provider response per call — via per-call mock configuration (`with_narrations(...)` / `with_prompt_responses(...)`) or a `.pipeline_fn(...)` closure.

**The standard.**

```rust
#[test]
fn test_<scenario>_<expected_outcome>() {
    let app = SqliteTestAppBuilder::default_test()
        .backends(MockBackend::default)                 // .backends / .mock_backend / .separate_backends
        .build_service()
        .unwrap();

    app.pipeline.execute_action("<input>".to_string());

    let state = app.latest_state();                    // via `PipelineHelpers` in tests/helpers/application_ext.rs
    assert_eq!(state.narrative.history().len(), <expected>);
}
```

For tests that need a different provider behaviour between actions, configure the mock per call — `MockBackend::default().with_narrations(vec![first, second, ...])` (narrator) or `.with_prompt_responses(vec![...])` (quantifier) — or supply the pipeline explicitly via `.pipeline_fn(...)`:

```rust
let app = SqliteTestAppBuilder::default_test()
    .pipeline_fn(move |storage, pg, settings, _token| {
        ActionPipeline::with_mock_quantifier(
            CancellationToken::new(),
            make_test_recorder_with_storage(Arc::new(MockBackend::new()), Arc::clone(storage)),
            Arc::new(MockBackend::default().with_prompt_responses(vec![
                r#"{"npcs_in_room": []}"#.to_string(),
                r#"{"npcs_in_room": ["gabriella"]}"#.to_string(),
            ])),
            Arc::clone(pg),
            Arc::clone(settings),
        )
    })
    .build_with_state()
    .unwrap();
app.pipeline.execute_action("enter shop".to_string());
app.pipeline.retry_last_response();   // second call consumes the next queued response
```

**When to use the Service-direct variant.** Use the Service-direct variant when testing collaborator methods that don't flow through `execute_action` — lifecycle ops live on `GameCatalogue` (`create_game`, `switch_game`, `delete_game`, `list_games`, `current_game_id`), status/cancellation on `GenerationGate`, and read-side queries on `GameViewQuery`; or any test needing explicit pipeline construction with `skip_seeding=true`. Don't use for `execute_action` pipeline tests — those use the primary `SqliteTestAppBuilder::default_test().backends(...).build_service()` form. Don't use for direct-storage tests with no service — that's Pattern 3.

**The standard (Service-direct variant).**

```rust
let app = TestAppBuilder::with_data(data)
    .storage(storage.clone())
    .pipeline(make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        crate::make_test_recorder(Arc::new(MockBackend::default())),
        AgentRegistry::default(),
    ))
    .skip_seeding(true)
    .build_service();

let result = app.game_catalogue.create_game(&world_key, "hero");
assert!(result.is_ok(), "create_game should succeed: {:?}", result.err());
```

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

## Pattern 3 — Storage-direct round-trip

**Purpose.** Exercise a `Storage` method directly against a real SQLite (or in-memory) backend, with no `ActionPipeline` and no application collaborators. The persistence layer is the system under test.

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

## Cross-cutting patterns

### Cross-cutting 1 — `SettingsTestGuard` for settings mutations

Every HTTP test that mutates `AppSettings` (via `connections/add`, `settings`, `prompt-presets`, etc.) starts with `let _guard = SettingsTestGuard::new();`. This is load-bearing because `AppSettings` lives in a global static and parallel HTTP test runs would race. The guard is a `Mutex<()>` with poisoning-recovery (`unwrap_or_else(|e| e.into_inner())`) so a panic in one test does not deadlock subsequent ones.

The guard is the near-universal idiom for HTTP tests; assume any HTTP test that mutates settings needs it. Read-only HTTP tests (e.g., `GET /fragment/<x>` against default settings) do not need it but use it anyway as a defensive blanket convention. If a test mutates settings without it, that's a real bug.

**Where.** `tests/test_utils/settings_guard.rs`. Used by every test in `tests/http/connections.rs`, every test in `tests/http/prompt_presets.rs`, 5 of 6 tests in `tests/integration/model/settings.rs`, etc.

### Cross-cutting 2 — File-locked port allocation on 3010–3050

`tests/test_utils/server.rs::get_available_port(min, max)` allocates ports from `tests/test_config.json::port_range` (default 3010–3050) via POSIX `OpenOptions::create_new` on `/tmp/chronicler_test_ports/port_<N>.lock`. This is atomic on POSIX; the test claims the port by creating the lock file and writing its PID to it. On contention, the helper GCs stale locks (PIDs no longer alive) and retries with exponential backoff (50ms → 500ms, 20 attempts).

The port range and lock-file path are **deliberately outside the cargo workspace** (`/tmp` rather than `target/`) so concurrent `cargo build` invocations can't see stale locks. When refactoring the helper, do not move the lock path into `target/`.

**Where.** All browser tests (`with_test_page`), `tests/integration/bootstrap/run_branches.rs`, and `tests/llm/flow_llm_tests.rs` use this allocator. Tests that bind `0.0.0.0:0` directly opt out of the 3010–3050 range by design.

### Cross-cutting 3 — Mock-backend auto-injection via `--settings-path`

`TestServer::start(port, world, persona, use_mock)` writes a temp JSON file at `/tmp/chronicler_test_settings_<pid>_<port>/settings.json` with two Mock connections (`openrouter-gpt-4o-mini`, `openrouter-euryale`) when `use_mock = true`, then passes `--settings-path` to the engine binary. The `Drop` impl removes the directory. This is what `TestServer::new_with_mock` defaults to and what `with_test_page` uses for browser tests.

The Mock auto-injection is gated by the `use_mock` boolean; `with_test_page` defaults to `true`. Tests that need real env-var configurations (e.g., `OPENROUTER_API_KEY`) call `TestServer::new(port, ...)` instead — the binary loads its default settings (which the env var populates).

**Where.** All `with_test_page` browser tests, all `with_real_llm` LLM tests, and any HTTP test that uses `TestServer::from_config(...)`. Tests that use `tower::ServiceExt::oneshot` (Pattern 4) bypass this entirely because the router runs in-process and the engine binary never starts — they construct the app directly with `TestAppBuilder`.

### Cross-cutting 4 — `HEADED` / `SLOW_MO` env-var overrides for Playwright

`tests/test_utils/browser.rs::launch_chrome()` reads two env vars before constructing Playwright launch options:

- `HEADED=1` → `options.headless = Some(false)` (run Playwright in headed mode, surfacing the browser window for interactive debugging).
- `SLOW_MO=<ms>` → `options.slow_mo = Some(<ms>)` (introduce a pause between Playwright steps).

Both default off. Run `HEADED=1 SLOW_MO=500 cargo nextest run --test browser <name>` to debug a single browser test interactively; default to headless in CI.

This convention exists **only** for the browser binary. The LLM binary does not override Playwright launch — when an LLM test runs through `with_test_page` (if it did), it would inherit the same `HEADED` / `SLOW_MO` discipline. Today it uses `TestServer::new` directly, not `with_test_page`.

### Cross-cutting 5 — `capture_failure_state` diagnostic dump

`tests/test_utils/browser.rs::capture_failure_state(page, name)` writes two diagnostic artifacts on demand:

- `chronicler_engine/tmp/screenshots/<epoch>_<sanitized_name>.png` (browser screenshot at the moment of failure).
- `chronicler_engine/tmp/test_diagnostics/<name>.html` (the DOM dump, full HTML at the moment of failure).

It is called **only** by the timed-out `wait_for_*` helpers in `tests/test_utils/wait.rs`. Tests do not intentionally produce diagnostics on success — diagnostics are a failure artefact, not a control artefact.

The convention: **never swallow `capture_failure_state`'s panic**. If a `wait_for_*` helper times out, it panics with the diagnostic already written to disk; the test runner's failure report includes the path so a developer can `cat` the HTML or `xdg-open` the PNG. Removing the panic (e.g., returning `Err` instead) would suppress the diagnostic dump and break the convention.

There is no equivalent for sync polling in the integration suite (no screenshots for sync-test failures). Sync tests use `cargo nextest`'s built-in failure reporting and the `--retries` / `--failure-output` flags.

### Cross-cutting 6 — `SqliteTestAppBuilder` over `TestAppBuilder` for snapshot assertions

When you need to assert state after `execute_action`, the canonical builder is `SqliteTestAppBuilder` (defined in `tests/helpers/sqlite_test_app_builder.rs`). It builds a full `AppState` with collaborators (`ActionPipeline`, `GameCatalogue`, `GameViewQuery`, `GenerationGate`, `PersistenceGate`) backed by in-memory SQLite, persists snapshots + messages the way production does, and exposes `app.storage`, `app.shutdown_token`. The alternative — `TestAppBuilder::default_test().skip_seeding(true)` — only goes through `:memory:` SQLite storage but skips the production service wiring, which is enough for tests that call `app.game_catalogue.create_game(...)` directly but **not** for tests that observe generation phases or assert through the snapshot path.

The decision rule:

- Service method under test that mutates state the test will then read? → `SqliteTestAppBuilder`.
- Service method under test on a pre-seeded state (read or write)? → `TestAppBuilder` (faster; no SQLite startup).
- `Storage` method under test (no service at all)? → `create_test_storage(...)` or `Storage::new_in_memory()` (Pattern 3).

This decision rule appears across `tests/integration/application/*.rs`.

### Cross-cutting 7 — `MockBackend` factory form vs closure form

Two patterns coexist for the same `MockBackend`; the choice matters when the test wants to attach a configuration (`with_fail`, `with_empty_response`, `with_prompt_responses`, `with_narrations`, `with_trigger_narration_fail`, `with_trigger_delay`) to a specific backend:

- **Function pointer form:** `MockBackend::default` — passed directly to `.backends(MockBackend::default)`. Used when no per-backend configuration is needed. Faster, no `move` capture needed.
- **Closure form:** `|| MockBackend::default()` (often chained: `|| MockBackend::default().with_fail()`) — wrapped in builder methods that expect `Fn() -> MockBackend + 'static`. Used when per-backend configuration is needed. The factory is invoked twice (once for narrator, once for quantifier), so one `MockBackend` instance cannot be reused — the closure must yield a fresh instance each call.

The factory is invoked twice because `SqliteTestAppBuilder` constructs one `MockBackend` for the narrator and one for the quantifier. This is non-obvious; if you bind `MockBackend` to a `let` and pass the variable, the test will panic with a "consumed twice" error. Pass a factory.

**Where.** Every test in `tests/integration/application/`, `tests/integration/flow/`, and `tests/infrastructure/invariant_contract.rs::test_p4_*`.

### Cross-cutting 8 — `#[ignore]` for real-LLM tests + `--llm-only` invocation

Two tests in `tests/llm/flow_llm_tests.rs` (`test_real_llm_smoke`, `test_real_llm_multi_step_stability`) are `#[ignore = "slow: requires OPENROUTER_API_KEY"]` AND short-circuit at runtime if the env var is unset:

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

The `--llm-only` invocation is the canonical one for the integration-tier LLM tests. When modifying `src/application/narrative_prompt/` or `src/adapters/driven/llm/`, or LLM-parsing code, run `--llm-only` once locally with a valid `OPENROUTER_API_KEY` to confirm the tests still pass — CI does not exercise the ignored tests.

## Document References

- [`unit_test_standards.md`](unit_test_standards.md) — unit-test standards (storage backend pair, LLM provider trait, LLM recorder, fragment renderer, concurrency invariant, proptest!; cross-tier alignment).
- `tests/AGENTS.md` — engine-side test-infrastructure policy; test-mirror convention, structure overview, `[binary] ↔ fixture-weight` mapping.
- `chronicler_engine/docs/adr/adr-028-test-module-header-convention.md` (ADR-028) — the `check_test_module_header` guardrail; enforces single-line `//! <summary>` on every `*_tests.rs` file.
- `tests/test_utils/server.rs` — port allocation, `TestServer` lifecycle, `SERVER_MANAGED` PID registry.
- `tests/test_utils/wait.rs` — smart-waiting helpers (`wait_for_llm_idle`, `wait_for_status_ready`, `wait_for_element_children`, etc.).
- `tests/test_utils/browser.rs` — Playwright setup, `with_test_page`, `capture_failure_state`, `HEADED` / `SLOW_MO` env-var conventions.
- `tests/test_utils/settings_guard.rs` — `SettingsTestGuard` (Cross-cutting 1).
- `tests/helpers/sqlite_test_app_builder.rs` — `SqliteTestAppBuilder` (Pattern 1), including the `.pipeline_fn(...)` provider override.
- `tests/helpers/fixtures.rs` — `create_test_storage`, `TestDataBuilder`, world-fixture seeders.
- `chronicler_engine/scripts/tests/test_validate_docs.py` — front-matter, mode vocabulary, DOC-anchor regression suite.
- `chronicler_engine/docs/AGENTS.md` — autogenerated catalogue of all docs in `chronicler_engine/docs/`.
