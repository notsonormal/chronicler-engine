# Thermo-nuclear review: the Rust rebuild, `bc7d6cd4...8036bbc3` plus the uncommitted fixes

**Verdict: not approved.** The rebuild deletes a lot of code: the action pipeline, the arrival opening, the snapshots and `GameState`. The writer rule in `gate.rs` is sound. But the two central files carry these problems:

- Some invariants live only in runtime checks, not in the types.
- The domain rules and the narrative work sit in the wrong layer.
- Several values that should come from one place are computed in two places, and some of those copies already differ.

I found no critical finding. I did not run any build or test. Each claim is labelled **known** (I read it in the tree) or **inferred** (I reasoned it).

## High

**H1. `domain/model/generation.rs:GenerationOrder` lets the kind and its data disagree, so runtime checks guard an invariant that a type could hold.** (known)
- Evidence: `kind: Kind` and `event: Option<EventRecord>` are separate fields. `inputs.impersonate_direction` is an `Option` too. So the rule "`Kind::Event` ⇔ an event record" is checked again and again:
  - `gate.rs:require_job_data` has `if order.kind == Kind::Event && order.event.is_none() { order.event = Some(...) }` and a matching check for an `Impersonate` with no direction.
  - `generation_record::message_kind` returns `Option<MessageKind>` only because the event can be missing.
  - `JobWriter::message_kind_of` turns `None` into `SaveFailed("the event record is missing")`.
  - `Job::apply_rules` and `Job::user_message` each have `.ok_or_else(|| Failure::missing("the event record"))`.
  - `user_message` has `impersonate_direction.clone().unwrap_or_default()`.
  - `Tail::retrigger` builds an order with `event: None`, and a later pass fills it.
- Why it matters: there are seven branches for a state the design calls impossible. Two of them write a misleading failure line.
- Fix: make the data part of the kind, for example `enum Generation { Narration, Event(EventRecord), Impersonate { direction: String }, UserRegen }`. Then `message_kind` is total, and every "missing event record" branch is deleted. `Tail::retrigger` would take the direction as a parameter, or return a pre-order that the planner completes once.

**H2. `job.rs:Job::run` and `Job::failed` track one fact with three booleans and two failure paths.** (known)
- Evidence:
  - `Generated.narration`, `Ended.narration_saved` and the local `saved_a_narration` all hold the same fact.
  - The patch `ended.narration_saved |= saved_a_narration` exists because of that split.
  - `into_event_failure()` is called at two sites: `Err(ended) if order.kind == Kind::Event` and the event branch.
  - `failed()` checks `options == OptionsRun::AfterNarration` again.
- Why it matters: this is the busiest flow in the engine, and it is hard to check by eye.
- Fix:
  - Move the event-failure wrap into `generate`, keyed on `order.kind`.
  - Write `run` as one loop: `next = first; while let Some(order) = next { … next = generated.fired.map(|r| event_order(r, at)) }`, with one `narration_saved` and one `Option<Failure>`.
  - After the loop: `if !stopped && (OnDemand || narration_saved) { options_step }`, then `end(failure)`.
  - `failed()` and `Ended::into_event_failure` disappear.

**H3. `job.rs:Job::apply_rules` holds game-rule logic that belongs to `domain/model/utils/game_rules.rs`.** (known)
- Evidence: the event branch builds `QuantifierResult { movement: MovementParseResult::default(), ..analysis.clone() }`. It calls `apply_game_rules` twice, ignores the Trigger the rules report, and sets `state.trigger_context = None`. The other branch sets `state.trigger_context = fired.as_ref().map(...)`.
- Why it matters: the module docs of `game_rules.rs` say the rules are "a pure function of the Swipe state". "An event moves nobody and fires no Trigger" is a game rule. Today it is outside that function and has no pure unit test.
- Fix: add `GameRuleRequest::EventNarration { analysis, trigger, text }`. Let the domain own both changes to `trigger_context`. `apply_rules` then becomes one call and a mapping.

**H4. The room report of the opening is computed in two layers, with two copies of the `count == 1` check.** (known)
- Evidence:
  - `gate.rs:JobWriter::save_text` has `match tail.count { 1 => self.opening_room_report(...) }`. `opening_room_report` reads `get_game` and `require_world` inside the hold, and drops their errors with `.ok()?`.
  - `job.rs:Job::is_the_opening` has `self.tail()?.count == 1`, and `apply_rules` then calls `room_name(...)`.
- Why it matters: the writer does narrative work and hides read failures. A change to the rule must be made in two files.
- Fix: add `Tail::is_opening()`. The Job already holds `bundle`, so it computes the text-save room report itself and passes it to `save_text(order, text, pre, room_report)`. Delete `opening_room_report` from `gate.rs`.

**H5. `gate.rs:GameWriter::opening_message` builds narrative content inside the writer, and the inputs are read three times.** (known)
- Evidence: `GameCatalogue::create_game` and `reset` read the World and the Persona. Then `GameWriter::create` and `replace` call `opening_message`, which reads `require_world`, `require_persona` and `list_characters`, and then runs `SwipeState::starting` and `opening::opening_message`.
- Why it matters: `gate.rs` should be "the writer rule and the guard". This one function is the reason the file imports `NpcCard`, `SwipeState` and `opening`.
- Fix: `GameCatalogue` builds the opening `Message` from the records it already read, and calls `game_writer.create(&new, &opening)`. `GameWriter` keeps only the unit.

**H6. Request planning makes up about 250 of the 910 lines of `gate.rs`.** (known for the size, inferred for the split)
- Evidence: the planning code is `plan_for`, `generation_plan`, `require_start`, `require_job_data`, `event_record`, `trigger_direction` and `story_has_text`.
  - `require_job_data` both checks and changes the plan (`plan: &mut JobPlan`), so a plan is built in two passes.
  - `generation_plan` and `require_start` take `&self` but do not use it.
  - The file grew from 145 to 910 lines in this range.
- Fix: read the inputs under the hold (the tail, the Game row, the start state and the World's Trigger direction). Pass them to a pure planner in `pipeline/`, for example `JobPlan::for_request(request, &PlanInputs) -> Result<JobPlan, PlanRefusal>`. `accept` then maps `PlanRefusal` to `Refusal`. The planner can then get pure unit tests, and `gate.rs` shrinks to the guard and its units.

**H7. Job failure lines name the wrong thing.** (known)
- Evidence:
  - In `job.rs`, `impl From<EngineError> for Failure` returns `Self::missing(&error.to_string())`. So every storage read failure becomes "The data for this request is missing: Configuration error: Failed to read …". The error text goes into both `named` and `raw`.
  - `gate.rs` does this correctly with `unreadable(name, error)` → `missing_record(name, Some(err))`.
  - `JobWriter::state_at`, `JobWriter::tail_of` and the dock read in `save_options` return `Stopped::SaveFailed`. So the user sees "The new text could not be saved." when a read failed.
- Fix:
  - Delete the blanket `From`. Each Job reader names its record: `.map_err(|e| Failure::unreadable("the story", e))`.
  - Split `Stopped` into `EntryMoved`, `WriteFailed` and `ReadFailed`. Or keep the reads out of `JobWriter` (see H4 and M3).

**H8. The `InMemory` storage backend is now half a backend.** (known)
- Evidence:
  - 14 production arms of the form `Backend::InMemory(_) => Err(no_in_memory(..))` in `stories.rs`, `docks.rs` and `transaction.rs`.
  - `touch_game_updated_at` returns `Backend::InMemory(_) => Ok(())`, a silent no-op.
  - `newest_game_of_world` has a separate in-memory sort rule.
  - `add_failure` swaps a placeholder `InMemory` backend in with `mem::replace`.
- Why it matters: every new storage method pays for a dead branch. A unit test on in-memory storage cannot create a Game, because `create` goes through `transaction`. The table in `tests/STRATEGY.md` still says that the unit tier fakes storage with "in-memory `Storage`".

| Option | Effect |
|---|---|
| Delete `InMemory`. Unit tests use `empty_sqlite_storage()`, which already exists. | About 14 dead arms and `in_memory_data.rs` go. There is one backend, and `with_backend_*` can hand out the connection. |
| Implement the story records in memory. | Two implementations of every query must stay equal. This is more code for no product value. |

Recommendation: delete it.

## Medium

**M1. The room lookup exists in four copies, and two of them miss dynamic rooms.** (known)
- Evidence: the copies are `game_rules::room_of` (private), `Job::room_of`, `GameViewQuery::get_current_room_view` (inline), and `Job::room_name` / `JobWriter::opening_room_report`. The last two read the map only and ignore `dynamic_rooms`.
- Fix: add `SwipeState::current_room(&MapDef) -> Option<&Room>` and `current_room_name`, and have all four sites use them.

**M2. A Retrigger's event direction skips the template render that an analysis Trigger gets.** (known divergence; inferred latent bug)
- Evidence:
  - `game_rules::analysis` builds `trigger_narration_prompt: TemplateVars::new(persona_name).render(...)`.
  - `gate.rs:trigger_direction` uses `trigger.narration.narration_prompt.clone()` without a render.
  - The old code stored the rendered prompt.
  - No Trigger in `data/characters` uses `{{…}}` today, so no user sees this yet (inferred).
- Fix: one domain function, for example `Trigger::direction(persona_name)`, used by both paths.

**M3. A Job reads the same records several times in one generation.** (known)
- Evidence:
  - `get_settings()` runs in `preset()`, `budget()`, `analysis_settings()` and `options_settings()`, each through its own call.
  - `tail()` runs in `place_before`, `user_message` (`UserRegen`), `is_the_opening` and `options_step`.
  - `game_row()` runs in `narrate_step` and `options_enabled`.
- Fix: load one local `GenerationInputs { game, settings, bundle, start, tail }` at the start of `generate` and pass it to each step. Rule 7 forbids record fields on the Job, not locals in one function (inferred). About eight small reader helpers then go.

**M4. `job.rs:preset_text` hides a read failure, and its own doc comment says it must not.** (known)
- Evidence: the doc comment says "A read failure is the Job's failure, not a silent no-preset". But it calls `pick(&self.storage, game, &settings)`, which is `Storage::active_quantifier_preset_id` or `active_options_preset_id`. Those run `self.get_game(..).ok().flatten()` and fall back to the registry with only a `warn`.
- Other problems:
  - The selection of a preset with a fallback is policy, and it lives in the storage adapter.
  - The Job already has the Game row.
  - `fn(&Storage, GameId, &AppSettings) -> String` is a function-pointer indirection over two fields.
- Fix: read `game.active_quantifier_prompt_preset_id` and `game.active_options_prompt_preset_id` from the row the Job has. Delete `preset_id_or` and both `active_*_preset_id` methods if nothing else calls them.

**M5. The `bool` parameter of `gate.rs:JobWriter::save_options` repeats `OptionsRun`.** (known)
- Evidence: every call site passes a value equal to `run == OptionsRun::AfterNarration`. `run()` passes `saved_a_narration`, which is `false` for `OnDemand` because `first` is `None`. `failed()` passes `true` only when the run is `AfterNarration`.
- Fix: `save_options(result, run: OptionsRun)` and `options_step(run)`.

**M6. Two places in the HTTP adapter start a Job, and the adapter holds the Job's internals.** (known)
- Evidence:
  - `actions.rs:dispatch_action` and `chat_window.rs:start_job` both call `Job::start(&state.game_writer, state.job_deps.clone(), game_id, request)`.
  - `AppState` carries `JobDeps`, and its `storage: Arc<Storage>` is `pub`. So the HTTP layer can reach `Storage` by going through `JobDeps`, which gets around the `server → storage` rule (inferred). The C5 ticket notes say test fixtures already did this.
- Fix: add an application service, for example `Jobs { writer, narrator, agent_runner, storage, tasks }` with `start(game, request) -> Result<(), Refusal>`. `AppState` then holds `jobs` and no `JobDeps`.

**M7. The new storage code wraps `rusqlite` errors in `EngineError::Config` by hand. `EngineError::Database(#[from] rusqlite::Error)` already exists.** (known)
- Evidence: 34 closures of the form `map_err(|e| EngineError::Config(format!(..)))` in `transaction.rs`, `stories.rs`, `docks.rs` and `records.rs`. `characters.rs` and `worlds.rs` use `EngineError::Database`.
- Why it matters: `GenerationFailure::from_engine_error` maps only `Database` and `Io` to `SaveFailed`. Any of these errors that reaches it becomes `Other`.
- Fix: use `?`, and add context only where the operation name matters.

**M8. The design rules depend on the build features, and the test machinery ships in production.** (known)
- Evidence:
  - `assert_state_consistency` is `#[cfg(feature = "diagnostics")]`, so the `Contradiction` line in spec 38.10 exists only with the default features. The C5 notes say: "With `--no-default-features` the rules step would accept the broken state instead".
  - `default = ["testing"]` builds `BackendKind::Test` and the write-failure injection into production. `JobTasks` says it is "built into production because the `testing` feature is a default one".
- Fix: make the consistency check unconditional, because it is cheap. Then remove `testing` from the default features, and turn it on for the test targets with `required-features` or dev-dependency features.

**M9. The docs still describe the cancel handle that the uncommitted fix removed.** (known)
- Evidence:
  - `CONTEXT.md:40`: "and a cancel handle".
  - `game_flow.md:23`: "the current `Step` and a cancel handle".
  - `game_flow.md:71`: "cancels its handle".
  - `rust_idioms.md` §"Cancellation of a Job": "A Job cancels through the handle in its job entry…".
  - Ticket 12 cites `Job::require_alive`, which does not exist.
- The removal itself is correct (known). `run_step → writer.step()` checks the id before every step, so the token was redundant.
- Fix: rewrite these four places to say "the id check before each step".

**M10. Some logic about kinds is written twice.** (known)
- Evidence:
  - `JobWriter::empties_the_dock` is `matches!(kind, Narration | Event)`, the same as `Kind::saves_a_narration`.
  - `JobPlan::first_step` repeats `Kind::step()`, with one extra rule for `None → Options`.
- Fix: use the canonical `Kind` methods. `first_step` becomes `self.first.as_ref().map_or(Step::Options, |o| o.kind.step())`.

**M11. The "one registry" invariant holds only by convention.** (known; inferred no impact at boot)
- Evidence: `run.rs:resolve_boot_game` builds a second `GameWriter::new(..)` and a second `ActiveGame` for boot. `GameWriter::new` is `pub`.
- Fix: resolve the boot Game through the wired graph. Or make the boot path a function that takes the wired `GameCatalogue`, and make `GameWriter::new` `pub(crate)`, called only from wiring.

**M12. `games/active.rs:ActiveGame::set` writes to the database and hides the failure.** (inferred)
- Evidence: `if let Err(e) = self.storage.touch_game_updated_at(game) { tracing::error!(..) }`, and the `InMemory` backend returns `Ok`.
- Fix: only `switch_game` needs the touch, so move it to `GameCatalogue::switch_game` and return its error. `ActiveGame` then becomes a plain `Mutex<Option<GameId>>` with no `Storage`.

**M13. Two enums describe the same status.** (known)
- Evidence: `view_query::Status { Busy, Failed, Idle }` and `GenerationStatus { Idle, Generating, Error }`. `GenerationStatus` is used only to map into `DebugStateView`, together with `Step::default()` as a placeholder when no Job runs.
- Fix: delete `GenerationStatus`. The debug DTO carries `Status` or a `step: Option<Step>`.

**M14. Several test files cross the 1k-line limit.** (known)
- Evidence:
  - `tests/http/actions.rs` went from 972 to 1006 lines.
  - `tests/http/races.rs` is new at 1145 lines.
  - `tests/http/swipe_new.rs` is still 1519 lines.
  - `narrator_tests.rs` is new at 1100 lines. It is a seam contract, so I waive it.
- Fix: split `races.rs` and `actions.rs` by spec section.

## Low

**L1. `transaction.rs:Tx::clear_dock` and `TxWrite::ClearDock` have only one caller, a test.** (known)
- Evidence: the caller is `tests/storage/story_storage.rs:652`. Production writes `put_dock(&DockRecord::default())` four times instead. `GameViewQuery::get_current_options` is also called only by tests, and it duplicates `dock().offered`. Neither has a doc comment that says only tests call it, as `tests/STRATEGY.md` requires.
- Fix: use `clear_dock` at the four production sites, or delete it. Delete `get_current_options`.

**L2. Three units repeat the same pair of writes.** (known)
- Evidence: `put_dock(default)` followed by `set_game_failure(None)` appears in `switch_swipe`, `delete_last` and `stop` (with variations).
- Fix: name this unit once.

**L3. `docks.rs:get` and `Tx::write_dock` skip the shared JSON helpers.** (known)
- Evidence: they call `serde_json::from_str` and `serde_json::to_string` directly, next to `from_optional_json` and `to_optional_json`.
- Fix: use `from_json` and `to_json` from `records.rs`.

**L4. `STORY_LIMIT = i32::MAX as usize` is a sentinel for "the whole story".** (known)
- Evidence: `render_story_log` reads the whole story, then calls `.take(50)` on a list that is oldest first. So it keeps the oldest 50 Messages (inferred). Check this against the intent of the story-log spec.
- Fix: use `entries(game, None, Some(limit))`, or add a separate `all_entries`.

**L5. `gate.rs:GameWriter::next_job_id` uses an `Arc<AtomicU64>` that the lock could hold.** (inferred)
- Evidence: the counter is incremented outside the registry mutex. `accept` takes that mutex anyway.
- Fix: keep the counter inside `JobRegistry`'s `Mutex`. This removes one shared primitive from the area that rule 18 watches.

**L6. Some types and items are unused or carry stale docs.** (known)
- `Step` is a fieldless enum but is not `Copy`, which causes `step.clone()` noise in `job.rs` and `busy_step`.
- `Narrated.backend_name` and `Narrated.model_name` have no production reader.
- The doc of `Narrator::assemble` says "the trigger context keeps it for the event narration to re-use", but no such reuse exists.
- `GameRuleRequest::NpcEvents` is called only by tests.

**L7. HTTP extractor and helper clean-up.** (known)
- `current_game.rs` writes the `async_trait` desugaring by hand. Axum 0.7 re-exports `axum::async_trait`, so this can be a plain `async fn`.
- `dispatch_with_status_headers` is `async` but never awaits anything.
- The DOC anchor of `current_game.rs` points to `storage.md`.

**L8. `endpoints.rs:generating_status_handler` hides read errors from the player.** (known)
- Evidence: on `Err`, it logs the error and answers `"idle"`, so a read failure shows as idle.
- Fix: return the error disclosure.

**L9. The two name scans for a new Game differ.** (known)
- Evidence: `GameCatalogue` checks against all Games in `create_game`, but only the same World without the current Game in `reset`. The doc comment admits that the two scans differ.
- Fix: decide on one uniqueness rule and put it in `domain/model/utils/game_name.rs`.

**L10. Guardrail rule 13 matches calls by the receiver's field name.** (known)
- Evidence: it matches on `narrator` and `agent_runner`, so a rename of a `JobDeps` field silently turns the rule off.
- Fix: at least add a walker test that fails when `JobDeps` loses those names.

## What would make this approvable

H1 through H4 together form one code-judo move:
1. Put the kind's data into the type (H1).
2. Move the event rules into `game_rules` (H3).
3. Let the Job compute the room report from one room lookup (H4, M1).
4. Rewrite `run` as one loop (H2).

Together these delete about 15 branches and three booleans in `job.rs` and `gate.rs`. H5 and H6 then cut `gate.rs` down to what its module docs promise: the guard and its units.