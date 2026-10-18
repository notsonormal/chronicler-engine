# P5: the storage `Tx`

Type: task
Status: claimed
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
