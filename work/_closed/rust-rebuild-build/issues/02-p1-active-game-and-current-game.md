# P1: `ActiveGame` and `CurrentGame`

Type: task
Status: resolved
Blocked by: 01

## Question

Build §4 of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) on branch `wf/rebuild-p1`, from the current `main`. Read §1, §2 and §4 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §4 pass. The owner has approved the last commit, and the branch has merged to `main`.

## Comments

**2026-10-17, implementer.** The slice is implemented on `wf/rebuild-p1` (from `main` at `241657b0`), and the full gate (`python build.py`) is green. Awaiting the commit gate and the owner's approval.

**2026-10-17, implementer (fix pass).** The accepted review findings are applied; see *The fix pass* below. The full gate is green again (log `logs/build_20261017_183824.log`).

### Content, item by item

- `application/games/active.rs` (new): `ActiveGame { storage, Mutex<Option<u64>> }` with `get` and `set`. `set` touches `games.updated_at` through the new `Storage::touch_game_updated_at(game_id)`, which keeps the old skip on the in-memory backend; it is `pub(super)`, so only `application/games/` can move the pointer (the compiler half of rule 2).
- `adapters/driving/http/utils/current_game.rs` (new): `CurrentGame(pub u64)`, a `FromRequestParts<AppState>` extractor; no active Game gives `400` (R-45). It needs the manual future signature because axum declares the trait through `async_trait`, and `async fn` in the impl does not match that desugared signature.
- `GameId` (`domain/model/game.rs`, lead decision in the fix pass): a `Copy` newtype over `u64` (`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Display`, serde-transparent). The pointer, `CurrentGame`, `GameCatalogue::active_game_id` and every game-scoped storage signature carry it; the storage adapter unwraps it where it meets a row, an SQL parameter or the in-memory maps, and the bootstrap/test helpers wrap raw row ids, so no caller converts.
- `Storage`: the `game_id` field, `game_id()`, `set_game_id` and `current_game_id` are gone; `new_sqlite(pool)` takes no Game. Every game-scoped read and write takes `game_id: GameId` as its first argument: `insert_message`, `delete_message`, `load_message_rows`, `get_active_swipe_index`, `require_active_swipe_index`, `update_active_swipe`, `soft_delete_message`, `restore_soft_deleted`, `purge_soft_deleted`, `load_messages_with_swipes`, `save_snapshot`, `load_latest_snapshot`, `load_snapshot_by_id`, `active_{system,quantifier,impersonate,options}_preset_id`.
- `MessageService`, `GameViewQuery` and `GameCatalogue` take the `GameId`; the pipeline entry points (`process_action`, `process_options`, `retry`, `retrigger`, `run_from_input`, `execute_action_with_inputs`, `execute_options_refresh`, `retry_last_response`, `retry_event_continuation`, `retry_user_regen`, `retry_main_narration`, `phase_trigger_continuation`, `heal_stale_status`, `reset_persisted_status`) take it too. `ActionPipeline` holds an `Arc<GameCatalogue>`, so `PipelineRun::check_game_unchanged` compares the request's Game with the active Game **read through `GameCatalogue`** (ticket 17 Q1); behaviour is unchanged.
- `GameCatalogue` (switch, create, reset, delete) owns the pointer: `active_game_id()`, `active_game()` and the constructor's `initial_game`. Bootstrap and the test builders point the app **through `GameCatalogue`**, driven by a new `initial_game: Option<GameId>` parameter on `build_app_graph` / `build_app_graph_for_tests`; the graph sets it before the boot heal, exactly where the old pointer was set. A later move goes through `switch_game`, which checks that the Game row exists.
- Handlers: every handler that read the Game id takes `CurrentGame`. The three exceptions take `Option<CurrentGame>`: the Games list (`list_games_fragment`, `delete_game_handler`), the status poll and the stop, which answers `failed` with no Game (target design §5).
- Tier 1 covers `ActiveGame` and `CurrentGame` through the Game switch, create and reset scenarios (ticket 26 R9); no new scenario was needed.

### Tests

- Removed, because their subject went: `core_tests::{test_new_in_memory_default_game_id, test_new_sqlite_game_id, test_set_game_id_updates_backend, test_current_game_id_returns_stored_value}` (the `game_id` atomic), `catalogue_tests::test_current_game_id_matches_storage` and `tests/storage/snapshot_storage.rs::test_current_game_id` (the `Storage` pointer). -6 legacy unit tests; the `tests/storage/snapshot_storage.rs::test_game_id_isolates_snapshots` case keeps its subject, now on explicit ids.
- Every other broken test changed in place (§2, ticket 17 Q8): a new Game argument or the app's own `active_game_id()`. `TestData::seed_into` now returns the seeded Game id instead of the unused world id, and `TestAppBuilder` gained `.active_game(game_id)` for the `skip_seeding(true)` tests, which the builder cannot infer.
- One shared accessor, `test_support::fixtures::active_game_of(&state) -> GameId` (panics with a clear message when no Game is active), replaced the six private copies and every inline pointer read in tests, including `.active_game_id().unwrap_or_default()`. `wait_idle` reads the Game once per wait.
- Scenarios 11.2 and 14.6 ("no game exists in storage") now drive the app with no Game and no active pointer (`skip_seeding(true)` with no `active_game`), so their `400` comes from the `CurrentGame` extractor.
- Run results: `python build.py integration` — 1628 passed, 0 failed, 2 skipped (1630 tests, `-6` on the legacy unit tests); `python build.py guardrails` — 187 passed; `python build.py browser` — 29 passed; `python build.py` — all 19 steps green in 141 s (log `logs/build_20261017_183824.log`).
- Generated index blocks regenerated by `scripts/precommit_regenerate.py` (which stages them, as the pre-commit hook does): `AGENTS.md` gains `active.rs` and `current_game.rs` in its Structure section, and `guardrails.md` carries the two landed rows.
- **Done when**: a search of `src/` and `tests/` for `set_game_id`, `current_game_id` and `game_id()` on `Storage` finds nothing (the only matches are prose in `docs/`).

### Guardrails

- Rule 1 (`BANNED_NAME_ROWS`): no `static` item and no `Atomic*` type under `adapters/driven/storage/`; reason on the row.
- Rule 2 (`CALL_SITE_ROWS`): `CallSiteRow.callee` became `callees: &[&str]`, so one rule can name several methods. The row guards `ActiveGame::get` and `ActiveGame::set`, allowed only in `application/games/catalogue.rs`; `ActiveGame::set` is also `pub(super)`, so the compiler holds the writer half. Building the value in `bootstrap/wiring.rs` is not a call the row matches.
- The rule-2 detection fixture mirrors the row (both callees, one allowed file, one disallowed file).
- `walker_tests.rs` gains a rule-1 case and a rule-2 case that mirror the landed rows: `registry_tests::every_walker_row_has_a_detection_case` requires a detection case per landed rule number.
- `docs/diataxis/reference/coding_standards/guardrails.md` regenerated with `scripts/generate_guardrails_doc.py`.

### Deviations, with reasons

1. **Rule 2's row names one file, and `CurrentGame` reads through `GameCatalogue`.** The row form puts each `allowed` value on one line (the parser contract of `generate_guardrails_doc.py`); the plan's two exact paths make a 100-character line, which rustfmt wraps and the parser then rejects. Options weighed: a folder-level allowlist entry (`adapters/driving/http/utils/`, broader than the rule), or two rows for rule 2 (breaks P0's `every_walker_rule_number_is_used_once` and the plan's seven-row count). `CurrentGame` therefore reads `GameCatalogue::active_game_id()`, the row allows exactly `application/games/catalogue.rs`, and `AppState` does not carry the pointer. Same behaviour, stricter guardrail, one row per rule.
2. **`ActiveGame::set` is `pub(super)`.** Only `application/games/` can move the pointer, so the write side of rule 2 is a compile-time fact rather than a walker row.
3. **`TestData::seed_into` returns the Game id, not the world id.** Nothing read the world id; the pointer cannot be set during seeding any more, because the app that owns it does not exist yet.
4. **`AgentContext` carries `game_id`.** The Agents still resolve their preset from storage; P4 moves that read to the caller and replaces the context with `HookInput`, which drops the field.
5. **`#[allow(clippy::too_many_arguments)]`** on `ActionPipeline::run_post_generation_agents` and `test_support::seed_swipe_with_stored_inputs`, both 8 parameters after the id (house precedent in `arrival_service.rs`).
7. **The writes of a switched-away Job go to its own Game.** Before P1 every write read the pointer at write time, so a Job that noticed a switch and cancelled (`PipelineRun::handle_cancellation`, and the other writes that follow a `check_game_unchanged` failure) wrote its `Idle` status into the **newly active** Game. Now those writes name the Job's Game (`started_for`), which is the design's intent (§3.3: a write names its captured Game) and the behaviour the core step's I5 scenario asserts. Inferred not player-visible in P1: the Game switch heals the stale status of the Game switched to, and the abandoned Game's status is the one the Job wrote either way, so no scenario in `docs/specs/` can see the difference. The three cancellation tests in `retry_tests`/`retrigger_tests` assert the cancellation itself, unchanged.
6. **Order inside the create and reset units.** `create_game` moves the pointer only after the fresh state is written, so a failed create leaves the previous Game active (the old code set it first and restored the previous pointer on failure — same end state). `reset` keeps the old order: pointer first, then the fresh state, because the old code never restored.

### The fix pass (2026-10-17)

Accepted review findings, all applied:

1. **`GameId` newtype** (lead decision; the plan and sketch 25 name `GameId`). It is `Copy` over `u64` and flows through `ActiveGame`, `CurrentGame`, `GameCatalogue::active_game_id`, every storage signature P1 gave a Game and their callers. Conversions live at the boundaries only: SQL parameters, the row-id reads (`get_game`, `require_game`, `world_bundle_for`) and the in-memory maps, which now key by `GameId`. Message ids, snapshot ids and other ids stay as they are.
2. **Rule 2 guards the writer too.** `CallSiteRow.callee: &str` became `callees: &[&str]`; the row names `ActiveGame::get` and `ActiveGame::set`, allowed in `application/games/catalogue.rs` only. The row form still keeps each value on one line, the fixtures and the detection case mirror the row, and `scripts/generate_guardrails_doc.py` needed no change (it reads only `rule:`).
3. **One pointer read per request in the Games list.** `posture_html` now comes from the active Game that `current_and_saved_games` already read, not a second `GameCatalogue` read.
4. **11.2 and 14.6 drive the extractor** (see *Tests* above).
5. **One test accessor** (see *Tests* above).
6. **`tests/http/games_config.rs`** takes its Game from the built app (`active_game_of(&state).0`) instead of a hard-coded `1`.
7. **Naming.** `GameCatalogue::current_game` is `active_game`, and `GameViewQuery::get_current_game_display_name` is `game_display_name(game_id)`. The error strings that said "current game" now name what they receive: "No active game" when the pointer is empty, "Game not found" when the row is missing, `no game state snapshot found for game {game_id}`, and the storage fallback warnings name the Game. No spec scenario asserts any of those strings.
8. **`GameCatalogue::set_active_game` is gone.** The constructor takes `initial_game: Option<GameId>` (the composition root and the test builders), and every later move goes through `switch_game`, which checks the row first; the three mid-run switch tests in `retry_tests`/`retrigger_tests` create a real second Game and switch to it, which is closer to what a player does. The wiring comment that claimed no other file could move the pointer is corrected.
9. **The status poll no longer builds an error to reach an `Err` arm**: `None` for the current Game answers the idle default directly.
10. **The `MISSING_GAME_ID`/`NO_GAME` constants** no longer describe the deleted in-memory default pointer; they are documented as a Game id with no row.
11. **Three WHY comments restored**: the `Test.base: Box<Backend>` replace-not-nest invariant, the debug registry-vs-persisted-status note, and "override backends must use this graph's persistence".
12. **Dead bindings**: the `_state` bindings that only kept a router are `.build()` again, the unused `_game_id` bindings in the two quarantined fixtures are gone, and the `TestAppBuilder` fixture call sites that never read the Game are unchanged.

No ticket, plan or requirement ids appear in code comments (swept `src/` and `tests/`).

**Reported, not changed.** No scenario states the no-Game answer of the three `Option<CurrentGame>` handlers: the Games list (`games.md` 20.8 to 20.10 all assume an active game), the status poll (`dashboard.md` 39.1 and 39.3 assume a game), and the stop (`/status/reset-generating` has no scenario at all, not even for its normal answer). Per the finding, I did not invent scenarios; the behaviour is only exercised by the extractor's `400` in 11.2 and 14.6 and by the quarantine test for the stop.

**Retro, 2026-10-17 (lead).** Plan §4 says "takes a `GameId` parameter" as a known fact, but the code at `d067420b` had no `GameId` type. The first pass used `u64`; the standards review caught it. Later slices: where the plan names a type, check that it exists before you build against it. Known gap: no scenario states the no-Game answer of the Games list, the status poll and the stop; the core step's I2 adds the stop scenarios to `dashboard.md`.

## Answer

P1 is built and merged to `main`. `ActiveGame` (`application/games/active.rs`) and `CurrentGame` (`http/utils/current_game.rs`) hold the active Game; `Storage` holds none, and every game-scoped read and write takes a `GameId` newtype. Rule 1 (no `static`/`Atomic*` in `storage/`) and rule 2 (only `catalogue.rs` calls `ActiveGame::get`/`set`) are walker rows. Full gate green: `logs/build_20261017_183824.log`.
