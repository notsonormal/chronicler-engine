I reviewed the whole Rust rebuild. The Spec axis has one high-severity defect: a bare `/impersonate` now fails with a 500. The Standards axis has no hard violation, only medium and low findings. I did not edit files or run the build.

**Scope.**
- I read both bundles (the committed range and the uncommitted fixes). The working tree was the final state.
- In full, I read `gate.rs`, `job.rs`, `catalogue.rs`, `view_query.rs`, `narrator.rs`, `tail.rs`, `transaction.rs`, `stories.rs`, `generation_status.rs`, the HTTP handlers, `walker_rows.rs` and `tests/STRATEGY.md`.
- I checked the plan, the target design (§1–§5), tickets 07 and 12, and the specs `dashboard.md`, `failure_display.md`, `reset.md` and `actions.md`.
- I do not report decisions that the tickets record with a reason. Examples: `replace` runs the create unit first, edit/switch/delete write the failure line, a stop does not cancel the model call, and the test-only reads.

## Standards

1. **Medium — `src/application/pipeline/job.rs`: `impl From<EngineError> for Failure`** (known)
   - Evidence: `fn from(error) -> Self { Self::missing(&error.to_string()) }`. `missing_record` puts that text into `named`, which is the part that the short line shows.
   - Why: every `?` on a storage or settings read in the Job gives "The data for this request is missing: Database error: …". The raw text goes into the status line. This breaks the rule in ticket 07 that "no call site can put raw text in a line", and the clamped-line rule of `failure_display.md`. The `gate.rs` refusals were fixed; the Job was not.
   - Fix: delete the blanket `From`. Map each read to `missing_record("<record>", Some(err))`, or use `GenerationFailure::from_engine_error` for internal errors.

2. **Medium — `src/application/generation/gate.rs`: `JobEntry` and `stop`/`replace`/`delete`, against the docs** (known)
   - Evidence: the uncommitted fix removed `CancellationToken`, `JobWriter::cancelled` and `Job::require_alive`. These docs still describe them:
     - `docs/diataxis/reference/game_flow.md` (§ job entries: "a cancel handle"; § Stop: "cancels its handle")
     - `docs/diataxis/explanation/rust_idioms.md` § "Cancellation of a Job"
     - `CONTEXT.md` (Job entry: "a cancel handle")
     - the Answer of ticket 12 ("`Job::require_alive` checks the cancel handle")
   - Why: the docs and the ticket describe a mechanism that does not exist. The behaviour is still correct: `run_step` → `JobWriter::step` gets the id check.
   - Fix: rewrite these four places. Say that the id check in `JobWriter::step` stops the Job before its next step.

3. **Low (judgement) — Duplicated Code / Feature Envy: the room-name and room lookups** (known)
   - Evidence: `map.get_room_by_id(id).map(|r| r.name.clone()).unwrap_or(id)` is in three places: `domain/model/utils/opening.rs::opening_message`, `gate.rs::JobWriter::opening_room_report` and `job.rs::Job::room_name`. The lookup `get_room_by_id(..).or_else(dynamic_rooms.get(..))` is in two places: `job.rs::Job::room_of` and `view_query.rs::get_current_room_view`.
   - Fix: put a `SwipeState::current_room(&MapDef)` method and a `MapDef::room_name(id)` method in `domain/model/`.

4. **Low (judgement) — Duplicated Code: `gate.rs::JobWriter::empties_the_dock`** (known)
   - Evidence: `matches!(kind, Kind::Narration | Kind::Event)` is the same body as `Kind::saves_a_narration` in `domain/model/agent.rs`.
   - Fix: call `kind.saves_a_narration()`.

5. **Low (judgement) — two status types: `domain/model/state/generation_status.rs::GenerationStatus` against `view_query.rs::Status`** (known)
   - Evidence: `get_debug_state` maps `Status` to `GenerationStatus` and `Step::default()`. So `/debug/state` reports `step: Narrating` for an idle or failed Game.
   - Fix: give the debug DTO `Option<Step>` (busy) and the failure. Delete `GenerationStatus`.

6. **Low — `adapters/driven/storage/transaction.rs`: `Tx::clear_dock` and `TxWrite::ClearDock`** (known)
   - Evidence: only `tests/storage/story_storage.rs:652` calls it. Production calls `put_dock(&DockRecord::default())`.
   - Why: production code that only a test calls must have a doc comment that names the stored fact (`tests/STRATEGY.md`). It has none, so it is speculative.
   - Fix: delete it, or use it in the stop, switch and delete units.

7. **Low — `transaction.rs`: `Storage::transaction` and `transaction_new_game`** (known)
   - Evidence: the two functions repeat the begin, `Tx::new`, closure and commit steps.
   - Fix: make one private helper that takes the optional Game-row write.

8. **Low (inferred) — blocking work on the async runtime: `Job::start`, `GameWriter::stop` and `busy_step`**
   - Evidence: async handlers call them directly. They take the one global `Mutex` and run SQLite reads under it, including `world_bundle_for` for a Retrigger. The status poll of every Game takes the same guard.
   - Fix: call them through `spawn_blocking`, as the Job body already runs.

9. **Low — `layout/handlers/endpoints.rs::generating_status_handler`** (known)
   - Evidence: `Err(e) => { tracing::error!(..); Html("idle") }`.
   - Why: a status read that fails shows "idle". The Game can be busy or failed.
   - Fix: answer a non-2xx, as 38.2 does for a poll that fails.

10. **Low — `gate.rs::JobWriter::opening_room_report`** (known)
    - Evidence: `.get_game(..).ok().flatten()?` and `.require_world(..).ok()?`. A read error silently gives no room report.
    - Fix: return `Result` and map the error to `Stopped::SaveFailed`.

11. **Low — `narration/narrator.rs::Narrator::assemble`** (known)
    - Evidence: the doc comment says "the trigger context keeps it for the event narration to re-use". `StoredTriggerContext` is gone, and only `narrate` calls the method. It is `pub`.
    - Fix: make it private and delete the stale sentence.

12. **Low (judgement) — Mysterious Name** (known)
    - `GenerationFailureKind::SceneMissing`: the design word is "invalid room", and the line says "The room … is not on the map".
    - `gate.rs::Units`: the struct opens units; it is not a set of units.
    - `CODING_STANDARDS.md` asks for names that map 1-to-1 to the docs.
    - Fix: `InvalidRoom`, `UnitOpener`.

13. **Low — tier-1 tests: `tests/http/support/http_fixtures.rs::story_of`** (known)
    - Evidence: many tests now count Narrations and Inputs with `story_of`, which calls `GameViewQuery::story_messages`. That read's doc comment justifies it only for "stored inputs and the room report of a Swipe that is not active".
    - Why: `tests/STRATEGY.md` allows a test-only read seam only for a fact that no production read returns. `GET /fragment/story-log` returns message counts.
    - Fix: widen the doc comment with a reason, or assert counts through the fragment.

14. **Low (judgement) — `tests/http/failure_display.rs::test_save_failure_clamps_with_the_save_line_and_saves_no_narration`** (known)
    - Evidence: the test comment says "The Input's save unit writes no dock, so the Narration's text save is the first write this override refuses."
    - Why: the test depends on which unit writes the dock record. A later change to the Input unit makes it fail for a reason that is not a regression.
    - Fix: inject on `InsertSwipe` after a count, or hold the Job and inject after the Input lands.

15. **Low — `application/games/active.rs::ActiveGame::set` doc** (known)
    - Evidence: "the newest Game of a World is picked by that column". `newest_game_of_world` orders by the newest `messages.timestamp` first, and `updated_at` is only the fallback. The uncommitted docs patch removed the same claim from the Reference doc.
    - Fix: align the comment.

## Spec

1. **High — `gate.rs::GameWriter::require_job_data` refuses a bare `/impersonate`** (code path known; player impact inferred, because no test covers it)
   - Evidence: `Action::parse("/impersonate")` → `Impersonate(None)` → `JobRequest::Impersonate(None)`. Then `if order.kind == Kind::Impersonate && order.inputs.impersonate_direction.is_none() { return Err(self.missing(game, "the impersonate direction", units)) }` gives a `500` and writes the failure line "The data for this request is missing: the impersonate direction."
   - Why:
     - `Action::Impersonate` documents "The optional payload steers…".
     - The slash menu offers `/impersonate` with no argument (`tests/browser/stub/slash_menu.rs`).
     - The old pipeline accepted `None`.
     - Design §4.5 asks for the direction only on a **Retry** of an Impersonate Input.
   - Fix: run the direction check only for `Place::NewSwipeOnLast`. Add an `actions.md` scenario and a test for a bare `/impersonate`.

2. **Medium — `job.rs` failure lines against design §4.7** (known)
   - Evidence: Standards item 1. Examples:
     - a read error in `world_bundle`, `game_row`, `tail`, `preset` or `budget` becomes the "missing data" kind with the error text as `<what>`;
     - a narration connection that is not configured (`settings.narration_connection()`) also reads as missing data.
   - Why: §4.7 keeps the line short and puts the raw text in the disclosure. An engine read failure is "internal error". Ticket 07 deviation 7 recorded this as a follow-up, but tickets 09–12 do not carry it.
   - Fix: as in Standards item 1. Add the follow-up to the map if it is deferred.

3. **Medium (inferred) — `job.rs::Job::failed` skips the Options run after a save failure that comes after the text save**
   - Evidence: `let stopped = matches!(ended.failure, Failure::Stopped(_)); if ended.narration_saved && !stopped && …`. `Stopped::SaveFailed` (for example, the state update fails) counts as "stopped".
   - Why: design §4.2 says "When a step fails … The Options run still runs when a Narration was saved". Only `EntryMoved` means that the Job no longer owns the Game.
   - Fix: test `Failure::Stopped(Stopped::EntryMoved)` only.

4. **Medium (inferred) — `dashboard.md` 39.4 / `races.rs` I2 do not cover all of §4.4**
   - Evidence: the Examples table has no stop at the **event Narration's analysis call**. Plan §9.1 I2 asks for "each stop point, for each kind".
   - Why: this is the one point where the pre-analysis state differs from the start state (§1: "its Trigger is also marked fired if it is one-shot"; R-16). No test pins that a stop there keeps the event text and marks the one-shot Trigger as fired.
   - Fix: add the row `event Narration | the analysis call | quantifying | the event Narration, with its one-shot Trigger fired`.

5. **Low (inferred) — `job.rs::Job::player_text` against §4.6**
   - Evidence: when `input` is `None`, the function returns `history.last()` of any type. For a Continue or a `/guide`, that is the previous **Narration**. The sibling function `previous_input` filters on `MessageType::Input`.
   - Why: the analysis reads "the player's text that led to it (the Input before it, if any)".
   - Fix: use `previous_input(history).unwrap_or_default()`.

6. **Low (inferred) — `job.rs::Job::end` uses the save-failed line for a refused state update**
   - Evidence: `Stopped::SaveFailed` always writes `GenerationFailureKind::SaveFailed`, "The new text could not be saved." When the state update fails, the text is saved and stays.
   - Why: the line is wrong for that case. §4.4 says the pre-analysis state stays.
   - Fix: let the state-update path map to the `Contradiction` line ("The text was saved, but the game state could not be updated.").

7. **Low (known) — `gate.rs::GameWriter::stop` when the dock read fails**
   - Evidence: `entries.remove(&game);` runs, then `self.dock_of(game)?` returns early. The Game is idle, but the stop unit (failure line none, notice none) does not land, and the handler answers "failed".
   - Why: §5 stop is one guard hold and one stop unit.
   - Fix: read the dock first, or remove the entry only after the unit lands. Alternative: write the notice-only change with a SQL update that needs no read.

8. **Low (known, the code predates the diff) — `chat_window.rs::reset_handler` with no active Game**
   - Evidence: `GameCatalogue::reset` returns `Validation("No active game")`, and the handler maps every `Err` to `internal_error(e.to_string())`, which is a `500`.
   - Why: design §4.1 and R-45 say no Game is a `400`.
   - Fix: return `e.into_response()`, which maps `Validation` to `400`.

**Checks with no finding:**
- The status order (busy, then the failure line, then idle) in `GameViewQuery::status`.
- The create unit (Game, empty dock record, opening with the World text, starting state and room report, in one `transaction_new_game`).
- The accept order (busy first, then `400`, then `500`, then the accept unit and the entry).
- The text-save dock rules (§3.2).
- The swipe switch and delete units.
- The event generation (movement taken out, Trigger ignored).
- The `Refusal` mapping to `200`/`400`/`500`.
- The rule rows 1, 2, 3, 6, 7, 13 and 18.

**Summary:** Standards has 15 findings; the worst is that `Job` puts raw engine error text into the failure line. Spec has 8 findings; the worst is that a bare `/impersonate` now answers `500` "missing data".