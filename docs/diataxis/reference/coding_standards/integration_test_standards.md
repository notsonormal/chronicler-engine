---
diataxis: reference
title: Integration Test Standards
---

Integration tests live under `tests/`, split into binaries by fixture weight
(architecture / guardrails / storage / bootstrap / http / browser / llm); the
placement rule is in `tests/STRATEGY.md`. Each pattern below names an exemplar —
read the exemplar for the current API shape rather than copying a snippet,
because the signatures it uses are the single source of truth. Only the rules an
exemplar cannot state are inlined here.

## Pattern 1 — In-memory app + builder handoff

**Purpose.** Test an application collaborator through the production wiring.
`TestAppBuilder` defaults to `Storage::new_in_memory()` (the `InMemory` backend);
pass a SQLite `Arc<Storage>` via `.storage(...)` when the test needs the sqlite
backend. State persists through snapshots and messages the way it does in
production, so a test can assert on the state a service left behind.

**The builder.** `TestAppBuilder` (`src/test_support/test_app_builder.rs`).
`default_test()` (or `with_data(data)`) seeds test data; `.storage(Arc<Storage>)`
overrides storage; `.pipeline(pipeline)` injects a mock-wired `ActionPipeline`;
`.skip_seeding(true)` leaves seeding to the test. Terminals: `build()` →
`Router`, `build_service()` → `AppState`, `build_with_state()` →
`(Router, AppState)`, `build_service_with_storage()` → `(AppState, Arc<Storage>)`.

**Wiring the mock pipeline.** Build the recorder with `make_test_recorder` /
`make_test_recorder_with_storage` (`src/test_support/fixtures.rs`), then the
pipeline with `make_test_pipeline_with_backends` or
`make_test_pipeline_with_mock_quantifier` (`src/test_support/context.rs`).
Configure narration per call with
`MockBackend::default().with_narrations(vec![...])`; quantifier responses with
`.with_prompt_responses(vec![...])` (Cross-cutting 7).

**Observing state.** `app.message_service.load_or_fresh()` is the persisted
state. The injected pipeline is rebound to the builder's storage, so pass the
same `Arc<Storage>` via `.storage(...)` when the test mutates storage the
pipeline should read.

**Exemplars.** `tests/http/retrigger.rs` (mock quantifier + `build_with_state`),
`tests/http/games_create.rs` (custom storage + `build`).

**Service-direct variant.** Use it for collaborator methods no request flows
through: lifecycle ops on `GameCatalogue` (`create_game`, `switch_game`,
`delete_game`, `list_games`, `current_game_id`), status/cancellation on
`GenerationGate`, read-side queries on `GameViewQuery`. Do not use it for
`process_action` — drive that through the builder + injected pipeline form above.
Direct-storage tests with no service are Pattern 3.

## Pattern 2 — Real `TestServer` lifecycle + port allocation

**Purpose.** Spawn the actual `chronicler_engine` binary as a child process on a
dynamically allocated port, drive it through Playwright or HTTP, and tear it down
at test exit. This is the integration tier's only end-to-end process shape.

**The standard.** `with_test_page(config_path, world, persona, closure)` in
`tests/test_utils/browser.rs` is the entry point: it allocates a file-locked
port, spawns the binary, waits for HTTP readiness, launches Chromium, and passes
`(page, port)` to the closure. Poll real conditions with the helpers in
`tests/test_utils/wait.rs` — e.g. `wait_for_element_children(&page, "#story-log .log-entry", n)` — or `count_log_entries` (`tests/test_utils/browser.rs`) — never a fixed `sleep`.

**Teardown.** `TestServer::Drop` kills the child (SIGKILL via `Child::kill`),
waits, releases the port lock, and deletes the SQLite DB file, so tests do no
manual teardown. A stale server on a port is reaped by `kill_existing_server`,
which sends SIGTERM through `terminate_pid`.

**Exemplars.** `tests/browser/*.rs`; `tests/llm/flow_llm_tests.rs` drives
`TestServer` and Chromium directly without `with_test_page`.

## Pattern 3 — Storage-direct round-trip

**Purpose.** Exercise a `Storage` method against a real SQLite (or in-memory)
backend, with no `ActionPipeline` and no application collaborators.

**The standard.** `create_test_storage(game_id)` in `tests/helpers/fixtures.rs`
opens `:memory:` SQLite and pre-seeds the games row so FK-bearing tables accept
inserts. Use `Storage::new_in_memory()` when the test needs no SQLite startup.

**Exemplar.** `tests/storage/*.rs`.

## Pattern 4 — HTTP one-shot via `tower::ServiceExt::oneshot`

**Purpose.** Dispatch a single `Request` against a `TestAppBuilder`-built router.
There is no listening port — the handler runs once and the response is the
assertion. This is the dominant HTTP test shape.

**The standard.** `TestAppBuilder::default_app()` (or `.default_test()...build()`)
returns the `Router`; call `.oneshot(req)`. Mutating global `AppSettings` requires
`SettingsTestGuard::new()` first (Cross-cutting 1). Install custom storage before
building with `.storage(Arc::new(...))`.

**Exemplars.** `tests/http/settings.rs`, `tests/http/games_create.rs`.

## Pattern 5 — Failure-injection via storage overrides

**Purpose.** Test that a storage failure propagates correctly through the layer
above it. The failure is the system under test.

**The standard.** `Storage::with_failure(method, TestOverride::internal(...))`
fails one method inline. `Storage::with_test_failures()` returns
`(Storage, TestFailureHandle)`; the handle's `.set(method, override)` /
`.clear(method)` toggle a failure mid-test. `TestOverride::internal` models
unexpected runtime errors and `TestOverride::config` models input-validation
failures; the two map to different `EngineError` arms downstream.

**Exemplars.** `tests/http/settings.rs`, `tests/http/games_config.rs`.

## Pattern 6 — Bootstrap `run(Args)` direct invocation

**Purpose.** Exercise the production `bootstrap::run(Args)` entry point end-to-end
and observe the SQLite file the binary opens. This is the only integration tier
that reaches the CLI startup path.

**The standard.** Build `Args` from the real struct in `src/utils/cli.rs` (it
carries `world`, `persona`, `list_worlds`, `port`, and `host`), allocate a port
with `get_available_port(3010, 3050)`, and call `run(args)`. `run` opens
`<exe_parent>/chronicler_<port>.db` plus WAL/SHM sidecars, so call the local
`cleanup_db_for_port(port)` first. Stale files make migrations re-apply `ALTER
TABLE` statements and surface as "duplicate column name: persona_key", or as a
transient "disk I/O error" when WAL files from concurrent runs collide.

**Exemplar.** `tests/bootstrap/run_branches.rs`.

## Pattern 7 — Arch-lint rule self-tests

**Purpose.** Test an arch-lint rule function itself with paired positive and
negative cases: feed synthetic source strings through `check_<rule>()` and assert
the violation set. The rule functions and their self-tests live in
`tests/infrastructure/guardrails/`.

**The standard.** One test feeds source that should violate and asserts the
violation count and message fragment; a paired test feeds source that should not
and asserts the set is empty.

**Exemplars.** `structure_tests.rs`, `free_fn_tests.rs`,
`inherent_impl_tests.rs`, `location_tests.rs`.

## Cross-cutting patterns

### Cross-cutting 1 — `SettingsTestGuard` for settings mutations

Every HTTP test that mutates `AppSettings` (settings, connections, prompt
presets) starts with `let _guard = SettingsTestGuard::new();`. `AppSettings`
lives in a global static, so parallel tests would race without it. The guard is a
process-wide `Mutex<()>` with poisoning recovery, so one panicking test cannot
deadlock later ones. Read-only tests do not need it. A settings mutation without
the guard is a real bug. `tests/test_utils/settings_guard.rs`.

### Cross-cutting 2 — File-locked port allocation on 3010–3050

`tests/test_utils/server.rs::get_available_port(min, max)` takes explicit bounds
and claims a port by creating `/tmp/chronicler_test_ports/port_<N>.lock` with
`OpenOptions::create_new` (atomic on POSIX), writing its PID.
`get_config_port(config_path)` is the wrapper that reads `tests/test_config.json`
and delegates. On contention the helper GCs locks whose PID is dead and retries
with backoff. The lock directory is deliberately under `/tmp` rather than
`target/` so concurrent `cargo build` invocations cannot see stale locks — do not
move it into the workspace. Used by `with_test_page` browser tests, the bootstrap
tests, and the LLM tests; the stub tier allocates from the same range through
`StubServer::start` but only binds a listener. Tests that bind an OS-assigned
port (`127.0.0.1:0`) directly opt out of the 3010–3050 range by design.

### Cross-cutting 3 — Mock-backend auto-injection over HTTP

`TestServer::new_with_mock` spawns the binary, waits for HTTP readiness, then
`inject_mock_connections(port)` POSTs a Mock connection to `/connections/add` and
points both the narrator and quantifier roles at it. Injection goes through HTTP
rather than writing the SQLite file because the engine holds that file for its
lifetime and `DbPool::new` replays migrations on open — a second writer would
collide. Use `TestServer::new` when the test needs real env-var configuration
(e.g. `OPENROUTER_API_KEY`); connections seeded from `data/settings.json` stay
listed but unused once both roles point at the Mock. `oneshot` tests (Pattern 4)
never start the binary and bypass this, as does the stub tier.

### Cross-cutting 4 — `HEADED` / `SLOW_MO` env-var overrides for Playwright

`launch_chrome()` (`tests/test_utils/browser.rs`) reads `HEADED=1` (headed browser
window) and `SLOW_MO=<ms>` (pause between Playwright steps); both default off.
Debug one test with `HEADED=1 SLOW_MO=500 python build.py test-pattern <name>`,
and default to headless in CI. The convention applies to the browser binary only;
the LLM tests call `TestServer::new` directly, not `with_test_page`.

### Cross-cutting 5 — `capture_failure_state` diagnostic dump

`capture_failure_state(page, test_name)` writes a screenshot to
`tmp/screenshots/<epoch>_<sanitized_name>.png`, and the DOM dump plus per-server
engine log tails under `tmp/test_diagnostics/`. The wait helpers that assert a
terminal condition panic on timeout after capturing; the polling helpers that
report a value — `wait_for_llm_idle` (`Result<(), ()>`),
`wait_for_element_persist` (`bool`), `wait_for_condition_async` (`bool`) — return
instead and do not dump. `wait_for_status_ready_or_error` captures because both
"ready" and "error" are terminal. Never swallow a capturing helper's panic:
returning `Err` would suppress the dump. See `tests/test_utils/wait.rs` for the
current list. There is no sync-polling equivalent; sync failures rely on
`cargo nextest`'s `--failure-output` reporting.

### Cross-cutting 6 — `TestAppBuilder` storage handoff for snapshot assertions

`build_service_with_storage()` returns the wired `AppState` and the exact
`Arc<Storage>` the pipeline holds, so a test can assert through the
snapshot/message path and still read storage directly. `skip_seeding(true)` suits
a test that seeds its own game state; `build()` / `default_app()` suit
router-only HTTP tests.

The decision rule:

- Service method that mutates state the test then reads, or asserts through the
  snapshot/generation path? → the wired `TestAppBuilder` app.
- Service method on a pre-seeded state (read or write)? → `TestAppBuilder`
  (faster; no SQLite startup).
- `Storage` method under test (no service at all)? → `create_test_storage(...)`
  or `Storage::new_in_memory()` (Pattern 3).
- Router-only HTTP assertion? → `default_app()` / `.build()` (Pattern 4).

### Cross-cutting 7 — Narrator vs quantifier mock wiring

Two pipeline constructors cover the common cases.
`make_test_pipeline_with_backends` forwards the caller's `AgentRegistry`; call
sites commonly pass `AgentRegistry::default()`, so the quantifier comes from the
registry rather than the recorder. Use `make_test_pipeline_with_mock_quantifier`
to supply a separate Mock quantifier provider. Configure narration text per call
with `with_narrations`, and quantifier JSON with `with_prompt_responses`, on the
`MockBackend`.

### Cross-cutting 8 — `#[ignore]` for real-LLM tests + `--llm-only` invocation

The two tests in `tests/llm/flow_llm_tests.rs` are `#[ignore = "slow: requires
OPENROUTER_API_KEY"]` and also short-circuit at runtime when `has_llm_api_key()`
is false. Defense in depth: `#[ignore]` keeps them out of `python build.py` and
`python build.py integration`; `--llm-only` forces them in via `--run-ignored`,
where the runtime check keeps an unset-key run to about a second. Run
`--llm-only` once locally with a valid key when modifying
`src/application/prompting/` or `src/adapters/driven/llm/` or LLM-parsing code —
CI does not exercise the ignored tests.

## Document References

- [`unit_test_standards.md`](unit_test_standards.md) — unit-test standards.
- `tests/AGENTS.md` — test-infrastructure policy; test-mirror convention, structure overview.
- `tests/test_utils/server.rs` — port allocation and `TestServer` lifecycle.
- `tests/test_utils/wait.rs` — smart-waiting helpers.
- `tests/test_utils/browser.rs` — Playwright setup, `with_test_page`, `capture_failure_state`, `HEADED` / `SLOW_MO`.
- `tests/test_utils/stub_server.rs` — the stub engine behind the stub-browser tier.
- `tests/test_utils/settings_guard.rs` — `SettingsTestGuard`.
- `tests/helpers/fixtures.rs` — `create_test_storage` and shared fixtures.
- `src/test_support/test_app_builder.rs` — `TestAppBuilder`.
- `src/test_support/context.rs` — `make_test_pipeline_*` constructors.
- `src/test_support/fixtures.rs` — recorders and test-data fixtures.
- `scripts/tests/test_validate_docs.py` — front-matter, mode vocabulary, DOC-anchor regression suite.
- `docs/AGENTS.md` — autogenerated catalogue of all docs in `docs/`.
