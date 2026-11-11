# Scan 2: implicit state machines in `src/application/`

Epistemic labels: **known** = I read the code path. **inferred** = reasoned from the code, not run. **unsure** = could not confirm.
I read every non-test file of `src/application/`. I also read the domain files that the application code leans on: `generation.rs`, `tail.rs`, `dock.rs`, `generation_status.rs`, `message_types.rs`, `agent.rs`. I also read `bootstrap/wiring.rs` for the provider resolver. I ran no code.
`src/application/` has no `Role health` or `Connection health` state of its own. `GameViewQuery::role_health` derives it on each read (finding 9).

---

## Already modeled (mapped briefly)

### A1. Busy registry and holder (the generation gate)
- Where: `src/application/generation/gate.rs:98-103` (`JobEntry`), `:119-161` (`Holder`), `:163-221` (`BusyRegistry`), `:250-266` (`Busy`).
- State variables: `Holders = HashMap<GameId, Holder>`. `Holder::Job(JobEntry{id, step})` or `Holder::Autoplay{run, progress, job: Option<JobEntry>}`. A missing key means idle.
- States: idle, Job running, Autoplay between Jobs (`job: None`), Autoplay with a Job running.
- Transitions (all under `BusyRegistry::hold`):
  - `accept` inserts `Holder::Job` (`:341-351`).
  - `reserve_for_autoplay` inserts `Autoplay{job:None}` (`:444-462`).
  - `accept_for_autoplay` inserts `Autoplay{job:Some}`, or removes the holder on Finish or NotAccepted (`:353-442`).
  - `JobWriter::step` changes `entry.step` (`:850-856`).
  - `JobWriter::drop` sets `job=None` for an Autoplay holder, or removes a `Job` holder (`:901-920`).
  - `end_autoplay` removes the holder (`:464-473`).
  - `stop`, `replace` and `delete` remove the holder (`:489-499`, `:576-593`).
- Explicitness: EXPLICIT (enum + one `hold`). Already modeled: yes.
- Risks seen:
  - (inferred) `save_config` (`:475-484`) refuses only when an Autoplay run holds the Game. A plain Job does not block it. See finding 8.

### A2. Job life (steps, failure line, entry check, Options run)
- Where: `src/application/pipeline/job.rs:142-420`, `domain/model/generation.rs:177-193`.
- Locals that hold the phase: `next: Option<GenerationOrder>`, `input: Option<String>` (taken on the first order), `narration_saved: bool` (`job.rs:148`), `failure: Option<JobFailure>`.
- Transitions: `run` (`job.rs:142-174`), `generate` (`:176-212`), `options_step` (`:334-375`), `end` (`:385-420`).
- Explicitness: EXPLICIT STATES (`Step`, `JobEnd`, `OptionsRun`), SCATTERED TRANSITIONS. The Step order is the code order of `generate` and `run`. No table names the legal order.
- Already modeled: yes.
- Risks seen:
  - (known) `end` (`job.rs:385-420`, the `Err(refused)` arm at `:399`): when `save_failure` fails with an error other than `EntryMoved`, the Job returns `JobEnd::Failed` but no failure line was stored. The Game then reads as Idle with no line. The Autoplay run ends with `JobFailed`.
  - (known) The `while let Some(order) = next.take()` loop (`job.rs:151`) has no counter. It ends only because `game_rules::apply` fires no Trigger for an event generation (`domain/model/utils/game_rules.rs:73-76`). That rule lives in another module.

### A3. One Autoplay run
- Where: `src/application/autoplay/driver.rs:21-141`. Locals `done: u32`, `after_job: bool` (`:69-70`). `RunEnd` enum.
- Cycle counting is in `decide` (`:109-120`). `done += after_job && story.ends_a_cycle()`.
- Explicitness: EXPLICIT (`RunEnd`, `Decision`, `AutoplayAccept`). Already modeled: yes.
- Risks seen:
  - none new. `Drop for AutoplayRun` (`:137-141`) ends the hold.
  - (known) `done` and `after_job` are loop locals. The gate stores a second copy as `AutoplayProgress`. The loop updates `done` from `progress.done` only on Accepted (`:90-92`).

### A4. Active-Game pointer
- Where: `src/application/games/active.rs:20,45`. `Mutex<Option<GameId>>`. `None` is "no active Game".
- Transitions: `PointerHold::set` / `set_new` (`:58-79`). Called from `catalogue.rs:30-36`, `:56-60`, `:63-70`, and `:259-262`.
- Explicitness: EXPLICIT (`PointerHold` guard). Already modeled: yes.
- Risks seen: none seen.

### A5. The two writes of a new Swipe state
- Where: `gate.rs:84-96` (`SavedText`), `:820-826` (`save_state_update`), `job.rs:195-212`.
- `SavedText` is a move-only token. `save_state_update` consumes it.
- Explicitness: EXPLICIT (type-state token). Already modeled: yes.
- Risks seen: none seen. The pre-analysis write is the text save.

---

## New findings (not in the modeled list)

### 1. Game failure line plus dock record: two stored status slots with scattered clear points
- Where: column `games.failure` (written by `set_game_failure`, `storage/transaction.rs:234`). Dock row `game_docks` (`DockRecord{offered: Vec<String>, failure: Option<OptionsFailure>}`, `domain/model/dock.rs:8-14`). `put_dock` is at `transaction.rs:300`, `put_options_failure` at `:307`.
- State variables: `game.failure: Option<GenerationFailure>`, `dock.offered: Vec<String>`, `dock.failure: Option<OptionsFailure>`.
- States:
  - Game line: none, or a line of one of 11 kinds.
  - Dock: (empty, none), (offered, none), (offered, Options failure). `put_options_failure` keeps the offered set (`transaction.rs:306`). The dock reads as "Options failure" in the third case.
  - Combined: failure line present while idle, the normal state after a failed Job.
- Transitions (event -> from -> to):
  - Job fails -> any -> Game line set (`job.rs:385-396` -> `gate.rs:828-834`).
  - Accept succeeds -> any -> Game line cleared (`gate.rs:700-704`, `accept_in_tx`). The dock is NOT touched.
  - Accept refused with a record error -> any -> Game line set (`gate.rs:648-658`).
  - Narration text saved -> any -> dock emptied (`gate.rs:793-818`, `:857-899`, `empties_the_dock`).
  - Options run offered -> any -> dock (offered, none) (`gate.rs:836-848`).
  - Options run unavailable -> any -> Options failure set, offered kept (`gate.rs:844-846`).
  - Stop -> any -> Game line cleared, offered kept, dock failure cleared (`gate.rs:489-499`).
  - `switch_swipe` and `delete_last` -> Game line cleared, dock emptied (`gate.rs:501-531`, `:547-574`).
  - `edit` -> neither is touched (`gate.rs:533-545`).
  - `create` -> dock emptied. `reset` replaces the Game.
- Guards / invariants: a stored Game line is only valid while no Job holds the Game. A Job that starts clears it first. `busy` shows before the line (`GameViewQuery::status`, finding 2).
- Explicitness: EXPLICIT STATES (the kinds and the Options), but the "which events clear which slot" rule is SCATTERED TRANSITIONS over 8 call sites. The two slots are cleared by different subsets of events.
- Already modeled: no. (`Busy` is modeled. This pair is not.)
- Risks seen:
  - (known) `accept` clears the Game line but not the dock Options failure. A new Job whose Options run does not follow (a failed Narration) leaves the old Options failure on screen. The line says one thing, the dock another.
  - (known) `edit` keeps a failure line standing, while `switch_swipe` and `delete_last` clear it. I found no comment that says this is a choice.

### 2. Status rule: busy registry (memory) + `games.failure` (SQLite) read in two steps
- Where: `src/application/games/view_query.rs:32-40` (`Status`), `:198-212` (`status`). `GenerationStatus` at `domain/model/state/generation_status.rs:10-18`.
- State variables: `writer.busy(game)` (memory, takes the registry mutex), then `game.failure` (storage). Two sources, two reads, no common lock.
- States: Busy(Job | Autoplay), Failed(line), Idle. `GenerationStatus::Busy` drops the Busy payload (`view_query.rs:127-134`).
- Transitions: none of its own. It derives from A1 and finding 1.
- Guards / invariants: "busy first, then failure, then idle." A failed Job writes the line before its `JobWriter` drops, so a reader that sees "not busy" also sees the line.
- Explicitness: EXPLICIT enum, derived. The rule is in one function.
- Already modeled: no.
- Risks seen:
  - (inferred) Race on the accept path. A reader calls `busy()`, gets `None`. An accept then clears the line and inserts the holder. The reader reads `games.failure` as `None` and returns Idle. One poll shows Idle while the Game is busy. The change counters (`raise_busy_changes`, `gate.rs:200-221`) should trigger a re-poll. I did not trace the HTMX side.
  - (known) `get_debug_state` reads the tail, the history and the status in separate reads (`view_query.rs:121-157`). The view is not a snapshot.

### 3. Quantifier retry loop: Confidence enum doubles as outcome status
- Where: `src/application/agents/quantifier/utils/orchestration.rs:94-158` (loop), `:162-195` (`process_quantifier_result`). Types in `quantifier/types.rs:8-24` (`QuantifierConfidence`), `:29-45`.
- State variables: loop counter `attempt` (literal `max_attempts = 2` at `:94`), `last_error: Option<String>`, `QuantifierConfidence {High, Medium, Low}`.
- States:
  - attempt 1, attempt 2.
  - Outcomes: parsed with High, parsed with Medium, parsed with Low, call failed.
  - "All attempts failed" is encoded as an empty `QuantifierReading` with `Low` (`:147-158`). That is the same value as "the text had no known Character ids".
- Transitions:
  - transport error on attempt 1 -> retry (`:139-142`).
  - Low confidence on attempt 1 -> retry (`:129-132`).
  - Low confidence or an error on the last attempt -> return as is or the empty Low.
  - Low -> `static_character_result` keeps the previous Characters present (`:187-190`, `:197-210`).
  - Hook-level fallback: the runner swallows an Agent `Err` and keeps the Characters present (`agents/runner.rs:70-79`).
- Guards / invariants: Low must mean "use the static Characters". Movement from a Low reading still passes (`process_quantifier_result` Low branch keeps `result.movement`). A call that failed has no movement.
- Explicitness: FULLY IMPLICIT for the retry (a counter in a `for` with `continue`). The confidence is an explicit enum, but one value serves two meanings.
- Already modeled: no.
- Risks seen:
  - (known) "Model call failed twice" and "model answered but named nobody" give the same `Low` reading. Nothing records that the Quantifier fell back.
  - (known) Role health (finding 9) reflects only transport failures. A Quantifier that always answers unparseable text shows as healthy.
  - (known) `max_attempts = 2` is a local literal here. Options uses a named constant (finding 4). The two policies are not tied together.

### 4. Options agent retry and the three-way end-of-Job result
- Where: `src/application/agents/options/utils/orchestration.rs:15-18` (`MAX_OPTIONS_ATTEMPTS = 2`), `:46-70` (loop), `:73-94` (`options_call_attempt`). Runner at `agents/runner.rs:84-98`. Job side at `job.rs:357-366` and `gate.rs:77-83` (`OptionsResult`).
- State variables: `attempt`, `last_error: Option<EngineError>`, `Result<Option<Vec<String>>>` from the attempt, `Result<Option<Vec<String>>>` from the runner, `AgentRunner.end_of_job: Option<Box<dyn EndOfJobAgent>>`.
- States:
  - attempt call: Ok(Some(options)), Ok(None) (call worked, zero parsed), Err.
  - runner: Ok(Some(offered)), Ok(None) (no Agent registered), Err.
  - Job: `Offered`, `Unavailable{reason, raw}`.
- Transitions: `Ok(None)` attempt -> retry once -> `Err(EngineError::Parse)`. `Ok(None)` at the runner -> `Unavailable("Options agent is not available")` (`job.rs:359-362`). `Err` at the runner -> `Unavailable("Options generation failed")` (`:363-366`).
- Guards / invariants: an `Offered` set is never empty. `parse_options` returns empty only as "none", and the attempt turns that into a retry. This holds by the shape of the parser, not by a type.
- Explicitness: FULLY IMPLICIT retry (a `for` loop with a counter). EXPLICIT result enum (`OptionsResult`).
- Already modeled: no. (The Options run as a Job step is modeled. The retry is not.)
- Risks seen:
  - (known) Two meanings of `Ok(None)` at two levels. At the attempt it means "zero options parsed". At the runner it means "no Agent registered". Both are written as `Option::None`.
  - (known) `AgentRunner` is built once with `Option` fields (`runner.rs:13-17`). A disabled or missing config leaves `None` for the life of the process. The 4 combinations of the 2 slots give different results: `after_narration` None gives a "keeping present" result (`runner.rs:70-73`), and `end_of_job` None gives "unavailable" (`runner.rs:84-88`).

### 5. `AgentRunner` slots filled by config order, last-wins
- Where: `src/application/agents/runner.rs:13-17,23-62`.
- State variables: `after_narration: Option<Box<..>>`, `end_of_job: Option<Box<..>>`.
- States: each slot empty or filled. 4 combinations. A second config of the same type replaces the first with a `warn` (`:44-48`, `:54-60`).
- Transitions: only during `from_configs` (build time). Later the runner is read only.
- Explicitness: FULLY IMPLICIT (a mode set by presence of two Options), but fixed at build.
- Already modeled: no.
- Risks seen:
  - (known) an empty `configs` slice swaps in the defaults (`:28-33`). A file with every Agent disabled is a non-empty slice and builds an empty runner. Both cases are legal and read differently ("no `[agents]`" versus "all off").
  - (known) the Agent name is a fixed string (`QuantifierAgent::from_config` ignores the config, `quantifier/agent.rs:35-43`), so the runner matches `config.agent_type` strings `"quantifier"` and `"options"` (a string used as a kind, signal 3).

### 6. Pause point (Autoplay test hold)
- Where: `src/application/autoplay/pause.rs:10-14` (`PauseSync{paused, released}`), `:20-27` (`armed: Mutex<HashMap<GameId, Arc<PauseState>>>`).
- State variables: map entry present or absent per Game; `paused: bool`, `released: bool`.
- States: unarmed (no entry), armed (entry, `paused=false`), paused (`paused=true`, `released=false`), released. The entry is removed at `point()` (`:30-46`), so "armed" ends when the loop reaches the point.
- Transitions:
  - `hold` -> unarmed -> armed (`:50-62`, panics if an entry is present).
  - `point` -> armed -> paused (removes the entry, sets `paused`, `:30-46`).
  - `release` -> any -> released (`:82-86`).
  - `PauseHandle::drop` -> release (`:90-94`).
- Guards / invariants: `released` after `paused` is a go. `released` before `paused` means the loop never blocks. A hold fires once.
- Explicitness: FULLY IMPLICIT (two bools plus presence in a map). Test support, but built into production ("the `testing` feature is a default one", `pause.rs:19-20`).
- Already modeled: no. (A3 mentions the call site only.)
- Risks seen:
  - (known) If the loop is never reached (the run ends before `point`), the entry stays armed. The next `hold` of that Game panics with the assert message.
  - (known) A hold that was never armed for a Game costs one mutex lock per loop turn (`point` always locks `armed`). Harmless.

### 7. `JobTasks` watch: a Vec of handles drained by `wait`
- Where: `src/application/generation/job_tasks.rs:9-11,14-18,34-51`.
- State variables: `handles: Mutex<Vec<JoinHandle<()>>>`. Handles that finished are dropped lazily at the next `watch` (`retain`).
- States: no handles, some running, some finished-but-not-removed.
- Transitions: `watch` adds and prunes. `wait_for_each` drains (`:34-51`).
- Explicitness: FULLY IMPLICIT (emptiness of a Vec is the "all Jobs ended" signal).
- Already modeled: no. (Test aid, but it is built into production.)
- Risks seen:
  - (known) `running()` counts unfinished handles. A drain in `wait_for_each` removes them from the Vec while they are still running, so `running()` can read 0 during a wait. Only a test observes this.
  - (inferred) `wait` with a timeout leaves the drained handles unawaited and unwatched. A later `wait` does not see them.

### 8. Game config writes: read, validate, then write in separate steps; gate checks Autoplay only
- Where: `src/application/games/catalogue.rs:148-156` (`set_posture`), `:158-190` (`switch_mode`), `:192-215` (`set_preset_selection`), `:217-221` (`save_config`). Gate side `gate.rs:475-484`.
- State variables: `Game.narrator_mode`, the four preset id slots, `narrative_perspective`, `narrative_tense`, `options_always_on` (columns of the Game row). Every Job step re-reads the row (`Job::narrate`, `hook_settings`, `options_enabled` in `job.rs`).
- States: the mode (Novel, InteractiveFiction) is the phase. The valid combinations are: each mode-tagged preset must allow the Game's mode (`catalogue.rs:207-212`).
- Transitions: `switch_mode` -> mode and presets change together in one `ConfigChange::Mode`. `set_preset_selection` -> one preset changes. Each does a read, then a check, then `save_config`, which takes the hold.
- Guards / invariants: "a preset allows the Game's mode." The check runs against a copy read before the hold.
- Explicitness: EXPLICIT enum (`NarratorMode`), but the legal moves are checked in the catalogue, outside the hold. They are not one transition function.
- Already modeled: no.
- Risks seen:
  - (inferred) TOCTOU: `set_preset_selection` validates against `game.narrator_mode` read earlier. A `switch_mode` between the read and `save_config` can leave a preset that the new mode does not allow. The route layer may serialise requests per page. I did not check.
  - (inferred) `save_config` refuses only for an Autoplay holder. A plain Job may run while the config changes. The Job re-reads the row at each of `narrate`, `analyse`, `options_step`. One Job can narrate with the old mode's preset and run the Quantifier with the new one. The `DockRecord` doc says the config routes write with no busy check, so this is by design. The cross-step effect is not stated anywhere I read.
  - (inferred) `rename_game` (`catalogue.rs:79-87`) does not take the pointer lock or the hold. `delete_game` does. A rename that races a delete of the same non-active Game updates a row that is going away. I did not check what the update returns for a missing row. **unsure**.

### 9. Role health (derived, with retention as the memory)
- Where: `src/application/games/view_query.rs:25-29` (`RoleHealth`), `:249-271` (`role_health`). Records written by `llm_recorder.rs:52-90`. Retention at `storage/llm_messages.rs` (`LLM_MESSAGE_RETENTION_LIMIT`).
- State variables: no state field. Health is the newest `llm_messages` row per `agent_name`. `backend_model: Option<String>`, `last_error: Option<String>`.
- States: never called (None, None), healthy (Some, None), failing (Some, Some(error)). The "recovered" state is the same as healthy: a newer success hides the old error.
- Transitions: an LLM call writes a row (`LlmCallRecorder::complete`, success `:78-90`, failure `:60-77`). The connection test writes none (`connection_test_service.rs:1-8`, `:21-24`).
- Guards / invariants: "newest" is `(created_at, id)`. The agent name must match `Role::agent_name()`.
- Explicitness: derived from stored rows. No enum. FULLY IMPLICIT.
- Already modeled: no.
- Risks seen:
  - (known) The recorder's `complete` can return `Err` from `save_fn` for a good call (`llm_recorder.rs:85`). The caller sees a failed call, and no row exists. Health does not move.
  - (known) A failed row is saved with `id: 0` and is saved only when the save succeeds (`:60-77`, warns on a save error at `:72`). A failure whose save fails is invisible to health.
  - (known) Impersonate calls use `role.agent_name()` of the narrator (`narrator.rs:115,136-140`), so Impersonate failures count against the Narrator role. The glossary says Impersonate has a slot and no Role.
  - (known) A Quantifier retry (finding 3) writes two rows. The newest decides, so a transport failure then a good retry reads healthy.
  - (known) An empty-text Narration (`narrator.rs:151-156`) is saved as a success row. Health shows healthy while the Job fails with `EmptyText`.
  - (inferred) Role `Trigger` and the event kind: `Kind::Event.role()` is Trigger, but retries that go through `Place::NewSwipeOnLast` use the same role. No problem seen.

### 10. Provider resolution on each call (no cached state, but a timing mode)
- Where: `src/application/llm_recorder.rs:16-56`. Built in `bootstrap/wiring.rs:40-69`.
- State variables: none held. The settings row is read at each call. `check_api_key_available` runs per call.
- States: resolved, unresolved (`Err`), mapped to `<unresolved: ..>` labels for logging (`:48-53`).
- Explicitness: pure function of settings at call time. Not a state machine, but two calls in one Job (the narrate call, then the Quantifier) can see different settings. Same cause as finding 8.
- Already modeled: no. Listed because Role health (9) and the "unresolved" label depend on it.
- Risks seen:
  - (known) `Narrator::backend_info` (`narrator.rs:90-95`) resolves the provider again, so the label can differ from the provider of the call that ran.

### 11. Prompt assembly: two-pass cut with an "only once" overflow guard
- Where: `src/application/prompting/assembler.rs:188-216` (`render_cut_prompt`), `src/application/prompting/token_budget.rs:46-65`, `domain/model/history_cut.rs`.
- State variables: `cut: HistoryCut` (mutable), `parts` rendered once, then maybe again, `excess_chars`.
- States: first cut (by tokens), guard-adjusted cut (by characters). The second render happens only if `kept_from` moved (`:205-213`).
- Transitions: `HistoryCut::of` -> `drop_to_free` -> render. One pass only.
- Explicitness: FULLY IMPLICIT. A fixed two-step order in a straight function.
- Already modeled: no.
- Risks seen:
  - (known) The guard runs once and `response_tokens` then re-checks (`token_budget.rs:67-90`). When the guard cannot free enough, the call ends in `ContextOverflow`, a different failure kind from the guard. The two checks use different units (characters, then tokens).

### 12. `PromptContext`: `guide` and `impersonate` fields that exclude each other
- Where: `src/application/prompting/assembler.rs:45,47` (public fields `guide: Option<String>`, `impersonate: bool`), builders `:130-143`. Use in `narrator.rs:126-131`.
- State variables: `guide`, `impersonate`.
- States: plain, guided, impersonate. "impersonate with a guide" is the invalid combination.
- Transitions: `with_impersonate(true)` clears `guide` (`:135-141`). `with_guide` sets it with no check. `Generation::Impersonate` sets one and `Generation::Narration` sets the other (`narrator.rs:126-131`).
- Explicitness: FULLY IMPLICIT. The rule holds only through the order of two builder calls and the match in `narrate`.
- Already modeled: no.
- Risks seen:
  - (known) The fields are `pub`. `with_guide(...)` after `with_impersonate(true)` gives the invalid pair. Today one caller exists and it avoids this.
  - (known) `render_persona_layer` drops the Persona when `impersonate` is true (`:415-418`). The guide layer stays rendered if `guide` is set.

### 13. Story shape: message `kind` + `message_type` + Swipe flags and optionals
- Where: `domain/model/state/message_types.rs:8-27` (`MessageType`, `MessageKind`), `domain/model/tail.rs:14-73`, `domain/model/generation.rs:94-136` (`write`), `:326-348` (`redo`). Application use: `autoplay/driver.rs:109-120`, `gate.rs:501-531`, `gate.rs:547-574`.
- State variables: `Message.message_type` (Narration/System/Input), `Message.kind` (TypedInput/ImpersonateInput/Narration/EventNarration(record)), `Tail.state_before_last: Option<..>`, `Tail.count`, `Swipe.{impersonated: bool, impersonate_direction, guide_text, event_header, maker}`, `SwipeState.event_record: Option<..>`.
- States (end of the story): no Message, opening only (`count == 1`, `state_before_last == None`), ends with a typed Input, ends with an Impersonate Input, ends with a Narration, ends with an event Narration. Each with or without a retrigger record.
- Transitions: Job writes (`save_input`, `save_text`), `delete_last`, `switch_swipe`, `edit`. The next Job request depends on the shape (`JobRequest::next_in_autoplay`, `JobPlan::redo`, `retrigger_record`).
- Guards / invariants: `message_type` is Input exactly when `kind` is TypedInput or ImpersonateInput. `impersonated` is true exactly when `kind == ImpersonateInput`. `event_header` is `Some` exactly when `kind` is EventNarration. `guide_text` belongs only to a Narration. The type allows other combinations.
- Explicitness: EXPLICIT STATES (`MessageKind`), SCATTERED TRANSITIONS. The shape rules are spread over `Tail` methods, `JobPlan`, `GenerationOrder::write`, `StoryEnd::ends_a_cycle` and the gate. `GenerationOrder::write` (`generation.rs:94-136`) is the one place that sets the pairs, which helps.
- Already modeled: no. (The Job plan is. The shape of the story is not.)
- Risks seen:
  - (known) `message_type` and `kind` are two fields with one meaning. A reader that trusts one and not the other can disagree. `awaits_narration` reads `message_type`. `retrigger_record` reads `kind`.
  - (known) `delete_last` at `count > 1` can delete down to the opening. `redo` of an opening uses `state_before_last.is_none()` as "this is the opening" (`generation.rs:330`). The opening is a Presence-as-phase case.
  - (inferred) `edit` writes `swipe.text` only, and leaves `guide_text` or `impersonate_direction` as is. A later Retry reuses the old direction.

### 14. Autoplay `Budget` and `AutoplayProgress` counters
- Where: `src/application/autoplay/budget.rs:7-34`, `gate.rs:108-117` (`AutoplayProgress{done, budget}`).
- State variables: `Budget(u32)` (1..=100), `done: u32`.
- States: counter 0..budget. Finished when `done >= budget` (`driver.rs:116`).
- Explicitness: a counter used as phase. The rule is in `decide` only. The gate stores the progress and applies no rule (`gate.rs:107` comment).
- Already modeled: yes, with A3. Listed for the counter signal.
- Risks seen: none seen. `AutoplayProgress::cycle()` is `done + 1`. It shows cycle N+1 even between Jobs after the last cycle ended, which is clipped by the Finish step. (inferred)

### 15. Catalogue create / reset: pointer move after a write that may fail
- Where: `catalogue.rs:42-62` (`create_game`), `:229-263` (`reset`). Gate `create` / `replace` at `gate.rs:576-586`.
- State variables: the pointer (A4), the new Game row, the old Game row, the old holder.
- Order: the pointer lock is taken, then `game_writer.create` or `replace`, then `set_new`. `replace` creates first, then deletes the old Game, then removes the old holder (`gate.rs:576-586`).
- Explicitness: a required call order held by comments and by the `PointerHold` guard.
- Already modeled: partly (A4).
- Risks seen:
  - (known) `replace` writes the new Game before the delete of the old one, in two transactions (`open_new_game`, then `tx.delete_game`). A crash or a failed delete leaves two Games and the pointer on the old one. The error returns, and the pointer does not move (`catalogue.rs:259-262`). (inferred) The new Game is then an extra saved Game with no error line beyond the one returned.

---

## Not state machines (checked and dropped)
`settings_service.rs`, `prompt_preset_service.rs`, `world_catalogue.rs`, `persona_catalogue.rs`, `text_check_service.rs` (a mode check `Disabled` returns early; no time-varying state), `errors.rs`, `ports/`, `prompting/sanitize.rs`, `prompting/prompt_merge.rs`, `prompting/builders/sections.rs`, `agents/*/prompt.rs`, `agents/options/utils/parser.rs`, `agents/quantifier/utils/parser.rs` (a fixed fallback order of three parse strategies, pure), `connection_test_service.rs` (one call; the `TokenBudgetSpent` error maps to success, a pure rule), `debug/dto.rs`.
- `PromptPresetService::next_copy_name` loops with a counter. It is a pure search over the stored names.

---

## Table of contents
name | file | explicitness
- Busy registry and holder (A1) | `src/application/generation/gate.rs` | EXPLICIT (already modeled)
- Job life (A2) | `src/application/pipeline/job.rs` | EXPLICIT STATES, SCATTERED TRANSITIONS (already modeled)
- One Autoplay run (A3) | `src/application/autoplay/driver.rs` | EXPLICIT (already modeled)
- Active-Game pointer (A4) | `src/application/games/active.rs` | EXPLICIT (already modeled)
- Two writes of a Swipe state (A5) | `src/application/generation/gate.rs` | EXPLICIT (already modeled)
- 1 Game failure line + dock record | `src/application/generation/gate.rs`, `storage/transaction.rs` | EXPLICIT STATES, SCATTERED TRANSITIONS
- 2 Status rule (memory + SQLite) | `src/application/games/view_query.rs` | EXPLICIT, derived from two sources
- 3 Quantifier retry + confidence as status | `src/application/agents/quantifier/utils/orchestration.rs` | FULLY IMPLICIT retry
- 4 Options retry + three-way result | `src/application/agents/options/utils/orchestration.rs`, `agents/runner.rs`, `pipeline/job.rs` | FULLY IMPLICIT retry
- 5 AgentRunner slots | `src/application/agents/runner.rs` | FULLY IMPLICIT (fixed at build)
- 6 Autoplay pause point | `src/application/autoplay/pause.rs` | FULLY IMPLICIT
- 7 JobTasks watch | `src/application/generation/job_tasks.rs` | FULLY IMPLICIT
- 8 Game config writes vs Jobs | `src/application/games/catalogue.rs`, `gate.rs` | EXPLICIT enum, SCATTERED checks
- 9 Role health | `src/application/games/view_query.rs`, `llm_recorder.rs` | FULLY IMPLICIT (derived)
- 10 Per-call provider resolution | `src/application/llm_recorder.rs`, `bootstrap/wiring.rs` | no stored state
- 11 Prompt overflow guard | `src/application/prompting/assembler.rs` | FULLY IMPLICIT
- 12 PromptContext guide / impersonate | `src/application/prompting/assembler.rs` | FULLY IMPLICIT
- 13 Story shape (kind, type, flags) | `domain/model/state/message_types.rs`, `tail.rs`, `generation.rs` | EXPLICIT STATES, SCATTERED TRANSITIONS
- 14 Budget counter | `src/application/autoplay/budget.rs` | counter (already modeled with A3)
- 15 Create / reset order | `src/application/games/catalogue.rs`, `gate.rs` | required call order
