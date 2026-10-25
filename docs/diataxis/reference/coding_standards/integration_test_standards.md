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

## Pattern 1 — SQLite `:memory:` app + builder handoff

**Purpose.** Test an application collaborator through the production wiring.
`TestAppBuilder` defaults to `empty_sqlite_storage()`: the production SQLite backend
over `DbPool::new(":memory:")`. Pass an `Arc<Storage>` via `.storage(...)` when
the test wires its own instance. Records persist through transactions the way
they do in production, so a test can assert on the state a service left behind.

**The builder.** `TestAppBuilder` (`src/test_support/test_app_builder.rs`).
`default_test()` (or `with_data(data)`) seeds test data; `.storage(Arc<Storage>)`
overrides storage; `.model(Arc<ScriptedModel>)` runs the real Agents on that
script (Cross-cutting 7); `.job_deps(job_deps)` injects the collaborators of a
Job; `.skip_seeding(true)` leaves seeding to the test. Terminals:
`build()` → `Router`, `build_service()` → `AppState`, `build_with_state()` →
`(Router, AppState)`, `build_service_with_storage()` → `(AppState, Arc<Storage>)`.

**Wiring the scripted model.** A test that does not seed through the builder
uses `app_with_script` / `app_with_model_and_storage`
(`tests/http/support/app_wiring.rs`); those build the recorder with
`make_test_recorder_with_storage` (`src/test_support/fixtures.rs`) and the
Job collaborators with `make_test_job_deps` (`src/test_support/context.rs`).
Either way the test scripts every model call a Job makes (Cross-cutting 7).

**Observing state.** `state.game_view_query` is the read seam: `story_messages`
for the whole story, `dock` for the offered set and the Options failure, `status` for
the step or the failure line. Take the Game from the app the test built,
through the shared `test_support::active_game_of(&state)` (it panics when no
Game is active, which names a broken fixture). Injected `JobDeps` are rebound to
the builder's storage, so pass the same `Arc<Storage>` via `.storage(...)` when
the test mutates storage the Job should read. A test that seeds its own storage
and skips the builder's seeding also names the Game the app should point at:
`.skip_seeding(true).active_game(game_id)`.

**Exemplars.** `tests/http/retrigger.rs` (scripted model through
`TestAppBuilder::model` + `build_with_state`), `tests/http/games_create.rs`
(custom storage + `build`).

**Service-direct variant.** Use it for collaborator methods no request flows
through: lifecycle ops on `GameCatalogue` (`create_game`, `switch_game`,
`delete_game`, `list_games`, `active_game_id`), the writes of `GameWriter`
(stop, edit, delete, swipe switch), read-side queries on `GameViewQuery`. Do
not use it to start a Job — drive that through a request on the builder's
router, as above.
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

**Purpose.** Exercise a `Storage` method against a real SQLite backend, with no
`Job` and no application collaborators.

**The standard.** `create_test_storage(game_id)` in `tests/helpers/fixtures.rs`
opens `:memory:` SQLite and pre-seeds the games row so FK-bearing tables accept
inserts. A case that needs no seeded row builds
`test_support::empty_sqlite_storage()` instead.

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

**The standard.** `Storage::with_failure(StorageCall::..., TestOverride::internal(...))`
fails one method inline. `Storage::with_test_failures()` returns
`(Storage, TestFailureHandle)`; the handle's `.set(call, override)` /
`.clear(call)` toggle a failure mid-test. `StorageCall` names one `Storage`
method, so a wrong name does not compile. `TestOverride::internal` models
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

### Cross-cutting 6 — `TestAppBuilder` storage handoff for record assertions

`build_service_with_storage()` returns the wired `AppState` and the exact
`Arc<Storage>` the Jobs hold, so a test can assert through the read seams and
still read storage directly. `skip_seeding(true)` suits
a test that seeds its own Game; `build()` / `default_app()` suit
router-only HTTP tests.

The decision rule:

- Service method that mutates state the test then reads, or asserts through a
  Job? → the wired `TestAppBuilder` app.
- Service method on a pre-seeded state (read or write)? → `TestAppBuilder`.
- `Storage` method under test (no service at all)? → `create_test_storage(...)`,
  or `empty_sqlite_storage()` when the case needs no seeded row (Pattern 3).
- Router-only HTTP assertion? → `default_app()` / `.build()` (Pattern 4).

### Cross-cutting 7 — The scripted model and the Agents a tier-1 app runs

A tier-1 test scripts every model call its Job makes. `ScriptedModel`
(`src/test_support/scripted_model.rs`) is a strict, holdable `LlmProvider`: it
holds one entry per call and fails the test on a call the test did not script.
Entries are keyed by role (`ModelRole`), so a test declares `.narration("…")`,
`.event_narration("…")`, `.analysis(analysis_report(&[], Some("room2")))`,
`.options(set_a)` and the shorthand `.turn("…")` (a narration and the analysis
of it) without depending on the order the roles interleave. The analysis entry
is a parsed Quantifier answer (`analysis_report`), which is what makes a
generation's analysis confident.

Hand the model to the app with `app_with_script` /
`app_with_model_and_storage` (`tests/http/support/app_wiring.rs`), or with
`TestAppBuilder::model(model)` when the builder's other knobs seed the fixture.
`app_with_script`, `TestAppBuilder::model` and `app_with_model` wire the
production Agents through `configured_runner` (`src/test_support/context.rs`),
so a tier-1 Job runs the real Quantifier and the real Options agent on that
script. The tests that leave an Agent out build their runner through a closure
over `configured_runner_without(recorder, "options")`, passed to
`app_with_script_and_agents`.

A stray call fails the test three ways: the fake answers the caller with an
error naming the role, `wait_settled` asserts no recorded attempt carries that
failure, and the fake panics when it drops with a stray call on record.

`ScriptedModel::hold(role)` returns a `ModelHold` that keeps the next call of
that role until `release()`; the hold also releases with the handle, so a
failing test cannot strand the held call. This is how a test arranges a real Job
whose Game is busy: `tests/http/races.rs` holds one Job and asserts the second
request's answer, and `swipe_new.rs` and `retrigger.rs` hold a Job the same
way.

`MockBackend` stays the lenient runtime provider of `LlmBackendType::Mock`,
which the browser tier and dev runs use.

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

- `tests/STRATEGY.md` — the tiers, and the one rule for unit tests.
- `tests/AGENTS.md` — test-infrastructure policy; test-mirror convention, structure overview.
- `tests/test_utils/server.rs` — port allocation and `TestServer` lifecycle.
- `tests/test_utils/wait.rs` — smart-waiting helpers.
- `tests/test_utils/browser.rs` — Playwright setup, `with_test_page`, `capture_failure_state`, `HEADED` / `SLOW_MO`.
- `tests/test_utils/stub_server.rs` — the stub engine behind the stub-browser tier.
- `tests/test_utils/settings_guard.rs` — `SettingsTestGuard`.
- `tests/helpers/fixtures.rs` — `create_test_storage` and shared fixtures.
- `src/test_support/test_app_builder.rs` — `TestAppBuilder`.
- `src/test_support/context.rs` — `make_test_job_deps` and the Agent runner constructors.
- `src/test_support/fixtures.rs` — recorders and test-data fixtures.
- `scripts/tests/test_validate_docs.py` — front-matter, mode vocabulary, DOC-anchor regression suite.
- `docs/AGENTS.md` — autogenerated catalogue of all docs in `docs/`.
