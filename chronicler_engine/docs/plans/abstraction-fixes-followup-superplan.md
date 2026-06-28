# Super-Plan: Abstraction-Fixes Follow-Up

**Date:** 2026-06-27 (last revised 2026-06-28)
**Status:** Planning / Sub-plans pending
**Scope:** `chronicler_engine/`

## Related

- Source plan (archived): `docs/plans/archived/abstraction-fixes-implementation-plan.md`
- Original investigation: `docs/reviews/abstraction-antipatterns-summary.md` + `docs/reviews/zone-{a,b}-*.md`
- Prevention plan: `docs/plans/abstraction-antipattern-healthcheck-plan.md`
- Reliability + concurrency findings extracted to: `docs/plans/reliability-and-cancellation-plan.md` (2026-06-28 split)

## Objective

Organise remaining abstraction/cleanup debt into independently-schedulable sub-plans. Each track is self-contained enough to become its own sub-plan. This super-plan does NOT re-litigate past decisions — the [Finding State](#finding-state) table is the single source of truth for what is done, deferred, or rejected. Each track section below describes only what its sub-plan must do.

**Scope narrowed 2026-06-28:** reliability and concurrency findings (N11/N17/N20/M7/M8 + the former T8 track) extracted to `docs/plans/reliability-and-cancellation-plan.md`. This plan now covers abstraction debt only.

Sub-plans MUST reference this super-plan as parent, verify against actual code (the source investigation had 4 misclassifications — verify everything), and define success criteria per `AGENTS.md` goal-driven execution.

## Track Listing

| # | Track | Readiness | Priority | Blocks |
|---|-------|-----------|----------|--------|
| T1 | Error Model Unification | ready — needs scoping decisions | P1 | none |
| T2-ARCH | Narration Deepening | needs grilling (G2–G5) | P1 | T1, ADR-018 |
| T3 | Service Layer Cleanup | done (2026-06-28) | P1 | none |
| T4 | MockBackend Modernization | done | P2 | none |
| T5 | Type Collapses (A3 + A6) | ready | P2 | none |
| T6 | MessageHistory Encapsulation | ready | P2 | none |
| T9 | Doc / Migration Debt | ready | P0 | none |
| T10 | Low-priority cleanup bundle | opportunistic | P3 | none |

Priority: P0 = no risk / anchors future work; P1 = structural/risk; P2 = debt prune; P3 = cosmetic.

T7 (Storage API Polish) is closed; see the Finding State table.

T2 (ArrivalTaskContext minimal cancellation chunk) was withdrawn 2026-06-28 — the underlying bug reframed as N21 (token registration gap) and extracted to `docs/plans/reliability-and-cancellation-plan.md` (R2). N5 (drift) remains in T2-ARCH below.

---

## T1 — Error Model Unification

**Findings owned:** A1, B2, B3, B9, N3 (also unblocks R1 in the reliability plan).

**Architecture-lens reframe:** the seam is misplaced. Pipeline methods return `PipelineResult<T>` BUT failure is also signalled via `state.narrative.input_buffer.status = GenerationStatus::Error(msg)`. `Ok` does not always mean success — the interface lies. ADR-018 + commit `8e4acf5` deliberately pinned errors onto `GenerationStatus`; revisit because the friction is real. Sub-plan should run skill ADR-conflict check before proceeding.

**Scope:**

- Pick a single error type at the pipeline boundary: `PipelineError` enum capturing `Cancelled` / `LlmFailed(String)` / `StorageFailed(EngineError)` / `QuantifierFailed(String)` / `TriggerFailed(String)`.
- Delete the `error_return` helper (`phases.rs:53-61`); replace with explicit `Err(PipelineError::...)` propagation.
- Retain `GenerationStatus::Error` purely for state-machine UI rendering — NOT for pipeline control flow.
- Convert every mid-flow `status.error_message().is_some()` check (~5 sites in `pipeline.rs` + `phases.rs`) to `?` propagation.
- Decide: fold `ActionOutcome::Cancelled` into `PipelineError::Cancelled` or keep separate as an exhaustiveness aid.

**Out of scope:** removing `GenerationStatus::Error` (UI status rendering depends on it).

**Blast radius:** ~3–5 files in `application/action_pipeline/`. Storage layer untouched.

---

## T2-ARCH — Narration Deepening

**Findings owned:** N5.

**Reframe:** T2's "two reimplementations of pipeline logic" is actually **one deep Narration module split across two adapters** (`phase_narrate` + `ArrivalTaskContext::run`). Per LANGUAGE.md two adapters = real seam. The deep module does not exist as an explicit module. A third LLM-call site (`phase_trigger_continuation_raw`) is a different seam (replays pre-assembled stored prompts; bypasses the assembler).

**Constraints any deepened module must resolve:**

1. **State ownership** — pipeline receives pre-loaded `GameState`; arrival owns `Arc<Storage>` and loads the snapshot inside `run()`.
2. **Preset acquisition** — pipeline calls `self.service.load_preset_and_response_length()` at call time; arrival receives `arrival_preset` + `response_length` baked into the struct at spawn time.
3. **Persistence policy** — both now route through `save_message_and_snapshot` (closed in T2-Q1 / `db8ab25`).
4. **Cancellation** — pipeline checks `cancel_token` pre+post LLM call; arrival task's token registration + checks are owned by R2 in `docs/plans/reliability-and-cancellation-plan.md`.
5. **Status reporting** — pipeline writes `GenerationStatus::Generating` and transitions through phases (`GeneratingEvent`, `Quantifying`); arrival only sets `Generating` then `Idle`/`Error`.
6. **Backwards data flow** — pipeline returns `(text, backend_name, model_name)` for caller to persist `LlmMessage` forensics; arrival discards `backend_name`/`model_name`.
7. **Domain role** — pipeline = "narrate user input"; arrival = "narrate arrival scene". Different domain meaning.

**Grilling decisions deferred to sub-plan (G1–G5):**

- **G2 (naming).** `Narrator` (conflicts with `AGENT_NARRATOR`?), `NarrationDriver`, `SceneNarration`, or two modules `ArrivalNarration` + `InputNarration` (rejects deepening).
- **G3 (state ownership shape).** (a) always receive `GameState`; (b) always own storage + load internally; (c) two entry points sharing internals.
- **G4 (cancellation).** (a) require `cancel_token` param; (b) `Option<&CancellationToken>`; (c) always require token with spawner supplying it.
- **G5 (ADR-018 conflict).** `architecture/system.md:208-209` currently documents `ArrivalTaskContext` as deliberate; revisit if deepening proceeds. Re-flag prevention is covered by this super-plan's Finding State table + T9 task 5 annotation on `abstraction-antipatterns-summary.md`.

(G1 — persistence difference — was the bug fixed in `db8ab25`.)

**Sub-plan deliverable:** full deepening (T2-ARCH). Don't propose interfaces yet — sub-plan runs `improve-codebase-architecture` skill grilling loop to resolve G2–G5 first. Coordinate with R2 in the reliability plan for the cancellation-plumbing grilling (G4).

**Side concerns:** ArrivalTaskContext has 12 fields (coupling smell); `phase_narrate` returns `backend_name`/`model_name` for forensics (deepened module must expose them or persist `LlmMessage` itself); `spawn_blocking` is caller's concern, the deepened module stays sync.

**Blast radius:** `bootstrap/init_game.rs` + `application/action_pipeline/phases.rs` + possibly new `application/narration.rs` (or `narrative/narration.rs`). High churn but mechanical-ish.

**Dependencies:** soft T1 (error shape); coordinate with T9 (ADR-018 doc update) + R2 in the reliability plan (cancellation plumbing).

---

## T3 — Service Layer Cleanup

**Status:** DONE — landed 2026-06-28. Archived plan: `docs/plans/archived/t3-service-layer-cleanup.md`. See CHANGELOG entry "T3 service-layer cleanup".

**Findings owned:** B10 (`spawn_pipeline_task` partial), N1 (9 identity-passthroughs).

**Resolved decisions:**
- 9 query-handler delegates deleted; server callers route to `query_handlers::X(ctx)` free fns directly.
- `MessageEditingService` struct deleted; 5 methods converted to module-level free fns in `application/message_editing.rs`. `retry` / `retrigger` take `&Arc<GameService>` via `game_service()` accessor; `switch_swipe` / `edit_history` / `delete_last` take only `ctx` (they don't touch game_service). Mirrors `query_handlers` pattern (decision: Option 1 — delete + free fns; struct was hollow, 3 of 5 methods didn't use game_service).
- Shared `spawn_pipeline_task(game_service, ctx, f)` free fn extracted to `application/spawn.rs`. Dedups only `Arc::clone` + `spawn_blocking`; cancel-check + `GenerationGuard` lifetime remain inside each caller's closure (zero behavior change).
- Observability: original 3 `tracing::debug!` lines from the private method moved into `process_action`'s closure only (per Option A); `retry` / `retrigger` stay unlogged as before.

**Out of scope (owned elsewhere):** retry/retrigger not setting `is_generating=true` — owned by R2 in `docs/plans/reliability-and-cancellation-plan.md`.

---

## T4 — MockBackend Modernization

**Status:** DONE — see `docs/CHANGELOG.md` Unreleased entry "MockBackend builder modernization (T4)" and archived plan `docs/plans/archived/t4-mockbackend-modernization.md`.

**Findings owned:** C6, N6, N7.

**Scope:**

1. Change the 6 AtomicBool/AtomicU64 + 2 Vec fields on `MockBackend` from `pub` to `pub(crate)`. Forces builder use externally.
2. Audit and remove 2 unused flags (likely `trigger_started` if no test checks it; `narration_started` likely still used).
3. Add `::succeeding()` builder (symmetric with `::failing()`).
4. Migrate ~100 `MockBackend::default()` test sites opportunistically. Irrelevant once fields go `pub(crate)` — old code still works.

**Blast radius:** `narrative/llm/mock.rs` + tests touching fields directly.

---

## T5 — Type Collapses (A3 + A6)

**Findings owned:** A3 (false seam per architecture-lens candidate #6), A6, N14.

**Scope:**

- **A3:** delete `QuantifierConfidence` (`model/quantifier.rs:8`), use `Confidence` everywhere. ~75 refs across ~15 files, all mechanical. Decide placement: keep `Confidence` in `model/agent.rs` or split to `model/confidence.rs`.
- **A6:** delete `TemplateVars` (replace signature `render_template(text, user: &str)`) — only one field, one function. (Accept keeping the struct only if a 2nd field is on the roadmap.)
- **N14:** derive `Ord` on `Confidence`, simplifying `StatePatch::merge` to `min()`.

**Blast radius:** A3 = 75 refs; A6 = 6 callers. Risk: test fixtures may rely on type names.

---

## T6 — MessageHistory Encapsulation

**Findings owned:** A5, N15.

**Architecture-lens reframe:** classic "tests want to test past the interface → module is wrong shape" (candidate #5). `replace`/`retain`/`iter_mut`/`as_slice` exist as implementation exposure for test setup; removing concentrates the MAX_MESSAGES cap-bypass bug into one place. Frame removal as interface correction.

**Scope:**

1. Remove `replace`, `retain`, `iter_mut`, `as_slice`, `clear` from the public API. Replace callers with the existing controlled API: `iter()`, `last()`, `len()`, `append()`, `delete_last()`, `edit()`.
2. `from_messages`: enforce the 1000-message cap (truncate), OR rename to `from_messages_trusted` + `#[doc(hidden)]` for storage loaders only. (N15: currently only `append` enforces the cap.)
3. Audit callers — `ArrivalTaskContext::run()` (`init_game.rs:135` area; `history.replace(msgs)`), `context.rs:180`, `retry.rs:75`, plus storage loaders + tests.

**Blast radius:** `model/message_history.rs` + callers.

---

## T9 — Doc / Migration Debt

**Findings owned:** N18, Phase 4 re-export shield, CHANGELOG Phase 4-7, `abstraction-antipatterns-summary.md` annotation, `docs/system/action_pipeline.md` missing, `docs/system/message_model.md` missing.

No code logic. Pure docs + ~5 import-path-only changes. Do FIRST so future audits have anchors.

**Scope:**

1. CHANGELOG — retroactive Phase 4-7 entries (module splits `state/`, `misc/`, `renderers/`; `GameLifecycleService` inline/delete; `Message` accessor pattern; `apply_to` deletion; `Operation` enum removal; `assemble_prompt_text` relocation).
2. `docs/system/action_pipeline.md` — spec covering `PipelineInputs` + `spawn_pipeline_task` helper contracts. Propose DELTA from current documented arch, not contradiction.
3. `docs/system/message_model.md` — spec covering accessor-pattern `Message` struct (reads from `swipes[active_swipe_index]`, no mirrored fields). ADR-017 alignment.
4. Migrate top-5 high-churn callers (`game_service.rs`, `application_service.rs`, `pipeline.rs`, `phases.rs`, +1) off the `state/mod.rs:12-18` re-export shield. Architecture-lens candidate #8: re-export shields are pure pass-throughs failing the deletion test; architecture debt, not migration debt. Leave the shield in place for opportunistic completion later.
5. Annotate `docs/reviews/abstraction-antipatterns-summary.md` with status notes ("resolved Phase 2", "deferred — see super-plan Finding State table"). Cross-reference each finding to its Finding State class + owner track, not just the 4 misclassifications (B2/B7/B8/B9).
6. Add `[DOC: ...]` anchors to the 7 `src/test_support/` files.

**Blast radius:** docs + ~5 import-path-only changes.

---

## T10 — Low-priority Cleanup Bundle

Bundle of 12 untouched + cosmetic items. All low severity, mechanical. Pick 3-5 during a sprint; each <1 hour. No structural risk.

- **A9** `push_section` — `narrative/prompt/assembler.rs:52`; 6 callers now (was 3). Keep-or-inline decision.
- **A11** `MessageEntry` DTO mirroring — `state/message_types.rs:16-32`. Collapse or `impl From<&Message>`.
- **D2** `empty_to_none` — `storage/backend/helpers.rs:5`; 5 callers. Inline or `String` extension trait.
- **D7** `ActionForm` reused for `check_text_handler` — `server/fragments/misc/text_check.rs:18`. Split `CheckTextForm`.
- **D9** `add_status_swap_headers` — 1 caller. Inline.
- **D11** `from_row` consistency — 4 of 9 Db* models have `from_row`. Low priority.
- **N13** `Ok(_) => unreachable!()` arms — 4 remain (`history.rs:23,36`, `misc/swipe.rs:17,30`). Invert to `let Ok(_) = ... else { ... }`.
- **N14** `Confidence` derive `Ord` — covered by T5 if A3 collapse happens.
- **N16** `list_personas` 3-line passthrough — `application_service.rs:434-439`. Move to direct storage calls (matches `list_worlds`).
- **M3** `response_length: Option<&str>` stringly typed — `narrative/prompt/assembler.rs:11`. Enum or token count.
- **M4** `QuantifierParseResult::is_high()` only — add `is_low()`/`is_medium()` or replace with derived `Ord` + comparison.
- **B12** `trigger_eval.rs` cohesion — `evaluate_triggers` + `NpcEncounterLog` CRUD helpers in same file.

---

## Finding State

Single source of truth for every original abstraction investigation finding (A/B/C/D/M series) + every abstraction-related NEW issue surfaced in the 2026-06-27 holistic re-review. Reliability + concurrency findings (N11/N17/N20/M7/M8 + former T8 track) extracted to `docs/plans/reliability-and-cancellation-plan.md` on 2026-06-28. For audit-visible annotations see T9 task 5 (`abstraction-antipatterns-summary.md`); architectural deferrals are also pinned in ADR-018 + CHANGELOG.

**Classes:**
- `closed` — work landed.
- `non-issue` — investigation got it wrong; nothing to fix. Locked.
- `deferred` — real finding; will fix in a future named sub-plan; locked until then.
- `active` — in a current track's scope; will be fixed when that track ships.
- `out-of-scope` — consciously not addressed (separate concern / other plan).

| ID | Class | Owner | Note |
|----|-------|-------|------|
| A1 `StatePatch` enum | deferred | T1 | single-variant enum; collapses when error seam unified |
| A2 `TriggerRequirement` enum | deferred | T5/T10 | single-variant enum; mechanical collapse |
| A3 `Confidence` vs `QuantifierConfidence` | deferred | T5 | false seam; ~75 refs |
| A4 `Message` mirrors `Swipe` | closed | — | accessor pattern landed (thermo-nuclear) |
| A5 `MessageHistory` encapsulation | active | T6 | `replace`/`retain`/`iter_mut`/`as_slice`/`clear`/`from_messages` all `pub`; cap bypass (N15) |
| A6 `TemplateVars` one field | deferred | T5 | mechanical; collapse with A3 |
| A7 `state.rs` grab-bag | closed | — | split into `state/` submodules (thermo-nuclear) |
| A9 `push_section` wrapper | active | T10 | 6 callers now (was 3); keep-or-inline |
| A10 (original) | closed | — | thermo-nuclear + `6a8531e` |
| A11 `MessageEntry` DTO mirroring | active | T10 | collapse or `impl From<&Message>` |
| A12 `apply_to` manual field clone | closed | — | deleted (thermo-nuclear) |
| B2 `ActionOutcome::Error` variant | deferred | T1 | unused at runtime; error channel moved to `GenerationStatus::Error` (`8e4acf5`) |
| B3 `run_from_input` monolith | deferred | T1 | state-machine rewrite scoped out per Phase 6.1 (Issue 9 constraint) |
| B4 `GameLifecycleService` | closed | — | flattened into `DefaultApplicationService` (thermo-nuclear); N1 passthroughs resolved by T3 |
| B7 retry mini-pipeline | non-issue | — | STALE finding: retry already delegates to `phase_trigger_continuation` + `run_from_input` (`architecture/system.md:52`, CHANGELOG L90) |
| B8 `ArrivalTaskContext` | deferred | T2-ARCH | deliberate extraction per ADR-018 / `system.md:208-209`; deepening deferred until a 3rd narration execution path emerges |
| B9 `error_return` returns `Ok` | deferred | T1 | deliberate arch per `8e4acf5` ("Unify error model onto GenerationStatus on state") |
| B10 `spawn_pipeline_task` helper | closed | T3 | extracted `application/spawn.rs`; reused by `process_action` + `retry` + `retrigger` (T3 landed 2026-06-28) |
| B12 `trigger_eval.rs` cohesion | deferred | T10 | CRUD helpers in same file as `evaluate_triggers` |
| C1 `NarratorAgent::narrate_continuation` | closed | — | zero prod callers; removed (thermo-nuclear) |
| C2 global `sanitize_llm_output` | out-of-scope | — | backend-agnostic sanitize policy, not a dedup bug |
| C4 `PromptAssembler` trait | closed | — | single-impl trait removed (thermo-nuclear) |
| C5 OpenRouter headers | out-of-scope | — | storage/LLM refactor, belongs to a separate plan |
| C6 `MockBackend` flag-bag | active | T4 | builders exist; fields still `pub`; `::succeeding()` missing |
| C7 `PromptLayer::Phi` | closed | — | unused variant removed (thermo-nuclear) |
| C8 `preprocess_user_text` hook | closed | — | single-backend override removed (thermo-nuclear) |
| C9 `NarratorAgent` `NoOp` | closed | — | removed (thermo-nuclear) |
| D1 `server/fragments/misc.rs` grab-bag | closed | — | split into `misc/` submodule (thermo-nuclear) |
| D2 `empty_to_none` one-fn module | active | T10 | `storage/backend/helpers.rs:5`; 5 callers |
| D3 `with_backend_mut` signature | closed | — | `(method: &'static str, f)` retained; achieves goal of no dummy closures |
| D7 `ActionForm` reused for `check_text` | active | T10 | `server/fragments/misc/text_check.rs:18`; split `CheckTextForm` |
| D8 `renderers.rs` cohesion | closed | — | split (thermo-nuclear) |
| D9 `add_status_swap_headers` | active | T10 | 1 caller; inline |
| D11 `from_row` consistency | deferred | T10 | 4 of 9 Db* models have it; low priority |
| M1 `Backend::Test` prod visibility | closed | — | feature-gated behind `testing` (LayeredBackend split `f914bae`) |
| M2 `with_failure` non-idempotent | closed | — | idempotent per `with_failure_adds_does_not_nest` test (LayeredBackend split) |
| M3 `response_length: Option<&str>` stringly | active | T10 | `narrative/prompt/assembler.rs:11`; enum or token count |
| M4 `QuantifierParseResult::is_high()` only | active | T10 | add `is_low/is_medium` or derive `Ord` |


| N1 9 identity-passthrough methods | closed | T3 | delegates deleted; server routes to `query_handlers::X` directly (T3 landed 2026-06-28) |
| N2 40 `Backend::Test` dead arms | closed | — | removed in thermo-nuclear `6a8531e` + `f914bae` |
| N3 new code self-invents `state.status` error side-channel | deferred | T1 | consequence of B9; T1 fixes root cause |
| N5 prompt-context + LLM + persist drift between `ArrivalTaskContext` and `phase_narrate` | deferred | T2-ARCH | one deep Narration module split across two adapters |
| N6 `MockBackend::default()` migration backlog | deferred | T4 | cosmetic; irrelevant once fields go `pub(crate)` |
| N7 builder ergonomics not enforced | active | T4 | fields still `pub` |

| N12 `context.rs:96-97` panics | non-issue | — | STALE: inside `#[cfg(test)] fn load_state_for_test`; `lib.rs:4-11` denies `clippy::panic` with `cfg_attr(test, allow(clippy::panic))` |
| N13 `Ok(_) => unreachable!()` arms in fragments | active | T10 | 4 remain (`history.rs:23,36`, `misc/swipe.rs:17,30`); invert to `let Ok(_) = ... else {}` |
| N14 `Confidence` derive `Ord` | deferred | T5 | collapses `StatePatch::merge` to `min()`; bundle with A3 collapse |
| N15 `from_messages` bypasses MAX_MESSAGES cap | active | T6 | only `append` enforces 1000 cap |
| N16 `list_personas` 3-line passthrough | active | T10 | `application_service.rs:434-439`; move to direct storage calls |
| N18 7 `test_support/` files lack `[DOC: ...]` anchor | active | T9 | add anchors |
| N19 no ADR protecting deliberate-defer | closed | — | resolved: deliberate-defers protected by super-plan Finding State table + T9 task 5 annotation on `abstraction-antipatterns-summary.md` + existing ADR-018 + CHANGELOG `8e4acf5`. No new ADR needed. |
| Phase 4 re-export shield | resolved | — | done 2026-06-28 via holistic migration; shield deleted, all callers migrated to direct submodule paths |
| CHANGELOG Phase 4-7 | active | T9 | only Phase 1-3 + thermo-nuclear logged |
| Reliability + concurrency findings | closed | — | extracted to `docs/plans/reliability-and-cancellation-plan.md` on 2026-06-28 (N11/N17/N20/M7/M8 + former T8 track) |
| `docs/system/action_pipeline.md` missing | active | T9 | Task 6.1 |
| `docs/system/message_model.md` missing | active | T9 | Task 7.1 |
| T2-Q1 arrival persistence bug | closed | — | `db8ab25` routes `ArrivalTaskContext::run` through `save_message_and_snapshot` |
| T7 Storage API Polish | closed | — | `f914bae` + `c4ff37b` + `6a8531e`; archived sub-plan `docs/plans/archived/t7-storage-backend-layered-split.md` |
| `Operation` enum | closed | — | removed (thermo-nuclear) |
| `GameLifecycleService` deleted | closed | — | thermo-nuclear |

**Re-flag rule:** any future audit referencing an item above with class `closed` or `non-issue` is itself a stale finding. Items `deferred` or `active` are owned by a named track; re-flag only against that track's sub-plan, not the raw finding.

---

## Decisions to Lock Before Sub-Plans

These decisions belong in each sub-plan, not this super-plan. Visible here:

- **T1:** Fold `ActionOutcome::Cancelled` into `PipelineError::Cancelled` or keep separate?
- **T2-ARCH:** Extract shared `NarrationDriver` now, or sync helper overlap informally and pin for future?
- **T3:** ~~Keep `MessageEditingService` (promote to HistoryMutation) or delete the 5 delegates?~~ Resolved 2026-06-28 — delete struct + convert to free fns (Option 1).
- **T4:** Migrate 100 `MockBackend::default()` test sites, or leave (fields go `pub(crate)` so old code still works)?
- **T5 A3:** Move `Confidence` to new `model/confidence.rs`, or keep in `agent.rs`?
- **T6:** `from_messages` for storage loaders: enforce cap, or add `from_messages_trusted`?

---

## Verification Strategy

- Each sub-plan defines its own test guarantees + verification commands.
- Project-level: `python build.py` (fmt + clippy + tests + coverage) is the gold standard.
- Healthcheck: `python scripts/healthcheck.py` enforces the jscpd duplicates check; the abstraction-antipatterns-healthcheck-plan will add advisory detection for `too_many_arguments` + `dead_code`.
- For structural tracks (T1, T3): require integration test coverage of new behaviour paths before merge.

## Sub-plan Creation Order (recommended)

1. **T9 (docs)** — P0; no risk, anchors everything else.
2. **R2 (arrival task cancellation, in reliability plan)** — P0; real production race (reset-during-arrival + ctrl-C-during-arrival).
3. **T1 (error model)** — P1; unblocks downstream cleanup; root cause.
4. **R1 (persistence reliability, in reliability plan)** — P1; behaviour risk, sequence after T1.
5. **T3 (service layer)** — P1; quick win after T1.
6. **T5 (A3 + A6)** — P2; mechanical, do alongside any of above opportunistically.
7. **T6 (MessageHistory)** — P2; isolated, can run parallel.
8. **T4 (MockBackend)** — P2; cosmetic migration; can run alongside anything.
9. **T10 (low-priority bundle)** — P3; pick off during sprints.
10. **T2-ARCH (Narration deepening)** — P1; needs G2–G5 grilling first.

## Plan Adherence

This super-plan **does not dictate implementation**. It enumerates:

- What's done (locked, won't re-litigate) — see Finding State table.
- What's deliberately deferred (with reasons) — see Finding State table.
- What remains (with sub-plan scope + blast radius) — see track sections.
- Recommended order (advisory).

Where a sub-plan deviates from this super-plan's recommendation, stop and report per `AGENTS.md` "Plan Adherence" rule.
