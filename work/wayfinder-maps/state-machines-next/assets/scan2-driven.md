## Scan 2: driven adapters and bootstrap

Scope read: every non-test .rs file in src/adapters/driven/ and src/bootstrap/. Read gate.rs, view_query.rs, active.rs and catalogue.rs only to confirm who calls a storage write. I did not read the earlier lists. Status words: known = I read the code path. Inferred = I reasoned from the code and did not trace it end to end.

### Dock record (`game_docks` row)
- Where: src/adapters/driven/storage/transaction.rs:300-345 (`put_dock`, `put_options_failure`, `clear_dock`, `write_dock`); src/adapters/driven/storage/docks.rs:19-43; type `DockRecord` / `OptionsFailure` in src/domain/model/dock.rs; table in plumbing.rs v27.
- State variables: row present or absent; `offered` (JSON list, `'[]'` by default); `notice` (nullable JSON `OptionsFailure`).
- States: NoRow (read gives `unwrap_or_default`, same as Empty); Empty (row, `[]`, no notice); Offered (non-empty list, no notice); FailureOnly (empty list, notice); Offered+Failure (non-empty list AND notice; implied, reachable).
- Transitions:
  - Create of a Game -> any -> Empty: `create_in_tx` gate.rs:595-610 (`put_dock(default)` at :602).
  - Options run offers -> any -> Offered: `save_options` gate.rs:836-841 (`put_dock{offered, failure:None}`).
  - Options run fails -> any -> same offered list + notice: `save_options` gate.rs:845 -> `put_options_failure` transaction.rs:307 (upsert keeps `offered`, new row gets `'[]'`).
  - Narration text saved -> any -> Empty: `write_new_message` / `write_new_swipe` gate.rs:872, :893 (only when `order.kind().saves_a_narration()`).
  - Stop -> any -> offered kept, notice cleared: `stop` gate.rs:489-498 (reads the dock first, then `clear_failure_and_put_dock`).
  - Switch Swipe / delete last Message -> any -> Empty: gate.rs:527, :564.
  - `clear_dock` (transaction.rs:321) has no caller in src outside tests (known, rg).
- Guards / invariants: `DockRecord` doc (dock.rs) says the failure "stands in for the offered set". The writes do not enforce that. A failed Options run keeps the old offered list (comment at gate.rs:834). The dashboard template (templates.rs:64) shows the notice and hides the strip when `failure` is set. `GameViewQuery::story_state` (view_query.rs:155) reads `dock.offered` and ignores `failure`.
- Explicitness: FULLY IMPLICIT (row presence + `Vec` + `Option`; no enum). Writes are in four gate.rs functions plus two `Tx` methods.
- Already modeled: no.
- Risks seen:
  - Offered+Failure is reachable and the code treats it two ways: the dock view lets the failure win, the debug state view shows the list. Known.
  - A new Job does not clear the notice at accept (`accept_in_tx` gate.rs:700 clears only `games.failure`). The notice stays until an Options run succeeds, a Narration saves, Stop, Switch or delete. Known. Probably by design; unsure.
  - `put_options_failure` on a Game with no row makes a row with `offered='[]'` (the default). A later read cannot tell it from Empty plus notice. Known, harmless.

### Failure line (`games.failure` column) and derived Status
- Where: column added in plumbing.rs v27 (`ALTER TABLE games ADD COLUMN failure TEXT`); `Tx::set_game_failure` transaction.rs:234-248; `DbGame::failure_json` models/game.rs; derived in src/application/games/view_query.rs:198-213 (`status`).
- State variables: `games.failure` (nullable JSON `GenerationFailure`); busy registry (memory); the Status is a function of both.
- States: Idle (no holder, failure NULL); Busy (holder; the column may still hold an old failure); Failed (no holder, failure set); implied: "Busy with a stale failure" (see below).
- Transitions:
  - Accept of a Job -> clears the line: `accept_in_tx` gate.rs:700-704.
  - A refused accept -> sets the line: `not_accepted_with_failure` gate.rs:648-657 (the write error is only logged).
  - A Job fails -> sets the line: `JobWriter::save_failure` gate.rs:828-832.
  - Stop / Switch Swipe / delete last -> clears: `clear_failure_and_put_dock` gate.rs:695-698, called at :497, :527, :564.
  - Edit (gate.rs:533) does not clear it (known).
  - Restart -> the column survives; the registry is empty. A Game that failed before the restart boots as Failed (known).
- Guards / invariants: the Status is derived, never stored. `status` checks the registry first, so Busy hides the column.
- Explicitness: EXPLICIT STATES (`GenerationStatus` enum in domain/model/state/generation_status.rs) built from two sources. The writes are scattered over gate.rs. The enum in generation_status.rs is not the type that `status` returns (`Status` in view_query).
- Already modeled: partly (the Job's failure line is). The column's persistence over a restart and the clear/not-clear rule per operation are not.
- Risks seen:
  - A refused accept whose failure write fails leaves the column NULL. The caller sees `NotAccepted::Failed`, a later read says Idle. Known (gate.rs:654-657).
  - `edit` leaves an old failure in place; `switch_swipe` and `delete_last` clear it. Inferred inconsistency; may be intended.
  - `set_game_failure` does not check the affected row count. For a deleted Game it succeeds and raises BUSY_STATE (see "Change counters"). Known.

### Message active Swipe (`messages.active_swipe_index` + `message_swipes` rows)
- Where: `messages.active_swipe_index` (plumbing.rs v28, :662-715); `message_swipes` UNIQUE(message_id, swipe_index); `Tx::insert_message` :132, `insert_swipe` :180, `set_active_swipe` :160, `delete_message` :170; readers in stories.rs and mappers/message.rs.
- State variables: integer index on the Message; the set of Swipe rows of that Message; Swipe `state` column.
- States: Active index in range (the valid state); index out of range or Message with zero Swipes (implied invalid).
- Transitions:
  - New Message: `insert_message` (index 0 written first) then `insert_swipe(0)` inside one unit: gate.rs:595-610, `write_new_message` :866-875.
  - New Swipe on last: `insert_swipe(len)` then `set_active_swipe(len)`: `write_new_swipe` gate.rs:878-895.
  - Switch: `set_active_swipe`: gate.rs:501-531.
  - Delete the Message: cascade removes the Swipes.
- Guards / invariants: "every Message has at least one Swipe and exactly one is active" (CONTEXT.md). No CHECK or FK enforces it. `set_active_swipe` checks only that the Message belongs to the Game, not that the Swipe row exists.
- Explicitness: FULLY IMPLICIT (integer pointer into a row set). The legal moves live in gate.rs.
- Already modeled: no. (The two writes of a new Swipe state are, and that is a separate item.)
- Risks seen (all known, read in code):
  - An index with no Swipe row is read four ways. `load_message` repairs it to 0 silently (mappers/message.rs:72 `ensure_valid_swipe_index`). `window_entries_of` fails with "message N has no swipe" (stories.rs:234). `entries_of` shows empty text through `COALESCE(s.text,'')` (stories.rs:20). `has_text` counts it as no text.
  - `set_active_swipe` can point at a Swipe that does not exist; nothing in the unit stops it.

### Schema version ladder (`PRAGMA user_version`)
- Where: src/adapters/driven/storage/utils/plumbing.rs:17 (`LATEST_VERSION = 34`), :27-799 (`migrate_up_to`), called from db.rs:17.
- State variables: integer `user_version`; per-step `column_exists` probes.
- States: 0 / 9 .. 34. Version 10 never stays: the v10 block sets 11 (plumbing.rs:161 and its end). There is no v11 block.
- Transitions: each `if due(n)` block (plumbing.rs:63 ... :785) -> sets `user_version = n`. All blocks run in one SQLite transaction, committed at :798.
- Guards / invariants: a failed step rolls the whole run back. Steps v28 and v30 delete all Games, Messages and Swipes (by design, plumbing.rs:662, :716). They run with `foreign_keys` OFF; the pragma is turned on after migration (db.rs:20). They delete child tables by hand, so this works.
- Explicitness: EXPLICIT (integer + one ladder function). Counter used as phase (signal 3).
- Already modeled: no.
- Risks seen:
  - No check for `user_version > LATEST_VERSION`: a database of a newer binary opens and no step runs (known: every `due` is false).
  - Some steps guard with `column_exists`, some do not (v13 `ADD COLUMN` has no guard). The transaction makes this safe; the mixed style is a trap if a step is ever split (inferred).

### Unit of record writes (`Tx`, `TxToken`, `WriteGuard`)
- Where: transaction.rs:19-31 (`TxWrite`), :39 (`TxToken`), :52 (`Tx`), :350 (`commit`), :372-412 (`transaction`, `transaction_new_game`, `run_transaction`); write_guard.rs:12-47; core.rs:93-111 (`WriteOverrides`).
- State variables: the `rusqlite::Transaction` (open / consumed); `WriteGuard.changed` (growing list of `ChangeKind`); `game` bound to the unit; `UnitGame::{Existing, New}`.
- States: Open (created; zero or more writes) -> Committed (changes raised) or RolledBack (dropped on `Err` or panic). The Create unit has one more step: the Game row is written inside `Tx::create` (:67-76) before the closure runs.
- Transitions:
  - Begin -> Open: `run_transaction` :398-410 (the backend mutex is held for the whole run).
  - Write -> Open: `Tx::write` :99 -> `WriteGuard::write` (fail-injection check, body, record kinds on success).
  - Closure `Ok` -> Committed -> counters raised: `tx.commit()` :350 then `changes().raise_unit` (changes.rs:103).
  - Closure `Err` / panic -> RolledBack by `Drop` of `rusqlite::Transaction`; `changed` is dropped with it, nothing raised.
- Guards / invariants: only gate.rs can make a `TxToken` (comment at transaction.rs:36); a `Tx` serves one Game; the closure must not call `Storage` (the backend mutex would deadlock, comment at :48).
- Explicitness: EXPLICIT by ownership (commit consumes `self`) but with no enum; lifecycle by `Drop`. One transition place (`run_transaction`).
- Already modeled: no (the gate that opens units is).
- Risks seen:
  - A closure that swallows a failed write and returns `Ok` commits the other writes. The guard counts only the writes that succeeded, so counters stay right. Known.
  - `raise_unit` runs after `commit` and is not part of it. A panic between them commits data and raises no number (inferred, very unlikely).
  - Nothing prevents a second `Tx` use after an `Err` from inside the same closure (the closure keeps `&mut Tx`). Inferred.

### Change counters (`ChangeCounters`)
- Where: src/adapters/driven/storage/changes.rs:17-24 (`Numbers`), :52-55 (`ChangeCounters`), :122 (`forget_game`); raised from core.rs:249-252 and transaction.rs:403 and gate.rs:208, :218.
- State variables: five engine counters, `revision`, `per_game: HashMap<GameId, GameVersions>` (entry present or absent), `server` UUID (new at each process start).
- States: per-Game entry absent (reads as zeros) / present; Game forgotten after delete; implied: "entry recreated for a deleted Game".
- Transitions: raise (raise_game/raise_engine/raise_unit) -> creates the entry via `entry().or_default()` (:30); `forget_game` -> removes it (:122), called after `GameWriter::replace` (gate.rs:584) and `delete` (gate.rs:592), outside the hold (the `inspect` runs after `hold` returns).
- Guards / invariants: every raise adds to `revision` (the two inner fns, :27-48). All numbers sit behind one lock. Raise of engine kinds happens while the backend lock is held (core.rs:247-252, comment).
- Explicitness: FULLY IMPLICIT lifecycle of the per-Game entry (presence in a map); the counters are plain.
- Already modeled: no.
- Risks seen:
  - After `forget_game`, a late raise for the old id creates the entry again. `set_game_failure` on a missing Game succeeds (UPDATE touches 0 rows, no check), so `not_accepted_with_failure` for a deleted Game can raise BUSY_STATE after the forget. Inferred; the doc comment at changes.rs:120 says no later raise names it.
  - Engine raise happens only on `Ok` of the whole call (core.rs:249). A call that wrote and then failed raises nothing (see next item). Known.

### Storage calls that write in two statements with no transaction
- Where: `save_llm_message` llm_messages.rs:14-52 (INSERT then prune DELETE); `update_world` worlds.rs:297-335 (UPDATE worlds then `write_map_row`); `update_settings` settings.rs (read, closure, write under one lock, one statement); `delete_world` worlds.rs:372-392 (count then DELETE under one lock).
- State variables: none stored; the order of the two statements is the phase.
- States: after statement 1 and before statement 2 (half-done), only if statement 2 fails.
- Transitions: statement 1 -> statement 2. `with_backend_writes` raises the engine kind only when the closure returns `Ok` (core.rs:249).
- Guards / invariants: the backend mutex stops other Storage calls from seeing the half state. A failure in statement 2 does not roll back statement 1.
- Explicitness: FULLY IMPLICIT (call order).
- Already modeled: no.
- Risks seen (known):
  - `update_world`: if `write_map_row` fails, the world row is already updated and no WORLDS number is raised. `write_world` (seed/create) does use a transaction (worlds.rs:175-183), so the two paths disagree.
  - `save_llm_message`: if the prune fails, the new row exists, the call returns `Err` and no LLM_MESSAGES number is raised.
  - The prune orders by `created_at` only (ties are not broken), so the 50-row window is approximate (inferred).

### Settings row (singleton `settings`, id = 1)
- Where: settings.rs:12-125 (`SettingsWrite::{Replace, OnlyIfAbsent}`, `read_settings` :23, `seed_settings`); bootstrap/load.rs:44-75; run.rs:64.
- State variables: row present or absent; nullable `impersonate_connection_id`, `options_connection_id` (models/settings.rs); file `data/settings.json` present or absent.
- States: NoRow (reads give `AppSettings::default()`, settings.rs:33); Seeded (row from file, `INSERT OR IGNORE`); Edited (`INSERT OR REPLACE`). Slot None = "use the Narration Connection" (CONTEXT.md).
- Transitions: boot seed -> NoRow to Seeded only when the row is absent: `seed_settings` (settings.rs:121) from load.rs:73; dashboard save / `update_settings` -> any to Edited (settings.rs:104-117).
- Guards / invariants: seed must not name a Connection that the list lacks (`check_connection_slots`, load.rs:69, also wiring.rs:141). After the first run the database wins over the file (comment settings.rs:118).
- Explicitness: FULLY IMPLICIT (row presence; `Option` slots as fallback mode).
- Already modeled: no.
- Risks seen:
  - `INSERT OR REPLACE` rewrites `created_at` to now on every save (settings.rs:81-88; known).
  - A read with no row returns defaults without writing, so the first `update_settings` creates a row from defaults. Known; harmless.

### Boot sequence and what a restart leaves
- Where: src/bootstrap/run.rs:44-95 (`prepare_data`), :97-119 (`resolve_boot_game`), :121-131 (`prepare_state`), :133 (`start_server`); load.rs; init_game.rs:8-24; wiring.rs:137-165.
- State variables: none held as one value. The phase is the position in this fixed order: open + migrate DB -> seed presets -> seed Worlds/Characters/Personas -> seed settings -> pick World -> check Persona -> resolve Game -> wire graph -> serve.
- States: implied by the call order only.
- Transitions (call order, run.rs:49-91): `ensure_presets` (warn on error, :54) -> `seed_game_data` (warn on error, :58) -> `seed_settings` (fatal, :64) -> `get_world` or fall back to the first World (:66-80) -> `require_persona` (:86) -> `resolve_boot_game` (:88) -> `build_app_graph` (wiring.rs:137) which reads settings and fails on a dangling slot (:141).
- Guards / invariants: seed order matters. Presets must exist before a Game is created (the Game copies preset ids from settings). Settings must be seeded before `build_app_graph` reads them (comment run.rs:62). A seed failure of presets or Worlds is only a warning, so a later step may fail for a reason the log shows earlier.
- Restart: the busy registry is memory only (gate.rs), so every Game boots Idle. A half-done Job leaves the data it already wrote: the Input, or the pre-analysis Swipe (modeled elsewhere), and `games.failure` and the dock row stay as they were. The change counters restart under a new server id (changes.rs:52 comment).
- Explicitness: FULLY IMPLICIT (order of calls).
- Already modeled: no. Boot Game pick uses the active-Game pointer (modeled).
- Risks seen:
  - `ensure_presets` uses `?` inside its loop (run.rs:178-180 area): one bad preset file aborts the seeding of the later files, then the caller only warns. Known for the `?` and the warn; the effect on later files is inferred.
  - `resolve_boot_game` builds a throwaway `GameWriter` and `ActiveGame` (run.rs:97-106). `build_app_graph` makes real ones, and `GameCatalogue::new` sets the pointer a second time (`PointerHold::set`, which touches `updated_at`, active.rs:54-60). Known, harmless today.
  - On resume, the `--persona` argument is checked to exist (run.rs:86) but is not compared with the persona of the resumed Game (init_game.rs:12-15). Known.
  - Two orders for "newest Game": see next item.

### "Newest Game" ordering: two definitions
- Where: games.rs:10-20 (`list_games`, `ORDER BY updated_at DESC`), :34-52 (`list_saved_games_page`, same), :135-160 (`newest_game_of_world`, `COALESCE(MAX(message.timestamp), games.updated_at)`); touch at core.rs:258-273 and active.rs:54; comment at changes.rs:181 says a Message "orders the Games by their newest Message".
- State variables: `games.updated_at`; the newest `messages.timestamp`.
- States: a Game with Messages (ordered by Message time at boot) vs a Game with none (ordered by `updated_at`).
- Transitions: `updated_at` changes on `set_game_config` (transaction.rs:259-286), `update_game_display_name` (games.rs:80-94), `touch_game_updated_at` (core.rs:262). It does not change on `insert_message` or `insert_swipe` (known: no UPDATE of games in those writes).
- Explicitness: FULLY IMPLICIT (two timestamps, two orderings).
- Already modeled: no.
- Risks seen:
  - The Games list and the boot pick can disagree about which Game is newest. The list moves a Game up on a rename or config change, the boot pick does not. Known (SQL read).
  - The comment at changes.rs:181 ("which orders the Games by their newest Message") does not match the list SQL. Known.
  - Every Game has an opening Message from the create unit, so the `updated_at` fallback in `newest_game_of_world` is reached only for Games made by old code or by tests. Inferred.

### Story tail read (`read_tail`) and the window read
- Where: stories.rs:62-93 (`read_tail`), :48-60 (`window`), :215-245 (`window_entries_of`).
- State variables: count of Messages; ids of the last two Messages; presence of `state` on the Swipe before the last.
- States: EmptyStory (count 0 -> `None`); OneMessage (`state_before_last` None); TwoOrMore (before-last state read from its active Swipe).
- Transitions: none stored; derived on each read. Domain rules on top are in domain/model/tail.rs (not read here).
- Explicitness: EXPLICIT derived value (`Tail`), computed in one place. Not an independent state machine.
- Already modeled: partly (Job uses the Tail).
- Risks seen: `load_swipe_state` returns `None` for a missing Swipe row, so `state_before_last` can be `None` with Messages before it (known; the Job then reports MissingData).

### LLM message log and role health
- Where: llm_messages.rs:9-52 (`LLM_MESSAGE_RETENTION_LIMIT = 50`, prune); table llm_messages (`error_message` nullable); readers view_query.rs:249-275 (`role_health`).
- State variables: nullable `error_message` per row; the newest row per `agent_name` inside the retained 50 rows.
- States per Role: Unknown (no row in the window); Healthy (newest row `error_message` NULL); Failing (newest row has an error); Recovered (a newer success after a failure; the failure is then gone from the view).
- Transitions: every model call -> new row (`save_llm_message`); prune keeps 50 newest. No stored health value.
- Guards / invariants: "retention keeps the whole table inside one window" (comment view_query.rs:251). Health is a pure derivation of the log.
- Explicitness: FULLY IMPLICIT (health derived from log rows; no enum, no stored state).
- Already modeled: no.
- Risks seen:
  - The prune is global, not per Role (llm_messages.rs:41-47, known). A busy Role can push the last row of a quiet Role out of the 50-row window. That Role then reads as Unknown, not Failing. This is inferred. The comment at view_query.rs:251 says the window holds every Role's newest attempt, and that holds only while all Roles stay recent.
  - The prune orders by `created_at` text only (known).

### LLM transport and providers
- Where: transport/utils/client.rs:6-73 (`call_chat_completions`), response.rs (`parse_chat_response`, `handle_response`), request.rs:8-12 (`REQUEST_COUNTER`), providers/chat_completions.rs:25-166.
- State variables: only `REQUEST_COUNTER` (static `AtomicU64`, log ids). No retry counter, no backoff, no stored health, no stream (`payload["stream"] = false`, request.rs).
- States: one call is a straight line: build client -> send -> status check -> read body -> parse (error object / choices / content / `finish_reason`) -> result. The failure kinds are the `LlmFailure` variants (Timeout, Network, Http, ParseError, EmptyResponse, TokenBudgetSpent).
- Transitions: all in `handle_response` / `parse_chat_response`; "what counts as an answer" is in one place (response.rs, doc comment).
- Explicitness: EXPLICIT (error enum, one parse place). Not a state machine; no time-varying mode.
- Already modeled: n/a.
- Risks seen: none seen. Note for the list: a new `reqwest` client is built for each call (client.rs:21), so no connection reuse state exists. The provider is resolved per call from settings (wiring.rs:46-65), so no cached Connection can go stale.

### Mock LLM provider (`MockBackend`)
- Where: providers/mock.rs:12-23 (fields), :91-100 (`guard`), :104-175 (`complete`).
- State variables: `should_fail`, `fail_first_n` (counter), `should_return_empty`, `trigger_narration_should_fail` (three flags); `per_call_narrations`, `per_call_prompt_responses` (scripted lists); `call_index` (one cursor shared by both lists).
- States: Failing-forever (`should_fail`); Failing-N (`fail_first_n` > 0) then normal; Empty-answer; Scripted at cursor i; Default. Flag combinations are meaningless in some cases (`should_fail` and `should_return_empty`; `should_fail` wins).
- Transitions: each narration call -> `guard` decrements `fail_first_n` (:92-95); scripted call -> `call_index += 1` (:127, :147).
- Guards / invariants: `fail_first_n` applies to Narrator and Dialogue calls only; other agents ignore it (doc at :42 says so for Trigger and Quantifier).
- Explicitness: FULLY IMPLICIT (flags and counters; test-only fake).
- Already modeled: no. (The strict, holdable `ScriptedModel` in src/test_support is outside my area.)
- Risks seen:
  - `guard` does `load` then `fetch_sub` as two atomic steps. Two concurrent narration calls can both see 1 and both subtract, which wraps `AtomicU32` to `u32::MAX`, so the mock then fails for good. Inferred (needs two concurrent callers; Jobs run one per Game).
  - One `call_index` serves two lists: a call to a non-narration agent moves the cursor of the narration list and the reverse. Known.

### Storage test failure layer (`BackendKind::Test`)
- Where: core.rs:78-91 (`BackendKind`), :93-111 (`WriteOverrides`), :154-215 (`add_failure`, `with_overrides`), test_support.rs:23-60 (`TestFailureHandle`). All behind `feature = "testing"`.
- State variables: `BackendKind::{Direct, Test}`; two maps of planted failures (per `StorageCall`, per `TxWrite`).
- States: Direct; Test(layer with maps, possibly empty).
- Transitions: `add_failure` / `with_test_failures` turn Direct into Test (core.rs:154-180). `clear_all` empties the maps but does not go back to Direct. A second `with_overrides` on a Test layer drops the old maps (`debug_assert!` only, core.rs:188-194).
- Explicitness: EXPLICIT (enum), transitions in two functions of core.rs.
- Already modeled: no.
- Risks seen: a planted failure persists until a test clears it; a failure on a `TxWrite` fires for every unit that makes that write (no "fail once" mode). Known. Test-only.

### Harper dictionary cache
- Where: text_check/harper_text_checker.rs:20-45 (`dictionary: Mutex<Option<(Vec<String>, Arc<MergedDictionary>)>>`).
- State variables: one `Option` holding (word list, dictionary).
- States: Empty; Cached(words).
- Transitions: `check` -> `merged` -> if the cached words equal the new ignored words, reuse; else rebuild and replace (:23-45).
- Explicitness: FULLY IMPLICIT but trivial (memo of one entry).
- Already modeled: no.
- Risks seen: none seen. A shared mutex is held while the dictionary is built (:24-45); two checks wait on each other. Known; minor.

### Seed files vs database (first run vs later runs)
- Where: load.rs:96-184 (`process_world_dir`, `seed_personas`), run.rs:138-277 (`ensure_presets`, `process_preset_file`), worlds.rs:146-153 (`seed_world` upsert), characters.rs:50-70 and personas.rs:53-70 (`INSERT OR IGNORE`).
- State variables: row presence per World key / Character key / Persona key / preset id; for a preset, whether the stored row has any of role, instructions, writing_style, output_format.
- States: NotSeeded -> Seeded (database now owns the row). Preset: Empty-shell row (all four text fields `None`) -> filled by a later seed (run.rs:241-253, keeps `allowed_modes`).
- Transitions: World seeded only when `get_world` finds no row (load.rs:116-123). Characters and Personas are added only when missing. A preset row is rewritten by the seed only when it has no content (run.rs:246).
- Explicitness: FULLY IMPLICIT (row presence as phase).
- Already modeled: no.
- Risks seen:
  - After the first run an edit to a World file never reaches the database (by design; "the database outranks"). `seed_world` documents "replace an existing World" (worlds.rs:143-145) but the only caller seeds only absent Worlds, so the upsert branch is not reached from boot. Known.
  - A World seeded without its map file errors (`read_json_file(&map_path)?`, load.rs:119) and aborts the whole `seed_worlds` loop for the other Worlds; the caller only warns (run.rs:58). Known.

### Items I looked at and did not count
- `DbPool` (db.rs): one connection under one mutex, with poison recovery; no mode. `foreign_keys` is on after migration (see schema ladder).
- Storage `Backend` enum: one variant (`Sqlite`), no transitions.
- Mappers and models: plain structs.
- logging.rs: a three-way choice at start (console env var / file / file failed -> console), once, no later change.

## Table of contents
Dock record | storage/transaction.rs, docks.rs, domain/model/dock.rs | FULLY IMPLICIT
Failure line (`games.failure`) and Status | storage/transaction.rs, application/games/view_query.rs | EXPLICIT STATES, SCATTERED TRANSITIONS (partly modeled)
Message active Swipe index | storage/transaction.rs, stories.rs, mappers/message.rs | FULLY IMPLICIT
Schema version ladder | storage/utils/plumbing.rs | EXPLICIT (integer + one ladder)
Unit of record writes (`Tx`) | storage/transaction.rs, write_guard.rs | EXPLICIT (by ownership + Drop)
Change counters (per-Game entry) | storage/changes.rs | FULLY IMPLICIT
Two-statement writes without a transaction | storage/llm_messages.rs, worlds.rs | FULLY IMPLICIT
Settings row (NoRow / Seeded / Edited, slot fallback) | storage/settings.rs, bootstrap/load.rs | FULLY IMPLICIT
Boot sequence and restart | bootstrap/run.rs, load.rs, init_game.rs, wiring.rs | FULLY IMPLICIT
Two "newest Game" orderings | storage/games.rs, core.rs | FULLY IMPLICIT
Story tail read | storage/stories.rs | EXPLICIT (derived)
LLM message log and role health | storage/llm_messages.rs, application/games/view_query.rs | FULLY IMPLICIT
LLM transport and providers | llm/transport/utils/*.rs, providers/chat_completions.rs | EXPLICIT (no mode; listed for completeness)
Mock LLM provider | llm/providers/mock.rs | FULLY IMPLICIT
Storage test failure layer | storage/core.rs, test_support.rs | EXPLICIT
Harper dictionary cache | text_check/harper_text_checker.rs | FULLY IMPLICIT (trivial)
Seed files vs database | bootstrap/load.rs, run.rs, storage/worlds.rs | FULLY IMPLICIT
