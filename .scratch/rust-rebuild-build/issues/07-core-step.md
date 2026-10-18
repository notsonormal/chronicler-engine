# The core step (C1 to C7)

Type: task
Status: claimed
Blocked by: 06

## Question

Build §9 of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) on branch `wf/rebuild-core`, from the current `main`. Read §1, §2 and §9 of the plan, and §8 (P5 wrote the target tests). The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §9 pass. The owner has approved the last commit, and the branch has merged to `main`.

## Comments

**2026-10-18, lead (handoff from P5).** (1) When C5 removes an `#[ignore]` from a target test, check that the test turns green for the right reason: change its expectation once, see it fail, and change it back. (2) The Job runner keeps the test-only Job-task tracker (`GenerationGate::watch_job`, `wait_for_jobs`) at its spawn site, in `job.rs` or `test_support/`, not in the new `gate.rs` (a second lock there is a rule-18 re-look trigger). (3) Run the target tests with `cargo nextest run --no-fail-fast --test http --run-ignored all -E 'test(races::)'`. (4) Ticket 06 lists the gaps of the stop matrix.

**2026-10-18, implementer (C1, uncommitted).** Branch `wf/rebuild-core` (base `f64f5d41`). Area 1, added next to today's records: the migration, the new domain types, `stories.rs`/`docks.rs`, the reads by `GameId`, the record writes on `Tx`, and the storage seam contract for the new records. Tier 1 stays green. No commit was made, and no branch was created.

Facts measured.

| Item | Result |
|---|---|
| `adapters/driven/storage/transaction.rs` (352 lines) | `TxWrite` grows to 11 keys (one per record write); `Tx` holds the Game of the unit, set by `Storage::transaction(game, f)` or, for `transaction_new_game`, by `insert_game`. New writes: `insert_message`, `set_active_swipe`, `delete_message`, `insert_swipe`, `set_swipe_text`, `set_swipe_state`, `set_game_failure`, `put_dock`, `clear_dock`. Each checks its key first, and each names the unit's Game: the update statements carry `game_id`, and `insert_swipe`/`set_swipe_text`/`set_swipe_state` check that the Message belongs to it (`MessageNotFound` otherwise). |
| `adapters/driven/storage/stories.rs` (275 lines, new) | `StoryReads` with `tail(game)`, `swipe_state(game, at)`, `entries(game, before, limit)`. Narrow reads: `tail` reads the ids, the last Message with its Swipes and one state; `entries` reads one row per Message through a join and no state. |
| `adapters/driven/storage/docks.rs` (57 lines, new) | `DockReads::get(game)`. |
| `adapters/driven/storage/games.rs` | `GameReads::get(game)`, the Game record with its failure line. |
| `adapters/driven/storage/utils/records.rs` (12 lines, new) | The one answer a read or a write of a new record gives on the in-memory backend (`Internal`); `storage/` is not an allowed folder for a top-level free fn, so the helper lives in `utils/`. |
| Migration v27 | `messages.kind TEXT`; `message_swipes.state TEXT`, `maker TEXT`; `message_swipes.location_header` renamed to `room_report`; `games.failure TEXT`; `CREATE TABLE game_docks (game_id PK, offered TEXT NOT NULL DEFAULT '[]', notice TEXT)`. The old steps stand, so a fresh database creates `location_header` at v9 and renames it at v27. |
| Domain types (new) | `dock.rs`: `DockRecord`, `Notice`. `tail.rs`: `Tail` with `current_state`, `start_at`, `retrigger_allowed`, `can_delete`. `generation.rs`: `GenerationOrder`, `Place`. `message.rs`: `SwipeAt`, `StoredInputs`, `Maker`, and `Swipe.state`/`maker`/`room_report`. `message_types.rs`: `MessageKind`, and `Message.kind`. `trigger_context.rs`: `TriggerRef`, and `EventRecord` moved here from `agent.rs` and nested as the sketch draws it. `game.rs`: `Game.failure`. `swipe_state.rs`: `trigger_context`. `generation_status.rs`: `GenerationPhase` renamed `Step`. |
| Storage seam contract | `tests/storage/story_storage.rs` (764 lines, new), 18 cases, SQLite only: tail (last Message, none, the state before the last), the state round trip (whole, none, another Game's Swipe), entries (window, entry fields, kind), delete, edit, the write injection (rule 14's test half on a new record), the Game failure line, the dock record (round trip, notice, clear) and the in-memory refusal. `validate_seam_cases.py`: 5 contracts, 83 cases. |
| Tests | `--test-target storage` 125 passed (was 107: 18 new). `--test-target lib` 1240 passed (was 1235: 5 new `domain/model/tail_tests.rs` cases; none of the 1235 changed behaviour). Full gate: integration 1646 passed + 7 skipped (1653), guardrails 189, browser 29. |

Decisions, with the reason.

1. **`room_report` renames `location_header`** (the field, `Message::room_report()`, `set_room_report`, `update_swipe_room_report`, `MessageEntry.room_report` and the DB column). Sketch 25 names `room_report` "today location_header", one concept with one field keeps rule 9 ("a save writes only what changed") true, and the plan's C6 list does not name the column. The rename is mechanical, so §2's mechanical rule covers the 28 files that carry it.
2. **`Swipe.maker` is `Option<Maker>` from C1 to C5**, like `Swipe.state` and `Message.kind` (ticket 17 Q5): the column is new, so a row today's code wrote holds none, and C6 makes the three options required. `room_report` stays a natural `Option`, as today's `location_header` is.
3. **`Storage::transaction` takes the `GameId`** (sketch 25: "every write on a Tx goes to the Tx's Game"). `transaction_new_game` sets it after the Game row lands. This changes P5's signature, which the plan's C2 row changes again for the token; the two seam cases of P5 changed in place (mechanical).
4. **`Tail.state_before_last` is `Option<(SwipeAt, SwipeState)>`.** The sketch holds the state alone, but `redo()` and `retrigger()` need the Swipe they start from, and a `SwipeState` carries no id. `current_state()` answers `Option<&SwipeState>` while `Swipe.state` is an `Option`; C6 drops the `Option`.
5. **`Tail::redo`/`retrigger` wait for `Refusal`** (C2 lands it). C1 builds the derived rules that need no refusal.
6. **`EventRecord` moved to `state/trigger_context.rs`** next to `TriggerRef` (sketch §7.2) and took the sketch's shape (`{ trigger, name, direction }`); `StoryEntry` stayed in `agent.rs` (P4's home) and gained the fields the `entries` read returns (`id`, `kind`, `room_report`, `event_header`, `swipe_count`), with `From<&MessageEntry>` filling all but `kind`. `Message` keeps `message_type` next to `kind` until C6, so the story reads carry both.
7. **`Tx` keeps its record writes as methods in `transaction.rs`.** The plan names no `GameWrites`/`MessageWrites`/`SwipeWrites`/`DockWrites` handles, P5 already put the Game writes on `Tx`, and rule 14's compiler half only needs the writes to live on `Tx`. The sketch's per-record handles are not built.
8. **`Notice { reason: String, raw: String }`**: the design's words are "a reason line with its raw text". C6 decides whether the reason becomes typed.
9. **`Step` renames `GenerationPhase`** (the fate table says "changed"), so no duplicate enum exists between C1 and C6. 34 files change mechanically, and `game_flow.md`'s one sentence follows.
10. **`StoryEntry.id` and `SwipeAt.message` are `u64`.** The crate has no `MessageId` newtype, and the plan's Area 1 list does not name one; `Message.id` stays `u64` until C6.
11. **No in-memory arm for the new reads and writes** (ticket 15 Q4). The seam contract pins the refusal.
12. **`merge` of `StoredInputs`**: `GenerationOrder.inputs` needs the three stored-input values, so `StoredInputs` exists with `Swipe::stored_inputs()`; `Swipe` keeps its three inline fields until C6, which the plan does not list as a C1 change.

Deviations and gaps.

- **`Tail` and `GenerationOrder` have no production caller in C1**, as the plan says ("nothing calls the new writes from production yet"). `Tail`'s derived rules have plain unit tests (`domain/model/tail_tests.rs`), where the one rule allows them.
- **`Tx::insert_message` writes the kind; there is no `set_message_kind` write.** The design decides a Message's kind when the unit writes it, and C4's save units only replace a Swipe's text or state. If a later slice needs to change a kind, it adds the write and its key.
- **The swipe-row SELECT exists twice** (`swipes.rs` for today's path, `stories.rs` for the new read). `stories.rs` runs inside one `with_backend_mut` hold, so it cannot call the `Storage` method of the old path (C9: the backend mutex is not re-entrant). C6 deletes the old path.
- **The rename's data preservation is not covered by a test.** A fresh database proves the rename runs (v9 creates `location_header`, v27 renames it, and every swipe write after it passes), but no case starts from a v26 file on disk. SQLite's `RENAME COLUMN` keeps the rows by definition, and C6 deletes those rows (OR-01).
- **The 5 `#[ignore]`d target tests of `races.rs` were not re-run in C1.** C1 changes their shape mechanically (`room_report` accessor, `state`/`maker` as `None`), and the http binary compiles and passes every other test; the direct `cargo nextest --run-ignored` call of the lead's note stays outside the build slot, so it is C5's check.
- **`AGENTS.md` and `tests/AGENTS.md` were regenerated** with the four `generate_*_index.py` scripts (not staged), so the structure index names the new files.

Validation. Full gate `python build.py` green at `logs/build_20261018_140847.log` (tree `f64f5d4157+d545f2ed`, 167.4 s): fmt, data, clippy, test structure, spec-coverage, seam-cases (5 contracts, 83 cases), Python, http_routes/guardrails freshness, docs, guardrails 189 passed, integration 1646 passed + 7 skipped, browser 29 passed. Earlier full gates of this work: `logs/build_20261018_135611.log` (192.2 s, first green) and `logs/build_20261018_140242.log` (177.3 s). Focused runs: clippy `logs/build_20261018_140026.log`, storage `logs/build_20261018_140745.log` (125 passed), lib `logs/build_20261018_140752.log` (1240 passed), guardrails `logs/build_20261018_135549.log` (189 passed). One full gate failed on a two-line `//!` module summary in the new test file (`logs/build_20261018_135611.log` preceded by the failing run); it is one line now.

Not done, with the reason. The commit gate (`/simple-after-plan`) and the owner's approval are the commit step, which this assignment excludes ("Do not commit or switch branches").

**2026-10-18, implementer (C1, fix pass, uncommitted).** The spec review's items 1 to 5 and the standards review's items 6 to 11 are applied. No commit was made.

Each number, to its result.

| # | Result |
|---|---|
| 1 | `Tail::start_at` follows the rule of sketch 25 §3.1 and target design §4.5: the `SwipeAt` in `state_before_last`, or `{last.id, 0}` when the story holds only its opening. `tail_tests.rs` pins both branches in their own tests (`start_at_names_the_active_swipe_of_the_message_before_the_last_one`, `start_at_names_the_first_swipe_of_the_opening`); the case that pinned the wrong rule is gone. |
| 2 | The v9 `CREATE TABLE message_swipes` says `location_header` again, so a fresh database runs the v27 rename. The earlier claim that the rename is exercised on a fresh database was **wrong**: the comment pass had changed the v9 DDL, and v27's `column_exists(location_header)` guard then skipped the rename. It runs now. |
| 3 | `Tx::clear_dock` writes `DockRecord::default()` through the same upsert as `put_dock`, so a Game keeps its one dock record with an empty offered set and no notice (target design §2.1 and D-15). `story-dock-clear` asserts the row stays and equals the default record. |
| 4 | `GameReads` and `Storage::games()` are gone: the one-method handle only forwarded to `get_game`, which already answers the Game with its failure line. The seam cases read through `storage.get_game`. `games.rs` is back to one `impl Storage` block. |
| 5 | `Tx` holds `game: GameId`, not an `Option`. The create unit writes the Game row first (`transaction_new_game`, with the `InsertGame` injection check before it) and then builds the `Tx` that holds its id; there is no `insert_game` method, so `transaction(game, ..)` cannot write another Game's row, and the missing-Game error path is gone. `delete_game()` takes no id and removes the unit's Game. **Reset is therefore two units**: the old Game goes in its own unit, and the new one is created in the create unit; `GameWriter::replace` (C4) sequences them under the guard. Sketch 25 §3.1's `games().remove` implies exactly this (a unit's `Tx` serves one Game), and target design §3.2's "a reset first removes the active Game with all its records" is its first unit. `set_active_swipe` and `delete_message` now call `require_message_of_game`, so they answer `MessageNotFound` for another Game's Message like the other writes. New case `story-write-other-game-message [failure]`. |
| 6 | `story_storage.rs`: the tautology `assert_eq!(message_id, message_id, ..)` is gone; `tail_answers_the_state_before_the_last_message_with_its_swipe` no longer calls `Tail::current_state()`; the state/room-report pair, the failure set/clear pair and the entry/kind cases are split or trimmed as asked; `an_entry_carries_the_active_swipe_and_its_headers` asserts the room report it promises; `the_new_records_refuse_the_in_memory_backend` no longer repeats P5's transaction clause; one `swipe(text)` helper (with `narration_swipe` and a functional update) replaces the repeated literal. 21 cases, one clause each. |
| 7 | `storage/utils/records.rs` owns two generic helpers, `to_json(label, value)` and `from_json(label, json)`, which replaced the three per-type wrappers and every hand-written `match … serde_json` in the new code paths; the label keeps the record name in the error. The same file owns `require_message_of_game`, now shared by `Tx` (5 writes) and `StoryReads`. |
| 8 | `DbSwipe.maker` is `maker_json`, and `models/message_tests.rs` writes valid JSON (`'{"movement":{}}'`, `'"Narrator"'`) through a raw string. The step rename is finished: `InputBuffer.phase` → `InputBuffer.step`, `DebugStateView.generation_phase` → `step` (so `/debug/state` answers `"step"`), and `game_flow.md`'s two headings and one sentence say step. `set_active_swipe` takes a `SwipeAt`. **Decision:** the Snapshot's `narrative` JSON column carries `input_buffer.step` from now on, so an old Snapshot row's `phase` key is ignored and the step defaults to `Narrating`; C6 deletes the saved Games (OR-01) and the Snapshot records, so no reader depends on the old key. |
| 9 | `stories.rs`: `tail` reads a `COUNT(*)` and the two newest Message ids (`ORDER BY id DESC LIMIT 2` in a subquery, oldest first) and uses `ids.iter().rev().nth(1)`; `entries` fills a row type directly through the new `FromSql` for `MessageType` and `MessageKind`, so no tuple and no manual parse remain. The story read's row type is `StoryRow` in `domain/model/state/message_types.rs` (a domain type, because the server scope may not import storage, and C4's story log needs it). P4's `StoryEntry` is back to `{message_type, text}` and the four Agent fixtures no longer fill id/kind/etc.; `From<&StoryRow> for StoryEntry` is the conversion the hook input uses from C4. No `From<&MessageEntry> for StoryRow` bridge was added (nothing in C1 needs it). |
| 10 | The doc line on `persist_swipes` is back: "Per-swipe errors are logged and swallowed." |
| 11 | `tail_tests.rs` has one branch per test, each named after its rule (13 tests). `retrigger_allowed` now covers a Narration, an Impersonate Input, an event Narration, a `TypedInput` with a trigger context, a Message with no kind, no trigger context and no current state; `start_at` covers both branches; `current_state` and `can_delete` each cover theirs. |

Ticket corrections.

- **The branch.** The earlier comment's "no branch was created" read as a claim that no branch exists. The truth: the work is on `wf/rebuild-core`, the branch cut from `main` `f64f5d41`; this assignment created no branch of its own and made no commit.
- **The migration claim** (item 2): the v9 DDL and the v27 rename are as above; the earlier "a fresh database creates `location_header` at v9 and renames it at v27" was true of the intended code and false of the tree at the time.
- **`/debug/state`.** `narration_history_tail[]` entries are `MessageEntry`, so their JSON key `location_header` is now `room_report`. Accepted: no spec, no test and no JavaScript reads it (`tests/http/requires_migration/debug.rs` only asserts that the key `narration_history_tail` exists).

Verification. `python scripts/validate_seam_cases.py` → 5 contracts, 86 cases (`story_storage.rs` 21, `transaction_storage.rs` 8, the three P4 contracts 57). Full gate `python build.py` green at `logs/build_20261018_150006.log` (tree `f64f5d4157+54a29e6c`, 150.0 s): fmt, data, clippy, test structure, spec-coverage, seam-cases, Python, freshness, docs, guardrails 189 passed, integration 1657 passed + 7 skipped (1664), browser 29 passed. The three gates before that are green with the same counts: `logs/build_20261018_145223.log` (182.2 s), `logs/build_20261018_144806.log` (189.9 s) and the C1 gate. Focused runs: clippy `logs/build_20261018_145640.log`, storage `logs/build_20261018_144610.log` (128 passed), lib `logs/build_20261018_144717.log` (1248 passed), guardrails `logs/build_20261018_144739.log` (189 passed). The comments the earlier pass trimmed stayed trimmed: this pass wrote docs only where it changed or added code. `AGENTS.md`, `tests/AGENTS.md` and `guardrails.md` regenerated (not staged).

Not done, with the reason. The commit gate (`/simple-after-plan`) and the owner's approval stay the commit step, which this assignment excludes. The rename's data preservation still has no test that starts from a v26 file on disk (SQLite `RENAME COLUMN` keeps the rows by definition, and C6 deletes those rows); the 5 `#[ignore]`d target tests in `races.rs` are still not re-run, because the direct `cargo nextest --run-ignored` call falls outside the build slot and C5 owns that check.

**2026-10-18, implementer (C2, uncommitted).** Branch `wf/rebuild-core`, base `622d5a5e` (core C1). Area 2, next to today's `GenerationGate`: `GameWriter` and `JobWriter`, the value types their signatures need, `TxToken` on the storage entries, and the three guardrail rows. No commit was made.

Facts measured.

| Item | Result |
|---|---|
| `application/generation/gate.rs` | Today's `GenerationGate` stays where the plan puts the new code ("next to today's `GenerationGate`"), and the new section follows it: 860 lines, so the file is 1078 lines. **Where the old gate goes:** it stays in `gate.rs` for C2 to C5, untouched, with its `RwLock` registry and the test-only `JobTasks` tracker; the new code never calls it, and rule 18's row counts `hold` calls, which the old gate makes none of. The tracker moves to the Job runner in C3, as the ticket 07 note says. **Size:** the sketch guessed 320 lines for the whole file; the new section is above that because it holds the save units, the accept checks and the create unit, which the sketch counted in `job.rs` or not at all. |
| `GameWriter` | `storage: Arc<Storage>`, `entries: Arc<Guard>`, `next_job_id: Arc<AtomicU64>` (a `SeqCst` counter; a weakly ordered atomic is a rule-18 re-look trigger). Methods: `accept`, `busy_step`, `stop`, `switch_swipe`, `edit`, `delete_last`, `create`, `replace`. `accept` is the busy check, the `400` checks, the `500` checks, the job entry and the accept unit, all inside one `hold`; the `400` checks are `/options` needs text, a Retry needs a last Message, a Retrigger needs a Trigger; the `500` checks read the Game and its Persona and write the missing-data failure line. `switch_swipe`, `edit` and `delete_last` check busy under the guard (rule 5's compiler half; its test is I6). `create` and `replace` share one create unit (the dock, the opening Message with the World's opening text, the starting state and the starting room as its room report); a reset removes the old Game in one unit and creates the new one in a second, both inside one hold, so no request slips between them. |
| `JobWriter` | Private `writer: GameWriter`, `game: GameId`, `id: u64`; only `accept` makes one; `Drop` removes the entry if it still holds this id. `hold` checks the entry's id under the guard and runs the closure with it. Save units: `save_input`, `save_text` (returns `SavedText`), `save_state_update` (takes `SavedText` by value, so move semantics forbid a second update), `save_failure`, `save_options`, `step` (memory only), `game`. |
| `Guard` | One `Mutex<HashMap<GameId, JobEntry>>` behind a small private type whose only method is `hold`, so Rust privacy keeps the lock in `gate.rs` (rule 18's compiler half). `hold` runs its closure while the guard is held, and the closure opens the unit it needs; the lock order stays guard, then backend mutex, then connection mutex. |
| `TxToken` | `storage/transaction.rs`, `pub struct TxToken(())` with a `pub(crate) fn new()`. `Storage::transaction` and `transaction_new_game` take it, so a unit opens where `gate.rs` opens it; `src/test_support/fixtures.rs::tx_token()` hands one to the storage seam contract, whose crate cannot call a crate-only constructor. The 22 seam-case call sites changed in place, mechanically. |
| Guardrail rows | Rule 3 (call-site: only `application/generation/gate.rs` and `test_support/` call `TxToken::new`), rule 6 (banned name in `application/generation/gate.rs`: `Narrator` and `Agent`), rule 18 (call-count: at most one `hold` call per function of `application/generation/gate.rs`, with the re-look triggers in the row's comment). `walker_tests.rs` gained one detection case that mirrors each row, and `guardrails.md` is regenerated. The machinery already had the call-count kind, so no walker was added. |
| Domain support | `Tail::current_at` (a new accessor: a generation of a new Message starts from the active Swipe of the last Message), `Tail::redo` (the retry table of design §4.5: the opening starts from its first Swipe with no stored inputs, any other Message from the state before it with its active Swipe's stored inputs, an event Narration carries its event record, the kind follows the last Message's kind), `Tail::retrigger` (a new event Message from the current state; the step fills the event record's direction from the World), and `SwipeState::starting(scenario, characters, room)` (the starting room, no added rooms, each starting Character met once and met now). |
| Tests | `tail_tests.rs` gained 10 cases for `current_at`, `redo` and `retrigger`; new `domain/model/state/swipe_state_tests.rs` has 3 cases for `starting`. `--test-target lib` 1261 passed (was 1248). `--test-target storage` 128 passed. Full gate: integration 1670 passed + 7 skipped (1677), guardrails 189, browser 29, seam cases 5 contracts / 86 cases. |

Decisions, with the reason.

1. **All of C2's new types live in `gate.rs`**: `Refusal`, `Stopped`, `JobRequest`, `JobPlan`, `OptionsRun`, `OptionsResult`, `SavedText`. The plan's C2 row names exactly these as the types Area 2's signatures need, and C3's `job.rs` imports them; the sketch draws `JobPlan` in `pipeline/job.rs`, which C3 can move with one edit if the lead prefers that home.
2. **`Refusal` is an application type, so `Tail::redo` and `Tail::retrigger` return `Option<GenerationOrder>`**, not the sketch's `Result<GenerationOrder, Refusal>`. `Tail` is a domain type and arch-lint denies `model → application`; the client-facing refusal text stays in `accept` ("No input to retry", "No trigger context available"), where today's texts live. The alternative was `Refusal` in `domain/model/`, which would put 200/400/500 language in the domain.
3. **`hold` takes `f: FnOnce(&mut HashMap<GameId, JobEntry>) -> T`**, not the plan's `|entries, tx| ..`: the closure opens the unit it needs, because a reset is two units inside one hold (C1's `Tx` split: a unit serves one Game, and only the create unit writes a Game row) and a step update needs no unit at all. Rule 18's row is unchanged (at most one `hold` call per function), and holding the guard across both units of a reset is what keeps an Action on the old Game out of the gap that rule's reason names.
4. **`Stopped` is `{ EntryMoved, SaveFailed(EngineError) }`.** The sketch names the type only; a storage failure must reach the Job, because design §4.7 gives the save failure its own line, and `Stopped` is the only answer a save unit gives.
5. **`OptionsResult` is `{ Offered(Vec<String>), Unavailable { reason, raw } }`.** The sketch names the type in `save_options` and never defines it; the two arms are exactly design §3.2's Options save table (offered set, or a notice with the reason and its raw text).
6. **`Tail::current_at` is new** (the sketch has `start_at` only). The C1 ruling made `start_at` the redo start; a generation of a new Message starts from the current state, which is the active Swipe of the last Message, so the writer needs the SwipeAt of it.
7. **`SwipeState::starting` takes `Option<&StartingScenario>`**: a World with no scenario has no starting Characters, and R-77 keeps the opening Message with empty text.
8. **The `Kind` mappings moved to the domain** (`Kind::message_kind`, `message_type`, `maker` in `agent.rs`): the mapping is what a generation of a kind writes, and naming `Maker::Narrator` in `gate.rs` tripped rule 6's banned-name row, whose purpose is that no model role is reachable from the writer file. **Gap:** those three methods have no unit test, because `agent.rs`'s test file is legacy and the one rule's list only shrinks.

Gaps and risks.

- **C2's writer bodies have no test until C4.** §9.4 places `gate.rs` at tier 1 and gives it no unit tests, and nothing calls `GameWriter` from production before C4, so C2's writer code is exercised by the compiler (private fields, the token, `SavedText` by value), the three guardrail rows, and the 13 pure-domain cases above. That is §9.3's order (C1 to C3 land next to today's code, C4 switches the routes, C5 reworks tier 1), and it is the price of the split; the C4 and C5 reviews must check the accept checks and the save units against design §3.2, §4.1 and §4.5, because no test does until then.
- **C4 will break the legacy in-memory create tests.** `GameWriter::create` uses `transaction_new_game`, which is SQLite only (no in-memory arm for the new records). The legacy lib tests that reach the create path on `Storage::new_in_memory()` — `application/games/catalogue_tests.rs` (4 storages) and anything else that calls `GameCatalogue::create_game` — fail at C4 unless they move to SQLite `:memory:`. Ticket 16 gap 1 option A said the legacy in-memory tests stay as they are, which held while today's Game writes stayed off `Tx`; C4 must move the create-path tests.
- **`edit` reads the active Swipe index through `Storage::require_active_swipe_index`**, today's read, which C6 replaces with a story read. The write itself is the new unit.
- **`/options`' `400` check reads the whole story's entries** (R-27 is a question about every Message) inside the accept hold. One query, no Swipe states; C4 may give it a narrower read if the story log's cost shows.
- **`missing-data` uses `GenerationFailureKind::Other`.** §4.7 gives the missing-data line its own kind, which `GenerationFailureKind` does not have yet; C6 can add the variant.
- **Rule 6's banned names are `Narrator` and `Agent`** (prefixes), not the four names the plan lists: `AnalysisAgent`, `OptionsAgent` and `AgentRunner` all start with `Agent`, and a one-line `banned:` list cannot carry four full paths — rustfmt wraps it past `max_width`, which `generate_guardrails_doc.py` rejects. `Agent` is the broader ban and covers any later Agent type.
- **Ticket correction.** The earlier comment says "no branch was created"; the work is on `wf/rebuild-core`.

Validation. `python scripts/validate_seam_cases.py` → 5 contracts, 86 cases. Full gate `python build.py` green at `logs/build_20261018_154401.log` (tree `622d5a5e70+9adc65bc`, 167.8 s); the gate before the last two edits is green too (`logs/build_20261018_153509.log`, 176.1 s): fmt, data, clippy, test structure, spec-coverage, seam-cases, Python, freshness, docs, guardrails 189 passed, integration 1670 passed + 7 skipped (1677), browser 29 passed. Focused runs: clippy `logs/build_20261018_153915.log`, guardrails `logs/build_20261018_153833.log` (189), storage `logs/build_20261018_153112.log` (128), lib `logs/build_20261018_153121.log` (1261). `AGENTS.md`, `tests/AGENTS.md` and `guardrails.md` regenerated (not staged). One full gate failed on the Python test of the guardrails-doc row form (`logs/build_20261018_153259.log`) because rustfmt had wrapped the rule-6 `banned` list; the list now uses short imports and stays on one line.

Not done, with the reason. The commit gate (`/simple-after-plan`) and the owner's approval are the commit step. Rules 4, 5 and 15 are held by the compiler and need no row; rule 5's test is I6 (C4). The 5 ignored target tests in `races.rs` are still not re-run (C5 owns that check).

**2026-10-18, implementer (C2, fix pass, uncommitted).** The logic review's items 1 to 10, the shape decision, the rule-6 row and the standards items are applied. No commit was made.

Each number, to its result.

| # | Result |
|---|---|
| 1 | `replace` removes the old Game's job entry under the same hold and cancels its call, so the old Job's next save unit answers `Stopped::EntryMoved` and its model call stops. |
| 2 | `accept` writes the accept unit **before** it inserts the entry, so a refused unit makes no entry and a panic in the unit cannot leave a half-accepted Game for the poison recovery. |
| 3 | `SavedText` is `#[derive(Debug, PartialEq, Eq)]`: no `Clone`, no `Copy`, so `save_state_update` takes it once. |
| 4 | `accept` runs busy, then `plan_for` (the `400` checks), then `require_job_data` (the `500` checks), then the unit and the entry. A Retry with nothing to retry on a Game with a missing Persona answers the `400`. |
| 5 | Every `500` refusal writes the failure line through one helper, `missing_data_with_line`: the tail read, `/options`' text read, the Game, the Persona, the start state, the impersonate direction, the trigger context and the Trigger, the accept unit, and the storage errors of switch, edit and delete. A `400` writes none: an edit of a Message the story does not hold answers `BadRequest("Message not found")`, which is what today's path answers as an internal error and the design gives no line for. |
| 6 | `GenerationFailureKind::MissingData` carries "The data for this request is missing: <what>."; `generation_failure_summary` now takes `(kind, raw)` and renders that line from the raw text, and the three call sites and the browser stub follow. No spec asserts a line this changes: `docs/specs/failure_display.md` names the missing-preset line and the unnamed-failure line only, and both are unchanged. |
| 7 | `save_text` writes the opening's room report: a new Swipe on a story of one Message resolves the starting room's name from the World's map. The state update of that Swipe must not overwrite it with none; the event step (C3) passes the room the rules moved to, and the opening keeps the starting room. |
| 8 | The `500` checks of a Retry read the start Swipe state and, for an Impersonate retry, the direction its stored inputs carry. A Retrigger's plan carries the event record: the current state's trigger context, with the narration direction the World's Trigger holds (a `500` when the World has no such Trigger). |
| 9 | `JobWriter::cancelled()` hands the Job its entry's cancel handle, so the Job observes the token and a stop has a reader. |
| 10 | `JobPlan::first_step()` reads the plan: a first order of the event kind starts at `generating-event`, everything else that narrates at `narrating`, and `/options` at `options`. |
| Shape | The lead's decision is in: `JobRegistry::hold` is the only place that locks **and** the only place that builds a `TxToken`; it hands the closure the entries and a `Units` opener, and the closure opens the units it needs. Every unit therefore opens inside a hold (the compiler guarantee the plan draws), a reset opens two units in one hold, and a step update opens none. `TxToken` derives `Clone`, because one hold hands a clone to each unit it opens; `TxToken::new` is called in `JobRegistry::hold` and in `test_support` only, so rule 3's row is unchanged. My earlier shape (the closure calling `Storage::transaction` itself) left the token reachable from any closure and is gone. |
| Rule 6 | The row bans `NamePrefix("Narrator")` and `NameContains("Agent")`, so it covers `AnalysisAgent`, `OptionsAgent` and `AgentRunner` and any later Agent type, and it stays on one line for `generate_guardrails_doc.py`. The banned-name walker gained the `NameContains` kind and a trait-bound walk (`Box<dyn AnalysisAgent>`: a bound's path is not a type path, so the type and import walks never reached it). The receiver form `self.narrator.narrate()` needs no walk of its own: the field that holds it names the type or the trait, which the walks catch. Detection cases: the rule-6 case now mirrors the row over four files, plus one case for `NameContains` and one for the trait bound. |
| Standards | `gate.rs`'s imports are std, external, crate. The import-ordering guardrail no longer stops at the first multi-line `use …::{}` (it tracks the statement to its `;`), with two detection cases; the repaired check then flagged `tests/http/requires_migration/fragment.rs`, whose `chronicler_engine::test_support` import followed a `crate::` one, now moved up. One `require_idle` helper holds the busy check of the four short writers, and one `JobWriter::unit` holds the six unit-opening chains. `/options`' text check is a real `EXISTS` read (`StoryReads::has_text`, one query, no Swipe state, no parameter), with two seam cases (`story-has-text`, `story-has-text-none`). The guard is `JobRegistry`, which cannot be read as `GenerationGuard`. `gate.rs`'s summary names both today's gate and the new writers. |
| Decision 3 in place | The `Kind` mappings moved from `agent.rs` to `domain/model/utils/generation_record.rs` as free functions (`message_kind`, `message_type`, `maker`) with 13 plain unit tests: a `_tests.rs` file must pair with a source file of the same name, and `agent.rs`'s test file is legacy, so a new source file was the only way to test them under the one rule. |

Corrections to the earlier comment.

- **`gate.rs`'s size.** The C2 comment said the new section is 860 lines and the file 1078. After this pass the file is **1224 lines**: today's gate is 186 code lines, and the new section is 1002 lines of which 796 are code. The sketch guessed 320 lines for the whole file; the new section holds the seven value types, the registry and its unit opener, the plan builder, the `400` and `500` checks, the create and reset units, and the six save units, so the guess is well exceeded. The counts come from `rustfmt`'s tall formatting and the `Result` and `match` shapes, not from duplicated code (no item is defined twice).
- **Rule 6's claim.** The C2 comment said the row bans `Narrator` and `Agent` as prefixes and that the four full names do not fit on one line. The row now bans `Narrator` by prefix and any `*Agent*` name by containment, and the walker reaches trait bounds, so the four names are enforced with two entries.

Validation. `python scripts/validate_seam_cases.py` → 5 contracts, 88 cases (two new: `story-has-text`, `story-has-text-none`). Full gate `python build.py` green at `logs/build_20261018_162305.log` (tree `622d5a5e70+5bea2024`, 184.2 s): fmt, data, clippy, test structure, spec-coverage, seam-cases, Python, freshness, docs, guardrails 191 passed, integration 1685 passed + 7 skipped (1692), browser 29 passed. Focused runs: clippy `logs/build_20261018_162136.log`, guardrails `logs/build_20261018_162157.log` (191 passed, two new style cases), storage `logs/build_20261018_162239.log` (130 passed), lib `logs/build_20261018_162216.log` (1274 passed, 13 new mapping cases), browser `logs/build_20261018_161858.log` (29 passed). One full gate failed in the browser tier (`logs/build_20261018_161525.log`): the clamp check rendered the missing-data line with the stub's long raw text, and the line then needed two lines; the stub now uses "the story", which a real failure carries.

**2026-10-18, implementer (C3, uncommitted).** Branch `wf/rebuild-core`, base `fcbe318f` (core C2). Area 3: `job.rs` with the runner, `generate`, the step functions and `run_step`, next to today's pipeline; no route reaches it until C4. No commit was made.

Facts measured.

| Item | Result |
|---|---|
| `application/pipeline/job.rs` (new) | 841 lines: `JobTasks`, `Started`, `Generated`, `Failure` and `Job` with `start`, `run`, `generate`, the step functions (`room_check`, `narrate_step`, `analyse_step`, `apply_rules`, `options_step`), `run_step`, the value builders and the reads. No unit tests (§9.4 places `job.rs` at tier 1). |
| The runner | `Job::start` accepts through `GameWriter::accept`, records the game, the job id, the kind and the first step in its span, spawns the plan on the blocking pool and registers the task in the Job watch. `Job::run` follows the request table: the first generation (with the request's Input text), the event generation when the game rules found a Trigger, then the Options run. `JobWriter` drops at the end, which frees the entry if it still holds this Job's id. |
| The steps, against §4.3 | 1. `room_check`: the start state's room on the map, with its added rooms; else the invalid-room line and nothing saved. 2. `narrate_step`: the preset of the kind, the room, the persona, the story before the place, the stored inputs, the event record for an event, one `Narrator::narrate` call. 3. `JobWriter::save_text` with the pre-analysis state (the start state for a user regen) and, for a Swipe of the opening, the room report the text save knows. 4. `analyse_step`: the analysis hook with the new text, the player's text, the room, the Characters and the 4-Message window. 5. `apply_rules`: `apply_game_rules` on the start state, `Analysis` for a Narration/Impersonate (movement, Characters, Trigger) and `NpcEvents` + `TriggerNarration` for an event (no movement, no Trigger from the rules, the fired mark, then no trigger context). 6. `JobWriter::save_state_update` with the state, the room report and the trigger context of the Trigger that fired. A user regen returns after step 3. `run_step` writes the status name before the step runs, and a stop is observed before each step. |
| §4.2's order and R-31 | The room check runs **before** the Input save, so an invalid room saves no Input (R-20). A step that fails ends the Job, and the Options run still runs when a Narration was saved (R-31, its example is the failed event Narration), then the failure line lands. An Impersonate generation saves an Input, so only its event Narration turns the Options run on. |
| The failure lines | `GenerationFailureKind` gained `Contradiction` and `EventNarrationFailed`, and `SceneMissing` now renders the design's line "The room <name> is not on the map." (the room name travels in `raw`). The runner writes the line for every failure: the step's own failure, a failed save (the save-failed line, best effort) and the event generation's failure (the event line, with the failed step's kind and raw text inside it). A moved entry writes no line. The Job logs each failure and each refused write at `warn`. |
| The tracker | `JobTasks` moved from today's `GenerationGate` in `gate.rs` to `job.rs` (its spawn site). `GenerationGate` holds an `Arc<JobTasks>` and keeps `watch_job` and `wait_for_jobs` working for the pipeline and the tier-1 helper, and gains `job_tasks()` for C4, which will hand the same instance to `job::start`. The `#[cfg(feature = "testing")]` attributes on the field and the two methods went: the type is always built, and `job::start` takes `Option<Arc<JobTasks>>`, so one watch serves both spawn sites. |
| Rule 7's row | `TypeCheckRow { rule: 7, scope: "application/pipeline/job.rs", banned_types: JOB_RECORD_TYPES, position: Fields }` in `walker_rows.rs`, with the six banned names (`Game`, `Message`, `SwipeState`, `Tail`, `DockRecord`, `WorldBundle`) in one named const, because the six names fit no single row field line and two rows of one rule break the registry's "one rule number once" test. `walker_tests.rs` gained the mirroring detection case (two files, two hits). |
| Domain support | `SwipeState::pre_analysis(event)` (the start state with no trigger context, a one-shot Trigger marked fired); `From<&TriggerMatch>` for `TriggerRef` and `EventRecord`; `GameRuleRequest::TriggerNarration` now takes a `TriggerRef` instead of a `StoredTriggerContext` (the design's record carries no assembled prompt), with today's two call sites converted; `StoryRow` gained the `timestamp` and the `active_swipe_index`, and `From<&StoryRow> for MessageEntry`, so the narrator's prompt context keeps the type P4's seam contracts use. |
| Tests | New plain domain tests: `pre_analysis` (3 cases in `state/swipe_state_tests.rs`) and the two `TriggerMatch` conversions (`state/trigger_context_tests.rs`, 2 cases). `--test-target lib` 1279 passed; `--test-target http` 277 passed + 5 skipped (tier 1 stays green); guardrails 191 passed; storage 130; seam cases 5 contracts, 88 cases. |

Decisions, with the reason.

1. **One `Narrator` call per narration, whatever the kind.** The design's "what each call reads" table has one Narrator row, and §4.3's step 2 is the narrator call for every kind, so an event Narration runs `Narrator::narrate` with the event record as its input, and its prompt input is built at call time from the Trigger's direction and the narration it continues (today's continuation text, moved from the trigger request to the call). **Consequence:** the event narration's call is recorded under the `narrator` role, so the dashboard's "trigger" role health stops receiving calls. P5's R-59 case reads the call by its answer, not by its role, so no test breaks; C4 or C6 must decide whether the trigger role keeps a call of its own or the banner loses the row.
2. **A Job-local `Failure`.** The steps answer `Stopped` (a refused save) or a `GenerationFailure` (the line to write), and the runner is the one place that writes the line, so every failure path writes exactly one.
3. **The event generation applies the Characters only.** `apply_game_rules(NpcEvents)` with the events computed from the start state's present Characters against the analysis's ids, then `TriggerNarration` for the fired mark, then `trigger_context = None`; the outcome's Trigger is ignored, which is the "Event" mode of the design's table.
4. **The room report of a Swipe**: the room the rules moved the player to; the opening keeps its starting room when nothing moved (the text save wrote it); an event Narration shows none.
5. **The 500 checks of a Retry and a Retrigger live in `accept`** (C2); a *step* whose read fails writes the missing-data line with the error text (R-46's data-integrity failure).
6. **`pre_analysis` marks the event's one-shot Trigger fired** on the pre-analysis state, because §1's definition says so and the Swipe between the text save and the state update must already count it.

Gaps and risks.

- **The "empty text" line is not reachable.** P4's `Narrator` fails an empty answer as `UnreadableAnswer` ("The language model's answer could not be read."), where §4.7 gives empty text its own line. Splitting it needs a `Narrator` change and a P4 seam-contract case; C6 or the owner decides.
- **A stop cannot cancel a call that is already running.** `Narrator::narrate` takes no cancel handle (P4's shape), so the Job observes the token before each step and the job id check refuses the later saves; the design's "cancel the LLM call that runs, when its connection can cancel it" is not reachable yet.
- **The event failure's disclosure holds the failed step's kind and raw text**, not its summary line: the summary lines live in the HTTP layer and the application cannot reach them.
- **`/options`' text check** is C2's `StoryReads::has_text` (one `EXISTS` query).
- **C3's code has no caller until C4**, so the compiler, rule 7's row and the five domain cases above are its only checks before then (§9.3's order); C4/C5 must check the step bodies and the failure lines against §4.2 to §4.7.
- **The `SceneMissing` line changed** for today's pipeline too (the old "The current scene could not be found." is gone). No test and no spec asserts the old words; `swipe_new.md` 11.7 now reads exactly as the spec asks. The rest of §4.7's re-wording (the "turn" lines, the internal-error line) stays for C6.
- **`GenerationGate::job_tasks()`** is unused until C4.

Validation. `python scripts/validate_seam_cases.py` → 5 contracts, 88 cases. Full gate `python build.py` green at `logs/build_20261018_165146.log` (tree `fcbe318f7f+cfb4ab79`, 167.4 s); the gate before the rename is green too (`logs/build_20261018_164737.log`, 196.1 s): fmt, data, clippy, test structure, spec-coverage, seam-cases, Python, freshness, docs, guardrails 191 passed, integration 1690 passed + 7 skipped (1697), browser 29 passed. Focused runs: clippy `logs/build_20261018_165108.log`, guardrails `logs/build_20261018_164617.log` (191), lib `logs/build_20261018_164631.log` (1279), http `logs/build_20261018_165121.log` (277 passed + 5 skipped), storage `logs/build_20261018_164326.log` (130). `AGENTS.md`, `tests/AGENTS.md` and `guardrails.md` regenerated (not staged).

**2026-10-18, implementer (C3, fix pass, uncommitted).** The logic review's items 1 to 9 and the accepted smells are applied. No commit was made.

Each number, to its result.

| # | Result |
|---|---|
| 1 | The room line names the room: `narration_generation.rs` passes the current room id as the failure's raw text, and `from_engine_error` returns `SceneMissing` with the room id for `EngineError::RoomNotFound`. The two tests that asserted "Room not found" now assert the room the line names — `actions.rs` (1.1's invalid-room case) and `swipe_new.rs` (11.7) — and three legacy lib tests assert the same. 11.7 now fails if the line stops naming the room. |
| 2 | `save_input` writes `MessageType::Input` and `input_swipe` takes `Maker::Player`; the opening Message's Swipe takes `Maker::WorldOpening`. |
| 3 | `Narrator::narrate` takes a `Role` and records the call under `role.agent_name()`, so an event Narration feeds the Trigger role again and the fake sees `AGENT_TRIGGER` (`ModelRole::Event`). `Kind::role()` names the role. The Narrator seam contract's entry gains the argument, and a new case (`narrator-role`) pins that the call record carries the role the caller names; its seven other cases keep one clause each. |
| 4 | The analysis window is the four Messages **before** the new text: it ends at the Message the text save made (`window_before(Some(at.message), STORY_WINDOW)`), for a new Message and for a Swipe alike. `player_text` is the Job's Input, else the Message the new text follows — never empty while the story holds one. |
| 5 | R-31: `generate` answers `Ended { failure, narration_saved }` — a failure after the text save carries the Narration it landed — so the Options run follows a failed analysis, a contradiction or a failed state update when a Narration was saved. A `Stopped` failure (a stop or a reset) runs no Options and logs no second line. |
| 6 | The Options window is the four newest Messages, the Narration the run serves among them. |
| 7 | A user regen's prompt is the last Message's active Swipe text, read at call time. |
| 8 | The event failure's raw text is the narrator failure's **line** (`GenerationFailureKind::line`), not `kind: raw`. |
| 9 | `Job::start` skips `request` (it holds the player's text) and the spawned blocking work runs inside the start span (`span.enter()`), so the steps' spans nest under it. |
| Smells | One room lookup (`resolve_room`) for the check and the prompts; `Failure::missing(what)` for the five copies; `stories.rs` parses the timestamp with the shared helper and maps its error into the row's; the dead `let _ = landed` and the impossible `map_err` are gone; `apply_rules` returns `RuleOutcome { fired, room_report }` and mutates the caller's `SwipeState` through a `&mut` parameter (rule 7 bans fields, not parameters); `Started` is gone and `Job::start` answers `()`; `Kind::step`, `Kind::role` and `Kind::saves_a_narration` carry the one-line kind maps; `Job::start` takes `JobDeps { narrator, agents, storage, tasks }`; `JobTasks` moved to `application/generation/job_tasks.rs`, so `gate.rs` and `job.rs` no longer import each other; `EventRecord::name` is `header_name` with `#[serde(rename = "name")]`, so the stored JSON is unchanged. |

Decisions, with the reason.

1. **The failure lines' words moved into the domain** (`GenerationFailureKind::line`), because §4.7's table is the design's and the event failure has to put the narrator failure's *line* in its raw text, which the application cannot reach while the table lives in the HTTP layer. `http/utils/error.rs::generation_failure_summary` delegates to it, so the templates and the stubs are unchanged.
2. **The whole table moved in one step, so three lines changed** beyond the ones C3 needed: prompt too long, save failed and the internal-error line now read as §4.7 gives them. The three tests that asserted the old internal-error line (`tests/browser/stub/announcements.rs`, `tests/http/failure_display.rs` twice) now assert the new one; the other two lines are asserted nowhere.
3. **`JobTasks` lives in `application/generation/job_tasks.rs`**: the tracker is a generation concern, and the move ends the mutual import between `gate.rs` and `job.rs`. `GenerationGate::job_tasks()` stays because C4's handlers hand that instance to `job::start`, so the pipeline's tasks and the new Job's tasks remain one watch.
4. **`RuleOutcome` instead of a state field**, because rule 7's row bans a `SwipeState` field anywhere in `job.rs` and the row keeps only its field half (the parameter half went in ticket 15 Q3).

Recorded for later.

- The empty-text failure line is unreachable until the Narrator reports an empty answer as its own kind (§4.7); C4 or C5 fixes it with the failure-line scenarios of §9.2.
- A stop does not cancel a running call: acceptable (D-11's "when it can", §7), and the job id check refuses the late save.
- The tracker is built into production because `testing` is a default feature (recorded in P5).

Validation. `python scripts/validate_seam_cases.py` → 5 contracts, 89 cases (the new `narrator-role`). Full gate `python build.py` green at `logs/build_20261018_173334.log` (tree `fcbe318f7f+572fe030`, 191.9 s): fmt, data, clippy, test structure, spec-coverage, seam-cases, Python, freshness, docs, guardrails 191 passed, integration 1691 passed + 7 skipped (1698), browser 29 passed. Focused runs: clippy `logs/build_20261018_172904.log`, guardrails `logs/build_20261018_172932.log` (191), lib `logs/build_20261018_173012.log` (1280), http `logs/build_20261018_173045.log` (277 passed + 5 skipped), storage `logs/build_20261018_173104.log` (130), browser `logs/build_20261018_173116.log` (29). `AGENTS.md`, `tests/AGENTS.md` and `guardrails.md` regenerated (not staged).

**2026-10-18, lead (C3 commit).** Accepted: C3 moved the failure-line table of target design §4.7 into the domain (`GenerationFailureKind::line`), so three lines a player sees today already use the design's words (prompt too long, save failed, internal error). No spec states the old words. Four specs (actions.md, retrigger.md, swipe_new.md ×2) still say "Trigger narration failed"; their tests pass today, and the failure-line scenarios of plan §9.2 rewrite them in C4 or C5.

**2026-10-18, implementer (C4, INCOMPLETE, uncommitted).** Branch `wf/rebuild-core`, base `959ca7d6` (core C3). The switch is largely built, but the reduced gate of §2 is **not green**: `--test-target lib` has 25 red tests, so this assignment does not meet C4's "Done" and must not be committed as it stands. No commit was made.

What is in the tree.

| Item | Result |
|---|---|
| The handlers | `actions.rs` maps the four `Action` kinds onto `JobRequest` and starts the Job; `chat_window.rs`'s retry and retrigger start a Job and its swipe switch calls `GameWriter::switch_swipe`; `history.rs`'s edit and delete call `GameWriter::edit` and `delete_last`; `endpoints.rs`'s status uses the status rule and its stop calls `GameWriter::stop`; `debug.rs` reads `GameViewQuery::is_busy`; the games switch no longer heals. Each handler answers the refusal where it happens. |
| The `Refusal` mapping | `impl IntoResponse for Refusal` in `http/error.rs`: `StillThinking` is `200` with the wait status, `BadRequest` is `400`, `MissingData` is `500`. `ConcurrentGeneration` is no longer returned by any route. |
| `GameViewQuery` | `story_log` (the story reads + `Tail::retrigger_allowed`), `dock` (the dock record), `status` (busy, then the failure line, then idle), `busy_step`, `is_busy`; `get_current_room_view`, `get_npc_headshots`, `get_debug_state`, `get_story_log_entries` and `get_current_options` now read the new records, the tail and the dock. Its `message_service` field went (the old reads are gone). |
| Create and reset | One create unit through `GameWriter::create` and `GameWriter::replace`, called from `GameCatalogue`; the switch keeps `games.get` + `ActiveGame::set` with no heal; the old `persist_initial_state_with_swipes` and the boot heal went. |
| The dock | `OptionsDockViewModel` carries the one notice and `OptionsDockTemplate` renders it, so a failed or missing Options run shows the dock notice instead of a `System` Message. The `System` producers were the old pipeline, which no route calls now. |
| The harness | `test_support::TestAppBuilder` writes its seeded story as the new records (a Message with one Swipe holding the app's starting state); `tests/http/support/http_requests.rs::wait_idle` reads the new busy state; the two dead walkers are re-pointed at `adapters/driving/http/` (`handler return type`, and `server layer boundaries` now bans `SwipeState`), with their unit cases updated; the cookbook TODO above `check_handler_return_type` is retired. |
| Specs (§9.2) | `actions.md`, `retrigger.md`, `swipe_new.md`: the four "Trigger narration failed" lines are "The event narration failed."; 12.1 and 15.1 read "(no Job is accepted)". `options.md`: the four System-message lines are the dock notice. `games.md` 17.1 asserts the new Game's opening Narration. `reset.md` 7.1 reads the World's opening text. `failure_display.md` and `dashboard.md` were checked and needed no edit (the §4.7 lines are not asserted there). |
| The empty-text line | `GenerationFailureKind::EmptyText` with §4.7's line; `Narrator::narrate` reports an empty answer as that kind; its seam case asserts the kind; the browser stub lists it. |
| The reduced gate | clippy green (`logs/build_20261018_175721.log`); guardrails 191 passed (`logs/build_20261018_174740.log`); storage 130 passed (`logs/build_20261018_175153.log`); **lib 1253 passed, 25 failed** (`logs/build_20261018_175737.log`). |

The 25 red lib tests, and why.

| Test | Why it is red |
|---|---|
| `actions_tests`: 14 tests (`test_action_handler_*`, `test_action_confirm_handler_*`, `test_action_check_handler_*`) | They arrange a turn with the old pipeline and assert `200`; the handler now starts a Job, which answers `500` on this app. The Job's accept reads the story the app seeded, and the seeded story and the Job's `JobDeps` need one more harness pass (the accept's missing data names it). **Not fixed**: it needs the same arrange rework C5 gives tier 1. |
| `actions_tests::test_action_handler_returns_503_on_shutdown`, `..._200_still_thinking_on_concurrent_generation` | Their subject went: no route answers `503` for shutdown, and a busy Game is arranged by a real Job. **Delete with C5** (I1 covers the busy answer at tier 1). |
| `debug_tests` (2) | The DTO's arrange seeds the old snapshot; its `current_room` now comes from the tail. |
| `endpoints_tests::test_generating_status_answers_from_live_registry` | Its subject changed: the status no longer reads the old slot registry. **Delete with C5** (the dashboard scenarios cover the status). |
| `games_tests::test_posture_fragment_renders_preset_load_error_span` (1) | Reads the story log, which now renders the new entries. |
| `catalogue_tests::test_reset_propagates_persist_failure` | The injected failure names a storage method the new reset unit no longer calls. |
| `view_query_tests` (4) | They seed the old snapshot and assert the old reads. |
| `gate_tests::test_boot_heal_resets_stale_generating_status` | The boot heal went with the design (D-05: memory is lost, so no Game is busy after a restart). **Delete with C6.** |
| `retry_tests::test_retry_empty_narration_sets_error` | The Narrator's empty answer is now `EmptyText`, so the old assertion on the line fails. |

Red at tier 1 and in the browser tier (§2's red window).

- `--test-target http`: 164 passed, 113 failed, 5 skipped (`logs/build_20261018_175206.log`).
- `--test-target browser`: 25 passed, 4 failed (`logs/build_20261018_175822.log`): `dashboard::test_status_updates_during_generation`, `dashboard::test_confirm_preview_leaves_form_ready_and_usable`, `options::test_options_use_click_submits_rendered_option_text`, `options::test_options_dock_survives_reload` — the old pipeline's status/dock behaviour against the new handlers.
- The ignored target tests (`cargo nextest run --no-fail-fast --test http --run-ignored all -E 'test(races::)'`): 10 run, **0 passed, 10 failed**. Four stop rows report "the Job should reach the held call", one test's Actions answer non-2xx, and the rest assert the design's records against the pre-C5 arrange. C5's rework is what turns them green.

What is missing before C4 can be committed.

1. The lib leg: the harness arrange (the seeded story, the scripted model reaching the Job) and the deletion of the tests whose subject the switch removed.
2. `--test-target lib` green; the http and browser lists above are the commit message's red list.
3. The remaining §9.2 spec edits I could not reach: `dashboard.md`'s status order (checked, no literal asserts it), `actions.md` 2.4's other line, and the `swipe_switch.md` 36.3 wording (already P5's).
4. The verification of the create/reset/stop paths against target design §3.2, §5 in a live run.

**2026-10-18, implementer (C4, verification pass, uncommitted).** Branch `wf/rebuild-core`, base `959ca7d6` (core C3). The previous C4 pass left the switch built and the reduced gate red (lib 1253 passed, 25 failed). This pass takes the reduced gate green and checks the switch live. No commit was made. The tree holds 46 changed/added files.

What each step did.

| Step | Result |
|---|---|
| 1. The harness after the revert | `src/test_support/test_data_builder.rs` is byte-identical to HEAD again (`seed_into` as it was; the only difference was one blank line, restored). `TestAppBuilder::seed_story_records` stays as the previous pass left it. clippy green. |
| 2. Five tests deleted | Each subject is gone (table below). |
| 3. Group 2, the seeded opening | 12 of the 14 red pipeline tests are a count or an index that the opening Narration shifts by one (`+1` Message, `+1` Swipe, the last Narration instead of the first). Two were not a count and are split out below. Each updated assertion says which entry it now names. |
| 4. Group 3 triaged | 2 arrange gaps fixed in place (`view_query_tests` ×4 in two fixtures, `games_tests`, `catalogue_tests`), 2 re-arranged on an empty-story fixture, 0 deleted for it, **1 product defect found and fixed** (below). |
| 5. §9.2 prose | `dashboard.md` 39.1 states the status order (the step of a running Job, then the failure line, then idle) and 39.3 speaks of busy instead of a persisted status and a slot; `actions.md` 2.4 names the disclosure that carries the narrator failure line and says story log. |
| 6. Live check | The dev server, not a tier-1 path: the create, reset and stop paths, an Action end to end, `/options`, and two stops held mid-Job (below). |
| 7. The gate | clippy, guardrails, lib and storage green; http, browser and the ignored `races::` run recorded for the commit message (below). |

The six deleted tests, each with its reason.

| Test | Reason |
|---|---|
| `actions_tests::test_action_handler_returns_200_still_thinking_on_concurrent_generation` | The busy answer is now arranged by a real Job, and the test arranges it with `GenerationGate::try_claim`. §9.4 says this test goes and I1 covers the busy answer at tier 1. |
| `actions_tests::test_action_handler_returns_503_on_shutdown` | The `503` goes with `ConcurrentGeneration` (§9.3's HTTP row); no route answers `503` for a shutdown, and the shutdown token is not read in the accept path. |
| `chat_window_tests::test_retry_handler_returns_400_when_no_input` | A design-valid Game always holds its opening Narration, and a retry reads the story (R-44), so "nothing to retry" cannot happen. |
| `endpoints_tests::test_generating_status_answers_from_live_registry` | The status is the §2.3 rule; it reads no persisted status and no slot registry. The dashboard scenarios cover the rule. |
| `gate_tests::test_boot_heal_resets_stale_generating_status` | The boot heal goes (D-05, D-24): memory is lost, so a restart leaves no busy mark to heal. |
| `retry_tests::test_retry_returns_internal_error_when_no_anchor_message` | The design's opening Narration is always the story's first Message, so `find_retry_anchor_msg`'s event path always finds a preceding non-event Message: the shape cannot exist in a design-valid Game. The same line is covered by `test_retry_no_snapshot` beside it, and `retry_tests.rs` sits at the 2000-non-blank-line guardrail, so a deleted subject is not kept at the price of the limit. |

**The product defect, found by the live check (step 6).** The boot wrote the opening twice and broke the room view.

- `bootstrap/run.rs::prepare_state` still called `init_game::load_game_state`, whose write "is the point". For a Game the boot itself creates, `resolve_boot_game` writes the design's opening through the create unit and `load_game_state` then found no Snapshot and wrote a **second** opening Narration (today's scenario Message, `kind` NULL, no Swipe state) plus a Snapshot.
- Measured on the dev server before the fix: the Game's story log held two openings, and `GET /fragment/visual-sidebar` answered "Room not found: the current state is missing" (`Tail::current_state` reads the last Message's active Swipe state, which the legacy Message has none of). A Retry on that Game would also have fallen into the legacy path.
- The fix: `prepare_state` no longer calls `load_game_state`, and `load_game_state` and `try_persist_initial_message` go with their last caller (the plan's C6 Deletions row names `init_game` down to `resolve_game_id`; the defect pulls that one deletion forward). `PreparedData` drops the four fields that only fed that call, and the boot's own persona check stays. Measured after the fix: one Game, one Message (`kind` Narration), one Swipe (`WorldOpening`, the starting room report, a Swipe state), no Snapshot, and the sidebar renders the room.
- The browser tier confirms it: it was 25 passed / 4 failed, and all 29 pass now (`logs/build_20261018_184222.log`).

Live evidence, on the dev server (`python build.py run -- --port 3113 --world test --persona test_player`, the `data/` world "test" with the OpenRouter connections).

| Check | What the client saw |
|---|---|
| The boot, with no Game for the World | `POST`-free boot creates Game 1: `games` 1 row with no failure line, `game_docks` the one empty record, `messages` 1 row (`kind` "Narration"), `message_swipes` 1 row (`maker` "WorldOpening", `room_report` "Test Tavern", the starting state), no `game_state_snapshots` row. The story log renders one entry, the sidebar renders the room, `GET /status/generating` answers `idle`. |
| A stop while idle | `POST /status/reset-generating` → `200`, body `reset`, the status stays `idle`. |
| `POST /games` (create) | `200` + `HX-Refresh: true`; a new Game with the design's opening (Message kind Narration, Swipe maker WorldOpening, the starting room report, the starting state) and the empty dock record. |
| `POST /reset` | `200` + `HX-Refresh: true`; the active Game and all its records (messages, swipes, dock) are gone, and a new Game with the design's opening stands. |
| An Action, end to end | `200` with the wait status; the story then holds the Input (`kind` TypedInput, maker Player, no room report, the current state), the Narration (maker Narrator), and the event Narration (a `MessageKind::EventNarration` record with the Trigger's name and the direction the World holds). No failure line, status `idle`. |
| `/options` | `200` with the wait status; the dock record holds the offered set and no notice, and no story Message is added. |
| A stop 1.1 s into the narrator call | `reset` at once; the answer that arrives ~30 s later writes **no** Narration, no failure line and no notice: the job-id check refuses the late save (D-08, D-11, §4.4). |
| A stop 0.65 s after the text save (during the analysis) | `reset`; the Narration stays with its **pre-analysis** state (no trigger context, no room report), the state update is refused, and no Options run follows (D-10, §4.4, R-31). |

Group 3, each item's result.

| Test | Result |
|---|---|
| `view_query_tests` ×4 | **Arrange gap.** `minimal_app` and `minimal_app_no_game` were in-memory fixtures of the old snapshot path; the reads are the story reads now. `minimal_app` builds its Game through `TestAppBuilder` with `TestWorld::minimal()` + `TestMap::single_room("start")` (so the assertions keep their room), and `minimal_app_no_game` is `TestAppBuilder` + `skip_seeding(true)` + `active_game(NO_GAME)`. `get_debug_state` needs a current state, so its case moved to `minimal_app` and asserts the opening as the story's one entry. |
| `games_tests::test_posture_fragment_renders_preset_load_error_span` | **Arrange gap.** The fixture's in-memory storage cannot carry the new records the create unit writes; the storage is `empty_sqlite_storage()` now. The injected failure stays the preset-library read. |
| `catalogue_tests::test_reset_propagates_persist_failure` | **Arrange gap, kept.** Measured on SQLite, with the failure injected into `TxWrite::InsertMessage` — a record write the reset's create unit makes — so the test still asserts that a failed reset write surfaces. It was not deleted. |
| `options_tests::test_process_options_empty_history_rejected` | **Arrange gap.** The old pipeline's guard reads the story, and a Game seeded through `TestAppBuilder` always holds its opening, so the shape is arranged on a fixture whose Game holds no Message (`make_test_app(GameState::new(..))`). |
| `options_tests::test_process_options_heals_stale_status_before_validation_error` | **Arrange gap**, the same empty-story fixture, with the stale status in the fixture's own snapshot. |
| `retry_tests::test_retry_target_and_anchor_agree_when_a_system_notice_trails_an_event_narration` | **Mechanical** (`anchor_idx` 1 → 2, and the redo's truncated history now names the opening first). |
| `retry_tests::test_retry_empty_narration_sets_error` | **The design's kind, not a count**: `Narrator::narrate` reports an empty answer as `GenerationFailureKind::EmptyText` (§4.7), so the case asserts that kind. |

The reduced gate, on the **final** tree (after `python build.py fmt`). All four green.

| Step | Log | Result |
|---|---|---|
| `python build.py clippy` | `logs/build_20261018_184935.log` | green |
| `python build.py guardrails` | `logs/build_20261018_185112.log` | 191 passed |
| `python build.py test-pattern "" --test-target lib` | `logs/build_20261018_185250.log` | 1272 passed, 0 failed |
| `python build.py test-pattern "" --test-target storage` | `logs/build_20261018_185317.log` | 130 passed |

Also green on the final tree: `fmt`, `validate-docs`, `spec-coverage`, `seam-cases` (5 contracts, 89 cases), `test-structure`, `validate-data`.

The red window of C4, grouped by cause — the commit message's list. Every group is C5's rework of a tier-1 arrange or expectation; no group is a missing behaviour.

1. **The seeded opening Narration is now in every story** (a count or an index off by one, or the last Narration is no longer the first): `actions::{test_action_succeeds_one_narration_idle_http, test_empty_command_continuation_no_input_http, test_input_persisted_before_narration_http, test_slash_guide_does_not_persist_input_http, test_trigger_continuation_reruns_quantifier_detects_new_npc_http}`, `story_log::{test_delete_last_between_actions_http, test_story_log_icon_buttons_have_accessible_names}`, `swipe_new::{test_retry_replaces_narration_with_new_swipe_http, test_retry_swipe_without_movement_reports_no_room_http, test_event_retry_after_retrigger_adds_a_swipe_http, test_event_retry_replaces_event_narration_with_new_swipe_http}`, `retrigger::{test_retrigger_creates_new_event_narration_no_rollback_http, test_retrigger_does_not_move_the_player_http, test_retrigger_trigger_narration_failure_sets_error_http}`, `races::{test_a_stopped_job_writes_nothing_and_the_next_job_saves_its_own_text, test_action_while_job_runs_is_refused_http, test_the_event_narration_call_uses_the_preset_active_when_it_runs, test_a_stop_at_each_model_call_keeps_the_records_of_that_step}`.
2. **The tier-1 arrange reaches the new Job but scripts or seeds the old path** (the ScriptedModel's call sequence, `try_claim`, `seed_*`, `snapshot`): `actions::{test_nonexistent_room_sets_error_status_http, test_moving_narration_reports_the_room_it_arrives_in_http}`, `settings::test_connection_context_window_applies_from_the_next_narration`, `story_log::test_delete_last_between_actions_http`, `swipe_new::{test_retry_no_input_returns_400_http, test_retry_concurrent_generation_returns_still_thinking_http, test_retry_anchor_no_snapshot_returns_500_http, test_retry_anchor_snapshot_deleted_returns_500_http, test_retry_llm_failure_sets_error_http, test_retry_empty_narration_sets_error_http, test_retry_room_not_found_sets_error_http, test_retry_that_wrote_its_narration_keeps_that_world_http, test_retry_reruns_quantifier_moves_player_http, test_failed_retry_does_not_roll_the_movement_back_http, test_retry_re_impersonate_generates_input_swipe_http, test_retry_re_impersonate_preserves_record_http, test_retry_re_impersonate_uses_current_preset_content_http, test_event_retry_does_not_move_the_player_http, test_event_retry_keeps_the_room_name_off_the_event_swipe_http, test_event_retry_after_a_deleted_action_keeps_its_own_trigger_http, test_event_retry_trigger_narration_failure_sets_error_http}`, `requires_migration::fragment::{test_delete_history_handler_empty, test_edit_history_handler_not_found}`.
3. **The status is the §2.3 rule now** (no persisted status, no slot, no heal): `dashboard::{test_generating_status_answers_from_live_registry, test_the_dock_and_the_debug_endpoint_answer_from_the_live_registry}`, `games_switch::test_switch_game_heals_stale_generating_status`, `requires_migration::{debug::test_debug_is_generating_reflects_state, fragment::test_switch_swipe_handler_concurrent}`.
4. **The dock notice replaces the `System` Message**: `options::{test_always_on_without_an_agent_empties_set_and_reports_http, test_failed_regeneration_keeps_prior_set_http, test_failed_always_on_regeneration_clears_set_and_reports_http, test_on_demand_options_without_an_agent_reports_unavailable_http, test_options_without_scene_history_errors_http}`.
5. **The failure line comes from the Game record now, and one line changed**: `failure_display::{test_failed_poll_answers_non_2xx_and_reswap_none, test_generation_error_clamps_with_raw_text_in_disclosure, test_missing_preset_failure_clamps_with_the_preset_line, test_spent_token_budget_clamps_with_its_own_line}`, `actions::test_trigger_narration_failure_preserves_main_sets_error_http` (the line is "The event narration failed.").

Counts of this pass, on the final tree: `--test-target http` 223 passed, 54 failed, 5 skipped (`logs/build_20261018_185340.log`); `--test-target browser` **29 passed, 0 failed** (`logs/build_20261018_185356.log`); the ignored target tests `cargo nextest run --no-fail-fast --test http --run-ignored all -E 'test(races::)'` → 10 run, **6 passed, 4 failed**, every failure in group 1 above (`tmp/races_run_final.log`).

Open, with the reason.

- **§9.2's "a swipe switch gives back the state of that Swipe" has no scenario yet.** `swipe_switch.md` 36.1 asserts only the status. The behaviour is built (§2.3's current state is the active Swipe's Swipe state), but a new scenario needs its test, and the tier-1 rework is C5's; adding the scenario alone would fail `spec-coverage`. C5 adds 36.4 with its test.
- **The event Narration's `maker` is `Narrator`.** The call record carries the Trigger role (`Kind::role`), but the Swipe's maker does not name the event. No spec states a value; C6 can decide.
- **The boot builds its own `GameWriter`** for `resolve_boot_game`'s create unit, while the app wires another one; `resolve_boot_game` creates a Game and runs no Job, so no second registry serves a request. C6 can hand the boot the app's writer.
- **`load_game_state`'s deletion moved from C6 to C4** (the defect above). The plan's C6 row still lists it; the file is already down to `resolve_game_id`.
- The five `#[ignore]`d `races::` cases and the http list above are C5's rework; the browser tier needed the boot fix and nothing else.

Not done, with the reason. The commit gate (`/simple-after-plan`) and the owner's approval are the commit step, which this assignment excludes.

**2026-10-18, implementer (C4, fix pass on the review, uncommitted).** Branch `wf/rebuild-core`, base `959ca7d6`. The two code reviews' items are applied. No commit was made. The tree holds 55 changed files (the comment pass edited 22 of them first; they were re-read before every edit).

Each number, to its result.

| # | Result |
|---|---|
| 1 | `TestAppBuilder::build_service_with_storage` no longer writes the seeded story through the old write path; `seed_story_records` is the one writer, so no entry is doubled and no blank entry (a Message with no Swipe) reaches the story reads. Every lib count the earlier pass changed is still the opening plus one, or the opening alone: lib 1272 passed. One tier-1 test was arranged on the doubled write — `requires_migration::fragment::test_delete_history_handler_success` (its story held two copies of "Test message", so the delete was allowed) — and its fixture gained a prior Message, a one-line arrange change forced by this fix rather than a rework of its expectation. The three lib `.log(...)` users (`action_tests`, `retrigger_tests`, `endpoints_tests`) pass unchanged. |
| 2 | `OptionsDockTemplate`: the strip renders in the notice's `else` branch, so the dock shows the notice in place of the offered set (D-06, §6). `options.md` 24.5 says the offered set stays and the dock shows the notice and renders no strip. Verified live (below). |
| 3 | See the decision below. |
| 4 | `history_tests`' delete case is `test_delete_history_handler_refuses_the_only_message` and asserts `400 Bad Request` (a Game holding only its opening refuses the delete, R-76). |
| 5 | `options.md` 24.2 now reads as the client's experience: a Game whose story holds no text answers `400 Bad Request` with the body naming "No scene to generate options for yet", and the dock renders no strip and no notice. `swipe_new.md` 22.1 says a re-impersonate runs the analysis: a quantifier that moves the player into "room2" leaves the player in "room2" (the old "the player has not moved" line contradicted §4.5's table, S-2 and R-60, and a client sees the room change). |
| 6 | `games.md` 18.3 (the switch heal), `swipe_new.md` 11.3 and 11.4 (the anchor Snapshot `500`s) are deleted with their tier-1 tests (`games_switch::test_switch_game_heals_stale_generating_status`, `swipe_new::test_retry_anchor_no_snapshot_returns_500_http`, `swipe_new::test_retry_anchor_snapshot_deleted_returns_500_http`). `spec-coverage` is green. |
| 7 | `assets/styles.css` styles `.options-notice`, `.options-notice-line`, `.options-notice-details summary` and `.options-notice-details pre` in the surrounding style: the red tint and border of `.error-message`, the muted small text of `.options-label`, and a `pre` like `.error-detail-raw`. |
| 8 | The two re-pointed guards get the dead-path assertion: `check_src_files_in_scope` fails a path-limited guardrail whose `HTTP_ADAPTER_SCOPE` matches no file, with `dead_scope_violation` naming the rule and the scope (`tests/infrastructure/guardrails/mod.rs`), and `a_dead_scope_names_the_guardrail_and_the_scope` pins its words. `check_server_layer_boundaries` bans `SwipeState` and `GameState` (`WHOLE_STATE_TYPES`), since the wrapper still exists; its unit case for `GameState` is new. Guardrails 193 passed; with the scope set to `nowhere/`, both guards fail with "the scope `nowhere/` matches no file under `src/`, so this path-limited guardrail guards nothing" (`logs/build_20261018_194852.log`). |
| 9 | `GameCatalogue::new` and `GameViewQuery::new` lost the `message_service` parameter and the `let _ =` at every caller (`wiring.rs`, `run.rs`, `test_support/context.rs` twice, `catalogue_tests.rs`, `view_query_tests.rs`). `WiredApp::job_deps` is gone (`AppState::from_wired` builds the one the app uses), and `JobDeps::tasks` is an `Arc<JobTasks>`. |
| 10 | `http/error.rs`'s `Refusal::StillThinking` answers `BUSY_STATUS` (renamed from `CONCURRENT_GENERATION_STATUS`, which named a refusal that no route returns; the stub server uses the new name), and the const carries the restored comment that `dashboard-action-area.js` keys the wait state on `.status.wait`. One `STORY_LIMIT` lives in `storage/stories.rs` and both readers import it. `GameViewQuery::current_state` reports `EngineError::Internal` ("stories_tail: the current state is missing"), because a design-valid Message always holds one — it is a broken record, not a room. The stub server's `/reset` comment is restored, naming scenario 16.34. `PreparedData::player_key` is `persona_key`. |
| 11 | `/debug/state` fills `backend_name` and `model_name` from `state.pipeline.backend_info()`, the provider `/debug/backend` already reports (the Game's record holds no call since D-05). Verified live: `OpenRouter` / `deepseek/deepseek-v4-flash`. |

The reset decision (item 3), recorded as asked.

- **What §3.2 fixes, and what it does not.** §3.2's row says "A reset first removes the active Game with all its records." C1 split the reset into two units (a unit's `Tx` serves one Game and only the create unit writes a Game row), so the row cannot order the two units. Read as the write order of one unit, removal-first would break the invariant the review names: a create unit that refuses after a successful removal leaves `ActiveGame` naming a deleted Game, and `ActiveGame` has no way to name none.
- **The order built:** the create unit first, the removal unit second, both inside the one hold. A refused create leaves the old Game, its job entry and the pointer exactly as they were; a refused removal leaves the old Game standing (a unit lands whole), so the pointer still names a live Game. Measured: `races::test_reset_while_job_runs_keeps_only_the_new_game_opening` passes, and `catalogue_tests::test_reset_propagates_persist_failure` now also asserts that the Game the pointer names still exists.
- **The one window left.** `ActiveGame::set` runs just after the hold, so between the hold's end and the
  pointer's move the old Game is gone while the pointer still names it; a request that captured that
  id in that window answers a refusal. Closing it means handing `replace` the `ActiveGame`, which the
  review did not ask for; recorded here as an open, non-failure window for C6.
- **The name.** `GameWriter::replace` cannot generate the name (it takes a `NewGame`), and the name is picked while the old Game still stands, so `GameCatalogue::reset` filters the Game being replaced out of `list_games()` — the same names today's main code saw after its removal. Measured live: a reset of "Test Realm — 18 Oct 2026 (2)" leaves the fresh Game with the display name "(2)", not "(3)". `catalogue_tests::test_reset_replaces_current_game` pins it.

Live checks of the fix pass, on the dev server (`python build.py run -- --port 3114 --world test --persona test_player`).

| Check | What the client saw |
|---|---|
| Boot and create | One Game, one opening Message (`kind` "Narration"), one Swipe (`maker` "WorldOpening", the starting room report, a Swipe state), one empty dock record, no Snapshot; the story log renders one entry and the sidebar renders the room. |
| Reset | `200` + `HX-Refresh: true`; the replaced Game and all its records are gone, and the fresh Game takes its display name and holds its opening only. |
| The dock notice in place of the set (item 2) | With the dock record holding a notice **and** an offered set of two options, `GET /fragment/options-dock` answers the notice markup once and `options-strip` not at all. |
| `/debug/state` (item 11) | `backend_name` `OpenRouter`, `model_name` `deepseek/deepseek-v4-flash`, matching `GET /debug/backend`. |

The reduced gate, on the final tree. Green.

| Step | Log | Result |
|---|---|---|
| `python build.py clippy` | `logs/build_20261018_193928.log` | green |
| `python build.py guardrails` | `logs/build_20261018_195036.log` | 193 passed |
| `python build.py test-pattern "" --test-target lib` | `logs/build_20261018_194008.log` | 1272 passed, 0 failed |
| `python build.py test-pattern "" --test-target storage` | `logs/build_20261018_194032.log` | 130 passed |
| `python build.py validate-docs`, `spec-coverage`, `seam-cases`, `test-structure`, `guardrails-doc-check` | `logs/build_20261018_1939*.log` | green (`guardrails.md` regenerated) |

The red window after this pass: `--test-target http` 274 tests, 223 passed, **51 failed**, 5 skipped (`logs/build_20261018_194036.log`); `--test-target browser` **29 passed, 0 failed** (`logs/build_20261018_194100.log`); the ignored `races::` run 10 run, **6 passed, 4 failed** (`tmp/races_final2.log`). The commit message groups the 51 by cause: the seeded opening (counts and indexes); the tier-1 arrange reaching the new Job while seeding or scripting the old path; the status rule (no persisted status, no slot, no heal); the dock notice instead of the `System` Message; the failure line coming from the Game record.

Recorded for later, as asked.

- The new failure-kind scenarios (empty text, invalid room, contradiction, missing data) and `swipe_switch.md` 36.4 ("a swipe switch gives back the state of that Swipe") come in C5, each with the tier-1 test that covers it. 36.4 has none yet, so the scenario is not written: a scenario without a covering test fails `spec-coverage`.
- `ApplicationError::ConcurrentGeneration` (`503`) is still produced by the old paths only (`settings_service`, `world_catalogue`, `persona_catalogue`, `prompt_preset_service` and the old pipeline), and it goes with the core step's deletions in C6.
- `WIREDAPP_SCOPE_ALLOWLIST_PREFIXES` (open follow-up): move `app_state.rs` into `adapters/driving/http/bootstrap` and shrink the allowlist to `bootstrap/` and `test_support/`.

**2026-10-18 ~19:55, lead: paused before the C4 commit (owner request: subagent plugin swap).**
- State: C1 `622d5a5e`, C2 `fcbe318f`, C3 `959ca7d6` committed on `wf/rebuild-core`. C4 (the switch) plus its review fix pass is uncommitted in the working tree (56 files). The untracked `.pi/extensions/delegate/` is the owner's, not C4's: do not stage it with C4.
- C4 review: both axes (c4-standards, c4-spec) handled; the fix pass (core4 result #2) did all 12 items. The lead has not yet re-read the fix-pass diff.
- C4 reduced gate on the final tree is green: clippy `logs/build_20261018_194629.log`, guardrails 193 `logs/build_20261018_195036.log`, lib 1272 `logs/build_20261018_194008.log`, storage 130 `logs/build_20261018_194032.log`. Red window: http 51 failed of 274 (`logs/build_20261018_194036.log`), browser 29/29 green, ignored races 6/10 pass. The grouped red list for the commit message is in core4's report above (5 groups).
- Also in the tree: `.agents/skills/simple-after-plan/SKILL.md` step 5 now says "Run the full gate once, here" (owner-approved); name it in the C4 commit message or split it out.
- Next: lead reads the fix-pass diff, commits C4 with the red list (reduced gate, plan §2), then C5 with a fresh implementer per commit. Implementer briefs ask for focused checks only; the commit gate runs the full gate once.

**2026-10-18 ~20:35, lead: C4 doc review (c4-docrev) and the C4 commit gate.**
- Lead read the fix-pass diff and restored four WHY comments the comment pass deleted (`job.rs` apply_rules opening room report and user_message per kind; `catalogue.rs` set_posture vs switch_mode, unchanged preset slot skips validation).
- Accepted edge of `GameWriter::replace`: the create unit lands first; if the removal then fails (storage failure only), two Games with the same name stand and the pointer stays on the old Game.
- c4-docrev spec-text fixes applied in C4: dashboard.md 39.1 title "else … else", case 1 "narrator call of a Narration", case 3 deleted (W1 option A: busy plus a failure line is not reachable by a client, because the accept unit clears the line and the Job writes its line after its Options run), case 4 duplicate line removed; 39.3 Givens name an offered set; 39.4 Options row status "options"; options.md 24.5 and 24.18 show set A again after POST /status/reset-generating; swipe_new.md 12.1 and retrigger.md 15.1 Given "a Game whose Job is running" and a client-visible Then; swipe_new.md 11.10 and I.9 "the state that its analysis saved"; swipe_new.md and retrigger.md I.7 restated as the missing-data 500; reset.md 7.1 restated in domain words; actions.md 2.4 disclosure wording.
- **C5 test items from c4-docrev** (tests assert the opposite of their changed scenario): swipe_new.rs 22.1 (two-room map, analysis moves to room2, assert room2); options.rs 24.5, 24.17, 24.18, 24.20 assert the dock notice in GET /fragment/options-dock, not a System Message (delete `system_messages`/`unavailable_agent_reports` helpers), and 24.5/24.18 assert set A after the stop; options.rs 24.2 asserts 400 and no notice with an empty opening text; actions.rs 2.4, retrigger.rs 14.5, swipe_new.rs 11.8/11.10 assert "The event narration failed." (2.4 also the disclosure); dashboard.rs 39.1 rewritten on the busy state and the failure line; 12.1/15.1 assert no Swipe / no event Narration added; reset.rs 7.1 per its new text; a test for the I.7 missing-data line comes with the C5 failure-kind scenarios.
- **C7 list additions from c4-docrev:** one pass replacing "the pipeline returns to idle" (deprecated Action Pipeline) in docs/specs; swipe_new.md 10.5 title and 11.9 Given name Snapshot; reset.md 7.2 names message_service/narrative.history; retrigger.md I.3, I.8, I.9 name narrative.history, process_action, last_trigger; retrigger.md 14.5 and swipe_new.md 11.8 mix "message history" and "story log"; one phrase for the opening ("its opening Narration"); CONTEXT.md gains "notice" and "failure line", and "step" (generation step) vs the avoided "step" for Hook; docs/diataxis/explanation/architecture.md:69,79 (heal to Idle, slot on persisted GenerationStatus); docs/diataxis/reference/narrative/options.md:10 ("persisted with the turn's snapshot"). Added by c5-docrev: docs/diataxis/reference/frontend/dashboard.md and docs/diataxis/reference/storage.md still say a swipe switch restores a Snapshot.
**2026-10-18, implementer (C5, uncommitted).** Branch `wf/rebuild-core`, base `51efaafc` (core C4). Area: the tier-1 rework. Tier 1 is green again with no `#[ignore]` left. No commit was made.

The tree holds 24 changed files. The red window of C4 (51 failing http tests, 4 of the 10 ignored `races::` tests) is closed.

What changed, per area.

| Area | Result |
|---|---|
| `tests/http/swipe_new.rs` | 20 red tests. The opening Narration shifts every count and index by one: a test that picked "the Narration" now picks the last one (`rfind`). The hand-seeded arranges (`seed_main_narration_flow`, `seed_event_flow`, `save_message_and_snapshot`, `save_snapshot`) are gone: a turn is arranged by `POST /action`, a user-regen by an Action then `POST /history/delete`, an Impersonate Input by `POST /action` with `/impersonate`. 11.1 arranges the empty story that R-44's `400` exists for; 11.7 breaks the start state's room through `corrupt_start_state`. 22.1 now arranges a two-room map and asserts the move into `room2`. 11.8 and 11.10 assert "The event narration failed.". |
| `tests/http/actions.rs` | 8 red tests: the first-Narration index and the story-log counts, the invalid-room arrange (now `break_start_room`), the 2.4 failure line and its disclosure (`The event narration failed.` plus the narrator line). New 2.6 (the whole missing-data line), and 2.3 is deleted (38.9 covers it). |
| `tests/http/retrigger.rs` | 3 red tests. `.last_trigger`/`.log` seeded state that no read sees, so every test now arranges a real turn: the Trigger fires and its continuation fails, which leaves the trigger context that makes Retrigger available. 14.3 arranges an Input-last story; 14.4 arranges a real event Narration. |
| `tests/http/options.rs` | 5 red tests. The dock notice replaces the `System` Message: the new `dock_notice` helper reads `GET /fragment/options-dock`, so `system_messages` and `unavailable_agent_reports` are gone. 24.5 and 24.18 assert that a stop gives the prior set back. 24.2 arranges a World with no starting scenario (an opening with no text) and asserts `400`. |
| `tests/http/dashboard.rs` | 2 red tests. 39.1 is the §2.3 rule: a held Job answers its step name, a Game with no Job and a failure line answers that line, and a plain Game answers idle. 39.3 arranges busy with a real held Job. |
| `tests/http/failure_display.rs` | 4 red tests. `TestAppBuilder::failure` writes the Game's failure line, so 38.3/38.5/38.7 read the design's record; 38.2 breaks the start state's room. New 38.9 (the empty-answer line) and 38.10 (the contradiction line). |
| `tests/http/story_log.rs` | 2 red tests: the first-Narration index, and the `retrigger-btn` fixture (a failed event continuation). New 8.8 (the log shows the text while the analysis holds the Job), moved here from `dashboard.rs` because this spec owns `/fragment/story-log`. |
| `tests/http/settings.rs` | 1 red test: the 150-Message story is seeded through `append_narrations` (the design's own record writes) instead of `save_message_and_snapshot`. |
| `tests/http/races.rs` | 2 red tests. The five §9.1 tests lose their `#[ignore]`. |
| `tests/http/requires_migration/` | `test_switch_swipe_handler_concurrent` arranges busy with a held Job and asserts `200` "Still thinking..."; the edit and delete refusals are `400`; `test_debug_is_generating_reflects_state` is deleted. |
| `tests/http/support/http_fixtures.rs` | The hand-seeded story writers are replaced by `corrupt_start_state`, `break_start_room` and `append_narrations`, and `seed_offered_set` writes the dock record through `Tx::put_dock` instead of the old snapshot path. |
| `src/test_support/test_app_builder.rs` | `generation_status` becomes `failure` (it writes the Game's failure line), and `claim_generation_slot` goes with the slot registry. The seeded opening now follows the create unit: the World's opening text and the starting room as its room report. |
| `src/test_support/fixtures.rs` | `seed_main_narration_flow`, `seed_event_flow`, `seed_swipe_with_stored_inputs` and `insert_message_with_swipe` are deleted: the hand-seeded story writers the plan names as fixtures that go. |
| `src/application/pipeline/action_pipeline/retry_tests.rs` | One legacy test changes in place (mechanical), and the four `test_retry_flow_*` tests go with the fixture they arranged through. |
| `src/application/pipeline/job.rs` | Two defects, below. |
| `src/application/generation/gate.rs` | Three refusal helpers replace `missing_data_with_line`: `missing(name)`, `unreadable(name, error)` and `save_failed(error)`, each writing the Game's failure line and answering `Refusal::Failed(failure)`. Every call site names its record or its failed write, so no site can put raw text in a line. |
| `src/domain/model/state/generation_status.rs` | `GenerationFailure` holds `named` (the line's own part: the room, or the record the request needs) and `raw` (the raw text the disclosure holds), with `new`, `missing_room`, `missing_record` and `line()`. The `MissingData` doc names a request, not only a Retry. |
| `src/domain/model/utils/opening.rs` | New: `opening_message(world, map, persona_name)`, the one build of a Game's opening Narration. |
| `src/adapters/driving/http/error.rs`, `layout/handlers/endpoints.rs` | The `500` body carries the failure's line (`Refusal::Failed`), and the status popover shows `failure.line()` with `raw`. |
| `scripts/validate_feature_spec.py` | The quarantine pin is 74 (one test deleted). |

Production defects found by the rework, each with the scenario that shows it.

1. **An event generation updated the encounter log only.** `Job::apply_rules` applied `GameRuleRequest::NpcEvents` for `Kind::Event`, so `scene.npcs_in_area` never changed and the event's analysis could never bring a Character present. Design §4.3 says the event generation applies the analysis with its movement taken out and the Trigger it reports ignored, so the Characters present are part of its state. The rework now applies `Analysis` with `movement: MovementParseResult::default()` and ignores `outcome.trigger_match`. Shown by `actions::test_trigger_continuation_reruns_quantifier_detects_new_npc_http` (actions.md 1.6: "the scene's NPCs include 'gabriella'") and by `swipe_new.md` 10.1/10.2.
2. **A failed event generation of a Retry showed the narrator's own line.** `run` wrapped only the second generation of a Job in `Ended::into_event_failure`, so the redo of an event Narration (whose first generation *is* the event) reported "The language model's answer could not be read." where §4.7 gives "The event narration failed." with that line in the disclosure. `run` now wraps a failed first generation when its kind is `Event`. Shown by `swipe_new::test_event_retry_trigger_narration_failure_sets_error_http` (swipe_new.md 11.8) and retrigger.md 14.5.

Deleted tests, each with its reason.

| Test | Reason |
|---|---|
| `requires_migration::debug::test_debug_is_generating_reflects_state` | Its subject was the persisted status plus `claim_generation_slot`; `/debug/is_generating` reads the Job entry now, and dashboard.md 39.3 covers that answer at tier 1. The quarantine pin is 74. |
| `TestAppBuilder::generation_status`, `TestAppBuilder::claim_generation_slot` | Not tests: the two builder methods. The first is replaced by `failure`, the second has no subject. |
| `seed_main_narration_flow`, `seed_event_flow`, `seed_swipe_with_stored_inputs`, `insert_message_with_swipe` | The hand-seeded fixtures: the plan names them as fixtures that go, and every tier-1 test arranges through the public write path now. The first two had no caller left; the last two went with their four legacy callers (the row above). |
| `swipe_new::test_retry_no_input_returns_400_http` (swipe_new.md 11.1), `retrigger::test_retrigger_no_messages_returns_400_http` (retrigger.md 14.2) | No client can make a Game whose story holds no Message: the create unit always writes the opening. The refusal stays in the code, and the same reason deleted 11.3 and 11.4 in C4. The requires_migration pin is unchanged (both tests were tier-1). |
| `retry_tests::test_retry_flow_impersonate_mode_runs_full_tail`, `::test_retry_flow_guide_only_narration_retries_with_empty_input`, `::test_retry_flow_guided_turn_retries_without_older_input_text`, `::test_retry_flow_user_regen_mode_stays_tail_less` | Their arrange was `seed_swipe_with_stored_inputs`, which goes with the other hand-seeded fixtures, and their subject is the legacy `ActionPipeline::retry` that C6 deletes. Tier 1 covers the same three modes over HTTP: `swipe_new.md` 22.1-22.5 (re-impersonate, user regen, stored inputs, the impersonate preset, the guided room report). Keeping them needed a local copy of the fixture, which put the legacy file at 2011 non-blank lines against the 2000-line guardrail; the allowlist guardrail's shrink-only rule rules out a new legacy test module. |
| `actions::test_empty_narrator_response_sets_error_no_narration_http` (actions.md 2.3) | 2.3 asserted the same behaviour at the same tier as `failure_display.md` 38.9 — the empty answer keeps its own line and saves no text — and 38.9 asserts it more: the exact line, that it is not the unreadable-answer line, and that the story log holds no Narration of the Action. |

New scenarios, each with its covering test.

| Scenario | Test |
|---|---|
| `swipe_switch.md` 36.4 (a swipe switch gives back the state of that Swipe) | `swipe_switch::test_switch_swipe_gives_back_the_state_of_that_swipe_http` |
| `failure_display.md` 38.9 (the empty-answer line) | `failure_display::test_empty_answer_keeps_its_own_line_clamps` |
| `failure_display.md` 38.10 (the contradiction line, and the text stays) | `failure_display::test_contradiction_clamps_with_its_own_line_and_keeps_the_text` |
| `actions.md` 2.6 (the missing-data `500`) | `actions::test_missing_persona_answers_500_with_the_missing_data_line_http` |
| `story_log.md` 8.8 (the story log shows the text while the analysis holds the Job) | `story_log::test_story_log_shows_the_text_while_the_analysis_holds_the_job` |

The invalid-room kind of §4.7 already has its scenarios (`actions.md` 2.1, `swipe_new.md` 11.7), so none was added; the other three kinds of §4.7 that no scenario had (empty text, contradiction, missing data) are the three above. The I.7 line of `swipe_new.md` and `retrigger.md` is the same accept check as `actions.md` 2.6 — `require_job_data` runs for every request kind — so 2.6 covers the missing-data **line** and the two Properties are not restated as scenarios. 2.6 does not cover I.7's **condition**: its Persona record is unreadable, while the Property is about a missing start state. That branch (`Ok(None)` for the start state) still has no scenario and no test, because no client request makes it and a broken record would have to drop the swipe state of the Message the Job starts from, which the `Tx` writes this slice added cannot do. Recorded as a gap, not as coverage.

The upload of §9.1: the flip check of each un-ignored test. Each expectation was changed once, watched fail, and changed back, while the test was still ignored.

| Test (case) | Flip | Result |
|---|---|---|
| `races::test_a_stopped_job_writes_nothing_and_the_next_job_saves_its_own_text` (I3) | the expected narration is `STOPPED_NARRATION`, not `NEXT_NARRATION` | fails at that assertion ("the stopped Job's late answer adds no Narration…") |
| `races::test_edit_and_delete_while_job_runs_are_refused` (I6) | the delete is expected to answer `400` | fails at the status assertion |
| `races::test_swipe_switch_while_job_runs_is_refused` (I6) | `assert_eq!` becomes `assert_ne!` on the active Swipe | fails at "a refused swipe switch must not change the active Swipe" |
| `races::test_a_job_goes_on_for_its_own_game_while_another_game_is_active` (I5) | Game A's log is expected to hold Game B's Narration | fails at "Game A's story log holds no Narration of Game B" |
| `races::test_a_stop_at_each_model_call_keeps_the_records_of_that_step` (I2) | the Impersonate arm expects a Narration | fails at "an Impersonate Job saves no Narration" |
| `races::test_a_stop_at_each_model_call_keeps_the_records_of_that_step` (I2, code review) | the Impersonate arm expects `MOVED_ROOM` where the refused state update leaves the player | fails at `left: "room_1" / right: "room2"`, so the read observes the current state and not the Snapshot the redo starts from |
| `races::test_a_stop_at_each_model_call_keeps_the_records_of_that_step` (I2, code review) | the Impersonate arm expects `[WORLD_OPENING_TEXT]` where the story holds the Input too | fails at `left: ["The World begins.", "The Job's narration."] / right: ["The World begins."]` |

The §9.4 items. The status test that checks each step name **already exists**: `dashboard.md` 39.4's generated table names the status while each model call is held (`narrating`, `quantifying`, `options`, `generating-event`), and the reworked 39.1 covers the three states a client can read without a call (a step name, the failure line, idle). No duplicate was added. The held-analysis test of R-11a is new: `story_log.md` 8.8 (`dashboard.md` 39.6 before the fix pass moved it).

Spec text fixed beyond the c4-docrev list. C4 introduced the opening Narration on create and reset, and several scenarios still counted a story as if it held no opening. The fix pass moved a scenario that belonged to another spec and deleted one that another spec covers. This table is exactly the diff of `git diff HEAD -- docs/specs/`.

| Spec | Change |
|---|---|
| `actions.md` 1.1, 1.2, 1.5, 1.9, 1.10, 1.12, 2.1, 2.2, 6.1, 6.2, 6.3 | The Given is "a Game whose story log holds only its opening Narration"; 2.1 says "And the story log holds only its opening Narration". |
| `actions.md` 3.5 | The Thens say the story log, and the Narration count names the opening: "the story log holds the opening Narration and exactly one Narration of that Action". Its test asserts the one Input and the one narrator call, so the count was a claim with no assertion behind it. |
| `actions.md` 1.14 | "Then the Action's Narration message reports the room …". |
| `actions.md` 2.3 | Deleted: `failure_display.md` 38.9 covers each of its lines at the same tier. |
| `actions.md` 2.6 | New, moved out of the Properties list into Error recovery after 2.5. It carries the whole line, the same line from the status, the raw text only in the disclosure, and no Message gained. |
| `swipe_new.md` 9.1, 10.1 | The Given is "a Game whose story log holds only its opening Narration", and the Thens count the opening: 2 Narration entries after one Action, 3 after a Trigger's event Narration. 9.1 names the Action's Narration and the ID it keeps. |
| `swipe_new.md` 9.8 | The two switch steps go; a sentence points at `swipe_switch.md` 36.4. |
| `swipe_new.md` 11.1 | Title and Given: a story with no Message. |
| `swipe_new.md` 11.7 | The Given names the state before the last Narration, which is what the Retry's room check reads. |
| `swipe_switch.md` 36.4 | New: a switch gives back the state of that Swipe, and "a Retry of that Action's Narration" names the Narration it means. |
| `failure_display.md` endpoints | `POST /action` joins the list. |
| `failure_display.md` 38.9, 38.10 | New. 38.9's Given is empty text. Both end the Job before any step reads the status, and 38.10's last Then counts the raw text once, inside the disclosure. |
| `retrigger.md` 13.1, 13.2, 14.1, 14.5 | One phrase for the context ("a Game whose current state holds a trigger context"), and the reads are domain words: "the story log holds N+1 Messages", "an event Narration (it carries an event header)", "the generation status is Idle", "the player still stands in \"room2\"". |
| `retrigger.md` 14.2 | The Given is a story with no Message. |
| `retrigger.md` 14.3 | The Given names the last Message's kind. |
| `retrigger.md` 14.4 | The trigger-context Given goes: an event generation clears the context, so the Given names the event Narration a Trigger appended. |
| `story_log.md` 8.8 | New, moved from `dashboard.md` 39.6 because this spec owns `/fragment/story-log`. `dashboard.md` returns to its HEAD text. |

Deviations from the plan.

1. **Two production defects are fixed, not worked around** (above). Both are small and both are decided by design §4.3 and §4.7.
2. **`TestAppBuilder` gains a `failure` knob and loses `claim_generation_slot`.** `generation_status` "changed" rather than went, because 38.3, 38.5 and 38.7 assert the clamping of one failure kind with its exact raw text, and no client path produces a spent token budget or a transport error.
3. **`TestAppBuilder::last_trigger` and `::log` stay.** `last_trigger` writes a field no read uses, and its last user is the legacy lib test `retrigger_tests::test_retrigger_returns_concurrent_generation_when_gate_busy`, which names the old gate and goes with the old pipeline in C6; `log` writes real Messages through the fixture's writer and has three users in the quarantine plus two legacy lib tests. C6 deletes both with their last callers.
4. **`log` left `dashboard.rs` 39.2 and `story_log.rs` 8.5**: both now use a real turn or the seeded opening.
5. **Two new fixture writers** (`corrupt_start_state`, `append_narrations`) write broken or long records through the design's own `Tx` writes, because no client request can make them: a room off the map, an encounter log that names a Character the World lacks, and a 150-Message story.
6. **`ReQUIRES_MIGRATION_TEST_COUNT` 75 → 74**, for the one deleted quarantined test.
7. **A missing-data failure carries two texts.** `GenerationFailure` gains `detail` (with `#[serde(default)]`, so a stored failure line from before the field still reads). The line names the record and the disclosure holds the unreadable error, which is design §4.7's split; `raw` keeps its meaning for every other kind (it is what the line names, as `SceneMissing`'s room id is). The three `require_job_data` paths use it. The other twelve `missing_data_with_line` call sites still pass the storage error text as the line's subject, because the review that asked for the split named those three and each of the rest needs its own name-and-kind judgement (several are save failures classified as missing data). Recorded as a follow-up, not as done.
8. **Load-bearing test setting.** The 38.10 contradiction comes from `assert_state_consistency` in `game_rules`, which the `diagnostics` feature gates. The default feature set turns `diagnostics` on (`Cargo.toml`: `default = ["testing"]`, `testing = ["diagnostics"]`), so the assertion runs in the gate as it runs in this suite. With `--no-default-features` the rules step would accept the broken state instead.
9. **`Named` and `raw` are the failure's two parts.** `GenerationFailure` holds the line's own part in `named` and the disclosure's text in `raw`. The review asked for one meaning and an honest name, and for the raw text to live where `detail` was. I merged `detail` into `raw` rather than keeping the name `detail`: `raw` is what this codebase calls the disclosure's text (`raw_error_detail`, and design §4.7's "the raw text goes in the disclosure"), so the thirty existing `.raw` readers keep their meaning and every one of them now reads only the raw text. A wholesale rename of those readers to `detail` is mechanical if the lead prefers the other name.
10. **No client request makes 38.10's state.** The Quantifier drops a Character id the World does not hold, so the state the test breaks is a corrupted or legacy record: the fixture writes it through `Tx::set_swipe_state`. A World data change between two boots makes the same state honestly — a Character removed from the World while a Game's encounter log still names it.

Open questions.

- **The event Narration's `maker` is `Narrator`** (C4's open item, unchanged): the call record carries the Trigger role, but the Swipe's maker does not name the event. No spec states a value.
- **A stop cannot cancel a call that is already running** (C3's item): the job id check refuses the late save, and 39.4/39.5 pin that, so nothing in this slice depends on a cancel.
- **`Refusal::BadRequest("No input to retry")`** is reachable only for a story with no Message; the design says such a Game is never created (R-76).
- **The start-state branch of I.7** has no test (above).
- **`break_start_room` stays.** The reviewed item asked whether the invalid-room arrange is still needed. It is: it breaks the state the Retry starts from, which no client request writes, and a World data change between two boots puts a room off the map the same way (the same reason 38.10 needs `corrupt_start_state`). The refusal that answers it is the design's own invalid-room line.
- **`seed_offered_set` reaches `state.job_deps.storage`** because `AppState` carries no `storage` field: `job_deps` is the app's own `JobDeps` and its `storage` is the same `Arc<Storage>` the Job writes through, so the fixture writes the app's storage and not a second one. A fixture that took `&Storage` would need every caller to hold the handle, and none of the four callers builds its app that way.

Validation on the final tree.

| Check | Log | Result |
|---|---|---|
| `python build.py clippy` | `logs/build_20261018_225232.log` | green |
| `python build.py guardrails` | `logs/build_20261018_224957.log` | 193 passed |
| `python build.py test-pattern "" --test-target http` | `logs/build_20261018_225136.log` | 280 passed, 0 failed, 0 skipped |
| `python build.py test-pattern "" --test-target lib` | `logs/build_20261018_225207.log` | 1269 passed |
| `python build.py test-pattern "" --test-target storage` | `logs/build_20261018_225156.log` | 130 passed |
| `python build.py spec-coverage` | `logs/build_20261018_225628.log` | green |
| `python build.py validate-docs` | `logs/build_20261018_225629.log` | green |
| `python build.py browser` | `logs/build_20261018_225421.log` | 29 passed |
| `python build.py seam-cases`, `test-structure`, `fmt` | `logs/build_20261018_2115*.log`, `224211` | 5 contracts, 89 cases; green |
| `cargo nextest run --no-fail-fast --test http --run-ignored all -E 'test(races::)'` | the flip runs above | 10 run, 10 passed, before the `#[ignore]`s came off |

Counts moved with the deletions: http 283 → 280 (2.3, swipe_new 11.1, retrigger 14.2) and lib 1272 → 1269 (the four `test_retry_flow_*` legacy tests). The requires_migration pin stays 74.

Not done, with the reason. The commit gate (`/simple-after-plan`) and the owner's approval are the commit step, which this assignment excludes.

### C5 fix pass 2 (the two-axis code review)

Per item, what changed.

| Item | Change |
|---|---|
| 1. The failure lines of `GameWriter` | `missing_data_with_line` is replaced by three helpers — `missing(name)` for a record the engine has no row for, `unreadable(name, error)` for one it cannot read, and `save_failed(error)` for a write that fails — plus `refuse(failure)`, which writes the Game's failure line and returns `Refusal::Failed(failure)`. All seventeen call sites are classified: the reads name their record ("the message", "the story", "the game", "the persona", "the start state", "the impersonate direction", "the trigger context", "the World", "the trigger") and the writes (the edit save, the delete, the swipe switch, the accept, the Job's four save units) take §4.7's save line. `Refusal::MissingData(String)` becomes `Refusal::Failed(GenerationFailure)`, so the `500` body and the Game's line come from one failure and no call site can put raw text in a line. |
| 2. `GenerationFailure.raw` | The failure now holds `named` (the line's own part) and `raw` (the disclosure's text), one meaning each, with `new`, `missing_room`, `missing_record` and `line()`. `from_engine_error` classifies as before and keeps the error as the raw text; a missing room names the room *and* keeps its error. Every user is updated: `endpoints.rs` shows `failure.line()` and `raw`, `job.rs` wraps an event failure with `failure.line()`, `view_query` and the legacy tests keep reading `.raw` (now only ever the raw text). The domain's seam tests assert both parts, including the two lines. |
| 3. `Refusal::MissingData` doc | The variant is gone; `Refusal::Failed` carries the whole failure and its doc says the payload's line is what the answer carries and the raw text stays in the disclosure. |
| 4. One opening build | New `domain/model/utils/opening.rs::opening_message(world, map, persona_name)` builds the opening Narration — the World's default scenario rendered for the Persona, the starting room as its room report, `Maker::WorldOpening`, kind `Narration` — and returns it with a state-less Swipe, because the state is the caller's. The create unit attaches `SwipeState::starting`, the test builder attaches its seeded state, and `retry_tests` computes its expected opening text from the same call. It lives in `domain/model/utils/` because it is World and Persona data rendered through the domain's `TemplateVars`, and because `gate.rs` and `test_support` may both call domain code. |
| 5. `swipe_new.md` 11.1 and `retrigger.md` 14.2 | Deleted with their tests. No client makes a Game with no Message, and the refusal stays in the code. The quarantine pin is unchanged. |
| 6. The I2 Impersonate arm | The arm now reads the room through `game_view_query.get_debug_state` and asserts the whole story list against `WORLD_OPENING_TEXT`, like the other arms. Its row arranges on the always-on World (via `active_game_on_an_always_on_world`), so that opening text is the row's own. Both halves are flip-checked (the flip table). |
| 7. `seed_swipe_with_stored_inputs`, `insert_message_with_swipe` | Deleted from `src/test_support/fixtures.rs` with their four legacy callers, which were the only callers left (the deletion row above carries the reason and the tier-1 coverage). |
| 8. `requires_migration/fragment.rs` | `test_switch_swipe_handler_success` and `test_switch_swipe_handler_not_last` are rebuilt on the design's writes and reads: an Action and a Retry (or an Action alone) arrange the story, and `game_view_query.get_story_log_entries` asserts the Swipe count, the active Swipe and the refusal. No test in that file names `load_latest_snapshot` or a `snapshot_id`. The stale comments in `games_create.rs` (the create unit's own records) and `swipe_switch.rs` 36.2 (the switch writes the empty dock record) are reworded. |
| 9. The disclosure check | `actions.rs` 2.6 and `failure_display.rs` 38.10 use `http_assertions::status_error_message`: the disclosure's text is read from the `<pre>` itself and the raw text is asserted to occur once, so nothing slices the fragment by hand. |
| 10. `append_narrations` doc | Reworded: it pads the story with Narration Messages that carry no Input, no maker and no room report, so a test that needs a long story reads padding and not a turned scene. |
| 11. `failure_display.rs` literals | The three seeded failures use `GenerationFailure::new(kind, raw)`; no test builds the struct by hand. |
| 12. `seed_offered_set` | Left as it is, with the reason in the open questions: `AppState` has no `storage` field, and `job_deps.storage` is the app's own handle. |


**2026-10-18 ~23:05, lead (C5 commit gate).** The first full gate on the C5 tree failed 2 tests of the `bootstrap` binary (`run_branches::test_run_persona_not_found_*`: `Validation("Persona not found")`, not `PersonaNotFound`). Cause: a C4 regression. C4 moved the persona check behind `resolve_boot_game`, so on an empty database boot created the Game first and failed inside `GameCatalogue::create_game`. Neither C4's reduced gate (clippy, guardrails, lib, storage) nor C5's focused checks (http, lib, storage) run the `bootstrap` binary. Fix: `prepare_data` checks the persona before it resolves the Game, and `PreparedData.persona_key` is removed. Full gate green on the final tree (`logs/build_20261018_230213.log`: guardrails 193, integration 1683 + 2 skipped, browser 29). Lesson for C6: a focused check list must name every test binary the change can reach; `python build.py integration` covers them all in ~35 s.
