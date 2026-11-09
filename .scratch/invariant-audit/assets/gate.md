# Invariants the gate relies on but does not enforce

Theme: busy state, Jobs, Autoplay and concurrency.
Scope: `src/application/generation/`, `src/application/pipeline/`, `src/application/autoplay/`,
`src/application/games/`, `src/adapters/driven/storage/{changes,transaction,write_guard,db}.rs`,
`src/adapters/driving/http/utils/{request_layers,current_game}.rs`,
`src/adapters/driving/http/layout/handlers/`, `src/bootstrap/`.
Read-only investigation. No build, no test run.

A note on the model check. `.agents/skills/verify-gate/SKILL.md` checks a TLA+ model of the
gate with TLC. A model check is verification, not runtime enforcement. The rules it covers
(`OneJob`, `BusyRefusesStory`, `StoppedJobWritesNothing`, `NextJobKeepsHolder`,
`RunHoldsItsGame`, `StoppedRunStartsNoJob`, `RunRefusesConfig`, `BudgetBound`, `RunEndsIdle`,
`DeleteNotActive`, `ActiveExists`) are claims about the code that a *test run* re-checks after
an edit. Nothing at runtime stops the code from breaking them, and three classes of violation
sit outside what the model can even describe: two reads of the pointer inside one request
(gate-02), a second process (gate-04), and the lock orders that the model assumes (gate-03).

---

## gate-01 — The active Game pointer names a Game whose row exists

1. Invariant: `ActiveGame` never names a `GameId` without a `games` row.
   The rule is stated as `ActiveExists` in the model check, and as "The active Game always
   names a Game that exists" in `docs/diataxis/reference/storage.md`.

2. Relied on at:
   - `src/adapters/driving/http/utils/current_game.rs:34` hands the id to every Game-bound
     handler without reading the row. A handler then works on a phantom Game.
   - `src/application/generation/gate.rs:690` writes the Game's failure line through
     `Tx::set_game_failure` (`transaction.rs:234-249`), which is
     `UPDATE games SET failure = ?1 WHERE id = ?2`: an update that matches no row is not an
     error, so the accept of a Job writes nothing and reports that it wrote.

3. Write paths that could violate it:
   - `src/application/games/catalogue.rs:32-35` — `GameCatalogue::new` moves the pointer to the
     boot Game with no existence read:
     ```rust
     if let Some(initial_game) = initial_game {
         active_game.lock().set(initial_game)?;
     ```
   - `src/application/games/active.rs:65-71` — `PointerHold::set`'s only guard is
     `self.storage.touch_game_updated_at(game)?`, and that call
     (`src/adapters/driven/storage/core.rs:260-273`) ignores the affected-row count, so a touch
     of a missing row answers `Ok`. `set` cannot fail on a missing Game.
   - `src/bootstrap/run.rs:115` — boot notices the missing row and continues:
     `Ok(None) => tracing::warn!("Starting on game {game_id}, which has no row")`, then
     `src/bootstrap/run.rs:127` builds the graph with that id.
   - External input: a `games` row removed by hand or by an older version of the app, or a
     second process deleting it between the boot resolve and the pointer move.
   `switch_game` (`catalogue.rs:71-80`) *does* read the Game first, under the pointer lock, so
   that path is safe.

4. Enforcement: PARTIAL. `GameCatalogue::switch_game` reads the Game under the pointer lock
   before it moves the pointer; `GameCatalogue::create_game`/`reset` pass an id that a committed
   unit just returned (`set_new`). `GameCatalogue::new` (the boot path) checks nothing, and
   `PointerHold::set` cannot detect a missing row.

5. Consequence if violated: silent wrong data plus a dead session. Every Game-bound handler
   fails on a missing row (`require_game` → "Game not found"), the failure-line writes of
   `accept`/`stop`/`swipe` land nowhere and still answer `200`, and the only recovery is a
   restart with another world.

6. Likelihood: only a manual DB edit, an older app version, or a second process. A player cannot
   cause it.

7. Epistemic status: VERIFIED (read every writer of the pointer: `set`, `set_new`, and the four
   callers in `catalogue.rs`; read `touch_game_updated_at`; read the boot path).

8. Suggested enforcement point: `PointerHold::set` reads the Game and fails when the row is
   absent, instead of relying on the touched `updated_at`; and `run.rs` treats a missing boot row
   as a boot error, not a warning.

---

## gate-02 — The Game a request writes is the Game its header check verified

1. Invariant: for a Game-bound `POST`, the `X-Chronicler-Game` header check and the `CurrentGame`
   the handler uses name the same Game.

2. Relied on at:
   - `src/adapters/driving/http/layout/handlers/action_controls.rs:14` renders Stop as
     `hx-post="/status/reset-busy"`; that route is game-bound, and its handler
     (`src/adapters/driving/http/layout/handlers/endpoints.rs:109-118`) writes the Game it gets
     from `CurrentGame`.
   - `src/adapters/driving/http/action/handlers/actions.rs:42-46` starts the Job on
     `CurrentGame`'s Game.

3. Write paths that could violate it: nothing writes the pointer here. The violation is a race
   between two *reads* of the pointer inside one request:
   - `src/adapters/driving/http/utils/request_layers.rs:44-50` reads the active Game once and
     compares it to the header.
   - `src/adapters/driving/http/utils/current_game.rs:34` reads the active Game a second time,
     later, inside the extractor. `ActiveGame::get` (`active.rs:33`) takes and drops the mutex,
     so it holds no guard across the two reads.
   A reset, a create or a switch that completes between the two reads moves the pointer, and the
   handler then writes the new Game while the page named the old one.

4. Enforcement: NONE. Two independent reads and no shared guard. The layer's doc comment claims
   the refusal comes "before the handler runs", which is true of the order and not of the Game.

5. Consequence if violated: silent wrong data. `/action` writes a Message into another Game's
   story; `/status/reset-busy` clears the wrong Game's failure line and Options failure
   (`gate.rs:515-526`), and answers `reset`.

6. Likelihood: a player with two dashboard tabs, or one tab plus a slow request, while the other
   tab resets, creates or switches a Game. The window is the gap between the layer and the
   extractor, so it is small but reachable on a real player action.

7. Epistemic status: VERIFIED (read both reads, the route layer wiring in
   `builders/router.rs:62`, and two handlers). Did not trace the browser side beyond
   `assets/dashboard-screen.js:421-428`, which does send the header on every non-GET request.

8. Suggested enforcement point: the layer reads the active Game once, and `CurrentGame` is built
   from that value (a request extension), so the check and the handler name one read.

---

## gate-03 — The lock discipline of the registry, the transaction and the pointer holds

Three rules, all stated only in comments:
- no `Storage` call inside a `Tx` closure (`src/adapters/driven/storage/transaction.rs:50`:
  "The closure makes no `Storage` call, which would wait on the backend mutex the unit already
  holds");
- no `GameWriter` call inside a registry hold, so no hold inside a hold
  (`gate.rs:371-375`: "`decide` runs under the registry hold ... and calls no `GameWriter`");
- no `ActiveGame` read while the guard is held (`gate.rs:188`: "No code that holds the guard may
  take the `ActiveGame` mutex").

1. Invariant: no thread takes a lock it already holds, in any of the three orders.

2. Relied on at: every hold and every unit. `BusyRegistry::hold` (`gate.rs:191-206`) keeps the
   guard across `f`, and `f` opens the backend mutex; `Tx` closures run with the backend and
   connection mutexes held; `ActiveGame::get` (`active.rs:33`) is a plain `std::sync::Mutex`,
   which is not reentrant. A second lock attempt on any of them blocks forever, not for a
   timeout.

3. Write paths that could violate it: any future edit inside
   - a `Tx` closure (`Storage::transaction` takes `FnOnce(&mut Tx)`, and a closure may capture
     `&Storage` freely — `job.rs:470` and `gate.rs:566` both capture `self.storage` in the
     surrounding scope),
   - the `decide` closure of `GameWriter::accept_for_autoplay` (`gate.rs:376-399`),
   - a `BusyRegistry::hold` closure that calls `GameCatalogue::active_game_id()`
     (`catalogue.rs:144`).
   The lock order is also recorded in `active.rs:25-28` and in the skill's model notes.

4. Enforcement: DOC-ONLY. Comments only. `TxToken::new` being crate-private limits who opens a
   unit, not what runs inside it.

5. Consequence if violated: the whole server hangs. Every request waits on the backend or the
   guard, including the Stop button, and no timeout exists.

6. Likelihood: only a code change. No player, LLM output or World JSON reaches it today.

7. Epistemic status: INFERRED. I read the three rules and the call paths of `gate.rs`,
   `catalogue.rs` and `transaction.rs` and found no current violation; I did not audit every
   `Tx` closure in `adapters/` (e.g. the seeds).

8. Suggested enforcement point: a thread-local "inside a unit / inside a hold" marker that panics
   on a second entry, checked in `Storage::with_backend_writes`, `BusyRegistry::hold` and
   `ActiveGame::get`.

---

## gate-04 — One server process per DB file

1. Invariant: the process that owns a `*.db` file is the only one that writes it. The busy
   registry (`gate.rs:191`, `struct BusyRegistry`, memory only) and the active-Game pointer
   (`active.rs:29`, `game: Mutex<Option<GameId>>`) live in the process, so a second process sees
   every Game as idle and every pointer as unset.

2. Relied on at:
   - `src/application/generation/gate.rs:353-358` — `require_idle` reads the in-memory map only.
     A second process's Job is invisible, so two Jobs can run on one Game and both pass the plan
     read.
   - `src/adapters/driven/storage/db.rs:16-25` — the pool holds **one** `Connection` behind a
     `Mutex` and sets one pragma (`foreign_keys`). There is no `busy_timeout` and no
     `journal_mode=WAL` anywhere in `src/` (grep: no match).
   - `src/bootstrap/run.rs:50` — the file name carries the port:
     `chronicler_{}.db`. The port bind, not a check, is what makes the assumption true; and
     `DbPool::new` (`run.rs:51`) runs `run_migrations` **before** the bind.

3. Write paths that could violate it: external — a second instance of the binary on the same port
   (it fails at bind, after it has migrated and read the file), a copy of a DB file, or a test
   probe pointed at the same file.

4. Enforcement: PARTIAL. Naming the file by port plus the exclusive port bind effectively keeps
   one owner per file today. Nothing checks it: no lock file, no PID row, no
   `PRAGMA locking_mode`, and no boot check that the file is not in use.

5. Consequence if violated: two writers race on one story — duplicate Messages, two holders for
   one Game, a `Swipe` index taken from a stale `tail` (`gate.rs:931-934`). The loser of a
   SQLite write gets `SQLITE_BUSY` at once (no `busy_timeout`) and answers a save-failure line.

6. Likelihood: a normal player cannot reach it. A developer who runs the server twice, or a
   tool that copies `chronicler_<port>.db`, can.

7. Epistemic status: VERIFIED for the code claims (single connection, no busy timeout or WAL, the
   file name, the migration order). INFERRED for the bind-before-migrate ordering being the only
   guard.

8. Suggested enforcement point: take an exclusive OS lock (or a `PRAGMA locking_mode=EXCLUSIVE`
   hold) on the DB file in `DbPool::new`, and fail boot when it is already held.

---

## gate-05 — A Job that ends without writing its records writes a failure line

1. Invariant: after every Job, the Game is idle and either the story grew or the Game holds a
   failure line. `docs/specs/actions.md` I.1 ("a Job always ends") is what the model check
   deliberately leaves out.

2. Relied on at:
   - `gate.rs:366` — the accept clears the failure line (`accept_in_tx`, `gate.rs:737-741`)
     before the Job runs, so a Job that dies silently leaves the Game looking clean and idle.
   - `src/adapters/driving/http/layout/handlers/endpoints.rs:50-64` — the status display then
     answers `Ready` for a Game whose Input never landed.

3. Write paths that could violate it: `Job::run` (`job.rs:142-175`) writes the line only on a
   `Result` error, through `Job::end` (`job.rs:385-412`). A panic anywhere in the blocking task
   (`job.rs:93`) unwinds past that call. So do an abort of the runtime and a `std::process::abort`.

4. Enforcement: PARTIAL. Every `Result` failure path writes the line (`save_failure`,
   `gate.rs:873-877`); a panic writes nothing. The holder *is* released on a panic, by RAII —
   `Drop for JobWriter` (`gate.rs:951`) — so the Game does not stay busy.

5. Consequence if violated: a lost request that looks successful. The player's Action is in no
   story, no failure line stands, the status says `Ready`. The same applies inside an Autoplay
   run: `Drop for AutoplayRun` (`driver.rs:138`) ends the run with no failure line.

6. Likelihood: a code bug only. No player, LLM output or World JSON triggers a panic today: the
   only `unwrap`/`expect` in the theme is `utils/response.rs:13` (a static body).

7. Epistemic status: VERIFIED for the gate and Job paths. INFERRED for panics inside
   `narrate`/`agent_runner`, which I did not audit line by line.

8. Suggested enforcement point: a guard type inside `Job::run` whose `Drop` writes the failure
   line when the Job did not finish (the panic path has no other writer).

---

## gate-06 — One Job reads one config

1. Invariant: the config a Job reads (Narrator Mode, posture, the four preset ids,
   `options_always_on`) is the same at its first part and at its last.

2. Relied on at: the Job reads the Game row again for each part, and each read can see another
   write:
   - `job.rs:240` — `narrate` reads the row for the perspective and tense.
   - `job.rs:493` — `hook_settings` reads it again for the Quantifier, the Trigger and the
     Options Agent, so a mid-Job config write makes the analysis use a different preset and a
     different Connection than the narration.
   - `job.rs:489` — `options_enabled` reads `options_always_on` at the end of the Job.
   `gate.rs:501-508` documents that a config write passes while a plain Job runs:
   `if matches!(holders.get(&game), Some(Holder::Autoplay { .. })) { ... }`.

3. Write paths that could violate it: `POST /games/:id/mode`, `/posture`, `/presets` →
   `GameCatalogue::save_config` → `GameWriter::save_config` (`gate.rs:501-508`), which refuses only
   an Autoplay holder. External input: a player (or a second tab) sending one of those three
   forms while a Job runs.

4. Enforcement: NONE. The refusal covers an Autoplay run only (`docs/specs/autoplay.md` 41.12),
   and the Job keeps no snapshot of the config it started with.

5. Consequence if violated: silent wrong data. A Narration written under preset A is analysed by
   the Quantifier of preset B, and the Options run can be skipped or run against the player's
   last-second toggle. Nothing fails loudly.

6. Likelihood: a normal player with two tabs, or one tab while the dashboard still allows the
   config forms — the write is accepted, so a single player can trigger it.

7. Epistemic status: VERIFIED for the three reads and the write path. Did not check whether the
   dashboard disables the config forms while busy; a disabled form is a client-side mitigation
   only.

8. Suggested enforcement point: read the config once per Job (`Job::call_plan`/`game_row`) and
   pass it down, or extend the `save_config` refusal to any holder.

---

## gate-07 — A refused reset leaves the Game, its holder and the pointer as they were

1. Invariant: `GameWriter::replace` is all or nothing — stated in its own doc comment
   (`gate.rs:602-606`: "a failure there leaves the old Game, its entry and the pointer as they
   were").

2. Relied on at:
   - `gate.rs:606-616` — the create unit commits first, then `holders.remove(&old)` runs, then
     the delete unit may return `Err`, and `.inspect(... forget_game(old))` is skipped.
   - `catalogue.rs:282` — `pointer.set_new(new_id)` is skipped when `replace` returns `Err`, so
     the pointer stays on the old Game; that part is correct.

3. Write paths that could violate it: a storage fault in the second unit of
   `opener.open(old, |tx| tx.delete_game())` (`gate.rs:610`), including the injected failures of
   `write_guard.rs` used by tests.

4. Enforcement: PARTIAL for the intent, NONE for the holder and for the orphan.
   The pointer and the old rows are kept, as documented. The old holder is removed before the
   delete unit runs, so after a refused reset the old Game is active with **no** holder and its
   running Job's next save is refused — the Job is stopped by a reset that reported failure. The
   new Game's row stays too, with its opening Narration, visible in the Games list.

5. Consequence if violated: a stopped generation the player did not ask for, plus one orphan Game
   per failed reset.

6. Likelihood: a storage fault only. The scope says storage faults are outside the model check.

7. Epistemic status: VERIFIED (read `replace`, `create_in_tx`, `delete_game` and the catalogue
   caller).

8. Suggested enforcement point: keep the holder until the delete unit lands (remove it after the
   second transaction), and delete the orphan Game when the second unit fails.

---

## gate-08 — The Games list order follows the newest Message

1. Invariant: a Game with new Messages moves to the front of the Games list.
   `src/adapters/driven/storage/changes.rs:180-181` states it as the reason for the raise:
   "A new or a removed Message also raises the Games list, which orders the Games by their
   newest Message."

2. Relied on at: `src/adapters/driven/storage/changes.rs:182-186` raises
   `ChangeKind::GAMES` on `InsertMessage`/`DeleteMessage`, so the client reloads the list after
   a Message lands.

3. Write paths that could violate it: no write path — the list SQL does not read the Messages.
   `src/adapters/driven/storage/games.rs:18` and `games.rs:49` order by
   `updated_at DESC, id DESC`, and no Message write touches `games.updated_at` (grep: only
   `insert` in `write_game_row`, `touch_game_updated_at`, `update_game_display_name` and
   `set_game_config` do). Only `newest_game_of_world` (`games.rs:154`) joins the Messages.

4. Enforcement: DOC-ONLY. The comment claims an order the list query does not implement.

5. Consequence if violated: silently wrong UI — a Game played for a long time keeps its creation
   time and stays below Games that were only re-configured; the `GAMES` raise on a Message is
   still correct for `newest_game_of_world`, and it does refresh the list.

6. Likelihood: a normal player sees it whenever two Games have different story activity than
   config activity.

7. Epistemic status: INFERRED. I read the four `updated_at` writers and the two list queries, and
   did not read the Games-list template or the client's reload rule.

8. Suggested enforcement point: order the saved-Games query by
   `COALESCE(MAX(message.timestamp), updated_at)`, as `newest_game_of_world` already does, or
   touch `updated_at` in the Message writes.

---

## Checked and enforced

- **At most one holder per busy Game.** `Holders` is a private `HashMap<GameId, Holder>` inside
  a private `Mutex` in `BusyRegistry`; the only mutation sites are in `gate.rs`, and each
  `insert` is reachable only after `require_idle` (`gate.rs:362`, `gate.rs:472`) or after the
  run-id check (`gate.rs:382-393`), in one critical section. Enforced by type plus the lock.
- **Only the holder writes the story.** Every `JobWriter` method goes through `JobWriter::hold`
  (`gate.rs:797-810`), which checks `entry.id == id` in the same hold as its transactions; `TxToken`
  is crate-private and only `TxOpener`, built inside `BusyRegistry::hold`, makes one
  (`transaction.rs:41-46`). A write that skips the check cannot compile.
- **Every Job and run releases its hold on every exit path.** `Drop for JobWriter`
  (`gate.rs:951`) and `Drop for AutoplayRun` (`driver.rs:138`) sit on the outermost value, which
  is moved into the blocking task, so an error, a panic, a `Stop`, and a dropped
  `spawn_blocking` closure all release. The Autoplay end checks the run id, so the end of an old
  run cannot release a new one (`gate.rs:489-496`).
- **A stopped Job writes nothing.** The holder removal and the id check share one mutex, so a
  save that finds no entry ends the Job with `EntryMoved` (`gate.rs:801-809`).
- **Change counters are raised only after the data committed.** `Tx::commit` runs first
  (`transaction.rs:350`), then `raise_unit` (`transaction.rs:407`), both under the backend mutex,
  so a load that sees the new numbers sees the new rows (`core.rs:249-252`).
- **Change counters only increase.** `raise_game`/`raise_engine` add one and bump `revision`
  (`changes.rs:29-49`). The one reset is `forget_game` (`changes.rs:122`) on a deleted Game;
  SQLite reuses a freed rowid, so a new Game can inherit a zeroed counter set — harmless because
  the raise also bumps `revision` and `games`, and the deleted Game is never the active one
  (`catalogue.rs:95-104` refuses to delete the active Game).
- **Budget is at least 1.** `Budget` has a private field and `parse` is its only constructor
  (`autoplay/budget.rs:14-35`); the HTTP form goes through `parse`
  (`autoplay/handlers/autoplay.rs:26`). `AutoplayProgress` itself is publicly constructible with
  any `budget`, so the gate trusts its caller (`gate.rs:467-483`, "the gate stores the budget as
  it comes"); every caller takes it from `budget.cycles()`.
- **`AutoplayRun::start` cannot leave a holder without a loop.** The holder is reserved before
  the task is spawned (`driver.rs:51-64`), and the captured `driver` releases it if the closure
  is dropped unrun.
- **The pointer moves only after the create unit landed.** `set_new` is called after
  `create`/`replace` return `Ok` (`catalogue.rs:64`, `catalogue.rs:282`); `PointerHold::set`
  touches `updated_at` before it moves, so a failed touch leaves the pointer (`active.rs:65-72`).
  gate-01 covers the one path that skips the existence read.
- **The pointer's outer lock covers every check with its write.** `ActiveGame::lock` is
  `pub(super)` and its `PointerHold` is held across the read, the unit and the move in
  `create_game`, `switch_game`, `delete_game` and `reset` — this is what `DeleteNotActive` and
  `ActiveExists` name.
- **SQLite writer concurrency in one process.** One `Connection` behind one `Mutex`
  (`db.rs:20-32`) plus the backend `Mutex` (`core.rs:217-228`) serialises every write; no
  `busy_timeout` or WAL is needed *in* one process. gate-04 covers the second process.
- **Poisoned-lock handling.** Every lock site in the theme takes the inner value
  (`gate.rs:198`, `changes.rs:126`, `active.rs:34`, `active.rs:47`, `db.rs:28`,
  `core.rs:225`, `job_tasks.rs:15`, `pause.rs:33`, `pause.rs:55`). The process does not abort; gate-05 covers
  what a panic leaves behind.
- **Accepted gap: `stop` is not atomic with the holder removal.** `GameWriter::stop` (`gate.rs:515-526`) reads
  the dock, removes the holder, then writes. A failed write leaves the Game idle with a stale
  failure line. The model check excludes storage faults on purpose; no data is lost.
