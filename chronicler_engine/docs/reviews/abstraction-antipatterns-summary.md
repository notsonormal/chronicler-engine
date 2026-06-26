# Chronicler Engine — Abstraction Anti-Pattern Investigation

**Date:** 2026-06-26
**Scope:** `chronicler_engine/src/` (232 files, ~32k LOC), all non-test production source
**Method:** 4 parallel fresh-context reviewers, one per zone, each read files fully.
**Per-zone reports:** `reports/zone-{a,b,c,d}-*.md`

---

## TL;DR

47 findings across 4 zones. Patterns cluster into 5 categories. The engine's biggest debt is **not** duplication — it is **premature generalization**: traits, enums, and service layers introduced for a second consumer that never appeared, leaving single-variant enums, single-impl traits, and identity-wrapper services paying tax forever.

Top 3 systemic issues:

1. **Pipeline + service layer architecture is half-finished** — `ActionPipeline` extracts phase methods but `run_from_input` stays a monolithic script; `retry.rs` and `init_game.rs::ArrivalTaskContext` each re-implement the pipeline by hand; `DefaultApplicationService` → `GameLifecycleService` adds 14 identity wrappers (TODO in source admits it).
2. **Error model split across 3 channels** — `EngineError`, `ActionOutcome`, `GenerationStatus::Error`. Produces dead enum variants, `error_return` helper that returns `Ok` while burying failures in state, and forces every storage method to thread an `Operation` enum used only by tests.
3. **Single-consumer abstractions everywhere** — `PromptAssembler` trait (1 impl), `StatePatch` enum (1 variant), `TriggerRequirement` enum (1 variant), `NarratorAgent` (returns `NoOp`), `narrate_continuation` trait method (zero prod callers), `preprocess_user_text` hook (1 backend), `Phi` prompt layer (unused).

---

## Categorization

### Category 1: Premature Generalization (single-consumer abstractions)

Wrong speculation about future needs. Trait/enum introduced for a second consumer never arriving.

| ID | Site | Evidence |
|----|------|----------|
| A1 | `model/agent.rs:96` | `StatePatch` enum — single `Scene` variant, 50+ lines merge boilerplate |
| A2 | `model/trigger.rs:15` | `TriggerRequirement` enum — single `TimesMet` variant |
| A4 | `model/message.rs:18` | `Message` mirrors `Swipe` fields; `from_db` constructs invalid Message |
| A12 | `model/state_snapshot.rs:45` | `apply_to` manually clones every field; messages skipped |
| C1 | `narrative/llm/backend.rs:86` | `narrate_continuation` trait method — zero prod callers, deepseek returns `not_implemented()` |
| C4 | `narrative/prompt/assembler.rs:27` | `PromptAssembler` trait — one impl `LayeredPromptAssembler` |
| C7 | `narrative/prompt/types.rs:19` | `PromptLayer::Phi` variant — only referenced by a discriminant test |
| C8 | `narrative/llm/backend.rs:70` | `preprocess_user_text` hook — only `OllamaBackend` overrides |
| C9 | `narrative/agents/registry.rs:86` | `NarratorAgent` registered, `execute` returns `NoOp` |
| B2 | `application/action_pipeline/pipeline.rs:28` | `ActionOutcome::Error` variant — `#[allow(dead_code)]`, errors flow via `GenerationStatus::Error` side-channel |

### Category 2: Coincidental Cohesion (grab-bag modules)

Modules grouped by "used together" rather than shared concept.

| ID | Site | Evidence |
|----|------|----------|
| A7 | `model/state.rs` | 11 unrelated types: MessageType, GenerationStatus, MovementState, StoredTriggerContext, NarrativeState, SceneState, GameState... |
| D1 | `server/fragments/misc.rs` | "Miscellaneous fragment utilities" — text_check, retry, retrigger, switch_swipe, reset handlers |
| D8 | `server/fragments/renderers.rs` | `render_header`, `render_story_log`, `ok`, `bad_request`, `app_err_to_response`, `html_escape` — 3 responsibilities |
| B12 | `engine/trigger_eval.rs` | `evaluate_triggers` + `NpcEncounterLog` CRUD helpers in same file |

### Category 3: False Deduplication (same shape, different intent merged)

Code looks similar, semantics differ. Merging couples unrelated callers.

| ID | Site | Evidence |
|----|------|----------|
| A3 | `model/agent.rs:88` vs `model/quantifier.rs:7` | `Confidence` vs `QuantifierConfidence` — identical enums, bidirectional `From` impls |
| C2 | `narrative/llm/sanitize.rs:12` | Global `sanitize_llm_output` strips gemma-4 markers from ALL backends, including ones that may legitimately emit those strings |
| C5 | `narrative/llm_client/request.rs:53` | "Generic" `configure_request` carries OpenRouter-specific `X-Title`/`HTTP-Referer` via `Option<&str>` |
| C11 | `narrative/llm/{deepseek,ollama,openrouter,mock}.rs` | Identical `save_message` copy-pasted across 4 backends (empty trait default caused this) |
| D7 | `server/fragments/actions.rs:12` | `ActionForm { command: String }` reused for `check_text_handler` where semantics = "text to check" |

### Category 4: Helper Smell / Utility Abuse

Helpers that grow flags, params, or become god-functions.

| ID | Site | Evidence |
|----|------|----------|
| B3 | `action_pipeline/pipeline.rs:55` | `run_from_input` — 60+ line monolith, mutable state threaded through 6 phase calls |
| B4 | `application_service.rs:145` | 14 identity-wrapper methods → `GameLifecycleService` (source TODO admits it) |
| B5/B6 | `action_pipeline/phases.rs:44,199` | `phase_narrate` (6 args) and `build_trigger_request` (7 args) both `#[allow(clippy::too_many_arguments)]` |
| B9 | `action_pipeline/phases.rs:33` | `error_return` returns `Ok` while stuffing error into `state.status` |
| B10 | `application/message_editing.rs:124` | `retry`/`retrigger`/`process_action` duplicate `spawn_blocking` boilerplate |
| B11 | `application/query_handlers.rs:9` | `QueryHandlers` — zero-field struct, stateless wrappers |
| C6 | `narrative/llm/mock.rs:21` | `MockBackend` flag-bag: 6 `AtomicBool`/`AtomicU64` + per-call Vecs |
| D2 | `storage/backend/helpers.rs:4` | `empty_to_none` — one-function module, inlined elsewhere inconsistently |
| D4 | `storage/backend/core.rs:~190` | `Operation` enum threaded through every prod storage call, only used by `Backend::Test` branch |
| D12 | `storage/backend/*.rs` | Every method has `Backend::Test { .. } => unimplemented!()` dead arm |

### Category 5: Refactor-be-damned Extraction

Extract-and-relieve-symptom instead of fix root cause.

| ID | Site | Evidence |
|----|------|----------|
| A5 | `model/message_history.rs:15` | `MessageHistory` promises encapsulation, exposes `replace`/`retain`/`iter_mut` bypasses |
| A6 | `model/template.rs:5` | `TemplateVars` struct — 1 field, 1 consumer |
| A8 | `model/prompt_preset.rs:68` | `PromptPreset::assemble_prompt_text` eats world rules + response length (preset = god-assembler) |
| A9 | `model/prompt_preset.rs:113` | `push_section` — 1 caller, inlines trivial `if let` |
| A10 | `model/message.rs:130` | `Message::from_db` produces invalid domain object (empty text, no swipes) |
| A11 | `model/state.rs:24` | `MessageEntry` DTO mirrors `Message`+`Swipe`, no behavior |
| B1 | `action_pipeline/actions.rs:13` | `_player_name` param never read, caller still passes it |
| B7 | `action_pipeline/retry.rs:118` | `retry_event_continuation` is a miniature re-impl of pipeline trigger branch |
| B8 | `bootstrap/init_game.rs:55` | `ArrivalTaskContext` — 13-field ad-hoc re-implementation of the pipeline |
| C3 | `narrative/agents/quantifier/parser.rs:65` | `_all_rooms` param + `extract_movement_from_text` orphaned, never wired |
| C10 | `narrative/prompt/sanitize.rs:8` | `sanitize_for_prompt` — 1 caller |
| D3 | `storage/backend/core.rs:~180` | `with_backend_mut` forces `_game_id` dummy on most closures |
| D5 | `storage/backend/swipes.rs:~164` | `count_swipes_for_message` piggybacks on unrelated `Operation::LoadSwipesForMessages` |
| D6 | `server/view_models.rs:~152` | `ActionAreaViewModel::new(_exits: &[String])` — unused param |
| D9 | `server/fragments/actions.rs:~110` | `add_status_swap_headers` — 1 caller |
| D11 | `storage/models/*.rs` | Inconsistent `from_row`: some Db* have it, some don't |

---

## Severity Tally

| Category | high | med | low | total |
|----------|-----|-----|-----|-------|
| 1. Premature generalization | 4 | 4 | 2 | 10 |
| 2. Coincidental cohesion | 0 | 2 | 2 | 4 |
| 3. False deduplication | 0 | 3 | 2 | 5 |
| 4. Helper smell / utility abuse | 1 | 6 | 3 | 10 |
| 5. Refactor-be-damned extraction | 2 | 4 | 9 | 15 |
| **Zone model (A)** | 4 | 5 | 3 | 12 |
| **Zone app/engine (B)** | 2 | 8 | 2 | 12 |
| **Zone narrative (C)** | 1 | 4 | 6 | 11 |
| **Zone server/storage (D)** | 0 | 2 | 10 | 12 |
| **TOTAL** | **7** | **15** | **21** | **47** |

Server/storage zone is mostly low-severity — architecture is sound there, just local smells.

---

## Fix Strategy

### Tier 1 — Surgical deletes (low risk, high value)

Quick wins. Remove dead code, no behavior change. Do as one PR.

- Delete `ActionOutcome::Error` (B2), `PromptLayer::Phi` (C7), `narrate_continuation` trait method (C1)
- Delete `_player_name` param (B1), `_all_rooms` param + `extract_movement_from_text` (C3), `_exits` param (D6)
- Inline `push_section` (A9), `add_status_swap_headers` (D9), `sanitize_for_prompt` (C10)
- Remove `NarratorAgent` (C9) or wire it
- Add `Operation::CountSwipes` (D5) — fix the piggyback

### Tier 2 — Collapse single-consumer abstractions

Replace trait/enum with concrete type. Re-introduce when second consumer appears.

- `StatePatch` enum → `struct ScenePatch` (A1)
- `TriggerRequirement` enum → `struct TimesMetRequirement` (A2)
- `PromptAssembler` trait → use `LayeredPromptAssembler` directly (C4)
- Drop `preprocess_user_text` from trait; inline in `OllamaBackend::call` (C8)
- Unify `Confidence` and `QuantifierConfidence` (A3)
- `TemplateVars` → `render_template(text, user)` (A6)
- `QueryHandlers` struct → free functions (B11)

### Tier 3 — Fix error model (root-cause fix, multi-file)

The error-splitting is the highest-leverage structural fix. Unblocks B9, D4, D5, D12.

- Pick one error type at application boundary (likely `EngineError` or new `PipelineError`)
- Remove `GenerationStatus::Error` as error channel — keep `GenerationStatus` for status only
- Make `ActionPipeline` methods return `Result<_, EngineError>`; delete `error_return` (B9)
- Move `Operation` enum test-interception into a decorator/wrapper; storage methods stop taking `Operation` (D4, D12)

### Tier 4 — Restructure pipeline (high risk, plan first)

The pipeline re-implementation problem (B3, B7, B8). Needs architecture doc update per project rules.

- Decide: pipeline as state machine (`PipelineStep` enum) vs free functions vs current method-blob
- Make `retry.rs` and `ArrivalTaskContext` feed into the pipeline rather than re-implement it
- Group `phase_narrate`/`build_trigger_request` args into `PipelineInputs<'a>` (B5, B6)
- Either flatten `DefaultApplicationService` → `GameLifecycleService` or expose inner service directly (B4)
- Extract `spawn_pipeline_task<F>` helper or move spawn into `ActionPipeline` (B10)

### Tier 5 — Module reorganization (low risk, mechanical)

Address coincidental cohesion.

- Split `model/state.rs` into `generation.rs`, `movement.rs`, `scene.rs`, `narrative.rs` (A7)
- Split `fragments/misc.rs` into `text_check.rs`, `swipe.rs`, `game_control.rs` (D1)
- Split `fragments/renderers.rs` into `response.rs` + `fragment_renderers.rs` (D8)
- Move `NpcEncounterLog` CRUD out of `trigger_eval.rs` (B12)
- Move `extract_movement_from_text` or delete (C3)

### Tier 6 — Domain model fixes (highest risk, needs schema work)

- `Message` mirrored fields (A4, A10): normalize DB so swipes are first-class, derive active values via accessor. Or accept DB hydration as a separate `DbMessageRow` type that doesn't claim to be a valid `Message`.
- `GameStateSnapshot::apply_to` (A12): make `GameState::from_snapshot` the only path.

---

## Prevention: How to stop recurrence

### 1. Add a lint/cache for single-variant enums

Custom clippy or `arch-lint.toml` rule: deny `enum` with exactly one variant (outside `#[derive]` macros). Forces author to justify or convert to struct.
Suggested rule name: `no-single-variant-enum`.

### 2. Add lint for single-impl traits

`arch-lint` rule: traits with exactly one `impl` block trigger a warning unless marked `#[allow(single_impl_trait)]` with a justification comment. Catches `PromptAssembler`, `narrate_continuation`-style speculative traits.

### 3. Add lint for unused `_`-prefixed params

Already partially covered by clippy. Tighten: any `fn foo(_x: T)` where param unused should fail build, not warn. Currently B1 and C3 shipped.

### 4. Add lint for `#[allow(clippy::too_many_arguments)]`

Forbid the attribute repo-wide. Forces grouping into a context struct (B5, B6).

### 5. Add lint for `#[allow(dead_code)]` on enum variants

Forbid `#[allow(dead_code)]` on enum variants without an issue link. Catches B2.

### 6. Forbid `misc.rs` and `utils.rs` filenames

Add to `arch-lint.toml`: deny new files named `misc.rs`, `util.rs`, `utils.rs`, `helpers.rs` (unless pre-existing). Forces authors to name modules by concept.

### 7. Pre-commit check: helper extraction requires ≥2 callers

Add a pre-commit script using call-graph (LSP `incomingCalls`): any new private function/trait method with <2 callers prints a warning asking author to justify extraction or inline.

### 8. Architecture review checklist item

Add to `chronicler_engine/AGENTS.md` anti-patterns section:
> **Never** introduce a trait, enum variant, or generic parameter for a single consumer. Wait for the second real caller. "Future-proofing" is not a sufficient justification. Use a struct or concrete type.

### 9. Decision log for abstractions

Every new trait/enum requires a 1-line entry in `docs/architecture/abstractions.md`:

- Name, purpose, current consumers (must be ≥2 for traits)
If only 1 consumer: tracker issue must exist to either add the second or remove the abstraction.

### 10. Test for invalid domain objects

Add `arch-lint` or unit test rule: no `from_db` / `new` constructor may produce an object violating a documented invariant. Catches A10 (`Message::from_db` returning invalid Message).

---

## Recommended execution order

1. **Now** — Tier 1 surgical deletes (safe, ~1 day, removes ~200 lines of dead code)
2. **Next sprint** — Tier 2 collapse single-consumer abstractions (safe, mechanical, ~2 days)
3. **Plan required** — Tier 3 error model consolidation (multi-file, touching every storage + pipeline method)
4. **Plan required** — Tier 4 pipeline restructure (B3/B7/B8 — needs architecture doc update per AGENTS.md)
5. **Background** — Tier 5 module reorg, Tier 6 domain model (low priority, can be done piecemeal)
6. **Immediately** — Tier prevention: add arch-lint rules (Tier 1+2 of prevention) before any new abstractions are added

---

## Reports index

- `reports/zone-a-model.md` — 12 findings, model layer
- `reports/zone-b-app-engine.md` — 12 findings, application/bootstrap/engine
- `reports/zone-c-narrative.md` — 11 findings, narrative subsystem
- `reports/zone-d-server-storage.md` — 12 findings, server + storage
- `reports/abstraction-antipatterns-summary.md` — this file
