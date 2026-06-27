# Super-Plan: Abstraction-Fixes Follow-Up

**Date:** 2026-06-27
**Status:** Planning / Sub-plans pending
**Type:** Overarching spec — organises remaining work after Phases 1–7 of `abstraction-fixes-implementation-plan.md` completed in commit `6a8531e`.
**Scope:** `chronicler_engine/`

## Related

- Source plan (archived): `docs/plans/archived/abstraction-fixes-implementation-plan.md`
- Original investigation: `docs/reviews/abstraction-antipatterns-summary.md` + `docs/reviews/zone-{a,b}-*.md` (zone-c/d originals removed in thermo-nuclear commit)
- Prevention plan: `docs/plans/abstraction-antipattern-healthcheck-plan.md`
- Post-implementation review: `docs/reviews/agent-comprehension-review.md` (stale context, may exist), plus informal holistic review captured in this plan

---

## Objective

Organise remaining abstraction/cleanup debt into independently-schedulable sub-plans. Each work-stream below is self-contained enough to become its own sub-plan. This super-plan does NOT re-litigate decisions — it locks past decisions and tracks what's left.

**What this is NOT:** Implementation detail. Each sub-plan holds its own decisions, blast-radius analysis, and verification. This plan only enumerates work + readiness.

**Headline state:** All 7 high-severity findings (A1/A4/A10/A12/C1/C4/B2 sister) cleared. Of 47 original findings: 18 fully fixed, 7 partial, 7 deliberate-defer, 15 untouched (mostly low). 18 NEW issues surfaced during holistic re-review (N1-N20).

---

## Track Listing

| # | Track | Sub-plan readiness | Priority | Blocks |
|---|-------|--------------------|----------|--------|
| T1 | Error Model Unification | ready — needs scoping decisions | P1 | none |
| T2 | ArrivalTaskContext / Pipeline Reuse | ready | P1 | T1 (soft) |
| T2-ARCH | Narration Deepening (architecture-lens reframing of T2) | needs grilling (G1–G5) | P1 | T1, ADR-018 |
| T3 | Service Layer Cleanup | ready | P1 | none |
| T4 | MockBackend Modernization | ready | P2 | none |
| T5 | Type Collapses (A3 + A6) | ready | P2 | none |
| T6 | MessageHistory Encapsulation | ready | P2 | none |
| T7 | Storage API Polish | ready | P2 | none |
| T8 | Persistence Reliability | needs design decision | P1 | none |
| T9 | Doc / Migration Debt | ready | P0 | none |
| T10 | Low-priority cleanup bundle | opportunistic | P3 | none |

P0 = no code change, just docs/process. P1 = structural/risk. P2 = debt prune. P3 = cosmetic.

### Architecture-lens pass (2026-06-27)

`improve-codebase-architecture` skill review surfaced 8 deepening opportunities (numbered in skill output). Per-track notes added inline below. Key reframes summarized:

- **T2 → T2-ARCH:** "extract helpers" → "one deep Narration module split across two adapters" (real seam per LANGUAGE.md).
- **T1:** "unify error types" → "seam misplaced (state mutation, not return value); interface lies (`Ok` ≠ success)."
- **T3:** "delete OR move-to-GameService" → deletion test says **delete only**. Move creates new shallow delegate.
- **T5 A3:** "type collapse, 75 refs" → **false seam** (one adapter each direction, no domain reason for two types).
- **T6:** "encapsulation cleanup" → **interface correction** (tests were testing past the interface).
- **T7:** "40 dead arms" are symptom; problem was misplaced seam (`Backend::Test` variant in prod enum). Mostly addressed by T7-Layered split (verify).
- **T8:** behavior decision, not module shape. Sequence after T1.
- **T9 Phase 4 shields:** re-export shields are architecture debt (zero leverage), not migration debt.

Full skill vocabulary in `improve-codebase-architecture` skill LANGUAGE.md: module / interface / implementation / depth / seam / adapter / leverage / locality.

---

## T1 — Error Model Unification

**State:** DEFERRED root cause. Triple channel still present:

- `EngineError` for storage/IO (`src/error.rs`)
- `GenerationStatus::Error(String)` for pipeline narration errors (`state/generation_status.rs`)
- `ActionOutcome` for pipeline cancellation (`action_pipeline/pipeline.rs`)
- Helper `error_return(&self, state, msg) -> PipelineResult<...>` returns `Ok` while writing `state.status = GenerationStatus::Error(msg)` (`phases.rs:53-61`). Each caller checks `status.error_message().is_some()` after.

**Why deliberate:** `8e4acf5 "Unify error model onto GenerationStatus on state"` consolidated narration errors onto state intentionally (CHANGELOG line 272). Documented at `architecture/system.md:49`, `system/game_flow.md` "Error Model" section. Original plan Tier 3 ("root-cause error model") explicitly excluded.

**What's left / risk:**

- Every new code path that needs to surface an error self-invents a side-channel write to `state.status` (N3 in holistic review).
- No structural barrier between transient (cancelled) and real-failure paths.
- Failure mode: silent corruption if caller forgets the `error_message().is_some()` check after a phase call.

**Sub-plan scope:**

- Pick single error type at pipeline boundary (`PipelineError` enum capturing Cancelled / LlmFailed(String) / StorageFailed(EngineError) / QuantifierFailed(String) / TriggerFailed(String)).
- Delete `error_return` helper; replace with explicit `Err(PipelineError::...)` propagation.
- `GenerationStatus::Error` retained purely for state-machine UI rendering, NOT for pipeline control flow.
- Audit: which phases currently check `status.error_message().is_some()` mid-flow (3+ sites in `pipeline.rs`)? Convert each to `?` propagation.
- Decide: `ActionOutcome::Cancelled` folds into `PipelineError::Cancelled` or stays separate (exhaustiveness aid).

**Out of scope:** removing `GenerationStatus::Error` variant entirely (UI status rendering depends on it).

**Blast radius:** ~3–5 files in `application/action_pipeline/`. Storage layer untouched.

### Architecture-lens note

`improve-codebase-architecture` skill review (2026-06-27) reframes this as skill candidate #3: **seam misplaced (state mutation, not return value)**. Pipeline methods return `PipelineResult<T>` BUT failure can also be signaled via `state.narrative.input_buffer.status = GenerationStatus::Error(msg)`. Caller's interface requires knowing "if `Ok`, check `state.status.error_message()`; if `Err(ActionOutcome::Cancelled)`, that's a real `Err`." Two error channels at one seam. Interface is **shallow on failure-mode surface** — it lies (`Ok` does not always mean success). ADR-018 + commit `8e4acf5` deliberately pinned errors onto `GenerationStatus`; revisit because the friction is real. Sub-plan should run skill ADR-conflict check before proceeding.

---

## T2 — ArrivalTaskContext / Pipeline Reuse

**State:** DELIBERATE per corrections #3 in original plan. `ArrivalTaskContext` (`bootstrap/init_game.rs:95-190`) is 12-field reimpl of pipeline: load snapshot → make prompt context → call LLM → persist. Documented at `architecture/system.md:208`.

**What's left / risk:**

- **N17:** `runtime.spawn_blocking(move || task_ctx.run())` at `init_game.rs:278` runs full LLM stack on shutdown with NO cancellation check inside `run()`. Compare `process_action` spawn (checks `cancel_token.is_cancelled()` before exec).
- **N5:** prompt-context assembly + LLM call shape + persistence pattern can drift between `ArrivalTaskContext::run` and `ActionPipeline::phase_narrate`.
- **N19:** no ADR protecting deliberate-defer decision. Future audits will re-flag.

**Sub-plan scope:**

1. (P0) Add cancellation check at start of `ArrivalTaskContext::run()` + before LLM call.
2. (P1) Add ADR-027 documenting the deliberate pipeline duplication + proposing a future `NarrationDriver` shared abstraction if a third execution path emerges.
3. (Optional, P2) Extract shared `make_prompt_context` + LLM-call + persist helpers into a `pipeline::shared` module used by both `ActionPipeline::phase_narrate` and `ArrivalTaskContext`. Decide: collapse now or pin helper overlap.

**Blast radius:** `bootstrap/init_game.rs` + new ADR + optional `application/action_pipeline/shared.rs` new module.

**Dependency:** Soft T1 — if error model unified first, the shared helper's error shape is cleaner.

---

## T2-ARCH — Narration Deepening (architecture-lens reframing of T2)

**Source:** `improve-codebase-architecture` skill review (2026-06-27). Reframes T2 from "extract shared helpers" to a deepening opportunity.

**Architecture vocabulary** (per skill LANGUAGE.md): module / interface / implementation / depth / seam / adapter / leverage / locality.

### Reframe

- T2 framed the gap as "two reimplementations of pipeline logic." Architecture lens says: this is **one deep module split across two adapters**. `phase_narrate` and `ArrivalTaskContext::run` both implement the same narration pipeline (load → context → assemble → complete → persist) behind disconnected interfaces. Per LANGUAGE.md: **two adapters = real seam**. The seam is misplaced; the deep module (Narration) does not exist as an explicit module.

### Deletion test

- Delete `ArrivalTaskContext` → complexity reappears across `bootstrap/init_game.rs` callers AND the same narration logic persists in `phase_narrate`. Module is earning its keep, just placed wrong.
- Inverse: delete `phase_narrate` → same outcome.

### What's actually shared (verified against code)

Three LLM-call sites producing narrative text. Only two are real adapters of the same seam:

| Site | File | Prompt source | Cancel check | Persist | Returns |
|------|------|---------------|--------------|---------|---------|
| `phase_narrate` | `phases.rs:78` | preset + assembler (`assemble`) | yes, pre+post call | `save_message_and_snapshot` (snapshot + message) | `(state, text, backend_name, model_name)` |
| `phase_trigger_continuation_raw` | `phases.rs:228` | `trigger.system_prompt`/`user_prompt` (PRE-ASSEMBLED in `StoredTriggerContext`) | yes | `save_message_and_snapshot` | `(state, continuation_text)` |
| `ArrivalTaskContext::run` | `init_game.rs:144` | preset + assembler | **no** | `save_snapshot` ONLY (no `save_message_and_snapshot`) — appears to NOT persist message | `()` unit |

Trigger-continuation site is a **different seam** (replay stored prompts; bypasses assembler). Narration deepening covers **two** real adapters: `phase_narrate` + `ArrivalTaskContext::run`.

### Constraints any deepened module must satisfy

These are the friction points. Two-call-site differences must resolve:

1. **State ownership** — pipeline receives pre-loaded `GameState`; arrival owns `Arc<Storage>` and loads snapshot inside `run()`.
2. **Preset acquisition** — pipeline calls `self.service.load_preset_and_response_length()` at call time; arrival receives `arrival_preset: Option<PromptPreset>` + `response_length: String` baked into struct at spawn time.
3. **Persistence policy** — pipeline uses `save_message_and_snapshot` (snapshot + newest message); arrival previously used `save_snapshot` only. **RESOLVED 2026-06-27: BUG confirmed. Fixed by routing `ArrivalTaskContext::run` through `save_message_and_snapshot` (Option A — `ArrivalTaskContext` now holds a `GameServiceContext`, calls helper directly). See CHANGELOG Q1 entry. Closes ADR-023 §4 for arrival path.**
4. **Cancellation** — pipeline checks `cancel_token` between assemble→LLM and after LLM; arrival has no `cancel_token` at all.
5. **Status reporting** — pipeline writes `GenerationStatus::Generating` and transitions through phases (`GeneratingEvent`, `Quantifying`); arrival only sets `Generating` then `Idle`/`Error`.
6. **Backwards data flow** — pipeline returns `(text, backend_name, model_name)` for caller to persist `LlmMessage` forensics; arrival discards `backend_name`/`model_name`.
7. **Domain role** — pipeline = "narrate user input" (always has `input: String`); arrival = "narrate arrival scene" (empty input, only when `!has_scenario`). Different domain meaning.

### Grilling decisions deferred (left for sub-plan)

User flagged low motivation for full grilling now. Sub-plan must answer these before designing the interface:

- **G1 (persistence difference).** RESOLVED 2026-06-27: BUG. Arrival path now routes through `save_message_and_snapshot` like the pipeline path. Constraint 3 collapses to one persistence policy. Deepening candidate #1 can resume at G3 (state ownership shape), already partially constrained by the `GameServiceContext` construction in `ArrivalTaskContext`
- **G2 (naming).** Pick a domain term: `Narrator` (conflicts with `AGENT_NARRATOR` const?), `NarrationDriver`, `NarrationService` (skill discourages "service" but ADR-018 uses it), `SceneNarration` (matches `state.scene.*` vocab), or two modules `ArrivalNarration` + `InputNarration` (rejects deepening). New term must be added to project vocab (`system.md` since no `CONTEXT.md` exists).
- **G3 (state ownership shape).** Three options: (a) always receive `GameState` — arrivals load state before calling; (b) always own storage + load internally — pipeline reframes to pass storage; (c) two entry points sharing internals — `narrate_input(state, input)` + `narrate_arrival(state)`. (a) = cleanest interface, (b) = highest locality, (c) = preserves both call shapes.
- **G4 (cancellation).** Three options: (a) require `cancel_token` param — arrival caller must obtain one (caller is `spawn_blocking` from `spawn_arrival_task_if_needed`); (b) make cancel optional `Option<&CancellationToken>` — leaks "is this cancellable?" into interface; (c) always require token — arrival task gets new token from spawner. Note: ADR-018 system.md:209 documents `ArrivalTaskContext` as deliberate; revisiting G3/G4 is an ADR-018 revisit.
- **G5 (ADR-018 conflict).** If deepening proceeds, `architecture/system.md:208-209` must be updated. If user rejects deepening with load-bearing reason, offer ADR-027 to prevent future re-flagging.

### Side concerns

- `ArrivalTaskContext` 12 fields = coupling smell. Deepened module may swallow fewer.
- `phase_narrate` returns `backend_name`/`model_name` for forensics (`LlmMessage`). Deepened module must expose them OR persist `LlmMessage` itself (more locality, larger interface).
- `spawn_blocking` is caller's concern (pipeline + `message_editing.rs:145,189` + `init_game.rs:278` all spawn). Deepened module stays sync; callers spawn.
- T2-ARCH is NOT exclusive of T2 — the P0 cancellation check + ADR-027 from T2 can still proceed independently if user defers deepening.

### Sub-plan deliverable

Sub-plan must pick: full deepening (T2-ARCH) OR minimal T2 (cancellation + ADR + helper extraction). Don't propose interfaces yet — sub-plan runs `improve-codebase-architecture` skill step 3 (grilling loop) to resolve G1–G5 first.

### Blast radius (deeper than T2)

`bootstrap/init_game.rs` + `application/action_pipeline/phases.rs` + possibly new `application/narration.rs` (or `narrative/narration.rs` — placement decision). High churn but mechanical-ish.

### Dependency

- Soft T1 — error model unified first means deepened module's error shape is `PipelineError` not `state.status = Error(...)`.
- ADR-018 revisit — coordinate with T9 doc track.

---

## T3 — Service Layer Cleanup

**State:** B4 partially fixed — `GameLifecycleService` flattened into `DefaultApplicationService` (file deleted). BUT N1 — `DefaultApplicationService` still has ~9 identity-passthrough methods to `super::query_handlers::*` (`application_service.rs:322-386`: `get_generating_status`, `reset_generating_status`, `get_current_game_name`, `list_latest_llm_messages`, `get_story_log_entries`, `get_input_status`, `get_current_room_view`, `get_npc_headshots`, `get_debug_state`). Service-layer sandwich relocated, not eliminated.

**Also:** B10 partial — `spawn_pipeline_task` helper extracted on `DefaultApplicationService:181` for `process_action`, but `message_editing.rs:145, 189` (`retry` / `retrigger`) still inline `tokio::task::spawn_blocking` with same shape (clone Arc, clone ctx, check cancel_token, spawn).

**Sub-plan scope:**

1. Decide query-handler surface: (a) delete 9 wrappers, callers `use crate::application::query_handlers::*` directly, OR (b) move query invocations into `GameService` methods (`game_service.rs` already owns storage; queries are state reads).
2. Extract single `spawn_pipeline_task<F>` helper on `GameService` or `DefaultApplicationService` — signature: `(ctx, f: F) where F: FnOnce(&GameService, GameServiceContext) + Send + 'static`. Used by `process_action`, `retry`, `retrigger`.
3. Decide: does `DefaultApplicationService` keep `MessageEditingService` field or inline too? (`editing.retry()` etc. are 1-line delegates — same shape as B4 GameLifecycleService).

**Blast radius:** `application_service.rs`, `message_editing.rs`, ~3–5 caller call sites (server fragments).

**Dependency:** none. Independent of T1.

### Architecture-lens note

`improve-codebase-architecture` skill review (2026-06-27) flags T3's option (a) vs (b) framing as wrong. Deletion test on the 9 query passthroughs: delete them → complexity vanishes (callers call `query_handlers::fn` directly). Pure pass-through; nothing hidden, zero leverage. Option (b) move-to-`GameService` creates **new** shallow delegate — same shape, different file. Don't.

Also flags 5 editing delegates (`retry`, `retrigger`, `switch_swipe`, `edit_history`, `delete_last`) as same shape (candidate #7 from skill review). Two valid framings: delete (if no mutation-coherence reason) OR promote `MessageEditingService` to a deep **HistoryMutation** module exposing all edits behind one interface (if mutation coherence matters). Sub-plan must pick per-skill rather than per-intuition.

---

## T4 — MockBackend Modernization (C6 follow-through)

**State:** Partial. `MockBackend` (`narrative/llm/mock.rs:22-35`) builders exist:

- `::failing()`, `::with_empty_response()`, `::with_failing_trigger_narration()`, `::with_delay()`, `::with_trigger_delay()`
- **No `::succeeding()`** (per plan; default fills role)
- **All fields still `pub`** — tests bypass builders via direct mutation (e.g., `backend.should_fail.store(true, ...)`)
- 6 AtomicBool/AtomicU64 + 2 `pub Vec<String>` + `per_call_*` pattern preserved (C6 Option C "minimal prune + builders" — flag-bag stays)

**Plan originally:** Decision 3 said Option C (minimal prune + builders). Migration of ~100 `::default()` call sites explicit follow-up Task 6.6b — never done.

**What's left / risk:**

- N7: builder ergonomics added but not enforced. New tests reach for `field.store(...)` not builders, propagating flag-bag smell.
- N6: cosmetic migration backlog.

**Sub-plan scope:**

1. Privacy: change `pub should_fail: AtomicBool` → `pub(crate)` for all 6 flags + Vec fields. Forces builder use externally.
2. Audit and remove 2 unused flags (likely `trigger_started` if no test checks it; `narration_started` likely still used).
3. Add `::succeeding()` builder (explicit symmetrical to `::failing()`).
4. Migrate ~100 `::default()` sites in test code — opportunistic, no deadline. Skip migration if step 1 lands; old code still works.

**Blast radius:** `narrative/llm/mock.rs` + tests touching fields directly.

---

## T5 — Type Collapses (A3 + A6 deferred)

**State:** Original A3 + A6 explicitly DEFERRED in source plan as "rippling struct field + many call sites" pattern.

**A3 `Confidence` vs `QuantifierConfidence`:**

- `model/agent.rs:76` `pub enum Confidence { High, Medium, Low }`
- `model/quantifier.rs:8` `pub enum QuantifierConfidence { High, Medium, Low }`
- Bidirectional `From` impls (`quantifier.rs:14-50`)
- **75 references** across `application/`, `engine/`, `model/`. Safety: identical variants; module boundary is only reason for split.

**A6 `TemplateVars`:**

- `model/template.rs:5` `pub struct TemplateVars { pub user: String }` + `pub fn render_template(text, vars)`. One field, one function, one consumer.
- 6+ callers in `assembler.rs:169`, `bootstrap/state.rs:28`, `bootstrap/scenario.rs:22`, `context.rs:124`, `quantifier/prompt.rs:21`, `types.rs:36` (per source plan).

**Sub-plan scope:**

- **A3:** delete `QuantifierConfidence`, use `Confidence` everywhere. Update 75 refs mechanically. Decision: keep `Confidence` in `model/agent.rs` or split out to new `model/confidence.rs`? Make invalid states unrepresentable — derive `Ord` (`StatePatch::merge` simplifies to `min()` — N14).
- **A6:** either delete `TemplateVars` (replace signature `render_template(text, user: &str)`), OR accept struct (if adding 2nd field is on the roadmap). Plan strongly suggested deletion.

**Blast radius:** A3 = 75 refs across ~15 files. A6 = 6+ callers. Both mechanical.

**Risk:** test fixtures may rely on the type names.

### Architecture-lens note

`improve-codebase-architecture` skill review (2026-06-27) reframes A3 from "type collapse, 75 refs" to a **false seam** (skill candidate #6). Two modules, bidirectional `From` impls each trivially wrapping the other. Per DEEPENING.md "one adapter = hypothetical seam" — there was never a domain reason for two types. Deletion test on `QuantifierConfidence`: complexity reappears as 75 mechanical refactors with no semantic gain; net = less code, fewer conversions. Sub-plan can use false-seam framing in ADR-027.

---

## T6 — MessageHistory Encapsulation (Task 7.4 skipped)

**State:** Original A5 finding NOT addressed. `model/message_history.rs` exposes:

- `pub fn replace(&mut self, messages: Vec<Message>)` — line 101 (bypasses MAX_MESSAGES)
- `pub fn retain(&mut self, f)` — line 89
- `pub fn iter_mut(&mut self)` — line 85
- `pub fn as_slice(&self)` — line 97
- `pub fn clear(&mut self)` — line 93
- `pub fn from_messages(messages: Vec<Message>) -> Self` — line 23, bypasses MAX_MESSAGES cap (N15 new risk; only `append` enforces 1000 cap)

**Why smell:** struct promises encapsulation ("Callers cannot bypass rules with direct `.push()`"), but multiple bypasses remain.

**Sub-plan scope:**

1. Remove `replace`, `retain`, `iter_mut`, `as_slice`, `clear` from public API. Replace callers with `iter()` + `last()` + `len()` + `append()` + `delete_last()` + `edit()` (existing controlled API).
2. `from_messages`: enforce cap (truncate to last 1000) OR rename to `from_messages_trusted` + mark `#[doc(hidden)]` for storage loaders only.
3. Audit callers: `ArrivalTaskContext::run()` uses `state.narrative.history.replace(msgs)` at `init_game.rs:135` — needs replacement pattern (likely `clear()` then `append()` loop, or a new `extend_from_storage(messages)` method).

**Blast radius:** `model/message_history.rs` + callers (mostly storage loaders + tests).

**Dependency:** none.

### Architecture-lens note

`improve-codebase-architecture` skill review (2026-06-27) flags this as skill candidate #5: classic **"tests want to test past the interface → module is wrong shape"** (LANGUAGE.md). `replace`/`retain`/`iter_mut`/`as_slice` exist as implementation exposure for test setup. Deletion test on `replace`: complexity reappears as `clear + append loop` at callers — concentrates the MAX_MESSAGES cap-bypass bug (N15) into one place. Earns keep *only by accident* (current callers exploit the bypass). Sub-plan should frame removal as **interface correction**, not encapsulation cleanup.

---

## T7 — Storage API Polish

**State:** Phase 5 partial — `Operation` enum removed ✅, stringly-keyed `HashMap<&'static str, TestOverride>` ✅, `TestFailureHandle` + `impl Drop` warn observability ✅. **Backend/LayeredBackend split sub-plan DONE (2026-06-27, uncommitted):** `Backend` enum now `Sqlite`/`InMemory` only; new `LayeredBackend` enum (`Direct(Backend)` | `Test { base: Box<Backend>, overrides }`, non-recursive); 40 dead `Backend::Test { .. } => unreachable!()` arms deleted across 10 storage files; test-infra types (`TestOverride`/`TestFailureHandle`/`ErrorKind`) moved to new `storage::backend::test_support.rs` module with re-export shim preserving `crate::storage::*` import path; 2 micro-tests pin replace-not-nest invariant; public `Storage` API unchanged. See archived sub-plan `docs/plans/archived/t7-storage-backend-layered-split.md`.

Still deferred from original T7 scope:

**What's left:**

- **D3 NOT fully done:** Plan said `with_backend_mut(game_id: Option<u64>, f)`. Actual signature: `with_backend_mut(method: &'static str, f: F)` — `_game_id` removed from closure entirely, `Option<u64>` API not adopted. *Cosmetic miss; achieves same goal (no dummy closures).*
- **D11 UNTOUCHED:** `from_row` consistency. 9 Db* storage models; 4 have `from_row` (character, persona, settings, world×2). 5 missing (game, game_state_snapshot, message, llm_message, prompt_preset). Inconsistent storage→model mapping patterns.
- **D2 UNTOUCHED:** `empty_to_none` one-function module (`storage/backend/helpers.rs:5`). 5 callers. Inline or move to storage mod.
- **M1 NEW:** `Backend::Test { ... }` variant is `pub` in production enum (`core.rs:60-67`). Test infrastructure ships in production type. Feature-gate or move to test module.
- **M2 NEW:** `Storage::with_failure` stacks `Backend::Test` wrappers (calling twice on same Storage nests). Non-idempotent; risky if test setup reuses storage instances.
- **N2:** 40 `Backend::Test { .. } => unreachable!()` dead arms across 10 backend files (49 total `unreachable!()` in src/). Thermo-nuclear revert deliberately preserved for exhaustive-match safety.
- **40 dead arms consideration:** could use `#[non_exhaustive]` on `Backend` enum for same safety with less repetition.

**Sub-plan scope:**

1. Audit `from_row` — add to 5 missing Db* models. Standardise storage→model mapping.
2. Inline `empty_to_none` or split into a `String` extension trait.
3. Decide `Backend::Test` prod-visibility: feature-gate behind `testing` feature (already exists, see `src/lib.rs` cfg_attr).
4. Decide: keep 40 explicit `Backend::Test { .. } => unreachable!()` arms (current, deliberate per thermo-nuclear commit body) OR replace with `#[non_exhaustive]` enum. If switched,antes preserves safety, drops noise.
5. Decide `with_failure` idempotency: document as "call once per Storage" or refactor to replace existing Test layer.
6. OPTIONAL: revisit Decision 1 Option C (trait-object `StorageBackend` test double) — deferred originally as ~60 storage method signatures touched. Re-evaluate if technical debt on stringly-keyed overrides grows.

**Blast radius:** storage layer only. Storage callers unaffected unless `with_failure` semantics change.

### Architecture-lens note

`improve-codebase-architecture` skill review (2026-06-27) reframes candidate #4: the 40 dead `Backend::Test { .. } => unreachable!()` arms were the **symptom**, not the problem. The problem was misplaced seam — `Test` variant lives inside the prod enum. Per DEEPENING.md seam discipline: proper two-adapter seam = trait boundary `StorageBackend` (prod Sqlite/InMemory + dev adapter). T7-ARCH-Layered split already addressed most of this (2026-06-27); verify the layered split matches the seam-discipline recommendation or document the divergence.

Note: M1/M2 items above are pre-split; check if still relevant post-split before sub-plan starts.

---

## T8 — Persistence Reliability

**State:** NEW risk surfaced by thermo-nuclear commit.

- **N11:** `DefaultApplicationService::reset` (`application_service.rs:285-296`) does `let _ = Self::persist_initial_state_with_swipes(&ctx);` — swallows all per-message/swipe persistence failures (only `tracing::error!` inside helper). Documented in thermo-nuclear commit body. **Behavior change from `?` propagation to silent swallow.**
- **N20/M7:** `ActionPipeline::run_from_input:108` — `if let Err(e) = save_message_and_snapshot(...) { tracing::warn!("Failed to save post-quantifier metadata: {e}"); }` — pipeline continues after warn. Tests that don't assert on tracing output pass despite persistence failure.
- **N12/M8:** `application/context.rs:96-97` — `panic!("no snapshots found")` + `panic!("failed to load snapshot: {e}")` under `#![deny(clippy::panic)]` in `lib.rs:9`. Either `#[allow]` present (verify) or these trip clippy.

**Why smell:** silent data partial-persistent state on disk failure. Game appears to work; later load fails on missing starting message. Worse for production: no alert.

**Sub-plan scope:**

1. **Design decision:** how should partial-persistence failure surface? Options:
   - (a) propagate to caller (`Result<(), ApplicationError>`) — currently partially reverted by thermo-nuclear
   - (b) mark `Game` row as "needs repair" + retry on next access
   - (c) keep silent + log + add healthcheck rule that fails on tracing `error!` containing "Failed to save"
2. Verify the per-phase `save_message_and_snapshot` warn paths — list all 3+ sites in `pipeline.rs`. Decide each: propagate vs. continue.
3. Resolve `context.rs:96-97` panics — either `#[allow(clippy::panic)]` with justification, or convert to `Result`/`expect` with documentation. Current state likely already tripping clippy or has allow.
4. Healthcheck: add check that greps tracing `warn!`/`error!` on save paths during test runs — surfaces silent failures.

**Blast radius:** small surface (`application_service.rs`, `context.rs`, `pipeline.rs`), but behavior-change risk.

**Risk:** touches user-facing reset behavior. Integration tests need new coverage for partial-failure scenarios.

### Architecture-lens note

`improve-codebase-architecture` skill review (2026-06-27) flags T8 as a **behavior** decision (propagate vs swallow), not a module-shape issue. Belongs in error-model discussion (T1) — once error seam unified (T1 candidate #3), swallow-vs-propagate becomes single-place policy, not a per-call-site decision. Sub-plan should NOT design T8 in isolation; sequence after T1.

---

## T9 — Doc / Migration Debt

**State:** Multiple doc/migration items pending. No code logic.

**What's left:**

- **Task 6.1 NOT DONE:** `docs/system/action_pipeline.md` spec covering `PipelineInputs` + `spawn_pipeline_task` helper contracts. Plan said "spec must propose DELTA from current documented arch, not contradiction."
- **Task 7.1 NOT DONE:** `docs/system/message_model.md` covering accessor-pattern Message struct (reads from `swipes[active_swipe_index]`, no mirrored fields). ADR-017 alignment.
- **Phase 4 re-export shield NOT migrated:** `state/mod.rs:12-18` keeps `pub use` for 7 submodules. 49 callers still use `crate::model::state::GenerationStatus` style (re-exported). Migrate high-churn callers (`game_service.rs`, `application_service.rs`, `pipeline.rs`, `phases.rs`) to direct paths, eventually drop re-exports. Plan Task 4.1b — never done. **Architecture-lens note (skill review 2026-06-27, candidate #8):** re-export shields are pure pass-throughs failing deletion test; provide zero leverage. Risk: become permanent indirection. Should be tracked as architecture debt, not migration debt. Original super-plan left this implicit; sub-plan should make it explicit.
- **CHANGELOG missing Phase 4–7 entries:** only Phase 1–3 + thermo-nuclear cleanup logged. Module splits (`state/`, `misc/`, `renderers/`), `GameLifecycleService` inline, `Message` accessor pattern, `apply_to` deletion, `Operation` enum removal, `assemble_prompt_text` relocation, `GameLifecycleService` delete — invisible in CHANGELOG.
- **ADR-027 MISSING:** recommended by source plan — document deliberate deferrals (B7 retry mini-pipeline stale, B8 `ArrivalTaskContext` deliberate, B9 `error_return` deliberate arch, B3 monolith). Without it, future audits re-flag same code. Also document the 4 investigation misclassifications (B2 reframed, B7 stale, B8 deliberate, B9 deliberate).
- **N18:** 6 files under `src/test_support/` lack `[DOC: ...]` anchor (`main.rs` too).
- **`abstraction-antipatterns-summary.md` NOT updated:** per source plan, "future readers see stale framing" of the 4 misclassifications.
- **`abstraction-fixes-plan.md` NOT updated** (now archived, may not matter, but annotations missing).

**Sub-plan scope:** Pure docs — no code changes. One sub-plan batches all of T9.

1. Write ADR-027 (deliberate deferrals + investigation misclassifications).
2. Update CHANGELOG with Phase 4–7 entries (retroactive; mark "completed in 6a8531e").
3. Write `docs/system/action_pipeline.md` + `docs/system/message_model.md` specs.
4. Migrate top-5 high-churn callers off re-export shield. Leave shield in place for opportunistic completion later.
5. Annotate `abstraction-antipatterns-summary.md` with status notes ("resolved Phase 2", "deliberate per ADR-027", etc.).
6. Add `[DOC: ...]` anchors to 7 files.

**Blast radius:** docs + ~5 import-path-only changes.

**Priority:** P0 — no logic changes, only context-recording. Do FIRST so future audits have anchors.

---

## T10 — Low-Priority Cleanup Bundle

**State:** 12 untouched items from original 47 + 8 NEW cosmetic issues. All low severity. Mechanical. Bundle as a single sub-plan or pick off opportunistically during other refactors.

**Items:**

- **A9 `push_section`** — `assembler.rs:52` 1-line wrapper, now 3-caller (so slightly more justified than original report). Inline or keep?
- **A11 `MessageEntry` DTO mirroring** — `state/message_types.rs`. Could collapse or `impl From<&Message>`.
- **D2 `empty_to_none`** — covered under T7.
- **D7 `ActionForm` reused for `check_text_handler`** — `server/fragments/actions.rs`. Split into `CheckTextForm`.
- **D9 `add_status_swap_headers`** — 1 caller. Inline.
- **D11 `from_row` consistency** — covered under T7.
- **N13 server fragments `Ok(_) => unreachable!()` arms** — `fragments/history.rs:23,36`, `misc/swipe.rs:17,30`, `games_fragment/handlers.rs:30,110,127`, `worlds_fragment/handlers.rs:203`. Invert to `let Ok(_) = ... else { ... }`.
- **N14 `Confidence` derive `Ord`** — collapses `StatePatch::merge` 4-arm match to `min()`. Covered by T5 if A3 collapse happens.
- **N16 asymmetric `list_personas` placement** — `DefaultApplicationService:383` is 3-line storage passthrough. Move to storage direct calls (matches `list_worlds`).
- **N18 test_support doc anchors** — covered under T9.
- **M3 `response_length: Option<&str>` stringly typed** — `assembler.rs:17`. Enum or token count.
- **M4 `QuantifierParseResult::is_high()` only** — add `is_low()`/`is_medium()` or replace with derived `Ord` + comparison.

**Sub-plan scope:** pick 3–5 of these during a sprint. Each is <1 hour mechanical work. No structural risk.

**Blast radius:** various; isolated per item.

---

## Triaged Away / Won't Fix (locked)

These are **deliberate decisions**, not cleanup items. Don't re-flag.

| ID | Decision | Source |
|----|----------|--------|
| B7 retry mini-pipeline | STALE finding — retry already delegates to `phase_trigger_continuation` + `run_from_input` per `architecture/system.md:52` + CHANGELOG line 90 | Plan corrections #2 |
| B8 `ArrivalTaskContext` | Deliberate extraction documented at `architecture/system.md:209`; ADR-027 (T2) will formalize | Plan corrections #3 |
| B9 `error_return` returns Ok | Deliberate arch per `8e4acf5` — error model unified onto `GenerationStatus` on state | Plan corrections #1, T1 will revisit if needed |
| B3 `run_from_input` monolith | State-machine rewrite scoped out in Phase 6.1 (Issue 9 constraint) | Plan Phase 6.1 spec |
| C2 global `sanitize_llm_output` | Backend-agnostic sanitize policy, not a dedup bug | Plan NOT in scope |
| C5 OpenRouter headers in generic request | Storage/LLM refactor, separate plan | Plan NOT in scope |
| 40× `Backend::Test { .. } => unreachable!()` arms | Deliberate revert in thermo-nuclear `6a8531e` for exhaustive-match safety; T7 may revisit via `#[non_exhaustive]` | Commit body of `6a8531e` |

---

## Decisions to Lock Before Sub-Plans

These decisions belong in their respective sub-plans, not this super-plan. Listed here so they're visible:

- **T1:** Fold `ActionOutcome::Cancelled` into `PipelineError::Cancelled` or keep separate?
- **T2:** Extract shared `NarrationDriver` now, or sync helper overlap informally and pin for future?
- **T3:** Delete 9 query-handler wrappers from `DefaultApplicationService`, or move queries to `GameService`?
- **T4:** Migrate 100 `MockBackend::default()` test sites, or leave (fields go `pub(crate)` so old code still works)?
- **T5 A3:** Move `Confidence` to new `model/confidence.rs`, or keep in `agent.rs`?
- **T6:** `from_messages` for storage loaders: drop cap enforcement, or add `from_messages_trusted`?
- **T7:** `Backend::Test` prod-visibility — feature-gate behind `testing`?
- **T7:** Keep 40 dead arms or switch `Backend` to `#[non_exhaustive]`?
- **T8:** Partial-persistence failure: propagate, mark-needs-repair, or silent+healthcheck?

---

## Verification Strategy

Per-track verification, not super-plan-level:

- Each sub-plan defines its own test guarantees + verification commands.
- Project-level: `python build.py` (fmt + clippy + tests + coverage) remains the gold standard.
- Healthcheck: `python scripts/healthcheck.py` already enforces jscpd duplicates check; abstraction-antipatterns-healthcheck-plan will add advisory detection for `too_many_arguments` + `dead_code`.
- For structural tracks (T1, T3, T8): require integration test coverage of new behavior paths before merge.

## Sub-plan Creation Order (recommended)

1. **T9 (docs)** — P0, no risk, anchors everything else
2. **T8 (persistence reliability)** — P1, behavior risk, do before more code accrues on top of swallow-semantics
3. **T1 (error model)** — unblocks downstream cleanup; root cause
4. **T3 (service layer)** — high visibility, quick win after T1
5. **T2 (ArrivalTaskContext)** — depends on T1 soft; isolated
6. **T5 (A3 + A6)** — mechanical, do alongside any of above opportunistically
7. **T6 (MessageHistory)** — isolated, can run parallel to above
8. **T7 (storage API)** — isolated, can run parallel
9. **T4 (MockBackend)** — cosmetic migration, lowest priority; can run alongside anything
10. **T10 (low-priority bundle)** — pick off during sprints

## Plan Adherence

This super-plan **does not dictate implementation**. It enumerates:

- What's done (locked, won't re-litigate)
- What's deliberately deferred (with reasons)
- What remains (with sub-plan scope + blast radius)
- Recommended order (advisory)

Sub-plans MUST:

- Reference this super-plan as parent
- State assumed decisions before writing code
- Verify against actual code, not plan claims (source plan's investigation had 4 misclassifications — verify everything)
- Define success criteria per `AGENTS.md` goal-driven execution

Where sub-plan deviates from this super-plan's recommendation, stop and report per `AGENTS.md` "Plan Adherence" rule.
