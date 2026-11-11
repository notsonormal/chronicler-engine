Findings from my area (read-only scout, no files changed).

### Schema migration ladder
- Where: `src/adapters/driven/storage/utils/plumbing.rs:17` (`LATEST_VERSION: i32 = 34`), `:25` (`migrate_up_to`), `:53` (`let due = |step| version < step && step <= up_to`)
- State variables: `PRAGMA user_version` read at `plumbing.rs:46-50`; each block writes a literal number (`:157` → 9, `:246` → 11, and one per block to `34`); `up_to` bounds the run (`run_migrations` at `:19-21`, tests call `migrate_up_to` with 27/28/29/30/31/32, e.g. `plumbing_tests.rs:511`)
- States: "schema at version N" for N in 0…34, plus "partially migrated" (only reachable inside the one transaction, so not observable). Implied states: "database written by a release whose `LATEST_VERSION` was lower" and "fresh database".
- Transitions: boot or `DbPool::new` (`db.rs:19`) → run every block where `version < step <= up_to` → each block sets `user_version` → single `tx.commit()` (`plumbing.rs:791-793`). Destructive transitions: v28 drops `messages`/`message_swipes`/`game_state_snapshots` and deletes every Game (`:662-714`); v30 deletes messages, swipes and Games again (`:716-728`).
- Guards / invariants: steps run in ascending order; a step is skipped when `version >= step`, so a step becomes a no-op exactly once. Blocks use `column_exists` / `CREATE TABLE IF NOT EXISTS` so a second run is *mostly* safe. Documented invariant: a test stops before a step that erases an earlier step's rows (`:21-24`).
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (34 unnamed blocks; no array of steps, no step name, no per-step "from" version check — the block's own number is duplicated in `if due(N)` and in the `pragma_update` literal).
- Already modeled: no.
- Risks seen:
  - known: `due(10)`'s block sets `user_version` to **11** and there is no `due(11)` block (`:246`). A database whose `user_version` is exactly 10 therefore skips all of v9→v11 schema creation. Whether a released build ever wrote 10 is unknown to me (inferred: the code intends "block 10 covers 10 and 11", which breaks the one-step-per-number rule that `migrate_up_to` depends on).
  - known: there is no per-step version guard inside a block, so `migrate_up_to(v, 5)` and the full run differ only by the `due` predicate; a test that raises `up_to` past a destructive step erases the fixture rows the earlier steps built (tests work around this with `UP_TO_V27`).
  - inferred: `LATEST_VERSION` and the last `if due(34)` block can drift silently — nothing asserts that the highest block equals `LATEST_VERSION`.

### Boot seeding precedence (which source wins, and when)
- Where: `src/bootstrap/run.rs:44-95` (`prepare_data`), `src/bootstrap/load.rs:30-46` (`seed_game_data`), `:49-76` (`seed_settings`), `:92-159` (`process_world_dir`), `:162-184` (`seed_personas`), `run.rs:143-272` (`ensure_presets`, `process_preset_file`); storage side: `src/adapters/driven/storage/settings.rs:13-16` (`SettingsWrite::{Replace, OnlyIfAbsent}`), `worlds.rs:396-421` (`WorldWriteMode::{Insert, Upsert}`)
- State variables: presence of a row (`existing_ids.contains`, `get_world(...)`, `get_persona(...)?.is_none()`, `existing_chars.contains`), the `has_content` test on a preset (`run.rs:259-268`), and the two file-existence tests (`run.rs:156`, `load.rs:59`).
- States: per entity kind, "no row → seed writes it", "row present → seed does nothing", "row present but empty of content → seed overwrites" (presets only), "world row present → seed **replaces** it on every boot" (worlds), "settings row present → `data/settings.json` is ignored forever" (`INSERT OR IGNORE`, `settings.rs:64-65`, `load.rs:73-75`).
- Transitions: boot → seed. `run.rs:54` and `:58` tolerate a seed error (warn only); `run.rs:64` `seed_settings` is fatal (`load.rs:63` malformed file, `load.rs:69` dangling connection slot).
- Guards / invariants: the documented rule is "after the first run the database outranks `data/settings.json`" (`settings.rs:118-120`). The four kinds do **not** share one rule: settings = insert-only, personas/characters = insert-only (`characters.rs:63`, `personas.rs:62` `INSERT OR IGNORE`), presets = insert-only unless the stored copy is bare (`run.rs:259-268`), worlds = upsert on every boot (`load.rs:123` → `worlds.rs:120-133` `WorldWriteMode::Upsert`).
- Explicitness: FULLY IMPLICIT (row presence is the whole state; three different precedences in four files).
- Already modeled: no.
- Risks seen:
  - known: an edited `World` in the database is overwritten on the next boot, while an edited `Character` or `Persona` is never updated from the file. The two halves of the same authored data set have opposite precedence rules (`load.rs:123` vs `characters.rs:63`).
  - known: a malformed `data/settings.json` fails the boot even when the settings row already exists and the file would be ignored by `INSERT OR IGNORE` (`load.rs:59-73`).
  - known: a `seed_settings` error after the settings row exists is invisible by construction, so a "seeded" log line and a "no-op" are indistinguishable in the log.
  - inferred: `process_preset_file`'s `has_content` test only looks at the four text columns; a preset whose text is present but whose `parameters`/`option_count` seed fields are missing keeps the old values with no warning beyond the skip.

### Storage backend mode: Direct vs Test (failure injection)
- Where: `src/adapters/driven/storage/core.rs:78` (`enum BackendKind`), `:92` (`enum WriteOverrides`), `:140-176` (`with_failure`, `with_test_failures`, `add_failure`), `:179-211` (`with_overrides`), `:217-255` (`with_backend_writes`); `write_guard.rs:14-45`
- State variables: `BackendKind::{Direct, Test}`; `Test.overrides: HashMap<StorageCall, TestOverride>`; `Test.writes: HashMap<TxWrite, TestOverride>`; `base: Box<Backend>` (the "replace, never nest" invariant).
- States: Direct (no injection) → Test (injection layer present, zero or more failures armed). Within Test, each entry is armed until cleared (`test_support.rs:32-50` `set`/`clear`/`clear_all`).
- Transitions: `with_failure`/`add_failure` → Direct→Test (`core.rs:154-176`); `with_test_failures` → Direct→Test with empty maps (`:146-152`); `with_overrides` → Direct→Test, or Test→Test which is the invalid move (`:179-211`); every call consults the map (`:240-250`); every `Tx` write consults the write map (`write_guard.rs:33`).
- Guards / invariants: at most one Test layer; `Box<Backend>` is the type-level pin. A failure key is a `StorageCall`/`TxWrite` variant, so an injection is compiler-checked.
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (three constructors, one of which is only a `debug_assert!`).
- Already modeled: no.
- Risks seen:
  - known: `with_overrides` called on an already-Test storage **drops the existing overrides silently** in a release build — the guard is `debug_assert!(false, ...)` at `core.rs:192-198`, so debug panics and release loses state.
  - known: an armed `TestOverride` never expires; a test that forgets `clear` fails every later call on that storage, and a write injection (`writes` map) is invisible to `add_failure` on a Direct storage (it returns early into the `Test` branch, `:158-165`).

### The unit of record writes (`Tx`) lifecycle
- Where: `src/adapters/driven/storage/transaction.rs:39` (`TxToken`), `:52-57` (`struct Tx`), `:350-355` (`commit`), `:369-412` (`transaction`, `transaction_new_game`, `run_transaction`); `write_guard.rs:14-45`
- State variables: the `Tx` itself — a live `rusqlite::Transaction`, a `WriteGuard` accumulating `changed: Vec<ChangeKind>`, and `game: GameId`. `TxToken` is a capability held by the caller.
- States: **open** (row lock held, backend mutex held, `changed` growing) → **committed** (`:406`, kinds raised at `:407`) or → **dropped** (implicit rollback; `changed` discarded, nothing raised).
- Transitions: `opener.open` → open; `f` returns `Ok` → `commit` → raise; `f` returns `Err` or panics → drop → rollback, no raise. `Tx::create` is the "new Game" entry and writes the Game row first (`:69-76`).
- Guards / invariants: a `Tx` write can only touch the `Tx`'s Game (`require_message_of_game`, `utils/records.rs:11-27`); the closure must make **no** `Storage` call (documented at `:44-46`) because the backend mutex is already held; `TxToken::new` is `pub(crate)` so only `gate.rs` opens a unit (`:35-39`).
- Explicitness: FULLY IMPLICIT (lifecycle is `Drop`, and "committed" has no type).
- Already modeled: partially (the pre-analysis/analysed write pair is the modeled part).
- Risks seen:
  - known: the "no `Storage` call inside the closure" rule is a comment, not a guard. The backend mutex is a non-reentrant `std::sync::Mutex` (`db.rs:27-32`, `core.rs:222-227`), so a nested call deadlocks rather than erroring.
  - known: `result.is_ok()` at `core.rs:249-253` raises the *call's* kinds after the closure returns; for `StorageCall::Transaction` the kind list is empty (`changes.rs:186`), so the raise path depends on `raise_unit` inside `run_transaction` — two raise sites for one logical commit, one of them conditional on the `StorageCall` table.
  - known: a delete unit raises after the row is gone (`transaction.rs:143-152` then `:407`), and the per-Game numbers are removed only by `ChangeCounters::forget_game` (`changes.rs:122`), which storage never calls — the gate calls it (`gate.rs:614`, `:624`). Two places must agree for the numbers of a deleted Game to disappear.

### Active Swipe index (`messages.active_swipe_index`)
- Where: column `active_swipe_index INTEGER NOT NULL DEFAULT 0` (`plumbing.rs:113`, `:690`); row struct `models/message.rs:10`; writer `transaction.rs:158-171` (`set_active_swipe`); readers `stories.rs:189-214` (`active_swipe`), `:243-278` (`window_entries_of`)
- State variables: `active_swipe_index: usize` per Message, plus the `message_swipes` rows it must point at.
- States: "exactly one active Swipe, index < count" (valid), "index ≥ count" (invalid but storable), "Message with zero Swipes" (impossible through `Tx`, since `insert_message` and `insert_swipe` share a unit, but readable as an error at `mappers/message.rs:63-67`).
- Transitions: `insert_swipe` appends → `set_active_swipe` moves the pointer → `delete_message` removes the Message and cascades its Swipes. Repair transition: `ensure_valid_swipe_index` silently resets an out-of-range index to 0 on read (`domain/model/message.rs:148-152`).
- Guards / invariants: the gate checks `index >= last.last.swipes.len()` (`gate.rs:543-551`) and the domain `switch` re-checks (`domain/model/message.rs:118-121`). **The storage write has no such check**: `set_active_swipe` verifies only that the Message belongs to the Game (`transaction.rs:159`).
- Explicitness: FULLY IMPLICIT (an integer plus a list length; no `ActiveSwipe` type).
- Already modeled: no.
- Risks seen:
  - known: `Tx::set_active_swipe` accepts an index with no matching `message_swipes` row. The subsequent window read LEFT JOINs on that index and turns the missing row into `EngineError::Internal("message {id} has no swipe")` (`stories.rs:248-258`) — one bad index makes the **whole** story window fail, not just that Message.
  - known: `ensure_valid_swipe_index` hides the corruption by silently resetting to 0 on the `Message` read path while `window_entries_of` (a different path, no `Message` involved) errors on it. Two readers, two behaviours for the same invalid state.

### The dock record (Options offered / Options failure)
- Where: table `game_docks (game_id PRIMARY KEY, offered, notice)` (`plumbing.rs:640-646`); read `docks.rs:19-41`; writes `transaction.rs:307-335` (`put_dock`, `put_options_failure`, `clear_dock`, `write_dock`); domain `domain/model/dock.rs:11-18`
- State variables: presence of the `game_docks` row; `offered: Vec<String>`; `notice: Option<OptionsFailure>`.
- States: **(a) no row** (a Game that never ran Options); **(b) row, offered empty, no notice** (cleared — the value `clear_dock` writes, `transaction.rs:321-325`); **(c) row, offered non-empty, no notice** (Options available); **(d) row, notice present** (Options failed; `put_options_failure` keeps the previous `offered`, `transaction.rs:307-319`).
- Transitions: Job Options success → (c); Options failure → (d) from any state (**including a; the fresh insert writes `offered` default `'[]'`**); Narration saved or Stop → (b) or (c); swipe switch / delete-last → (b) (`gate.rs:585-591`); `put_dock(DockRecord::default())` is the same as `clear_dock`.
- Guards / invariants: documented as "present when a reason line stands in for the offered set" (`dock.rs:17-18`) — i.e. (c) and (d) are meant to be exclusive. "The row stays: a Game holds one dock record" (`transaction.rs:320`).
- Explicitness: FULLY IMPLICIT (a nullable column plus row presence; `DockRecord::default()` is the erased state).
- Already modeled: no (the dock as a concept is not in the already-modeled list).
- Risks seen:
  - known: (c)+(d) together — `offered` non-empty **and** `notice` present — is reachable by design (`gate.rs:876-882` keeps the offered set), and the template then shows the notice and hides the offered set (`driving/http/templates.rs:64`). The stored `offered` is invisible until something clears the notice, so the dock holds data the UI cannot show.
  - known: "no row" (a) and "cleared row" (b) are the same mode to every caller but different values in the table; only `DockReads::get` distinguishes them (`docks.rs:31-33`), and nothing normalises (a) to (b).
  - inferred: `put_options_failure`'s `INSERT ... ON CONFLICT` leaves `offered` at SQLite's `'[]'` default on a fresh insert, so the fresh-insert path and the conflict path write different column sets from one function.

### Persisted Game failure line (`games.failure`) vs the in-memory holder
- Where: column added at `plumbing.rs:648-650`; row field `models/game.rs:24` (`failure_json: Option<String>`), decode `:74-88`; writer `transaction.rs:263-275` (`set_game_failure`); reader `application/games/view_query.rs:198-212` (`Status`); renderer `driving/http/layout/handlers/endpoints.rs:48-61`
- State variables: `games.failure` (persisted, survives a restart) and the busy registry (`application/generation/gate.rs`, memory only).
- States: `Idle` (no failure, no holder), `Failed(kind, line)` (failure row, no holder), `Busy(step)` (holder, **failure may or may not be set**), and the implied `Busy + stale failure` state.
- Transitions: accepted Job → clears the failure (`gate.rs:738-742` `accept_in_tx`); a Job that cannot run → writes the failure (`gate.rs:687-693`, `:873-877`); swipe switch / delete / stop → clears it (`gate.rs:554`, `:591`, `:521-525`); restart → the failure **stays**, the holder is gone.
- Guards / invariants: `Status` gives the holder priority (`view_query.rs:199-201`), so a failure written while a Job still holds the Game is masked until the hold drops. Documented: "a failure line is not busy" (`driving/http/layout/handlers/action_controls.rs:43`).
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (the `GenerationStatus` enum exists; the persisted column and the memory holder are its two halves and five places write them).
- Already modeled: yes for the Job/gate part; the **persisted** half is not.
- Risks seen:
  - known: `Busy` + non-absent `failure` is reachable (the hold is released after `save_failure`), and the two readers disagree about it: `Status` says Busy, the `games` list/row shows the failure text. Nothing asserts the combination is impossible.
  - known: nothing at boot clears or reconciles `games.failure`, so a Game that failed before a restart reads `Failed` forever until another accepted request touches it (grep of `src/bootstrap/` finds no failure handling).
  - inferred: a crash between `save_text` and `save_state_update` leaves a Game that reads `Idle` with a Swipe state that the game rules never updated — the one mid-Job state that leaves no failure line at all.

### Change counters and the restart stamp
- Where: `src/adapters/driven/storage/changes.rs:19-26` (`Numbers`), `:40-56`, `:84-101` (`versions`), `:122` (`forget_game`), `:127-138` (`raise_unit`), `:155-186` (`kinds_of_call`), `:190-207` (`kinds_of_write`); raise sites `core.rs:249-253`, `transaction.rs:407`
- State variables: six monotone counters plus `revision`, `per_game: HashMap<GameId, GameVersions>`, and `server: uuid` (a fresh id per process).
- States: "client's stamp matches this server → numbers are meaningful" vs "different `server` → reload everything"; per Game, "never raised → zeros" (no map entry) vs "raised".
- Transitions: a successful `Storage` call → raise engine kinds; a committed unit → raise unit kinds; `forget_game` → drop the Game's numbers (`gate.rs:614`, `:624`); process start → new `server` id.
- Guards / invariants: raise only after the data landed ("a load that reads the committed data waits for the lock", `core.rs:247-248`); one lock for all numbers so a snapshot is coherent (`changes.rs:18`); every raise adds to `revision` (`:40`, `:46`).
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (raise sites are correct but the "which kind does this write raise" table is a third place that must track `TxWrite`).
- Already modeled: partially (busy state raises from the gate).
- Risks seen:
  - known: after `forget_game`, `versions(Some(deleted_id))` returns zeros again (`:84-101`), which is the same answer as a Game that has never changed. A client that asks about a deleted id cannot distinguish "gone" from "fresh".
  - inferred: `kinds_of_write` and `kinds_of_call` both need to be updated when a `TxWrite`/`StorageCall` variant is added; the `#[deny(clippy::wildcard_enum_match_arm)]` keeps them exhaustive, but nothing checks that the kind is the *semantically* right one (e.g. `SetGameFailure` raising `BUSY_STATE` at `:199`).

### MockBackend scripted state
- Where: `src/adapters/driven/llm/providers/mock.rs:14-22` (four `AtomicBool`/`AtomicU32` plus two `Vec<String>` and `call_index: AtomicUsize`), `:91-99` (`guard`), `:118-160` (the two branches)
- State variables: `should_fail`, `fail_first_n` (a spendable counter), `should_return_empty`, `trigger_narration_should_fail`, `per_call_narrations`, `per_call_prompt_responses`, `call_index` (a rotating index, `idx % len`).
- States: narration branch — plain (`[MockNarration] …`), scripted (`per_call_narrations` non-empty → rotate by `call_index`), empty (`should_return_empty`), failing (`fail_first_n > 0`, then `should_fail`). Non-narration branch — failing (`should_fail` **or** `trigger_narration_should_fail`), scripted (`per_call_prompt_responses`), the Options default text, the generic continuation.
- Transitions: `guard` decrements `fail_first_n` **only** on the narration branch (`:121-123`); each scripted read advances `call_index` (`:127`, `:147`); no transition resets either counter.
- Guards / invariants: branch priority is `fail_first_n` → `should_fail` → `should_return_empty` → scripted → default (`:91-135`). The two scripted lists share **one** `call_index`.
- Explicitness: FULLY IMPLICIT (flags plus counters plus rotating lists; the priority order is only the `if` chain).
- Already modeled: no.
- Risks seen:
  - known: `call_index` is shared between the narration list and the prompt-response list (`:127` and `:147` both `fetch_add`). A test that arms both lists gets interleaved indices, so neither list is consumed in order.
  - known: `with_trigger_narration_fail` fails **every** non-narration agent — Quantifier and Options included (`:137-139`), because the branch keys on "not a Narrator/Dialogue call", not on the Trigger Role (`:118-120`).
  - known: `with_prompt_responses` is silently ignored on a narration call (the narration branch never reads it, `:121-135`), so a test that expects a scripted narration text gets `[MockNarration] …`.
  - known: `fail_first_n` is spent by narration calls only; an armed budget survives any number of Quantifier/Options failures and reappears at the next narration.
  - known: in production wiring the provider is rebuilt for every call (`bootstrap/wiring.rs:52-76`, resolver called per `complete` at `application/llm_recorder.rs:42-56`), so a `LlmBackendType::Mock` connection in a real database starts with a fresh scripted state on every call — `fail_first_n` can never reach zero there.

### Harper dictionary cache
- Where: `src/adapters/driven/text_check/harper_text_checker.rs:21` (`Mutex<Option<(Vec<String>, Arc<MergedDictionary>)>>`), `:29-47` (`merged`), `:62-64`
- State variables: the single cached `(ignored_words, dictionary)` pair.
- States: cold (no cache) → warm with key K → warm with key K' (rebuilt once per key change).
- Transitions: `check` → `merged` compares the cached word list by value (`:31-34`) and rebuilds on mismatch (`:36-47`).
- Guards / invariants: the cache is valid only for the word list it was built from; one slot only, so two different `TextCheckSettings` in one process thrash it.
- Explicitness: FULLY IMPLICIT (a memo whose key is the whole state).
- Already modeled: no.
- Risks seen: none seen — the key is compared by value, so a stale dictionary is not reachable; the cost is a rebuild per alternating key (known, from reading `:31-34`).

### LLM message retention window
- Where: `src/adapters/driven/storage/llm_messages.rs:11` (`LLM_MESSAGE_RETENTION_LIMIT: usize = 50`), `:38-44` (prune after every insert); `models/llm_message.rs:10` (`error_message: Option<String>`)
- State variables: the row count of `llm_messages`; `error_message` null vs set.
- States: "fewer than 50 rows" (history preserved) vs "at the limit" (each insert deletes the oldest); per row, a success record vs a failure record.
- Transitions: `save_llm_message` → insert → prune (`:38-44`). No other writer.
- Guards / invariants: a read of the whole retained window must ask for exactly `LLM_MESSAGE_RETENTION_LIMIT` (`:9-10`).
- Explicitness: FULLY IMPLICIT (a counter as the state; the limit is a constant in this file and a caller convention).
- Already modeled: no.
- Risks seen:
  - known: retention is by `created_at` string ordering (`:40-43`), while `get_llm_message` is by `id`; two rows with equal `created_at` (same second, no sub-second tie-break in the ORDER BY) can prune in an order that disagrees with the read order.
  - inferred: the limit is duplicated as a caller convention (`list_latest_llm_messages(limit)` at `:75-95` takes any limit), so a reader asking for more than 50 silently gets a truncated window with no marker.

### Boot Game resolution
- Where: `src/bootstrap/init_game.rs:8-27` (`resolve_game_id`), `src/bootstrap/run.rs:97-118` (`resolve_boot_game`), `:121-130` (`prepare_state`), `wiring.rs:105-112` (`GameCatalogue::new` with `initial_game`)
- State variables: `Option<GameId>` of the newest Game of the World (`storage.games.rs:145-176`, `newest_game_of_world`), the throwaway `ActiveGame` inside `resolve_boot_game`, and the second `ActiveGame` the wired graph builds.
- States: "no Game for this World → create one" vs "a Game exists → resume it"; implied: "the resolved Game has no row" (warned at `run.rs:110-114`, but the id is still used).
- Transitions: `newest_game_of_world` → `Some` → resume; `None` → `catalogue.create_game` (`init_game.rs:12-24`) → new Game with its opening Narration. Then `build_app_graph` → `GameCatalogue::new(..., Some(id))` → `PointerHold::set` → `touch_game_updated_at` (`catalogue.rs:32-34`, `active.rs:71-79`).
- Guards / invariants: the World and the Persona are checked before the Game is resolved (`run.rs:77-86`); the pointer must name an existing Game (`active.rs:16-18`).
- Explicitness: FULLY IMPLICIT (an `Option`; the "create vs resume" choice is one `match` in `init_game.rs`).
- Already modeled: the pointer is; the boot phase split is not.
- Risks seen:
  - known: `resolve_boot_game` builds its own `GameCatalogue` with its own `ActiveGame` (`run.rs:99-104`), and `GameCatalogue::new` calls `PointerHold::set` → `touch_game_updated_at`. The wired graph then sets its own pointer and touches the Game a **second** time (`wiring.rs:105-112` → `catalogue.rs:32`). The boot writes `updated_at` twice on the same Game, and `updated_at` is a tie-break key of `newest_game_of_world` (`games.rs:170`).
  - known: `run.rs:108-114` logs a warning when the resolved Game has no row and then keeps going with that id, so the process starts with an active pointer that names nothing.
  - inferred: the throwaway catalogue's pointer is never released, but it is dropped, so no other observer can see it — the only lasting effect is the extra `updated_at` write.

Contents:

| name | file | explicitness |
|---|---|---|
| Schema migration ladder | `src/adapters/driven/storage/utils/plumbing.rs:17,25,53` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Boot seeding precedence | `src/bootstrap/run.rs:44-95,143-272` + `src/bootstrap/load.rs:30-184` | FULLY IMPLICIT |
| Storage backend Direct/Test mode | `src/adapters/driven/storage/core.rs:78,92,140-255` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Unit of record writes (`Tx`) lifecycle | `src/adapters/driven/storage/transaction.rs:39,52,350,390-412` | FULLY IMPLICIT |
| Active Swipe index | `src/adapters/driven/storage/transaction.rs:158` + `stories.rs:243-278` | FULLY IMPLICIT |
| Dock record states | `src/adapters/driven/storage/transaction.rs:307-335` + `docks.rs:19-41` | FULLY IMPLICIT |
| Persisted Game failure line | `src/adapters/driven/storage/transaction.rs:263` + `models/game.rs:24,74` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Change counters and restart stamp | `src/adapters/driven/storage/changes.rs:19-138` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| MockBackend scripted state | `src/adapters/driven/llm/providers/mock.rs:14-160` | FULLY IMPLICIT |
| Harper dictionary cache | `src/adapters/driven/text_check/harper_text_checker.rs:21,29-47` | FULLY IMPLICIT |
| LLM message retention window | `src/adapters/driven/storage/llm_messages.rs:11,38-44` | FULLY IMPLICIT |
| Boot Game resolution | `src/bootstrap/init_game.rs:8-27` + `src/bootstrap/run.rs:97-130` | FULLY IMPLICIT |

Checked and rejected as plain data or pure functions: `models/{character,persona,message,swipe,map,world,prompt_preset}.rs` row structs; `mappers/*`; `utils/harper.rs` span helpers; `llm/transport/utils/{request,response}.rs` (one-shot call, no retry, no streaming; the only mutable state is `REQUEST_COUNTER`, a log-only id at `request.rs:14-18`); `text_check` mode dispatch (`harper_text_checker.rs:71-86`, a pure `match` on `TextCheckMode`).