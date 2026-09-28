# Settings: database as the sole source of truth

**Status:** Decision-complete
**Date:** 2026-09-24
**Depends on:** none
**Blocks:** none

---

## Summary

Engine settings are loaded once at boot into `Arc<RwLock<AppSettings>>` and the LLM providers built from them are frozen. Three defects follow from that:

1. **The seed never runs.** `data/settings.json` is never read at startup. `AppSettings::default()` fills the gap, so the engine serves a hardcoded 3-connection list. The live DB at `target/debug/chronicler_3000.db` has zero `settings` rows, which proves it.
2. **Providers are frozen at boot.** `build_app_graph` reads the settings lock once (`src/bootstrap/wiring.rs:146-164`) and bakes each connection into a concrete backend. Changing the narrator connection in the dashboard updates the card, the Arc, and the DB — and does not change the provider that answers narrations. The UI reports "Settings saved!" and lies.
3. **External writes are invisible.** Reads go to the boot-time Arc, not the DB. Nothing reloads it.

This plan removes the Arc. Settings resolve from `Storage` per call, matching how `PromptAssembler` already reads `max_context_tokens` live (`src/application/prompting/assembler.rs:118-125`). All three defects close together.

The `deepseek-v4-flash` narrator default is one line in `data/settings.json`. It is trivial on its own and is not the reason for this plan.

---

## Key Changes

### 1. Read source (`Storage` replaces the Arc)

Remove `Arc<RwLock<AppSettings>>` from the graph. Settings resolve from `Storage::get_settings()` per call.

Affected holders (production):

| File | Change |
|---|---|
| `src/adapters/driving/http/app_state.rs:41,58,71-73` | Drop the `settings` field **and** the `AppState::settings()` accessor at `:71-73`, which clones the whole struct under the lock. Its two production callers — `action/handlers/actions.rs:94` and `chat_window/handlers/chat_window.rs:89` — become `storage.get_settings()` calls and gain a `Result` to handle. |
| `src/application/pipeline/action_pipeline/core.rs:39,45,56,79,100,133` | Replace `settings` with `storage` (already a field) |
| `src/application/prompting/assembler.rs:53,70` | `settings: Option<Arc<RwLock<AppSettings>>>` → `storage: Option<Arc<Storage>>` |
| `src/application/games/catalogue.rs:18,25` | Replace `settings` with storage |
| `src/application/games/view_query.rs:21,28` | Same |
| `src/application/agents/quantifier/agent.rs:24,40` | Same |
| `src/application/agents/options/agent.rs:23,39` | Same |
| `src/application/agents/registry.rs:26` | Same |
| `src/bootstrap/wiring.rs:70,84,143,177` | Drop the parameter |
| `src/bootstrap/init_game.rs:19,129` | Replace with storage |
| `src/test_support/context.rs:95,102,113,120,127,136,139,185,186` | Drop the Arc from the four test constructors and from `build_app_graph_for_tests` |
| `src/test_support/test_app_builder.rs:190-195` | Stop building the Arc; pass storage only |

`tests/helpers/sqlite_test_app_builder.rs` also threads the Arc, but it compiles into no test binary (see NOT in scope) and needs no change here.

The 18 `settings.read()` sites and 8 `settings.write()` sites convert accordingly. Writes become `save_settings` only.

Arc site counts per production file (for the diff estimate): `pipeline/action_pipeline/core.rs` 5, `bootstrap/wiring.rs` 4, `bootstrap/init_game.rs` 2, `prompting/assembler.rs` 2, `games/view_query.rs` 2, `games/catalogue.rs` 2, `agents/quantifier/agent.rs` 2, `agents/options/agent.rs` 2, `agents/registry.rs` 1, `http/app_state.rs` 1. The 19 HTTP handler read/write sites reach the Arc through `app_state.settings`; they change when the field does.

Application code importing `storage` is established practice — 15 application files already do it. `arch-lint.toml` has no rule against it.

### 2. Provider resolution

`LlmCallRecorder` resolves its provider at call time instead of storing a frozen `Arc<dyn LlmProvider>`.

- `src/application/llm_recorder.rs` — hold `Arc<Storage>` plus a resolver, resolve inside `complete()`.
- `src/bootstrap/wiring.rs:32-64` — `recorder_for` takes the storage handle and a resolver closure instead of `config: &LlmProviderConfig`. Three construction sites (`:148`, `:150`, `:151`) each pass their own resolver:
  - narration → `narration_connection()`
  - quantifier → `quantifier_connection()`
  - options → `find_connection("options")`, else narration (preserves the existing rule at `wiring.rs:55-64`)
- `provider()` (6 call sites, all `tracing::info!` — `core.rs:60-61,116-117`, `quantifier/orchestration.rs:91-92`, `options/orchestration.rs:61-62`) resolves then logs. Output is unchanged.

Provider construction is cheap and pure: `OpenRouterBackend::from_config` copies four fields and does no I/O (`openrouter.rs:19-26`). The `reqwest::blocking::Client` is built per request inside `call_chat_completions` (`transport/utils/client.rs:24`). No connection pool exists to preserve.

In-flight semantics: a running generation keeps the provider it captured. The next call uses the new one. No coordination with `GenerationGate`.

### 3. Fail loud on invalid configuration

Replace the silent Mock fallbacks.

| Site | Today | New |
|---|---|---|
| `src/domain/model/settings.rs:373-384` (`narration_connection()`, `quantifier_connection()`) | Falls back to a Mock config | Return `Result`; unknown id is `EngineError::Config` |
| `src/domain/model/llm_backend.rs:21-26` (`LlmBackendType::from`) | Unknown string → `LlmBackendType::Mock` | Return `Result`; unknown string is an error |
| `src/adapters/driving/http/settings/handlers/settings.rs:110,173` | Silent Mock on bad form input | Surface the error in the fragment |
| Seed file load | No validation | A bad `data/settings.json` fails boot |

`LlmBackendType::from` has **three** callers, not two: the two form handlers plus `from_env` (`llm_backend.rs:32-36`), which is itself `.map_or(..., Self::from)`. Changing `from` to return `Result` changes `from_env` too. Note that `from_env` has no production caller — `LLM_BACKEND` is read nowhere outside `llm_backend_tests.rs`. Decide its fate as part of this change rather than leaving a fallible wrapper with no caller.

Four `*_connection()` consumers already return `Result`, so propagation is mechanical: `wiring.rs:148,150`, `assembler.rs:125`, `init_game.rs:153`.

Error-swallowing policy (Q7=A): a DB read failure still falls back to defaults (`src/utils/settings.rs:20-23` and `run.rs:95,154` keep their `unwrap_or_else`). A bad seed *file* fails boot. The two cases differ because a corrupt DB row is a runtime fault and a malformed seed file is an authoring error.

### 4. Seed settings from `data/settings.json`

- `src/adapters/driven/storage/settings.rs` — add a settings read-modify-write entry point. `get_settings()` returns `AppSettings::default()` on an absent row (`:22`), which cannot be distinguished from a row equal to the default, so the seed needs either an explicit existence check or an `INSERT OR IGNORE` write.
- `src/bootstrap/load.rs` — add a settings seed pass. Read `data/settings.json`, validate, write only when the row is absent.
- `src/bootstrap/run.rs` — call it in `prepare_data` beside the two existing seed calls (`:62`, `:66`), before the settings load at `:95`.

**Insert-if-absent is already the established pattern.** `seed_persona` (`personas.rs:63`) uses `INSERT OR IGNORE`, and `seed_character` (`characters.rs:61`) does the same. The InMemory arm guards with an explicit `iter().any()` check (`personas.rs:79-86`). `seed_settings` (`settings.rs:78-80`) is currently an alias for `save_settings`, which is `INSERT OR REPLACE` — it overwrites. Change it to mirror `seed_persona`: `INSERT OR IGNORE` on SQLite, an existence guard on InMemory. That lands the seed with no separate `has_settings_row()` query at all, because the guard is the insert itself.

This also fixes a real bug in the current alias: `test_seed_settings_idempotent` (`settings_tests.rs:23`) passes only because it rewrites identical data. Under `INSERT OR REPLACE` a second boot overwrites the user's edited settings with the file's contents.

### 5. `data/settings.json` corrections

The file must match `AppSettings` to be a valid seed.

| Field | Action |
|---|---|
| `agents` | Add the `options` agent. Only `quantifier` is present; absent, `run_options_generation` returns `Ok(empty)` and callers report the agent unavailable (`src/application/pipeline/action_pipeline/options.rs:52-57`) |
| `mode_preset_registry` | Add explicit novel + IF bundles |
| `active_options_prompt_preset_id` | Add `"options_default"` |
| `active_system_prompt_preset_id` | Remove. Not an `AppSettings` field; serde drops it silently |
| `active_quantifier_prompt_preset_id` | Remove. Same |
| `narration_connection_id` | Keep `deepseek-v4-flash` |
| `deepseek-v4-flash.provider` | Keep `OpenRouter`, or switch to `DeepSeek`. Both route correctly; they differ on which backend serves the call (`wiring.rs:41`) |

Preset IDs verified present: `system_default`, `system_if_default` (`data/prompt_presets/system/`), `quantifier_default`, `impersonate_default`, `options_default`. Connection IDs must stay consistent with `narration_connection_id` and `quantifier_connection_id`, now enforced by the validation in change 3.

### 6. Schema catch-up

`data/schemas/settings.schema.json` still declares `active_system_prompt_preset_id` and `active_quantifier_prompt_preset_id` as top-level properties, and omits `mode_preset_registry`, `active_options_prompt_preset_id`, and `agents`. `scripts/validate_data.py` reports `PASS` on the file against this stale contract. Update the schema to match `AppSettings`.

### 7. Remove `--settings-path`

Seed replaces its only human use. With the DB as the read source, the browser tests no longer need it either.

- `src/utils/cli.rs:34` — drop the flag
- `src/bootstrap/run.rs:135-150` — drop the import branch
- `src/utils/settings.rs:13` — drop `get_settings_path`, or keep if still wanted for tooling
- `src/utils/settings_tests.rs:26-28` — drop the path test
- `tests/test_utils/server.rs:248-285` — replace the temp-file write with the HTTP injection below
- `tests/bootstrap/run_branches.rs:55,91` — drop the field
- `docs/diataxis/reference/coding_standards/integration_test_standards.md:98,264-266` — rewrite the mock-injection pattern

**Mock injection goes over HTTP, not through the DB file.** The 8 affected tests spawn the engine as a child process, so a post-start write through `Storage` would mean a second process opening the live SQLite file. Two blockers rule that out: `DbPool::new` replays migrations unconditionally (`db.rs:18-21`), and neither `busy_timeout` nor WAL is set anywhere in `src/`, so SQLite's 0 ms default timeout makes a concurrent write fail with `SQLITE_BUSY` immediately. The engine also holds the file for its lifetime.

Instead, `start_server_with_env` POSTs the mock connections to the real endpoints after `wait_for_server` returns:

1. `POST /settings/connections/add` for each mock connection, with `conn_provider=mock` and a model string
2. `POST /settings/set-narrator/<id>` and `POST /settings/set-quantifier/<id>`

This keeps the injection inside the same process boundary the rest of the test uses, exercises the real settings path, and needs no test-only production endpoint. `add_connection_handler` generates `conn-<millis>` ids, so the harness reads the id back from the rendered card rather than assuming one; the browser tests do not reference the mock connection ids anywhere, so the naming is free.

Ordering constraint: this harness rewrite must land **with or before** change 1. If the Arc removal lands first, the spawned tests have no way to inject Mock connections and break in between.

### 8. Spec correction

`docs/specs/settings.md` Scenario 20.5 pins the current unvalidated behavior:

> Then the response is 200 (the handler does not validate the id)

Change 3 makes an unknown connection id an error, so this scenario is now wrong. Rewrite it to declare rejection, and invert `tests/http/settings.rs:117-132` (`test_post_settings_accepts_unknown_connection_id`).

### 9. Read-modify-write on `Storage`

Removing the `Arc<RwLock<AppSettings>>` removes more than a cache. The `write()` guard currently spans the whole read-modify-write in every settings handler:

```rust
let mut settings = try_lock!(app_state.settings.write());   // acquire
settings.narration_connection_id = id;                      // mutate
app_state.settings_service.save_settings(&settings)         // persist
```

That guard makes the sequence atomic. Without it, each handler becomes `get_settings()` → mutate → `save_settings()` across two independent calls, and two overlapping requests can lose one update. `Storage`'s backend mutex does not close the gap — it serializes each call, not the span. Seven handlers use this shape: `save_settings_handler`, `save_text_check_handler`, `add_connection_handler`, `edit_connection_handler`, `delete_connection_handler`, `set_narrator_handler`, `set_quantifier_handler`.

Add a read-modify-write entry point to `Storage` that holds the backend mutex once:

- `Storage::update_settings(f: impl FnOnce(&mut AppSettings) -> Result<T>) -> Result<T>` — acquire `with_backend_mut` once, read, apply `f`, persist, return `f`'s value.
- The 7 handlers call it instead of `try_lock!` + `save_settings`. Each closure returns the rendered HTML string, so the handler keeps its single response path.
- `save_settings` stays for callers that already hold a complete value; `update_settings` is the path for handlers.

This reuses the mutex `Storage` already has rather than introducing a settings repository type whose only job would be to hold a second lock. `SettingsTestGuard` (`tests/test_utils/settings_guard.rs`) exists because settings mutations are process-global; it stays, and the tests keep passing.

---

## Documentation

1. `docs/diataxis/explanation/storage_design.md:28` — the sentence "seeded once from `data/settings.json`, then read from the database for the lifetime of the process" is false today and true after this plan. Add the design rationale for resolving from the database rather than caching.
2. `docs/diataxis/reference/storage.md:33` — "Bootstrap loads the engine's settings once; reload happens only on process restart." Becomes false. Rewrite.
3. `docs/diataxis/reference/startup.md:46` — "Settings are loaded once during bootstrap and reload on restart." Becomes false. Rewrite, and add a concrete settings-seeding line to the Seeding Order section.
4. `docs/diataxis/reference/architecture_system.md:32-34,50` — describes the `Arc<RwLock<AppSettings>>` and `AppState.settings`. Both go. The `load_settings` and `AppState.settings` mentions are mechanics leaks under `docs/AGENTS.md`; rephrase or drop.
5. `docs/CHANGELOG.md` — under "Unreleased": settings resolve from the database per call; seeded from `data/settings.json`; `--settings-path` removed; invalid connection ids and unknown provider strings now error.
6. `CONTEXT.md` — no change. `Settings` and `Connection` are general programming concepts; `CONTEXT-FORMAT.md` excludes those.

No ADR. The repo has no `docs/adr/` convention, `docs/AGENTS.md` does not mention one, and the Diátaxis tree is the declared home for "why" content.

---

## Implementation Order

1. Plan doc (this file).
2. `data/settings.json` corrections (change 5) and schema catch-up (change 6). `python scripts/validate_data.py` must pass.
3. Fail-loud changes (change 3): `narration_connection()` / `quantifier_connection()` return `Result`; `LlmBackendType::from` returns `Result`. Fix the four call sites and the two form handlers.
4. Unit tests for change 3.
5. Read-modify-write entry point on `Storage` (change 9) and the 7 settings handlers rewritten onto it.
6. Seed (change 4): `INSERT OR IGNORE` semantics on `seed_settings`, the seed pass in `bootstrap/load.rs`, the call in `prepare_data`.
7. Seed unit tests: absent row seeds, present row is not overwritten, malformed file fails boot, DB read failure still falls back to defaults.
8. Provider resolution (change 2): `LlmCallRecorder` resolves per call; three resolvers at `wiring.rs`.
9. HTTP mock injection in `start_server_with_env` (change 7) and `--settings-path` removal. Lands before step 10.
10. Read-source removal (change 1): drop the Arc across the holders listed above, including `AppState::settings()` and the test-support constructors. Largest diff; lands last and alone.
11. Spec correction (change 8) and its test.
12. Doc updates.
13. `python build.py`.

Steps 3-4 precede 5-7 because the seed validates on load. Step 5 precedes 10 because the handlers must have a read-modify-write path before the lock that provided it goes away. Step 9 precedes 10 because the spawned tests need Mock injection that survives the Arc removal.

---

## Failure Modes

1. **Seed validation rejects the shipped `data/settings.json`.** The engine fails to boot. Mitigation: step 2 runs `scripts/validate_data.py` before the seed code exists, and step 6 tests the exact shipped file.
2. **A dangling `narration_connection_id` in a user's DB after upgrade.** Upgrading with an existing DB skips seeding, so the row is whatever the UI last wrote. If it references a deleted connection, resolution errors where it previously returned Mock. Mitigation: the dashboard delete handler already reassigns (`settings.rs:207-213`), so this requires hand-edited data. The error message must name the missing id.
3. **A test relies on the Mock fallback.** Removing it changes behavior for any test that feeds an invalid id. Grep for tests asserting Mock narration with a bad reference before landing change 3.
4. **Per-request `get_settings` cost.** Every resolution now takes `Storage`'s backend mutex and runs one query plus JSON deserialization of `connections`, `agents`, and `mode_preset_registry`. LLM calls take seconds, so the relative cost is small. Resolution happens per generation and per request, not per token.
5. **`Storage`'s mutex now sits in the generation path.** `with_backend_mut` (`core.rs:126-151`) serializes all storage operations. Settings reads now join that queue. SQLite is synchronous today, so this adds no new class of blocking.
6. **`PromptAssembler` gains a `Storage` dependency.** It is currently storage-free. 15 application files already import storage and `arch-lint.toml` permits it, but this is new for the prompting layer.
7. **HTTP mock injection races with boot.** The harness POSTs mock connections after `wait_for_server` returns (`tests/test_utils/server.rs:321-329`). Boot has completed by then and the seed is skipped because the row exists. A test that generates immediately after startup must inject before its first request.
8. **Scenario 20.5 inversion breaks the spec-tag check.** `scripts/validate_feature_spec.py` requires every declared scenario to have a covering test. Rewriting 20.5 requires updating its test in the same change.
9. **Lost update between concurrent settings edits.** Removing the `Arc<RwLock<AppSettings>>` also removes the `write()` guard that currently spans read-modify-write across the 7 handlers. Two overlapping requests can clobber each other. Mitigated by change 9: `Storage` owns a read-modify-write method that takes the backend mutex once, restoring the guarantee under a lock `Storage` already holds.

---

## NOT in scope

- `connections[0]` reassignment on delete (`settings.rs:215-221`) silently switches to an arbitrary connection rather than the previous one. Separate defect.
- Removing `AppSettings::default()`'s 3-connection list. It becomes test-only after this plan; shrinking it is a follow-up.
- Caching layer for settings reads. Listed as a possible mitigation for failure mode 4, deliberately not adopted.
- `GenerationGate` coordination with settings writes.
- `tests/helpers/sqlite_test_app_builder.rs` is orphaned. `Cargo.toml` declares seven test targets and none references it (`tests/storage/mod.rs` pulls only `../helpers/fixtures.rs` and `../helpers/storage_ext.rs`), so it compiles into no binary. It appears to be dead code, not part of this change.

---

## Verification

```bash
python build.py
```

Must pass: fmt + clippy `-D warnings` + guardrails + architecture + nextest. Run `python scripts/validate_data.py` and `python scripts/validate_feature_spec.py` early — both guard contracts this plan changes.

Required new coverage:

- **Unit** — `seed_settings` inserts when absent and leaves an edited row untouched; malformed seed file fails boot; DB read failure falls back to defaults; unknown connection id errors; unknown provider string errors; `Storage::update_settings` applies the closure and persists.
- **Integration** — a fresh DB serves the narrator connection from `data/settings.json`; changing the narrator connection takes effect without restart; deleting the last connection is refused; `POST /settings` with an unknown id is rejected.
- **Harness** — spawned-binary tests get Mock connections through the HTTP injection path after startup.

Manual check: start the engine with a deleted DB, confirm the narrator connection is `deepseek-v4-flash`, change it in the dashboard, and confirm the next narration uses the new model without a restart. Confirm `llm_messages` records the new `backend_name`.
