# Anti-pattern report: Rust rebuild (`bc7d6cd4...8036bbc3` and the uncommitted fixes)

I found 25 findings: 3 high, 10 med and 12 low. I read the final working tree for every file I name.

## Findings

### High

**1. [Re-implementation] Every `EngineError` in the Job becomes "missing data", named by the error text**
- **File:** `src/application/pipeline/job.rs:87` `impl From<EngineError> for Failure`
- **Evidence:**
  ```rust
  impl From<EngineError> for Failure {
      fn from(error: EngineError) -> Self { Self::missing(&error.to_string()) }
  }
  ```
- **Why smell:** Two classifiers already exist:
  - `GenerationFailure::from_engine_error` maps typed errors. For example, `Database`/`Io` go to `SaveFailed` and `PresetNotFound` goes to `PresetMissing`.
  - `GameWriter::unreadable(game, "the World", &error, ..)` names the record and puts the error in `raw`.

  The Job bypasses both. A read failure in `world_bundle()`, `budget()`, `preset()` or `game_row()` puts the full error string in `named`. The status line then reads "The data for this request is missing: Config error: …". If the accept hits the same read failure, the line names "the World" instead. One fault gives two different lines.
- **Label:** known for the code. Inferred for the visible line.
- **Fix:** Remove the blanket `From`. At each read site, name the record, as the gate does (`Failure::unreadable("the World", err)`). Or use `GenerationFailure::from_engine_error`.

**2. [Re-implementation] The Retrigger direction is not rendered, but the direction of a fired Trigger is**
- **File:** `src/application/generation/gate.rs:573` `trigger_direction`, compared with `src/domain/model/utils/game_rules.rs` `analysis`
- **Evidence:**
  ```rust
  // gate.rs
  .map(|trigger| trigger.narration.narration_prompt.clone())
  // game_rules.rs
  trigger_narration_prompt: TemplateVars::new(persona_name).render(&trigger.narration.narration_prompt),
  ```
- **Why smell:** Two code paths build `EventRecord.direction`. One substitutes `{{user}}` and the other does not. `PromptAssembler` does not render `user_message`, so a placeholder goes to the model as raw text on a Retrigger.
- **Label:** known for the divergence. The visible bug is inferred and latent: no shipped `data/characters/*` prompt uses a placeholder.
- **Fix:** Build `EventRecord` in one domain function from `(TriggerRef, &NpcCard, persona_name)`. Call it from `TriggerMatch` and from `trigger_direction`.

**3. [False deduplication] The `after_narration: bool` flag repeats `OptionsRun`**
- **File:** `src/application/pipeline/job.rs:512` `options_step(run, after_narration)` and `gate.rs:744` `JobWriter::save_options(result, after_narration)`
- **Evidence:**
  ```rust
  if options == OptionsRun::OnDemand || saved_a_narration {
      if let Err(failure) = self.options_step(options, saved_a_narration) {
  ```
- **Why smell:** `/options` always plans `first: None`, so `saved_a_narration` is always false for `OnDemand`. For `AfterNarration`, the call runs only when it is true. `failed()` passes `true` only for `AfterNarration`. The bool therefore always equals `run == AfterNarration`. It is a second, unchecked copy of a value that has a type, and nothing stops the two from disagreeing later.
- **Label:** known.
- **Fix:** Pass `OptionsRun` to `save_options` and remove the bool.

### Med

**4. [Coincidental cohesion] `gate.rs` holds the lock and also request planning, refusal policy and record building**
- **File:** `src/application/generation/gate.rs` (910 lines)
- **Evidence:** `fn plan_for`, `fn generation_plan`, `fn require_start`, `fn event_record`, `fn trigger_direction`, `fn opening_message`, `fn text_swipe`, `fn message_kind_of`, `fn opening_room_report`, together with `JobRegistry`, `Units` and `JobWriter`.
- **Why smell:** The reads must happen under `hold`. The decisions do not need the lock: which order a `Retry` makes, and which `Swipe` fields a save copies. Most of that logic is a pure function of `(&JobRequest, Option<&Tail>)`. `JobRequest`, `JobPlan`, `OptionsRun` and `OptionsResult` are plan types, but they live in the writer file.
- **Label:** inferred.
- **Fix:** Move the planning into a pure domain function that returns `Result<JobPlan, PlanRefusal>`, next to `Tail::redo`/`retrigger`. Move the plan types to `domain/model/generation.rs` or to `pipeline/`. `gate.rs` keeps the reads, the lock, the refusal write and the units.

**5. [Re-implementation] Two places decide the room report of the opening**
- **File:** `gate.rs:708-712` `save_text` and `gate.rs:806` `opening_room_report`, compared with `job.rs:650` `is_the_opening` and `apply_rules`
- **Evidence:**
  ```rust
  let room_report = match tail.count { 1 => self.opening_room_report(&pre.movement.current_room_id), _ => None };
  ...
  None if self.is_the_opening(order)? => Some(Self::room_name(bundle, &state.movement.current_room_id)),
  ```
- **Why smell:** The writer reads the Game and the World inside the hold. It also drops read errors with `.ok()?`. The Job already has the `WorldBundle` and computes the same value again.
- **Label:** known.
- **Fix:** The Job computes the opening room report once and gives it to `save_text` as a parameter. `JobWriter` then writes ids and texts only.

**6. [Re-implementation] The room lookup and the room name exist three times each**
- **Files:**
  - Room lookup: `game_rules.rs:184` `room_of` (private), `job.rs:701` `room_of`, `view_query.rs` `get_current_room_view` (inline).
  - Room name: `job.rs:711` `room_name`, `gate.rs` `opening_room_report`, `opening.rs`.
- **Evidence:**
  ```rust
  map.get_room_by_id(room_id).or_else(|| state.movement.dynamic_rooms.get(room_id))
  ```
- **Why smell:** The three name versions ignore `dynamic_rooms`. `game_rules::room_name` includes them. The same room can therefore get two different names.
- **Label:** known.
- **Fix:** Add `SwipeState::current_room(&self, &MapDef) -> Option<&Room>` and one name-or-id helper. Every caller uses them.

**7. [Refactor-be-damned] `AgentConfig.hook` is required, but nothing reads it**
- **File:** `src/domain/model/agent.rs` `pub hook: Hook` and `src/application/agents/runner.rs:39`
- **Evidence:**
  ```rust
  match config.agent_type.as_str() { "quantifier" => runner.analysis.push(..), "options" => ..
  ```
- **Why smell:** The schema requires `hook` and `data/settings.json` sets it. No production code reads `.hook`. A config with `"agent_type":"options","hook":"analysis"` is accepted, and `hook` has no effect. The old `ExecutionPhase` was the dispatch axis. Its replacement does not dispatch anything.
- **Label:** known.
- **Fix:** Dispatch on `hook`, or remove the field from the struct and the schema.

**8. [Refactor-be-damned] `StoryRow` copies `MessageEntry`, and `StoryEntry` has two conversion sources**
- **File:** `src/domain/model/state/message_types.rs` `StoryRow`, `impl From<&StoryRow> for MessageEntry`, and `agent.rs` with two `From` impls for `StoryEntry`
- **Evidence:** 8 of the 9 `StoryRow` fields are the same as in `MessageEntry`. 4 of the 5 production callers of `entries()` (`view_query.rs:131,176`, `job.rs:278,369`) convert to `MessageEntry` at once. `StoryRow.kind` is read only by `tests/storage/story_storage.rs:433`.
- **Why smell:** This is the corpus's `MessageEntry` DTO smell again. `analyse_step` builds `StoryEntry` from `MessageEntry`. `options_step` builds it from `StoryRow`. The same window takes two paths.
- **Label:** known.
- **Fix:** Make `entries()` return `MessageEntry`, or make `PromptContext` take `&[StoryRow]`. Then delete one type and one `From`.

**9. [False deduplication] The analysis result makes a round trip from `QuantifierResult` to `StatePatch` and back**
- **File:** `quantifier/agent.rs` `run` and `runner.rs` `analysis`
- **Evidence:**
  ```rust
  confidence: Confidence::from(result.npcs.confidence),   // agent
  result.npcs.confidence = confidence.into();             // runner
  ```
- **Why smell:** The new runner continues the corpus's `Confidence`/`QuantifierConfidence` pair. The round trip silently drops `movement_type` and `movement.confidence`. `apply_game_rules` reads only the `StatePatch` fields.
- **Label:** known.
- **Fix:** `AgentRunner::analysis` returns `StatePatch`, and `GameRuleRequest::Analysis` takes it.

**10. [Refactor-be-damned] `Message.message_type` repeats `Message.kind`**
- **File:** `src/domain/model/message.rs` `Message`, `gate.rs:676`, `generation_record::message_type`
- **Evidence:**
  ```rust
  Message::new(MessageType::Input, MessageKind::TypedInput, swipe.clone())
  ```
- **Why smell:** The type is a pure function of the kind. Nothing in production produces `MessageType::System`, but four consumers still match it. `save_input` hardcodes the pair, and `opening.rs` goes through `generation_record`. The two columns can disagree.
- **Label:** known.
- **Fix:** Add `MessageKind::message_type()`. Remove the field, or compute the column at insert time.

**11. [Re-implementation] `JobWriter` has its own reads, with a different failure class**
- **File:** `gate.rs:776,786,796` (`dock`, `state_at`, `tail_of`), compared with `job.rs:731,739` (`tail`, `start_state`), `GameWriter::dock_of` and `GameViewQuery::dock`
- **Evidence:**
  ```rust
  Ok(None) => Err(Stopped::SaveFailed(EngineError::Validation("the start state is missing".to_string()))),
  ```
- **Why smell:** A missing start state seen by `JobWriter` becomes `SaveFailed`, and `Job::end` writes "The new text could not be saved." The same condition seen by `Job::start_state` writes `MissingData`. `docks().get(g)?.unwrap_or_default()` appears three times.
- **Label:** known.
- **Fix:** Add `DockReads::get_or_default`. Give the `JobWriter` misses a distinct variant, so `Job::end` can write the missing-record line.

**12. [Refactor-be-damned] The `TxToken` capability has a hole**
- **File:** `transaction.rs` (`_token: TxToken`), `src/test_support/fixtures.rs:511` and `walker_rows.rs` rule 3
- **Evidence:**
  ```rust
  pub fn tx_token() -> TxToken { TxToken::new() }   // test_support, built by default
  ```
- **Why smell:** The doc says "constructor is crate-only, so a unit opens in `gate.rs` and nowhere else". `pub(crate)` does not limit callers inside the crate. `Cargo.toml` has `default = ["testing"]`, and rule 3 matches only `TxToken::new`. So any `src/` file can call `crate::test_support::tx_token()`, and no guardrail fires.
- **Label:** known.
- **Fix:** Add `tx_token` to rule 3's callees, with `allowed: ["test_support/"]` and the storage seam. Alternatively, make `testing` a non-default feature.

**13. [Re-implementation] Two places mark a one-shot event Trigger as fired**
- **File:** `swipe_state.rs` `pre_analysis` and `game_rules.rs` `trigger_narration` (through `Job::apply_rules`, Event branch)
- **Evidence:**
  ```rust
  if !record.trigger.trigger_repeat { state.npc_encounter_log.mark_trigger_fired(..) }
  ```
- **Why smell:** One rule has two writers: one for the pre-analysis state and one for the analysed state. A change to one leaves the other stale.
- **Label:** known.
- **Fix:** Keep the mark only in `game_rules`. `pre_analysis` calls `apply_game_rules(TriggerNarration)`.

### Low

**14. [Re-implementation] The `Kind` maps exist twice**
- **Evidence:**
  - `JobPlan::first_step` (`gate.rs:88`) is `Kind::step`.
  - `JobWriter::empties_the_dock` (`gate.rs:894`), `matches!(kind, Kind::Narration | Kind::Event)`, is `Kind::saves_a_narration`. The doc of `saves_a_narration` already says "the text save empties the dock".
- **Label:** known.
- **Fix:** Call the `Kind` methods.

**15. [Refactor-be-damned] Only tests use `Tx::clear_dock` and `TxWrite::ClearDock`**
- **Evidence:** The only caller is `tests/storage/story_storage.rs:652`. `gate.rs` writes `tx.put_dock(&DockRecord::default())` four times.
- **Label:** known.
- **Fix:** Use `clear_dock` at those four sites, or delete it and the `TxWrite::ClearDock` key. Check the `TxWrite::PutDock` injection in `tests/http/failure_display.rs:484` first.

**16. [Re-implementation] Two hand-written `serde_json` calls remain**
- **Evidence:** `Tx::write_dock` uses `serde_json::to_string(&record.offered)`. `DockReads::get` uses `serde_json::from_str(&row.0)`.
- **Why smell:** Ticket 07 item 7 says `to_json`/`from_json` replaced every hand-written call in the new code.
- **Label:** known.
- **Fix:** Use `to_json("offered set", ..)` and `from_json`.

**17. [Re-implementation] `Storage::transaction` and `transaction_new_game` repeat the same body**
- **Evidence:** Both repeat the same begin, `Tx::new`, `f`, commit and `InMemory` refusal (about 10 lines each).
- **Label:** known.
- **Fix:** Make one private `open_unit(game_source, f)`.

**18. [Refactor-be-damned] `from_config(_config: &AgentConfig, ..)` never reads its parameter**
- **Evidence:** Both `QuantifierAgent` and `OptionsGeneratorAgent` ignore it. They return `Result`, but it is never `Err`. This diff renamed them and kept the dead parameter.
- **Label:** known.
- **Fix:** Use `new(recorder) -> Self`, or read `config.name`.

**19. [Refactor-be-damned] `Narrated` and `Narrator::assemble` carry unused surface**
- **Evidence:**
  - Nothing reads `Narrated.backend_name` or `model_name`. `job.rs:405` keeps only `.text`.
  - `Narrator::assemble` is `pub` but has no outside caller. Its doc ("the trigger context keeps it for the event narration to re-use") is stale.
- **Label:** known.
- **Fix:** Return `String` and make `assemble` private.

**20. [Refactor-be-damned] Production code has variants and fields that only tests read**
- **Evidence:** `GameRuleRequest::NpcEvents`, `GameRulesOutcome::unknown_location` and `trigger_narration_landed`. Only `game_rules_tests.rs` uses them.
- **Label:** known.
- **Fix:** Delete them, or test through `Analysis`.

**21. [Refactor-be-damned] Two `storage` handles can diverge**
- **Evidence:** `JobDeps.storage` repeats the storage that `JobWriter` holds through `GameWriter`. `wiring.rs` `build_app_graph_for_tests` must re-point it: "The override's Job writes through this graph's storage".
- **Label:** known.
- **Fix:** `Job` reads through `JobWriter`'s storage, or `JobDeps` holds the `GameWriter`.

**22. [Refactor-be-damned] The test-only `JobTasks` sits on the production spawn path**
- **Evidence:** `job.rs` calls `tasks.watch(handle)` on every Job. The doc says: "built into production because the `testing` feature is a default one."
- **Label:** known.
- **Fix:** Put it behind a non-default feature, or use it for graceful shutdown so that production uses it.

**23. [Refactor-be-damned] Functions named like constructors write a row**
- **Evidence:** `GameWriter::missing`, `unreadable` and `save_failed` (`gate.rs:394-420`) call `refuse`, which opens a unit. `ok_or_else(|| self.missing(..))` hides a DB write.
- **Label:** known.
- **Fix:** Rename them `refuse_missing`, `refuse_unreadable` and so on.

**24. [Re-implementation] `ScriptedModel::ModelRole` copies `Role`**
- **Evidence:** Four variants map to `Role`'s four. The only addition is `AGENT_DIALOGUE`. `of_agent_name` reverses `Role::agent_name`.
- **Label:** known.
- **Fix:** Key the replies by `Role` and map `dialogue` to `Role::Narrator`.

**25. [Refactor-be-damned] Two walker options have no production row**
- **Evidence:** `TypeCheckPosition::Parameters` and `CallCountRow.file: Some(..)` occur only in `walker_tests.rs:424,437,552`. Every real row uses `Fields` or `file: None`.
- **Label:** known.
- **Fix:** Delete them until a row needs them.

## Cross-cutting notes

- **Many classifiers for one failure.** `From<EngineError> for Failure`, `GameWriter::unreadable`, `GenerationFailure::from_engine_error` and `Stopped::SaveFailed(Validation)` all decide the failure line. Findings 1 and 11 come from this split. One rule for "read failed, named X" would remove both.
- **Rule copies between the Job and the writer.** These rules each exist in two or three places: the room lookup, the room name, the opening report, the Kind maps and the dock read (findings 5, 6, 11, 14). Each is small, but together they show that `job.rs` and `gate.rs` grew side by side without shared domain helpers.
- **Two guardrail limits (inferred).** Rule 18 counts `hold` calls per function. It cannot see a helper that calls `hold` inside another function's hold closure, which is a deadlock with a non-reentrant `Mutex`. I found no such case now. Rule 13 matches calls by receiver name (recorded), so a renamed local defeats it.

## Checked and clean

- **`StoryReads`/`DockReads` handles, the boot's own `GameWriter`, `GameViewQuery::story_messages`, `Narrator::recorder`:** the recorded reasons hold. I report nothing on them.
- **`JobRegistry`/`Units`/`hold`:** one lock, and every unit opens in a hold. The shape is sound.
- **`TxWrite`:** it extends the existing failure-injection pattern, not a new anti-pattern. I report nothing.
- **`Tail`, `GenerationOrder`, `Place`:** these are narrow, ids-only domain values. Apart from findings 2 and 14, I found no duplicates.
- **Rule 7 type check:** `Job` holds no record field.
- **Re-implementation search:** I searched for existing versions of `Tail`, `GenerationOrder`, `DockRecord`, `SwipeState::starting/pre_analysis`, `opening_message`, `JobTasks`, `ActiveGame` and `CurrentGame`. I found none, except the hits above.