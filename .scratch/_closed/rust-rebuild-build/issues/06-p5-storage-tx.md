# P5: the storage `Tx`

Type: task
Status: resolved
Blocked by: 05

## Question

Build §8 of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) on branch `wf/rebuild-p5`, from the current `main`. Read §1, §2 and §8 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §8 pass. The owner has approved the last commit, and the branch has merged to `main`.

## Comments

**2026-10-17, implementer (P5 commit 1, uncommitted).** Branch `wf/rebuild-p5`, base `1c3ab297` (main). All of §8 except "The target tests of the core step" and "Spec scenarios", which are the second commit. No commit was made.

Facts measured.

| Item | Result |
|---|---|
| `src/adapters/driven/storage/transaction.rs` (new) | 122 lines. `TxWrite { InsertGame, DeleteGame }` is the typed key of one record write; `Tx<'a>` wraps one rusqlite `unchecked_transaction` and the `WriteOverrides` of the backend layer; `Tx::insert_game` and `Tx::delete_game` check their key before they write (`WriteOverrides::fail_if_injected`); `Tx::write_game_row` is the one Game-row insert, `pub(crate)`, shared with the kept `Storage::create_game_from_request`. |
| `Storage::transaction` / `transaction_new_game` | No token (C2 adds `TxToken`). `transaction` runs the closure inside one `with_backend_writes` call, so one unit is one hold of the backend mutex. `transaction_new_game(request, \|game_id, tx\| ..)` inserts the Game row first and hands the id to the closure. The in-memory arm answers `EngineError::Internal`. No production caller. |
| The record-write injection | A second map on the `Test` layer (`BackendKind::Test.writes`, keyed by `TxWrite`) plus `TestFailureHandle::set_write` / `clear_write`; `assert_no_unconsumed` and the `Drop` warning cover both maps. `WriteOverrides` (`core.rs`, `pub(crate)`) is `Direct` or a clone of that map, held by the `Tx`; `with_backend_writes` hands it over, and `with_backend_mut` is now a 1-line wrapper, so the 47 existing call sites are untouched. |
| `Storage::newest_game_of_world(world_key) -> Option<GameId>` | `games.rs`, both backends, through `with_backend_mut`. SQLite keeps today's SQL and order (newest `messages.timestamp`, else `games.updated_at`) and adds the tie-break `, g.id DESC`; the in-memory arm compares the same key and then the Game id, so both backends answer the same Game. |
| `init_game::resolve_game_id` | Takes `&GameCatalogue` and `(world_key, persona_key)`; it reads `GameCatalogue::newest_game_of_world` and creates a missing Game through `GameCatalogue::create_game`, so a boot Game gets the one create unit. The second creation path (`NewGame`, `bundle_for`, `insert_game_from_request`) is gone. |
| `bootstrap/run.rs` | `resolve_boot_game` builds the boot catalogue (a throw-away `ActiveGame` + `MessageService` + `GameCatalogue`, `initial_game: None`), because the wired graph is built later. `find_latest_game_for_world`, `list_game_names_for_world` and their raw SQL are gone. `load_game_state` stays with its result discarded (the P2 note). |
| `DbPool::insert_game_from_request` | gone; the Game-row SQL has one home, `Tx::write_game_row`. |
| Tier 1 on SQLite `:memory:` | `test_support::empty_sqlite_storage()` (named `tier1_storage()` in the first pass); the `TestAppBuilder` default and the self-built storages in 15 `tests/http/` files use it (`grep new_in_memory tests/http` = 0). No tier-1 test needed a behaviour fix: `--test-target http` 275 passed, unchanged. Legacy `src/` tests that build `Storage::new_in_memory()` themselves keep the in-memory backend; a legacy `src/` test that takes the `TestAppBuilder` default now runs on SQLite, because there is one builder default (the owner's decision, below). The seam cases that pin a clause on both backends build both themselves. |
| `tests/storage/transaction_storage.rs` (new) | 8 cases, on the tag-above-the-test form: `storage-tx-rollback` (a unit whose second write fails leaves no row; rule 14's test half), `storage-tx-new-game`, `storage-tx-new-game-rollback`, `storage-tx-no-in-memory-arm`, and `storage-newest-game-message` / `-updated-at` / `-tie` / `-unknown-world` (each on both backends). `--test-target storage` 107 passed. |
| The lookup cases of `run_tests.rs` | The 3 `test_find_latest_game_for_world_*` cases moved to the storage seam as `newest_game_of_world` cases on both backends, which check the picked Game by id. The 5 `test_list_game_names_for_world_*` cases go with their function (see Deviation 5). `test_restart_with_existing_game_does_not_duplicate_scenario` reads the new seam. |

Measured times, `python build.py integration`.

| Run | Step | nextest | Tests |
|---|---|---|---|
| Before the change (`logs/build_20261017_224808.log`) | 21.80 s | 16.5 s | 1624 passed, 2 skipped |
| After the SQLite switch (`logs/build_20261017_230445.log`) | 44.54 s (16.79 s of it compile) | 27.2 s | 1623 passed, 2 skipped |
| Warm, after the switch (`logs/build_20261017_230540.log`) | 28.46 s | 27.1 s | 1623 passed, 2 skipped |

So the tier-1 switch costs about 10.6 s of test time (16.5 s to 27.1 s), which matches the inferred risk of §8: each tier-1 test now runs the schema migrations on a fresh database. The test count is 1626, the same as `main`'s: 8 legacy cases left `run_tests.rs` and 8 seam cases arrived (Deviation 5; the eighth is the tie-break case below), and one seam case was added by the fix pass.

Findings.

1. **One boot-visible delta, and it is narrow.** A boot Game now gets the starting state of the one create unit (`MessageService::build_fresh_initial_state`) instead of `GameState::inject_scenario_logs`. The two agree for every World whose default scenario has text, which is both shipped Worlds (`data/worlds/test`, `data/worlds/redmist_estate`) and every test. They differ when the default scenario's `text` is empty: the create unit sets `pending_location` to the starting room, so the next Narration carries a location header, where the old path left it `None`. This follows from ticket 11 R6 (one create unit) and is not papered over; no scenario covers a World with an empty scenario text, so this is an owner item, not a silent change.
2. No other hidden defect surfaced: the lib unit tests, the tier-1 binary and the browser binary are green, and the real server boot (the browser and bootstrap tiers, real seed data) still starts and resolves the same Game.

Deviations, and what they need.

1. **The Game-row SQL has two callers in P5.** `Tx::write_game_row` (`pub(crate)`) holds it, and the kept `Storage::create_game_from_request` calls it. Ticket 16 Q1 keeps today's Game writes off `Tx` until the core step, and duplicating the insert would let the two drift; C6 deletes the kept path.
2. **The write-injection map is a second map on the test layer.** Ticket 11 R4 says the `Tx` carries the override map from `BackendKind::Test`, and the typed key must not mix with the 87 name-keyed whole-call overrides. So `BackendKind::Test` gained a `writes` map keyed by `TxWrite`, and the `Tx` holds it as `WriteOverrides`. C2 adds `TxToken` to the two entries.
3. **`resolve_game_id` takes a `GameCatalogue`, and boot builds a throw-away one.** `run.rs` builds the wired graph after the data phase, so `resolve_boot_game` builds its own `ActiveGame` + `MessageService` + `GameCatalogue` for the resolve, and `build_app_graph` still receives the resolved `GameId`. An empty-database boot touches `updated_at` twice, not once: the throw-away `ActiveGame` inside `create_game` touches the fresh row, then the wired graph's `ActiveGame::set` touches it again. Both writes set the same column to `now`, no read observes the difference, and the first pass's "stays one write" was wrong. The alternative (resolve after the wired graph) would skip the boot heal and change boot behaviour more.
4. **`Tx` on the in-memory backend answers `EngineError::Internal`.** No ticket names the error; a unit on the fake is a programmer error, because tier 1 runs on SQLite.
5. **The 5 `list_game_names_for_world` cases are deleted, not moved.** Their subject is a function that goes, and its purpose (a unique name for a new Game) moved into `GameCatalogue::create_game`, which takes the names from `list_games()` — ticket 11 R6 says so, and `generate_game_name` and the catalogue keep their tests. The 3 lookup cases moved instead (above).
6. **The two `resolve_game_id_*` cases changed in place** (legacy file, §2 mechanical rule): they seed the World and the persona into storage and build a boot catalogue, because the resolve path takes a catalogue and the create path reads the World row back. Their assertions are unchanged.
7. **The `Tx` cases are in a new file**, `tests/storage/transaction_storage.rs`, registered in `tests/storage/mod.rs`. It is the first `tests/storage/` file that declares cases: `validate_seam_cases.py` scans that tree, and `SEAM_CONTRACT_FILES` (the allowlist guardrail) scans `src/**/*_tests.rs` only, so a declaring file under `tests/storage/` needs no table entry. The test file carries a plain `//!` summary and no `DOC` anchor (both guardrails).
8. **The `newest_game_of_world` name is ticket 11 R6's placeholder**, kept as decided.

Docs. `storage.md` gained **Transactions** (`Tx`, `TxWrite`, the two entries, one hold of the backend mutex, the whole-or-nothing rule, the write injection) and **The Newest Game of a World**. `integration_test_standards.md`: Pattern 1 is now "SQLite `:memory:` app + builder handoff" and names the new default; Pattern 3 names `empty_sqlite_storage()` for a case with no seeded row and the in-memory fake for a clause it also pins; Cross-cutting 6 names all three. `tests/STRATEGY.md`: the tier-1 row's storage column is "SQLite `:memory:` storage". `startup.md`: Game-State Initialization states the newest-Game lookup and the create unit. `AGENTS.md` and `tests/AGENTS.md` regenerated. A search of `src/`, `tests/` and `docs/diataxis/` for `insert_game_from_request`, `find_latest_game_for_world` and `list_game_names_for_world` finds nothing.

Validation. Full gate `python build.py` green at `logs/build_20261017_231423.log` (tree `1c3ab297a1+7269ef2e`): fmt, data, clippy, test structure, spec-coverage, seam-cases, Python 393, http_routes/guardrails freshness, docs, guardrails 189 passed, integration 1623 passed + 2 skipped, browser 29 passed. Total 174.1 s against the warm-baseline 137.2 s: the whole delta is the integration step (40.1 s against 21.8 s), which is the tier-1 storage switch. The first full gate of this commit took 253.3 s (`logs/build_20261017_231320.log`), mostly compiling the touched targets (clippy 78.6 s there against 12.8 s warm), and it failed on one arch-lint error: `.expect()` in `src/test_support/fixtures.rs` (`no-unwrap-expect`), fixed with the file's `// arch-lint: allow(no-unwrap-expect)` shape. `validate_seam_cases.py`: 4 contracts, 64 cases. Focused runs: clippy `logs/build_20261017_225619.log` and `logs/build_20261017_230540.log`, storage `logs/build_20261017_225939.log` (106 passed), http `logs/build_20261017_225814.log` (275 passed), guardrails `logs/build_20261017_231406.log` (189 passed).

Not done, with the reason. The target tests of §9.1 and their spec scenarios are the second commit of P5 (ticket 16 Q2), together with the `#[ignore = "red until the core step: <case>"]` rule and the `validate_feature_spec.py` check. The commit gate (`/simple-after-plan`) and the owner's approval are the commit step, not run here. The stale sentence in `explanation/storage_design.md` (a `Storage` instance carries its game id) is fixed in a later pass of this branch, by the lead.
**2026-10-17, owner (via lead recommendation).** Accepted: a Game made at boot goes through `GameCatalogue::create_game` and gets the same starting state as a Game the client creates (not yet a `Tx` unit; that comes in C4). The only visible change is a location header on the next Narration when the default scenario has no text. One creation path is the point of ticket 11 R6. The lead fixed the stale sentence in `explanation/storage_design.md` (a `Storage` holds no Game).

**2026-10-18, implementer (P5 commit 1, fix pass, uncommitted).** The review findings A to K are applied. No commit was made.

Each finding, to its result.

| # | Finding | Result |
|---|---|---|
| A | Use the Pattern 3 helper where it fits, else a neutral name | `create_test_storage(game_id)` seeds a games row (`world_key = 'test'`, `updated_at = now`), which the "a failed unit leaves no row" clause of the `Tx` cases cannot have. The helper is renamed `test_support::empty_sqlite_storage()` (from `tier1_storage`) in 18 files and named in Pattern 3 and Cross-cutting 6 of `integration_test_standards.md`. |
| B | Restore two reason comments in `tests/http/prompt_presets.rs` | Both are back: "No mode param resolves to Novel, which the preset does not allow." and the short "Captured before activation: equality (not inequality) proves IF activation leaves the Novel bundle's system slot untouched." |
| C | One function that says what it does; inline the forwarder | `WriteOverrides::check` is `fail_if_injected` now, and `Tx::check` is gone: `insert_game` and `delete_game` call `self.writes.fail_if_injected(TxWrite::..)` directly. |
| D | Drop the stale allow; use `cfg_attr` for the testing-only parameter | `#[allow(unused_variables)]` is gone from `with_backend_mut` (kept on `with_backend_writes`, where `method` is unused without the testing feature). `fail_if_injected` takes `write` and carries `#[cfg_attr(not(feature = "testing"), allow(unused_variables))]`, the `lib.rs` shape. |
| E | One helper for the two lock-and-collect blocks | `TestFailureHandle::unconsumed_overrides()` answers the two key lists; `assert_no_unconsumed` and `Drop` call it. |
| F | Call the boot wiring from the test | `resolve_boot_game` is `pub(super)` (its only caller outside `run.rs` is the sibling `bootstrap::run_tests`), `boot_catalogue` is gone from `run_tests.rs`, and both cases call `resolve_boot_game`. The legacy file keeps its test count. |
| G | Keep the error kind; log the Game's name again | `boot_error` maps each `ApplicationError` variant to its `EngineError`: `Validation` stays `Validation`, an engine failure keeps its own variant, and the two variants a boot cannot reach map to `Config`. `resolve_boot_game` logs "Starting on game '<name>' (id=..) with persona '<key>'", with `warn` for a missing row and for a read failure. |
| H | One clause per case; fixed timestamps | The `newest_game_of_world` asserts left `storage-tx-rollback` and `storage-tx-new-game`; `assert_ne!` is gone. The `insert_message_at` helper writes a fixed time (`FUTURE_MESSAGE_TIME`, `PAST_MESSAGE_TIME`). `-message` keeps its second Game, because that Game is the clause (a Message beats a newer `updated_at`): the binding went and the row stayed. `-updated-at` compares a fixed past Message with a Game that has no Message, so no two `Utc::now()` values are compared. |
| I | Tie-break on the higher Game id | SQL `ORDER BY COALESCE(m.last_message, g.updated_at) DESC, g.id DESC`; in-memory `.then(a.id.cmp(&b.id))` with `max_by`. New case `storage-newest-game-tie` on both backends: two Games with the same fixed Message time answer the higher id. |
| J | Merge the import lines | Nine `tests/http/` files take the name inside their `test_support::{..}` group; the six files with no such group keep one `use chronicler_engine::test_support::empty_sqlite_storage;` line. |
| K | Ticket corrections | Applied above: the line count (122), the lookup cases check the Game by id, the double `updated_at` touch on an empty-database boot, the seam case count, and the corrected legacy-test claim. |

Owner decisions recorded.

- **Legacy `src/` tests and the builder default.** §8 says the legacy unit tests keep the in-memory backend, and the same slice puts the one `TestAppBuilder` default on SQLite. The plan conflicts with itself; one default wins. A legacy `src/` test that takes the default now runs on SQLite; a legacy test that builds `Storage::new_in_memory()` itself stays on the fake. The measured integration time already includes those tests. Accepted by the owner.
- **An unreadable settings row stops an empty-database boot.** `load_settings` (`utils/settings.rs`) swallowed a read failure with a warning and used the defaults; the boot no longer calls it, and `GameCatalogue::create_game` reads the settings row with `?`. A boot that creates its first Game now stops where it warned. Accepted: the row is the engine's only settings source, and a corrupt row is not a default.
- **`load_settings` now has no production caller**, only its two unit tests (`utils/settings_tests.rs`). It is a `pub` function of `utils/settings.rs`, so no lint fires. Owner item: delete it with its tests, or keep it as the lenient reader for a later caller.

Validation (this pass). Full gate `python build.py` green at `logs/build_20261018_112231.log` (tree `1c3ab297a1+f708bf3f`, 175.6 s): clippy, data, structure, spec-coverage, seam-cases (4 contracts, 65 cases), Python 393, http_routes/guardrails freshness, docs, guardrails 189 passed, integration 1624 passed + 2 skipped (1626, one more than the previous pass: the tie case), browser 29 passed. Two earlier full gates of this pass are also green: `logs/build_20261018_111416.log` (176.8 s) and `logs/build_20261018_111858.log` (175.2 s). Focused runs: clippy `logs/build_20261018_111209.log`, storage `logs/build_20261018_111232.log` (107 passed), http `logs/build_20261018_111243.log` (275 passed), lib `logs/build_20261018_111303.log` (1238 passed), guardrails `logs/build_20261018_111329.log` (189 passed). The `not(feature = "testing")` build that the `cfg_attr` covers is not built by any `build.py` step (`clippy` runs `--all-features`), so that shape is reasoned, not measured.

Not done, with the reason. The `run.rs` comment that named `load_settings` is updated, so no code comment points at a call the boot no longer makes. The rest of the not-done list of the first pass stands: the target tests of §9.1 and their scenarios are the second commit of P5, and the commit gate is the commit step.

**2026-10-18, implementer (P5 commit 2, uncommitted).** The last piece of §8: the target scenarios and their tests (§9.1: I2, I3, I4, I5, I6, R-59), the `validate_feature_spec.py` check, and the `load_settings` deletion. No commit was made.

Scenarios added, each written before its test.

| Spec file | Number | Title | Test (`tests/http/races.rs`) | Today |
|---|---|---|---|---|
| `docs/specs/dashboard.md` | 39.4 | A stop at each stop point leaves the records of the step it reached | `test_a_stop_at_each_stop_point_keeps_the_records_of_that_step` | **red** |
| `docs/specs/dashboard.md` | 39.5 | A stopped Job's late answer changes nothing, and the next Job saves its own text | `test_a_stopped_job_writes_nothing_and_the_next_job_saves_its_own_text` | **red** |
| `docs/specs/reset.md` | 7.3 | A reset while a Job runs leaves the new Game with its opening only | `test_reset_while_job_runs_keeps_only_the_new_game_opening` | green |
| `docs/specs/games.md` | 20.12 | A Job goes on for its own Game while another Game is active | `test_a_job_goes_on_for_its_own_game_while_another_game_is_active` | **red** |
| `docs/specs/story_log.md` | 8.7 | An edit and a delete while a Job runs are refused | `test_edit_and_delete_while_job_runs_are_refused` | **red** |
| `docs/specs/swipe_switch.md` | 36.3 | A swipe switch while a Job runs is refused | `test_swipe_switch_while_job_runs_is_refused` | **red** |
| `docs/specs/actions.md` | 6.4 | The event Narration uses the preset that is active when it runs | `test_the_event_narration_uses_the_preset_set_after_the_turn_started` | green |

`dashboard.md` gained `POST /status/reset-generating` and `story_log.md` gained `POST /history/:id` in their endpoint lists: the scenarios name those routes and neither list held them.

Each test, with what a red one does today. Five carry `#[ignore = "red until the core step: <case>"]`, in words and with no plan id.

| Case | Test | Today | Measured (actual → expected) |
|---|---|---|---|
| 39.4 (the stop table) | `test_a_stop_at_each_stop_point_keeps_the_records_of_that_step`, `#[ignore = "red until the core step: a stop at each stop point"]` | red | 3 of the 6 rows fail. (a) Narration held at the analysis call: the Narration carries `Some("Room room2")` because the state update landed → the pre-analysis state names no room. (b) Impersonate held at the narrator call: the story log gains the Narration → the target saves no Message of the turn. (c) Event held at the event narrator call: 2 Narrations → the target keeps the Narration that fired the Trigger and no event Narration. Green today: the two rows the stop already cancels (Narration held at the narrator call, Narration held at the Options call) and every held-status assertion (`narrating`, `quantifying`, `generating-event`). |
| 39.5 (a stopped Job's late answer) | `test_a_stopped_job_writes_nothing_and_the_next_job_saves_its_own_text`, `#[ignore = "red until the core step: a stopped job's late answer"]` | red | the log holds 2 Narrations, the target 1: the stopped Job's late text joins the story. |
| 7.3 (reset while busy) | `test_reset_while_job_runs_keeps_only_the_new_game_opening` | green | — (today the reset deletes the Game the Job targets, so the late save fails on its own) |
| 20.12 (a Job on another Game) | `test_a_job_goes_on_for_its_own_game_while_another_game_is_active`, `#[ignore = "red until the core step: a job on another game"]` | red | Game A's story log holds nothing, the target one Narration: the switch discarded the Job's text (today's game-changed check). |
| 8.7 (edit and delete while busy) | `test_edit_and_delete_while_job_runs_are_refused`, `#[ignore = "red until the core step: an edit and a delete while busy"]` | red | the edit answers `<span class="status ready">Edited</span>` and the delete removes the last Message → the target answers "Still thinking..." and writes nothing. |
| 36.3 (a swipe switch while busy) | `test_swipe_switch_while_job_runs_is_refused`, `#[ignore = "red until the core step: a swipe switch while busy"]` | red | status `503` → the target `200` "Still thinking...". |
| 6.4 (R-59, the event Narration's preset) | `test_the_event_narration_uses_the_preset_set_after_the_turn_started` | green | — (the narration resolves the Game's preset per generation, so the event Narration already reads the preset the client set while the narrator call was held) |

Two things the I2 matrix measured, worth keeping.

1. **Its failing rows are not stable today.** Two runs of the same test failed different rows (run 1: Narration at the analysis call, Impersonate, Event; run 2: Impersonate, user regen, Event). Today the stop and the Job's late save race for the slot, so which save lands depends on the interleaving. The target's job id check under the guard decides this (D-08, R-74a), so the test should be deterministic once C5 removes the ignore; C5 confirms it.
2. **The I2 form: one scenario with an Examples table and one generated test.** `validate_feature_spec.py` pairs a declared scenario with at least one covering tag and does not parse the body, so both forms validate; the design names **one** test over (kind × stop point) with the §4.4 table as data (ticket 05 item 4), so the scenario carries the rows as an Examples table. One scenario per row would need one tagged test per row.

What the design left open, and how the tests work around it.

1. **The Options run's status name.** §4.4's last row calls the status "the next step" and never names the Options run's status, while §4.3 names the generation's steps (`narrating`, `generating-event`, `quantifying`). The Options stop row therefore asserts no held status; the other five rows assert §4.3's names. If the core step names it, that row gains its status.
2. **A stop cannot land inside a save unit from tier 1.** The stop points are the model calls, so a stop at the text save or at the state update is not observable; the matrix maps the narrator call to §4.4's first row, the analysis call to its third and fourth, and the Options call to its fifth. Ticket 05 recorded the same inference for the prototype.

`validate_feature_spec.py`: **no change was needed.** The scanner anchors a tag on the test attribute and ignores `#[ignore]`, so an ignored test is cover already. Measured: 289 declared, 289 covered, 0 gaps. Its Python tests gained one case that pins this (`IgnoredTestAnchorTests.test_tag_above_an_ignored_test_is_an_annotation` in `scripts/tests/test_validate_feature_spec.py`), so the red-window mechanism cannot lose coverage silently.

How a reader runs the target tests. `python build.py --include-llm` switches the test command to `--profile llm --run-ignored all` (`build.py:274-292`), and `[profile.llm]` inherits `default` with no filter (`.config/nextest.toml:31-33`), so the target tests run in the same run as the ignored `tests/llm` tests. They are not hidden — each fails under its own name (`chronicler_engine::http races::…` beside the LLM binary's `chronicler_engine::llm flow_llm_tests::…`) — but the run is red until C5, as §2 says. A reader who wants only the target tests runs:

```
cargo nextest run --no-fail-fast --test http --run-ignored all -E 'test(races::)'
```

Measured: 10 tests, 5 pass (I1's three cases, 7.3, 6.4) and 5 fail (the ignored targets). Gap, for the owner: `build.py test-pattern` passes no `--run-ignored`, and `--include-llm` is gate-only, so that focused run has to be a direct `cargo nextest` call outside the build slot. A `test-pattern --run-ignored <mode>` flag would keep it inside the slot; not applied, because the `test-pattern` CLI shape is P0's and the flag is not in this slice.

`load_settings` deletion.

- `src/utils/settings.rs` is gone (the module held only `load_settings`), with `pub mod settings;` out of `src/utils/mod.rs`.
- Of its unit tests, the two named for it go. `test_save_settings_roundtrip` is deleted as a measured duplicate of `test_save_then_get_settings_roundtrip` and its `_sqlite` twin in `src/adapters/driven/storage/settings_tests.rs`.
- The other six tests are `LlmProviderConfig` tests (`resolve_api_key`, blank-key handling, `check_api_key_available`, `resolve_base_url`, the two `resolve_max_context_tokens` cases), and their subject's file is `src/domain/model/settings_tests.rs`; they move there. Reason: the `test file location` guardrail fails an orphan `_tests.rs` whose source file is gone, and a `domain/model/` file is place 2 of the one rule, so the move adds no legacy debt — while adding them to another legacy file would.
- The `utils/settings_tests.rs` entry is dropped from `LEGACY_UNIT_TEST_FILES` (a legacy entry that names no file is a guardrail error), and the generated `AGENTS.md` and `guardrails.md` are regenerated.
- The lib binary goes 1238 → 1235 tests: 3 deleted, 6 moved, none added.

Validation (this commit). Full gate `python build.py` green at `logs/build_20261018_120218.log` (tree `389ddc3ea8+b662b63d`, 342.5 s; the guardrails step is the long one, 94 s of it, because the test files recompile): fmt, data, clippy, test structure, spec-coverage (289 declared, 289 covered), seam-cases (4 contracts, 65 cases), Python, http_routes and guardrails-freshness, docs, guardrails 189 passed, integration 1623 passed + 7 skipped (1630 tests: 7 new, 3 deleted), browser 29 passed. The first green gate of this commit is `logs/build_20261018_115525.log` (tree `389ddc3ea8+3e4ebed0`, 177.0 s). Focused runs: clippy `logs/build_20261018_115321.log`, guardrails `logs/build_20261018_115341.log` (189 passed), lib `logs/build_20261018_115406.log` (1235 passed), http `logs/build_20261018_115132.log` (277 passed, 5 skipped). The red-row measurements: `logs/build_20261018_113853.log` (the five new cases without ignores) and `logs/build_20261018_114143.log` (the stop matrix). One earlier gate failed on `guardrails.md` freshness after the legacy-list edit (`logs/build_20261018_115452.log`); the generator is run and the next gate is green.

Done when (§8), for this commit. `validate_feature_spec.py` counts each ignored target test as its scenario's cover (289/289) ✓. The target tests exist with the fate §9.1 gives them: 5 red and ignored, 2 green ✓. C5 of the core step removes the 5 ignores.

**2026-10-18, implementer (P5 commit 2, fix pass, uncommitted).** The doc-review findings E1 to E3, W1 to W6 and I1 to I2, and the comment-pass renames, are applied. No commit was made.

Each id, to its result.

| id | Result |
|---|---|
| E1 | `actions.md` 6.4 is restated through the LLM call records (`GET /fragment/llm-messages`): the event Narration's call shows the system prompt of the preset the client set, and the Narration's call shows the preset that was active when the Job started. "Turn" is gone, and the title names the call, not the Narration's text. `actions.md`'s endpoint list gained `GET /fragment/llm-messages`. The test reads the panel, splits it into call cards, and attributes a card by the answer in its Response block, so "which call" is never guessed from a text that both prompts contain. |
| E2 | 18.4's test asserts each Game's story log holds its own Narration and no Narration of the other Game: a create saves its own opening, so an exact-one assertion was wrong. |
| E3 | Mechanism below. |
| W1 | 39.4's title is "A stop at each model call leaves the records of the step it reached", and the scenario states that a stop inside a save unit is not observable (a save unit takes milliseconds, so a stop just before one leaves the records of the model call before it). |
| W2 | The Impersonate × analysis-call row is in: the Impersonate text save landed before the stop, so the story log holds the Input of the Job and no Narration. The event × analysis-call row is a recorded gap: the event generation's analysis is the Job's *second* `Analysis` call, and `ScriptedModel::hold` arms the next call of a role only. |
| W3 | The Narration × analysis-call row fires the Trigger now: the always-on World seeds `triggered_npc()` and the scripted analysis reports `npc_1`, so its "no event Narration" line proves the stop ended the Job before the Trigger's event Narration. |
| W4 | The scenario states what the design states: the status while held as an Examples column (§4.3's `narrating` / `quantifying` / `generating-event`; the Options row reads "the step that runs next", because the design names no status for the Options run), and the stop's "reset" answer (§5). 39.5 states that the stopped Job's Input stays (§5's "the Input of a free-text Action"). The tests assert those lines, no more. |
| W5 | 36.3's held Job is a continue (an empty Action), which saves no Input, so the Narration with two Swipes is still the Message the switch names; the scenario speaks of "the Message that has two Swipes" and "the other Swipe of that Message". |
| W6 | The scenario is 18.4, in `games.md`'s Switch section, and it says Game B is *created* from Game A's World. The test releases Game A's answer first, waits for Game A's Job to end, then runs Game B's Job, then switches back and asserts. |
| I1 | "of the Job" (not "of the turn"), "Retry of a typed Input" (not "Typed user regen"), "event Narration" (not "Event"), Game capitalised, 39.4's Then split into separate lines, 8.7 "the Message the client tried to edit", 7.3 "the Job the reset ended". `JobKind` and `StopJob` replace `TurnKind` and `StopTurn`, so no code or spec says "turn". |
| I2 | 8.7 and 36.3 read "the response is 200 carrying the \"Still thinking...\" status". |

Renames: `Stays` / `StopRow::stays` → `StoryLogAfterStop` / `after_stop`; `TURN_TEXT` / `OPENING_TEXT` → `SCRIPTED_NARRATION` / `WORLD_OPENING_TEXT`; `StopTurn::before` → `story_before`.

**The E3 mechanism.** A stop and a reset end a Game's *slot* at once, while the Job's thread lives on until its refusals return, so no Game-side read can say whether a released Job's late answer has run — a test that read the story log right after the stop could pass on an empty answer. The smallest seam is at today's spawn site (`ActionPipeline::claim_and_spawn`): `GenerationGate` now watches each spawned Job task (`JobTasks`, `watch_job`) and tier 1 waits for the Job to end (`state.generation_gate.wait_for_jobs()`, wrapped as `wait_for_jobs(state)` in `tests/http/support/http_requests.rs`). The watch is `#[cfg(feature = "testing")]`, like the other test-support seams. **The core step's Job runner must keep it**: register its Job's task where it spawns it, and let a test await it. 39.4, 39.5 and 7.3 wait there before they read.

Sensitivity, measured.

- 39.4 and 39.5: today the late save *does* land, and the tests fail on it. With the wait in place, three more stop rows fail than before the fix: the rows that the pre-fix suite reported green — Narration at the narrator call, Impersonate at the narrator call, Retry of a typed Input at the narrator call — were green only because the read ran before the late save. The fix turned them red, which is the honest target signal.
- 7.3: the late save cannot land in the new Game (the reset deleted the Game the Job targets), so its assertion was shown live by temporarily flipping it to expect the stopped Job's text; the flipped test failed (`logs/build_20261018_123035.log`, "FLIP: the new Game must hold the late answer of the Job the reset ended"), and the flip is reverted.

Per test, re-measured on today's code. Five tests carry `#[ignore = "red until the core step: <case>"]`; two are green and carry none.

| Case | Test | Today | Red reason (actual → expected) |
|---|---|---|---|
| 39.4 | `test_a_stop_at_each_model_call_keeps_the_records_of_that_step`, `#[ignore = "red until the core step: a stop at each model call"]` | red | 5 of the 7 rows fail. Narration at the narrator call: the log gains the Narration → the Input only. Narration at the analysis call: the log gains the Narration *and* an event Narration → the Narration only, with no room and no event. Impersonate at the narrator call and Retry of a typed Input at the narrator call: the log gains the Job's Message → no Message of the Job. event Narration at the event narrator call: 2 Narrations → the Trigger's Narration only. Green: Narration at the Options call, and Impersonate at the analysis call. |
| 39.5 | `test_a_stopped_job_writes_nothing_and_the_next_job_saves_its_own_text`, `#[ignore = "red until the core step: a stopped job's late answer"]` | red | the log holds 2 Narrations → 1: the stopped Job's late answer joins the story. |
| 7.3 | `test_reset_while_job_runs_keeps_only_the_new_game_opening` | green | — (the flip demonstration above) |
| 18.4 | `test_a_job_goes_on_for_its_own_game_while_another_game_is_active`, `#[ignore = "red until the core step: a job on another game"]` | red | Game A's story log holds nothing → its own Narration: the switch discarded the Job's text. |
| 8.7 | `test_edit_and_delete_while_job_runs_are_refused`, `#[ignore = "red until the core step: an edit and a delete while busy"]` | red | the edit answers `<span class="status ready">Edited</span>` (and the delete removes the last Message) → 200 carrying "Still thinking..." and no write. |
| 36.3 | `test_swipe_switch_while_job_runs_is_refused`, `#[ignore = "red until the core step: a swipe switch while busy"]` | red | `503` → `200` carrying "Still thinking...". |
| 6.4 | `test_the_event_narration_call_uses_the_preset_active_when_it_runs` | green | — (a generation resolves the Game's preset when it runs, so the event Narration's call already shows the preset the client set) |

Gaps recorded. (1) The event × analysis-call row (W2) needs a "hold the Nth call of a role" ability the fake does not have. (2) The Options run's status name: the design's last stop row says "the next step" and names none, so that row's status cell reads "the step that runs next" and the test asserts no status there. (3) A stop inside a save unit is not reachable from tier 1 (W1), which the scenario states.

One fix outside the findings: the comment pass had shortened `check_unit_test_allowlist`'s doc comment to its second paragraph, so `generate_guardrails_doc.py` rendered a truncated row ("The one rule is `tests/STRATEGY.md` "The one rule": a unit test lives in a"). Its one-line summary is restored, and the generated `guardrails.md` is regenerated.

Validation (this pass). Full gate `python build.py` green at `logs/build_20261018_124123.log` (tree `389ddc3ea8+ba4cc2f3`, 266.0 s as the history records it): fmt, data, clippy, test structure, spec-coverage (289 declared, 289 covered), seam-cases, Python, freshness, docs, guardrails 189 passed, integration 1623 passed + 7 skipped (1630), browser 29 passed. `--test-target http` three times: `logs/build_20261018_123619.log`, `logs/build_20261018_123634.log`, `logs/build_20261018_123647.log` — 277 passed, 0 failed, 5 skipped each, and no flaky non-ignored test (one earlier single run: `logs/build_20261018_122646.log`). The ignored target tests once, in one run: `cargo nextest run --no-fail-fast --test http --run-ignored all -E 'test(races::)'` → 10 tests run, 5 passed, 5 failed (output in `/tmp/fix2_ignored_final.out`; a single ignored test cannot run through `build.py test-pattern`, which passes no `--run-ignored`). Guardrails `logs/build_20261018_123705.log` (189 passed). Clippy `logs/build_20261018_123910.log`. The 7.3 flip: `logs/build_20261018_123417.log` (FLIP assertion failed, then reverted). One earlier gate failed on `guardrails.md` freshness after the walker table's line number moved (`logs/build_20261018_124042.log`); the doc is regenerated and the next gate is green.

**2026-10-18, implementer (P5 commit 2, fix pass 2, uncommitted).** The two code reviews' items (S1 to S8, P1 to P5) and the ticket notes are applied. No commit was made.

Each id, to its result.

| id | Result |
|---|---|
| S1 | `races.rs` reads the LLM call records through the typed seam `GameViewQuery::list_latest_llm_messages`, as `tests/http/llm_messages.rs` does: the event Narration's call is found by its answer and its stored system prompt is checked for the preset text, and the Narration's call likewise. `llm_call_cards`, `card_answered` and their HTML scraping are gone. The scenario keeps its client-facing wording (the LLM call records). |
| S2 | `LlmProviderConfig::resolve_api_key_with(lookup)` and `check_api_key_available_with(lookup)` take the environment read as a function; `resolve_api_key` and `check_api_key_available` are one-line wrappers that pass `std::env::var(name).ok()`. The three moved tests that read env vars are pure now (a closure supplies the value): no `set_var`, no `ApiKeyEnvGuard` in `src/domain/model/settings_tests.rs`, and the six moved tests lost their `test_` prefix. |
| S3 | `JobTasks::wait(timeout)` wraps the wait in `tokio::time::timeout` and answers `bool`; `GenerationGate::wait_for_jobs(timeout_ms)` and the test helper `wait_for_jobs(state, timeout_ms)` follow `wait_idle`'s shape, and every caller asserts with a message. |
| S4 | One constant per text: `SHARED_NARRATION` is gone and 39.5 scripts `STOPPED_NARRATION` and `NEXT_NARRATION`; `input_texts` is gone and the I1 tests read through `texts_of`; the event-Narration fixture is one pair of helpers, `trigger_world()` (the default World plus `triggered_npc()`) and `triggered_script(narration, event_narration)` (its four calls), shared by 6.4, the `EventNarration` branch and the Narration row that fires a Trigger. |
| S5 | `StopRow` has no `fires_trigger`, and its `after_stop` variants carry none of the Input texts: `JobKind::Narration { fires_trigger }` owns the flag, and `start_stop_job` owns the Action text through `JobKind::action_input()` (`ACTION_INPUT`, `EVENT_ACTION_INPUT`), which the assertions reuse. |
| S7 | The sample text names the renamed scenario (`a stop at each model call`), and the new class has no docstring: the reason is in the test name, `test_tag_above_an_ignored_test_still_covers_its_scenario`. |
| S8 | `SEAM_CONTRACT_FILES` carries its reason comment again: a `_tests.rs` file that carries `CASE` tags fails unless it is on the list. |
| P1 | 18.4 holds Game A's Job at its narrator call and Game B's Job at its narrator call at the same moment (the Action on B arrives while A is active and busy), then releases A, then B, waits for both Jobs with the timeout, switches back to Game A and asserts both story logs. |
| P2 | 39.4 checks that the Game is idle and that the status poll answers `idle` with no failure line **before** it releases the held call, and keeps the after-release checks on the story log. |
| P3 | 39.5 scripts two Narration texts, so it checks that the Narration after `listen` is the second Job's text (`NEXT_NARRATION`). |
| P4 | The Options row reads the dock and asserts the offered set stays empty after the late answer (the text save emptied it, and the refused Options run leaves it empty); the Impersonate × analysis-call row scripts an analysis that moves the player, and asserts the saved Input carries no room and the current room is still the starting room. |
| P5 | 7.3 asserts that the status poll answers `idle` with no failure line after the late answer. |

**One mechanism the two review sides together forced.** P1 and P3 need two Jobs held at once, and two Jobs that both pop from one scripted role queue race: whichever thread wakes first takes the first answer. Today's release order does not decide it (`ModelHold::release` only sets a flag). So the fake gained one signal: `ModelHold::wait_taken(timeout_ms)`, which waits until the released call has taken its scripted answer, set by the fake right after it pops the reply. The call released first takes the first answer, deterministically, with no sleep. Both new tests use it: 39.5 between the two releases, and 18.4 before releasing Game B.

**Lead decisions recorded.**

- **(a) `load_settings` is deleted.** It had no production caller once the boot read moved to `GameCatalogue::create_game`; its module, its two tests and its legacy-list entry went, and the six tests of `LlmProviderConfig` that lived beside it moved to `src/domain/model/settings_tests.rs` (where they are pure tests of a domain function).
- **(b) The Job-task tracker moves in the core step.** It is a test-only watch on the spawned Job threads, and it sits in `GenerationGate` today because that is the type the spawn site already holds. The core step gives `gate.rs` its one `hold` and rule 18 counts a second lock there as a re-look trigger, so the tracker must move to the Job runner (`pipeline/job.rs`) or `test_support` when the runner arrives. The tracker adds a `Mutex<Vec<JoinHandle<()>>>` beside the registry and takes no lock the registry's paths take, so it does not nest today.
- **(c) The tracker is in the default build.** It is `#[cfg(feature = "testing")]`, and `testing` is a default feature, so a release build carries it too. That gap is older than this slice (the `BackendKind::Test` layer has it as well) and is not fixed here.

Per test, re-measured on today's code. Five carry `#[ignore = "red until the core step: <case>"]`; two are green and carry none.

| Case | Test | Today | Red reason (actual → expected) |
|---|---|---|---|
| 39.4 | `test_a_stop_at_each_model_call_keeps_the_records_of_that_step` | red | **all 7 rows** fail now. Narration at the narrator call: the log gains the Narration (expected the Input only). Narration that fires a Trigger at the analysis call: the log gains the Narration *and* an event Narration. Narration at the Options call: the dock still shows the offered set (`options-strip`) and the Narration reports `Room room2`. Impersonate at the narrator call and Retry of a typed Input at the narrator call: the log gains the Job's Message. Impersonate at the analysis call: the saved Input reports `Room room2` where the pre-analysis state names no room. event Narration at the event narrator call: 2 Narrations → the Trigger's own. |
| 39.5 | `test_a_stopped_job_writes_nothing_and_the_next_job_saves_its_own_text` | red | the log holds `["The next Job's narration.", "The stopped Job's narration."]` → only the next Job's text. |
| 7.3 | `test_reset_while_job_runs_keeps_only_the_new_game_opening` | green | — |
| 18.4 | `test_a_job_goes_on_for_its_own_game_while_another_game_is_active` | red | Game A's story log holds nothing → its own Narration: the switch discarded the Job's text. |
| 8.7 | `test_edit_and_delete_while_job_runs_are_refused` | red | the edit answers `<span class="status ready">Edited</span>` → 200 carrying "Still thinking...". |
| 36.3 | `test_swipe_switch_while_job_runs_is_refused` | red | `503` → `200` carrying "Still thinking...". |
| 6.4 | `test_the_event_narration_call_uses_the_preset_active_when_it_runs` | green | — |

The table's score changed with P4: the two rows that used to pass (Narration at the Options call, Impersonate at the analysis call) were green because the state update landed before the stop, and the new dock and room-report checks see that write. So all seven rows are red today, and the whole matrix is one honest target.

**Where the gate's extra time goes.** The full gate took 266.0 s (`logs/build_20261018_124123.log`) and 271.8 s (`logs/build_20261018_131303.log`) against 175.6 s for the same gate before this commit (`logs/build_20261018_112231.log`). The whole delta is compiling test binaries inside the steps, not running tests:

| Step | `112231` | `131303` | Of which compile |
|---|---|---|---|
| clippy | 12.2 s | 0.5 s | — (warm from a run just before) |
| guardrails | 15.0 s | 94.4 s | 4.4 s → 84 s (`Finished test profile in 1m 24s`), run 9.6 s |
| integration | 36.2 s | 59.7 s | 8.6 s → 29.0 s, run 30.1 s |
| browser | 102.4 s | 106.4 s | 0.6 s, run 105.3 s |

`build.py` builds each test binary inside its own step, and this commit's `src/` edits (the storage `Tx`, `ActiveGame`, the Agents, the gate, the settings helpers, the fake) invalidate the library and every harness: the guardrails step runs first in the gate order and pays the library and architecture binary, the integration step then pays its own harnesses. The *run* times are essentially unchanged (guardrails 10.0 s → 9.6 s, integration 27.1 s → 30.1 s with 5 more tests, browser 101.3 s → 105.3 s). A gate re-run with no source change returns to the ~175 s shape, which the clippy step's 0.5 s in the second run already shows.

Validation. Full gate `python build.py` green at `logs/build_20261018_131303.log` (271.8 s in the step summary, 275.7 s in the history): fmt, data, clippy, test structure, spec-coverage (289 declared, 289 covered), seam-cases (4 contracts, 65 cases), Python (399 tests, the feature-spec file's 11 among them), freshness, docs, guardrails 189 passed, integration 1623 passed + 7 skipped (1630), browser 29 passed. Focused runs: clippy `logs/build_20261018_130540.log` (89.1 s, green), guardrails `logs/build_20261018_130709.log` (91.5 s, 189 passed), `--test-target http` three times: `logs/build_20261018_130844.log`, `logs/build_20261018_130859.log`, `logs/build_20261018_130912.log` — 277 passed, 0 failed, 5 skipped each, no flaky non-ignored test (and one earlier single run, `logs/build_20261018_130219.log`). The ignored target tests once, in one run: `cargo nextest run --no-fail-fast --test http --run-ignored all -E 'test(races::)'` → 10 run, 5 passed, 5 failed (`/tmp/fp2v_ignored.out`). One earlier gate failed on `guardrails.md` freshness after the restored comment moved the walker table's line (`logs/build_20261018_131118.log`); the doc is regenerated and the next gate is green.

**2026-10-18, lead (retro).** The first version of the target tests had vacuous checks: the late-answer assertions ran before the late answer could reach its save, and two stop rows could not show a late write. The doc review and the spec review caught them. Two rows that were green for the wrong reason are red now. Lesson for the core step (recorded in ticket 07): when C5 removes an `#[ignore]`, each target test must turn green for the right reason, so C5 checks each one against a deliberately wrong expectation, as P5 did for 7.3. Infrastructure: one agent run stopped on a model error; the resumed session finished the work with no loss. Owner items from P5: (1) `build.py test-pattern` cannot run ignored tests, so the target tests run through a direct `cargo nextest` call outside the build slot; (2) the target design names no status for the Options run; (3) the fake can hold only the next call of a role, so the stop matrix has no row for an event stopped at its analysis call.

## Answer

P5 is built and merged to `main` in two commits. Commit 1: `Tx` and `TxWrite` on SQLite, `Storage::transaction`/`transaction_new_game`, `newest_game_of_world` with a tie-break, the boot lookup through `GameCatalogue`, tier 1 on SQLite `:memory:`, the storage seam contract (8 cases). Commit 2: the seven target scenarios of §9.1 and their tests in `races.rs` (5 red and ignored until the core step, 2 green), a test-only Job-task tracker the core step moves to the Job runner, and the deletion of `load_settings`. Full gate green: `logs/build_20261018_131303.log`.
