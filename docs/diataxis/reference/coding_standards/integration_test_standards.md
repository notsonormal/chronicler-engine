---
diataxis: reference
title: Integration Test Standards
---

Integration tests live under `tests/`, split into binaries by fixture weight.
`tests/STRATEGY.md` holds the tiers and the placement rule. Each pattern below
names an exemplar: read the exemplar for the current API shape. This doc keeps
only the rules that an exemplar cannot state.

## Pattern 1 — SQLite `:memory:` app through `TestAppBuilder`

**Purpose.** Test an application collaborator through the production wiring.
`TestAppBuilder` (`src/test_support/test_app_builder.rs`) opens the production
SQLite backend on `:memory:`, so records persist through transactions as they do
in production.

- Read state through `state.game_view_query`. Take the Game through
  `test_support::active_game_of(&state)`.
- Injected `JobDeps` are rebound to the builder's storage. Pass the same
  `Arc<Storage>` through `.storage(...)` when the test changes storage that the
  Job reads.
- A test that seeds its own storage and skips the builder's seeding names the
  active Game: `.skip_seeding(true).active_game(game_id)`.
- Start a Job through a request on the builder's router. Call a collaborator
  directly only for a method that no request flows through, such as the
  lifecycle of `GameCatalogue`, the writes of `GameWriter` and the reads of
  `GameViewQuery`.

**Exemplars.** `tests/http/retrigger.rs`, `tests/http/games_create.rs`.

## Pattern 2 — Real `TestServer` process

**Purpose.** Spawn the `chronicler_engine` binary on a file-locked port, drive
it through Playwright or HTTP, and tear it down at test exit. This is the only
end-to-end process shape. The entry point is `with_test_page`
(`tests/test_utils/browser.rs`).

- Poll real conditions with the helpers in `tests/test_utils/wait.rs`. Never use
  a fixed `sleep`.
- `TestServer::Drop` kills the child, releases the port lock and deletes the DB
  file, so a test does no manual teardown.
- The browser tests need Node 18+ and `npx playwright install chromium`.

**Exemplars.** `tests/browser/*.rs`. `tests/llm/flow_llm_tests.rs` drives
`TestServer` and Chromium without `with_test_page`.

## Pattern 3 — Storage-direct round-trip

**Purpose.** Exercise a `Storage` method against a real SQLite backend, with no
`Job` and no application collaborator. `create_test_storage(game_id)`
(`tests/helpers/fixtures.rs`) pre-seeds the games row so FK-bearing tables
accept inserts. A case that needs no seeded row uses
`test_support::empty_sqlite_storage()`.

**Exemplar.** `tests/storage/*.rs`.

## Pattern 4 — HTTP one-shot

**Purpose.** Dispatch one `Request` against a `TestAppBuilder` router through
`tower::ServiceExt::oneshot`. There is no listening port. This is the dominant
HTTP test shape.

**Exemplars.** `tests/http/settings.rs`, `tests/http/games_create.rs`.

## Pattern 5 — Failure injection through storage overrides

**Purpose.** Test that a storage failure propagates through the layer above it.
`Storage::with_failure` fails one method. `Storage::with_test_failures()` gives a
handle that sets and clears a failure in the middle of a test.
`TestOverride::internal` models an unexpected runtime error and
`TestOverride::config` models an input-validation failure. The two map to
different `EngineError` arms.

**Exemplars.** `tests/http/settings.rs`, `tests/http/games_config.rs`.

## Pattern 6 — Bootstrap `run(Args)`

**Purpose.** Exercise the production `bootstrap::run(Args)` entry point and
observe the SQLite file that it opens. This is the only tier that reaches the
CLI startup path.

`run` opens `<exe_parent>/chronicler_<port>.db` and its WAL/SHM sidecars, so
call `cleanup_db_for_port(port)` first. A stale file makes migrations re-apply
`ALTER TABLE` statements ("duplicate column name"), and WAL files from
concurrent runs give a transient "disk I/O error".

**Exemplar.** `tests/bootstrap/run_branches.rs`.

## Pattern 7 — Arch-lint rule self-tests

**Purpose.** Test an arch-lint rule function with paired cases: one source string
that violates the rule, with the violation count and message fragment, and one
that does not, with an empty set. The rules and their self-tests live in
`tests/infrastructure/guardrails/`.

**Exemplars.** `structure_tests.rs`, `free_fn_tests.rs`.

## Cross-cutting rules

### 1 — `SettingsTestGuard`

Every HTTP test that changes settings (settings, connections, prompt presets)
starts with `let _guard = SettingsTestGuard::new();`. A read-only test does not
need it.

### 2 — Port locks under `/tmp`

`get_available_port(min, max)` (`tests/test_utils/server.rs`) claims a port in
3010–3050 with a lock file under `/tmp/chronicler_test_ports/`. The lock
directory stays under `/tmp`, not `target/`, so that a concurrent `cargo build`
cannot see stale locks. A test that binds an OS-assigned port (`127.0.0.1:0`)
stays out of the range.

### 3 — Mock connections go in over HTTP

`TestServer::new_with_mock` adds a Mock connection through `/connections/add`
and points the narrator and the quantifier at it. It does not write the SQLite
file, because the engine holds that file for its lifetime and `DbPool::new`
replays migrations on open. Use `TestServer::new` when the test needs real
env-var configuration.

### 4 — Debug a browser test

`HEADED=1 SLOW_MO=500 python build.py test-pattern <name>` runs one browser test
in a visible window. Both variables default off.

### 5 — Failure diagnostics

`capture_failure_state` writes a screenshot to `tmp/screenshots/` and a DOM dump
and engine log tails to `tmp/test_diagnostics/`. A wait helper that asserts a
terminal condition captures and then panics. Never catch that panic: a returned
`Err` suppresses the dump. A polling helper that returns a value does not
capture. `tests/test_utils/wait.rs` holds the list.

### 6 — Which harness

- A service method that changes state the test then reads, or a Job → the wired
  `TestAppBuilder` app (Pattern 1).
- A `Storage` method with no service → `create_test_storage(...)` or
  `empty_sqlite_storage()` (Pattern 3).
- A router-only HTTP assertion → `default_app()` / `.build()` (Pattern 4).

`build_service_with_storage()` returns the `AppState` and the exact
`Arc<Storage>` that the Jobs hold, so a test can assert through the read seams
and still read storage directly.

### 7 — The scripted model

A tier-1 test scripts every model call its Job makes. `ScriptedModel`
(`src/test_support/scripted_model.rs`) is a strict, holdable `LlmProvider`.
Entries are keyed by role, so a test does not depend on the order in which the
roles interleave. The app wiring runs the production Agents on the script, so a
tier-1 Job runs the real Quantifier and the real Options agent.

- A call that the test did not script fails the test.
- A scripted entry that no call used fails the test when the fake drops. Mark an
  entry that a stop or a cancel can skip with `.optional()`. No switch relaxes
  the whole script.
- `ScriptedModel::hold(role)` keeps the next call of that role until
  `release()`. This is how a test makes a Game busy with a real Job
  (`tests/http/races.rs`).

`MockBackend` is the lenient runtime provider that the browser tier and dev runs
use.

### 8 — Real-LLM tests

The tests in `tests/llm/` are `#[ignore]`d and also return early when the key
of their provider is not set: `OPENROUTER_API_KEY` or `OPENCODE_GO_API_KEY`.
`python build.py --llm-only` runs them. CI does not run them, so run
`--llm-only` once with both keys after a change to
`src/application/prompting/`, `src/adapters/driven/llm/` or LLM-parsing code.

### 9 — XSS regression

A test of a renderer that puts user data into HTML keeps its escaping
assertion. Do not delete an XSS assertion with no replacement. Do not move it
out of the scenario or the case that sees it.

## Document References

- `tests/STRATEGY.md` — the tiers, the placement rule, and the one rule for unit tests.
- `tests/AGENTS.md` — the structure index of the integration test tree.
