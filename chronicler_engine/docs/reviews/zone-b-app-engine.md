# Zone B: application/bootstrap/engine — Abstraction Anti-Pattern Report

## Summary (12 findings)

- **High:** 2 — dead enum variant forcing side-channel errors; arrival task re-implements entire pipeline.
- **Med:** 7 — dummy arg, god-method pipeline run, 14 identity service wrappers, param-heavy phase methods, retry duplication, error-return helper masking control flow, spawn-blocking duplication.
- **Low:** 3 — stateless query utility struct, trigger-eval mixes evaluation with CRUD helpers.

All evidence comes from files listed in scope; no test files reviewed.

---

## Findings

### B1. [Premature generalization] Dummy arg `_player_name` in `execute_action_impl`

- **File:** `src/application/action_pipeline/actions.rs:13`
- **Evidence:**

  ```rust
  pub fn execute_action_impl<B: ActionPipelineBackend>(
      backend: &B,
      ctx: GameServiceContext,
      input: String,
      _player_name: String,
  ) {
      let mut state = load_or_fresh(&ctx);
      state.narrative.last_trigger = None;
      let pipeline = ActionPipeline::new(backend, &ctx);
      ...
  }
  ```

- **Why smell:** Parameter exists in public signature but is never read. Caller (`application_service.rs`) passes `game_state.player.sheet.name.clone()` for no reason. Suggests abstraction was designed for a player-name concern that was never implemented or was removed.
- **Severity:** med
- **Proposed fix:** Remove `_player_name` from signature and all call sites.

---

### B2. [False abstraction] Dead `ActionOutcome::Error` variant

- **File:** `src/application/action_pipeline/pipeline.rs:28-31`
- **Evidence:**

  ```rust
  pub enum ActionOutcome {
      Completed,
      #[allow(dead_code)] // Errors flow through GenerationStatus::Error
      Error { message: String },
      Cancelled,
  }
  ```

- **Why smell:** Enum pretends to model pipeline results, but errors are punted into `GenerationStatus::Error` inside `GameState`. Variant is never constructed, kept alive only by `#[allow(dead_code)]`. This is a wrong abstraction: the pipeline uses `Result` for cancellations but smuggles real failures through a side-channel (`state.status`).
- **Severity:** high
- **Proposed fix:** Delete `Error` variant and make pipeline return `Result<(), ActionOutcome>` where genuine failures become `Err(EngineError)` or a dedicated `PipelineError`; stop using `state.status` as an error accumulator.

---

### B3. [Helper smell / god-function] `run_from_input` is a monolithic imperative blob

- **File:** `src/application/action_pipeline/pipeline.rs:55-120`
- **Evidence:**

  ```rust
  pub fn run_from_input(&self, mut state: GameState, input: String) -> PipelineResult<()> {
      ...
      state = self.phase_pre_main_snapshot(state)?;
      let (mut state, narration_text, backend_name, model_name) = self
          .map_cancelled(self.phase_narrate(state, &input, &world, &map, &player, &all_npcs))?;
      ...
      let turn_result = match Self::phase_engine_commit(&state, &narration_text, &quantifier_result) { ... };
      ...
      if let Some(request) = trigger_request {
          match self.phase_trigger_continuation(next_state, &request) { ... }
      }
      self.phase_finalize(&mut next_state);
      Ok(())
  }
  ```

- **Why smell:** 60+ lines with mutable `state` / `next_state`, interleaving snapshotting, LLM calls, engine commit, trigger continuation, and finalize. Phases are extracted as methods but orchestration is one giant script. Hard to unit-test phases in isolation; any change risks breaking ordering.
- **Severity:** med
- **Proposed fix:** Introduce a small state machine or explicit `PipelineStep` enum so each transition is declarative and separately testable. Do not simply extract more helpers (refactor-be-damned).

---

### B4. [Helper smell / utility class] `DefaultApplicationService` contains 14 identity wrappers

- **File:** `src/application/application_service.rs:145-215`
- **Evidence:**

  ```rust
  // TODO(#tech-debt): Worlds CRUD methods are pure passthroughs to GameLifecycleService.
  // Combined with lifecycle layer, this creates 14 identity wrappers for zero logic.
  pub fn list_worlds(&self, ctx: GameServiceContext) -> Result<Vec<WorldCard>, ApplicationError> {
      self.lifecycle.list_worlds(ctx)
  }
  pub fn get_world(...) { self.lifecycle.get_world(...) }
  pub fn create_world(...) { self.lifecycle.create_world(...) }
  ...
  ```

- **Why smell:** Service forwards calls verbatim to an inner service, adding no validation, logging, or mapping. The TODO itself admits the abstraction is worthless. Layers stack up purely for "architecture ceremony."
- **Severity:** med
- **Proposed fix:** Flatten `GameLifecycleService` into `DefaultApplicationService`, or expose the inner service directly where callers need it.

---

### B5. [Helper smell / parameter accumulation] `phase_narrate` carries 6 positional args

- **File:** `src/application/action_pipeline/phases.rs:44`
- **Evidence:**

  ```rust
  #[allow(clippy::too_many_arguments)]
  pub(super) fn phase_narrate(
      &self,
      mut state: GameState,
      input: &str,
      world: &WorldCard,
      map: &MapDef,
      player: &PlayerCard,
      all_npcs: &[NpcCard],
  ) -> PipelineResult<(GameState, String, String, String)> { ... }
  ```

- **Why smell:** Clippy suppression is a red flag. `world`, `map`, `player`, `all_npcs` are all immutable read-only inputs that never change during a pipeline run, yet they are threaded through every phase call individually. This is a missing domain concept (a `PipelineInputContext` or `WorldView`).
- **Severity:** med
- **Proposed fix:** Group immutable inputs into a `PipelineInputs<'a>` struct and pass one reference.

---

### B6. [Helper smell / parameter accumulation] `build_trigger_request` carries 7 positional args

- **File:** `src/application/action_pipeline/phases.rs:199`
- **Evidence:**

  ```rust
  #[allow(clippy::too_many_arguments)]
  pub(super) fn build_trigger_request(
      &self,
      state: &GameState,
      narration_text: &str,
      world: &WorldCard,
      player: &PlayerCard,
      all_npcs: &[NpcCard],
      trigger_match: &TriggerMatch,
  ) -> Option<StoredTriggerContext> { ... }
  ```

- **Why smell:** Same as B5 but worse (7 args). `world`, `player`, `all_npcs` are the same read-only trio passed again. `state` already contains `world`, `map`, `player`, `npcs`, yet the method receives them separately, suggesting the abstraction doesn't trust its own domain model.
- **Severity:** med
- **Proposed fix:** Reuse the same `PipelineInputs` context; derive `world`/`player`/`npcs` from `state` when possible.

---

### B7. [Refactor-be-damned extraction] `retry_event_continuation` re-implements the trigger branch of the pipeline

- **File:** `src/application/action_pipeline/retry.rs:118-135`
- **Evidence:**

  ```rust
  pub(crate) fn retry_event_continuation<B: ActionPipelineBackend>(...) -> ActionOutcome {
      let pipeline = ActionPipeline::new(backend, ctx);
      let mut state = match pipeline.phase_trigger_continuation(state, &trigger) {
          Ok((s, continuation_text)) => {
              if !continuation_text.is_empty() {
                  pipeline.reconcile_post_trigger_npcs(s, &input_text, &continuation_text)
              } else { s }
          }
          Err(outcome) => return outcome,
      };
      if let Some(target) = state.narrative.retry_target.take() {
          state.narrative.history.append(target);
      }
      pipeline.phase_finalize(&mut state);
      ActionOutcome::Completed
  }
  ```

- **Why smell:** This is a miniature copy of the trigger-handling block inside `run_from_input` (pipeline.rs). Instead of teaching the pipeline to replay from a snapshot, the retry module extracts the same steps and reassembles them by hand. Any change to trigger finalization or NPC reconcile must be made in two places.
- **Severity:** med
- **Proposed fix:** Parameterize `run_from_input` (or a new `run_from_state`) so retry can feed an existing state + trigger context into the same orchestration.

---

### B8. [False deduplication / refactor-be-damned] `ArrivalTaskContext` is a one-off pipeline reimplementation

- **File:** `src/bootstrap/init_game.rs:55-175`
- **Evidence:**

  ```rust
  struct ArrivalTaskContext {
      storage: Arc<...>, world: Arc<...>, map: Arc<...>, player: Arc<...>,
      npcs: Arc<...>, room_id: String, arrival_preset: Option<PromptPreset>,
      response_length: String, max_context_tokens: u32, max_tokens: Option<u32>,
      nearby_npcs: Vec<NpcCard>, all_npcs: Vec<NpcCard>, connection: Connection,
  }
  impl ArrivalTaskContext {
      fn run(self) {
          let mut state = match self.storage.load_latest_snapshot() { ... };
          ...
          let context = make_prompt_context(...);
          let narration = match self.arrival_preset.as_ref() { ... };
          match narration { ... }
          self.storage.save_snapshot(...);
      }
  }
  ```

- **Why smell:** 13-field ad-hoc struct exists solely for `spawn_arrival_task_if_needed`. It manually loads snapshots, assembles prompts, calls the LLM backend, and persists state — all logic that already lives in the action pipeline. Rather than reuse the pipeline, the code duplicates it under a new abstraction.
- **Severity:** high
- **Proposed fix:** Model arrival as a `FreeAction("")` or dedicated `Action::Arrive` and feed it through `ActionPipeline`. If that is impossible, extract a shared `NarrationDriver` that both the pipeline and arrival task call.

---

### B9. [Refactor-be-damned extraction] `error_return` helper stuffs errors into state instead of fixing error flow

- **File:** `src/application/action_pipeline/phases.rs:33-39`
- **Evidence:**

  ```rust
  fn error_return(
      &self,
      mut state: GameState,
      msg: String,
  ) -> PipelineResult<(GameState, String, String, String)> {
      state.narrative.input_buffer.status = GenerationStatus::Error(msg);
      self.persist(&state);
      Ok((state, String::new(), String::new(), String::new()))
  }
  ```

- **Why smell:** Name says "error" but returns `Ok`. It buries the failure in `state.status`, forcing callers to ignore the empty strings and inspect a side-channel. This is a symptom-relief extraction: rather than making the pipeline use `Err` for errors, a helper was created to hide them inside success variants.
- **Severity:** med
- **Proposed fix:** Return `Err(ActionOutcome::Error(...))` or a real `EngineError`, and let the top-level runner decide how to render it.

---

### B10. [Premature extraction / duplication] `retry` and `retrigger` duplicate spawn-blocking boilerplate

- **File:** `src/application/message_editing.rs:124-160`
- **Evidence:**

  ```rust
  pub fn retry(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
      ...
      let game_service = Arc::clone(&self.game_service);
      let ctx_clone = ctx.clone();
      tokio::task::spawn_blocking(move || {
          if ctx_clone.cancel_token.is_cancelled() { return; }
          retry_last_response_impl(&*game_service, ctx_clone);
      });
      Ok(())
  }
  pub fn retrigger(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
      ...
      let game_service = Arc::clone(&self.game_service);
      let ctx_clone = ctx.clone();
      tokio::task::spawn_blocking(move || {
          if ctx_clone.cancel_token.is_cancelled() { return; }
          retrigger_event_impl(&*game_service, &ctx_clone);
      });
      Ok(())
  }
  ```

- **Why smell:** Two methods with identical shape: clone `Arc`s, clone `ctx`, check `cancel_token`, spawn blocking. The duplication is hidden inside a "service" layer that adds no behavior beyond the spawn. `DefaultApplicationService::process_action` repeats the same pattern again.
- **Severity:** med
- **Proposed fix:** Extract a single `spawn_pipeline_task<F>(ctx, service, f)` helper, or better, move the execution concern into `ActionPipeline` so services don't know about `tokio`.

---

### B11. [Helper smell / utility class] `QueryHandlers` is a stateless struct with no behavior of its own

- **File:** `src/application/query_handlers.rs:9-109`
- **Evidence:**

  ```rust
  pub struct QueryHandlers;
  impl QueryHandlers {
      pub fn new() -> Self { Self }
      pub fn get_generating_status(&self, ctx: GameServiceContext) -> Result<(...), ApplicationError> {
          let game_state = load_or_fresh(&ctx);
          Ok((game_state.narrative.input_buffer.status.clone(), game_state.narrative.input_buffer.phase.clone()))
      }
      pub fn get_input_status(&self, ctx: GameServiceContext) -> Result<(...), ApplicationError> {
          self.get_generating_status(ctx)
      }
      ...
  }
  ```

- **Why smell:** Struct has no fields. Every method is a thin wrapper around `load_or_fresh` plus a field access. `get_input_status` is literally an alias for `get_generating_status`. This is a utility class in disguise; the methods could be free functions or methods on `GameServiceContext` without losing anything.
- **Severity:** low
- **Proposed fix:** Convert to free functions or impl blocks on `GameServiceContext`.

---

### B12. [Coincidental cohesion] `trigger_eval.rs` mixes trigger evaluation with encounter-log CRUD

- **File:** `src/engine/trigger_eval.rs`
- **Evidence:**

  ```rust
  pub fn evaluate_triggers(state: &GameState) -> Vec<(NpcCard, Trigger, usize)> { ... }
  pub fn check_condition(...) -> bool { ... }
  pub fn increment_times_met(npc_encounter_log: &mut NpcEncounterLog, npc_id: &str) { ... }
  pub fn mark_trigger_fired(...) { ... }
  pub fn set_currently_meeting(...) { ... }
  pub fn get_times_met(...) -> u32 { ... }
  pub fn is_trigger_fired(...) -> bool { ... }
  ```

- **Why smell:** File name promises trigger evaluation, but half the module is low-level accessor/mutator functions for `NpcEncounterLog`. These CRUD helpers have nothing to do with trigger logic; they are used by `action_processing.rs` to mutate log state after movement. Mushing them together creates a grab-bag module.
- **Severity:** low
- **Proposed fix:** Move log helpers to `model/trigger.rs` (or a new `npc_encounter_log` module) and keep `trigger_eval.rs` focused on `evaluate_triggers` and `check_condition`.

---

## Cross-cutting notes

1. **Pipeline abstraction is both over- and under-abstracted.** `ActionPipeline` extracts phases as methods (good) but the central `run_from_input` script still manually threads state and handles cancellations inline. The existence of `retry.rs` with its own mini-pipeline proves the abstraction isn't reusable.
2. **Service layer sandwich.** `DefaultApplicationService` → `GameLifecycleService` / `MessageEditingService` / `QueryHandlers` creates deep delegation with no added value. The TODO in B4 is the canary; flattening would remove ~300 lines of boilerplate.
3. **Error model is split across three channels:** `EngineError`, `ActionOutcome`, and `GenerationStatus::Error`. This confusion produces dead variants (B2) and `error_return` hacks (B9). Consolidate to one error type at the application boundary.
4. **`GameServiceContext` is already a context object, but phases ignore it.** Phases receive `world`/`map`/`player`/`npcs` separately (B5, B6) even though `ctx.world` etc. are available. This suggests the context object was introduced but phases were not migrated, leaving a half-finished abstraction.
