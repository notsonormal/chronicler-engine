# Super-Plan: `simpler-hexagon` Pre-Merge Cleanup

**Date:** 2026-07-09
**Status:** In progress — T2 COMPLETE (tickets 00–04 resolved 2026-07-09); T3 glossary renames COMPLETE 2026-07-10 (4 of 4 in-scope terms; glossary-2 carved to T9); T1, T4–T8, T9 pending
**Scope:** `chronicler_engine/` on branch `simpler-hexagon` (HEAD `1eda563` at plan creation; T2 work landed post-HEAD)
**Total estimated effort:** ~60 SP across 8 tracks

## Related

- Architecture review HTML: `/tmp/architecture-review-1783626107.html` (47 KB, 4 candidates + glossary drift + decision verdicts)
- Antipattern fresh pass: `/tmp/antipattern-fresh-2026-07-09.md` (30 findings, 19 files)
- Antipattern prior pass: `/home/moridin84/projects/mrn-general/tmp/antipattern-check-simpler-hexagon-2026-07-09.md` (16 findings)
- Code review (Standards + Spec): `/home/moridin84/projects/mrn-general/tmp/code-review-simpler-hexagon-2026-07-09.md`
- Code simplification pass 2: `/home/moridin84/projects/mrn-general/tmp/code-simplification-pass2-simpler-hexagon-2026-07-09.md`
- Thermo-nuclear code quality review: `/home/moridin84/projects/mrn-general/tmp/simpler-hexagon-review.md`
- Depth analysis: `/tmp/depth-analysis-2026-07-09.md` (7 modules)
- Domain reconciliation: `/tmp/domain-reconciliation-2026-07-09.md` (14 terms + glossary gaps)
- Original umbrella plan: `chronicler_engine/docs/plans/opcontext-kill-plan.md` (now partly stale)
- ADR-027 (hexagonal architecture): `chronicler_engine/docs/adr/adr-027-hexagonal-architecture-migration.md`
- ADR-030 (is_generating invariant): `chronicler_engine/docs/adr/adr-030-is-generating-invariant.md`

## Objective

Consolidate findings from 4 prior reviews + 2 fresh antipattern passes + depth analysis + domain reconciliation + 4 doubt decisions into a single super-plan. Each track is rated for **Practical Benefit** (low / med / high) and **Certainty** (low / med / high). After user approves this super-plan, tracks split into independently-schedulable sub-plans.

## Rating Definitions

- **Practical Benefit** — what the work buys. "High" = removes real defect or unblocks future work. "Med" = code quality with measurable test/readability gains. "Low" = cosmetic / taste.
- **Certainty** — confidence the work lands as described. "High" = clear evidence, mechanical refactor, well-scoped. "Med" = depends on choices during grilling (G2-G5 style). "Low" = speculative, may turn into deeper rewrite.

## Track Listing

| # | Track | Readiness | Priority | Practical Benefit | Certainty | Blocks | SP est |
|---|-------|-----------|----------|-------------------|-----------|--------|--------|
| T1 | Tier 1 blockers (4 bugs) | ready | **P0** | High | High | T2-T5 | ~8 |
| T2 | C1 god-class split (4 modules + PresetStore) | **COMPLETE** (2026-07-09, tickets 00–04) | P1 | High | Med-High | T4 | ~16 |
| T3 | Glossary drift (4 terms + doc sweep) | ready | P1 | Med | High | none | ~4 |
| T4 | C2 PhaseError consolidation | needs grilling G3 | P2 | Med | Med | none | ~5 |
| T5 | C3 TestApp builder collapse | ready | P2 | Med | Med | none | ~16 |
| T6 | Plan/ADR honesty | ready | P0 | Med | High | none | ~1 |
| T7 | Mechanical antipattern polish (~25 findings) | opportunistic | P3 | Low-Med | High | none | ~10 |
| T8 | Workflow gates (D3/D4 prevention) | needs scope | P2 | Med | Med | none | ~3 |
| T9 | WorldSnapshot removal (immutable-world-data out of GameState) | needs grilling | P2 | High | Med | none | ~13 |

P0 = defects / required for merge; P1 = structural / load-bearing; P2 = debt prune; P3 = cosmetic.

**Total**: ~77 SP. **Tasks 8+ SP must break down per AGENTS.md.** T2 and T5 each split into sub-plans; T7 splits per finding cluster. T9 will need its own sub-plan.

---

## T1 — Tier 1 Blockers (4 Bugs)

**Findings owned:** `simpler-hexagon-review.md` blockers 2-4 + `antipattern-fresh-2026-07-09.md` #2, #14, #29 + `code-review-simpler-hexagon-2026-07-09.md` HARD on `layers.rs`.

**Practical Benefit: HIGH** — these are defects in HEAD, not taste calls. Three cause wrong behavior under realistic inputs; one falsifies a CI guardrail.

**Certainty: HIGH** — every finding has file:line evidence and a concrete fix. No grilling required.

### Scope

1. **`load_or_fresh` type lie** (`application_service.rs:260-279`)
   - Change `fn load_or_fresh(&self) -> Result<GameState, EngineError>` → `fn load_or_fresh(&self) -> GameState`
   - Drop dead `?` operators: `application_service.rs:451,470`; `message_editing.rs:85,98,117,144`; `tests/helpers/pipeline_helpers.rs:122`
   - Delete `load_state_lossy` (`query_handlers.rs:13-29`) entirely — its `Err` arm is unreachable

2. **`make_test_app_with_storage_and_service` cancel_token drop** (`src/test_support/context.rs:197-202`)
   - Either inline `finalize_app`'s token-preserving variant, or delete this helper and migrate its callers to `app_with_storage_from` (`tests/helpers/fixtures.rs:409`)
   - Decision: sub-plan runs `app_with_storage_from` audit + migrates callers; deletes broken helper

3. **Stale guardrail** (`tests/infrastructure/guardrails/layers.rs:5-9`)
   - Drop `application/context.rs` from `APPLICATION_STORAGE_GRANDFATHERED`
   - Drop the corresponding test assertion (lines 222-229)
   - Verify the other 4 entries still match real files

4. **`arrival_service::run` silent return** (`src/application/arrival_service.rs:52-58`)
   - Restore the prior fresh-state fallback + scenario-log injection on `load_expecting_valid_state` Err
   - Use `app.build_fresh_initial_state()` then `inject_scenario_logs` if needed
   - Add `arrival_service_tests::falls_back_to_fresh_state_on_load_error` regression test

**Out of scope:** god-class split (T2); test factory consolidation (T5); behavior change beyond the 4 bugs above.

**Blast radius:** 4 source files + 1 test file + 1 guardrail file. ~150 lines changed.

**Validation per task:** `cargo test` + `python build.py` green; one regression test per bug.

**Honest tradeoff:** T1 is non-negotiable. The 4 fixes are independent — can land in any order, even as 4 small PRs. Pair them with one ADR capturing "OpContext-kill regression audit lessons" so the same class of defect doesn't reappear.

---

## T2 — C1 God-Class Split (4 Modules + PresetStore Newtype)

**Status: COMPLETE (tickets 00–04 resolved, 2026-07-09).** `python build.py` green; `application_service.rs` 723 → 275 LOC; 4 modules + 3 type extractions landed; façade-first preserved (~30 caller sites untouched). Map + per-ticket resolutions: `.scratch/t2-god-class-split/`. Follow-ups (AppState token/phantom-storage, ADR-032, ≤200 LOC stretch, full caller-site migration G1-B) live in the map's §Not-yet-specified — separate efforts, not part of T2.

**Findings owned:** `antipattern-fresh-2026-07-09.md` #1, #4, #5, #6, #11, #17, #19 + `simpler-hexagon-review.md` blocker #1 + `code-simplification-pass2-simpler-hexagon-2026-07-09.md` #4 (partial) + doubt verdicts D1, D2 + depth analysis M1, M2.

**Practical Benefit: HIGH** — root cause of the entire defect cluster. 49-method god-object + 2 phantom `Arc<Storage>` + leaked `AtomicBool` was the deepest problem on the branch.

**Certainty: MED-HIGH** — the carve-out was mostly mechanical (existing method clusters became modules). G1 was the controlling grilling decision.

### Scope

Carve `DefaultApplicationService` (723 LOC, 49 methods, 6 pub(crate) fields) into:

1. **`PersistenceGate`** (`application/persistence_gate/`) — ✅ landed (ticket 02)
   - Owns `Arc<Storage>` (game) + `Arc<PresetStore>` newtype (preset)
   - Methods: 14 persistence helpers (`load_world_snapshot`, `load_or_fresh`, `load_expecting_valid_state`, `save_state`, `save_message_and_snapshot`, `delete_and_remove_message`, `load_messages_with_swipes`, `load_messages_into_state`, `build_fresh_initial_state`, `load_messages`, `update_message_text`, `find_retry_anchor`, `set_game_id`, `world_snapshot_or_empty`)
   - The PresetStore newtype (landed in ticket 01) is the load-bearing fix for the phantom-Storage seam

2. **`GenerationGate`** (`application/generation_gate/`) — ✅ landed (ticket 03)
   - Owns `cancel_token`, `is_generating`, `start_action` (moved from `process_action` body — 47 LOC preserved verbatim), `claim_generation_slot`, `release_generation_slot`, `heal_stale_generating`
   - Still uses current 5-style error handling — T4 PhaseError integration deferred

3. **`GameCatalogue`** (`application/game_catalogue/`) — ✅ landed (ticket 04)
   - Methods: `create_game`, `switch_game`, `delete_game`, `list_games`, `current_game_id`, `reset`, `persist_initial_state_with_swipes`
   - Storage orchestration; reads `is_generating` (borrowed `Arc<AtomicBool>` clone, ADR-030 single-writer preserved — read-only) but does not own the token

4. **`WorldCatalogue`** (`application/world_catalogue/`) — ✅ landed (ticket 04)
   - Methods: `list_worlds`, `get_world`, `create_world`, `update_world`, `delete_world`, `list_personas`

5. **`DefaultApplicationService` façade** (`application/application_service.rs` — shrunk to 275 LOC, stretch target ≤200 reclassified as T7-class polish)
   - Holds 6 fields: 4 module structs (`persistence_gate`, `generation_gate`, `game_catalogue`, `world_catalogue`) + 2 collaborators (`settings`, `game_service`) — **Decision A** (ticket 04); collaborators not storage-state, never moved
   - All migrated methods become 1-line delegates (façade pattern, G1=A)

6. **Module extraction** (free) — ✅ landed (ticket 04, except `WorldSnapshot` which moved in ticket 02):
   - `ApplicationError` + `ProcessActionResult` → `application/errors.rs`
   - `map_llm_error` → `application/mappers.rs`
   - `WorldSnapshot` → `application/persistence_gate/dto.rs`
   - `DebugStateView` → `application/debug/dto.rs`
   - Re-exported from `application/mod.rs` + `application_service.rs` so ~14 external caller import paths stay valid (façade-first)

### Grilling Decisions (resolved)

- **G1: façade vs full migration → A (façade-first).** `DefaultApplicationService` keeps method signatures as thin delegates; ~30 caller sites untouched. Full migration (G1-B) deferred to a follow-up PR. Decided in ticket 00.
- **G2: PresetStore newtype location → B (port + adapter).** `adapters/driven/storage/preset_store.rs`, hexagonal ADR-027-aligned. Decided in ticket 00.

### Out of scope

- Test factory consolidation (T5) — orthogonal; T5 can land before or after.
- C2 PhaseError (T4) — adjacent; can run in parallel.
- C4 AppState shrink — T2 confirms `AppState` still holds its own `Arc<Storage>` + `Arc<RwLock<CancellationToken>>` (phantom storage + dual-token concerns); separate smaller effort, sharp enough to ticket now (map §Not-yet-specified).

### Blast Radius (actual)

- ~30 caller files in `src/application/`, `src/adapters/driving/http/`, `src/bootstrap/`, `tests/integration/` — signatures preserved
- 1 new type (`PresetStore`)
- 4 new modules + 3 type extraction files + 1 façade shrinkage
- ~10 forced field-access → accessor-call rewrites (`app.storage.X` → `app.storage().X`) in `message_editing.rs`, `query_handlers.rs`, `phases.rs`, `action_pipeline/retry.rs` — no signature changes (not caller-site migration)
- `tests/infrastructure/guardrails/layers.rs`: `game_catalogue/gate.rs` + `world_catalogue/gate.rs` added to `APPLICATION_STORAGE_GRANDFATHERED`
- `docs/architecture/system.md` §2.5 updated with new module + type entries

### Validation (actual results)

- `cargo build` + `cargo test` + `python build.py` green (1241 passed, 2 LLM skipped) — one pre-existing flaky race in `fragment::test_action_confirm_empty_command` (passes in isolation + on retry; not caused by T2)
- `grep -rn 'self.storage' src/application/application_service.rs` → 0 hits ✓ (storage routed via `self.persistence_gate.storage()`)
- `grep -rn 'self.is_generating' src/application/application_service.rs` → 0 hits ✓ (routed via `self.generation_gate.is_generating()`)
- `wc -l application_service.rs` = **275** (not ≤200 — Decision B in ticket 04; ≤200 would require breaking façade-first or moving ticket's "STAYS" items (`active_quantifier_prompt`, `load_messages_with_swipes` free fn); real destination (god-class gone, no phantom seam, 4 cohesive modules, façade-first) achieved)

### Honest Tradeoff (post-implementation)

T2 was the load-bearing fix; façade-first (G1=A) was the right call. Real cost: LOC target slipped 200 → 275 (honest payoff — the alternative was breaking façade-first or migrating ~30 caller signatures, both out of scope this wave). `Settings`/`game_service` collaborators stayed on the façade (Decision A — they're not storage-state, no category-error module invented to hit a literal field count). AppState phantom-storage + dual-token concerns persisted as fog — graduate to separate tickets now that T2's shape is stable.

---

## T3 — Glossary Drift (4 Terms)

**Findings owned:** `domain-reconciliation-2026-07-09.md` Persona + Turn + Action Pipeline + avoid-aliases.

**Practical Benefit: MED** — code matches docs; future agents don't trip on contradictions. Avoid-alias leakage is real but minor.

**Certainty: HIGH** — renames are mechanical (with `sed`-class scripts + careful review).

### Scope

1. **`PersonaCard` rename** (~60 sites, ~3 SP)
   - `PlayerCard` → `PersonaCard` everywhere (struct, fields, prompts, tests)
   - `PlayerCardWithKey` → `PersonaCardWithKey`
   - `TestPlayer` → `TestPersona`
   - `PromptLayer::Player` → `PromptLayer::Persona`
   - `PromptContext::player` field → `PromptContext::persona`
   - Update `domain/model/character.rs:51`, `adapters/driven/storage/backend/core.rs:51`, `adapters/driven/storage/models/persona.rs:5` (note: type alias already correct, fields are wrong)
   - ADR-026 already renamed bindings; this finishes the value type

2. **`TurnResult` → `ActionResult`** (~6 sites, ~0.5 SP)
   - `domain/engine/action_processing.rs:27` + callers + `action_pipeline/pipeline.rs:90-100`

3. **Action Pipeline phase split OR glossary amend** (~0.5 SP)
   - Either split `phase_trigger_evaluation` out of `phase_engine_commit` (glossary alignment) OR amend `CONTEXT.md` to document the merge. Pick one. Recommend amend glossary — code is working; docs should match.

4. **Avoid-alias sweep** (~0.5 SP)
   - `StoryLogTemplate` → `NarrativeLogTemplate` (`templates.rs:28`)
   - `parse_command` → `parse_action` (`engine/parser.rs:6`)
   - Doc comments using `session`/`command`/`text`/`output` → glossary terms
   - One-line fixes; ~10 sites

### Out of scope

- New glossary terms (gap analysis identified `NarrativeState`, `SceneState`, `MovementState`, `InputBuffer`, `GenerationStatus`, etc.) — separate doc-only plan

### Blast Radius

- ~80 sites across domain, application, adapters, tests
- 1 ADR-026 amendment (Persona rename)
- 1 CONTEXT.md amendment (Action Pipeline phase merge)

### Honest Tradeoff

T3 is safe to land mechanically. Risk: test fixture naming has been around long enough that grep misses are likely. Sub-plan should run `grep -rn 'PlayerCard\|Player\b'` (word-boundary!) before declaring done. The Persona rename is the largest; can split T3.1 from T3.2-T3.5.

**WorldSnapshot removed from T3 scope (2026-07-10 grilling).** Decision A-Deep locked: remove immutable world fields from `GameState` + add AppState Arc cache for world data (CONTEXT.md already claims this cache exists but it does not — `app_state.rs:41` has no world cache; `GameService` has no cache). `WorldSnapshot` dies naturally once `GameState::from_snapshot(snapshot)` takes 1 arg. This is architectural work adjacent to T2 arch-3 (AppState phantom-storage, Ticket 05) — same seam. Moved to new Track T9. The T3 plan's original D1 framing ("err path unreachable in practice") was factually wrong: `world_snapshot_or_empty()` runs on every `load_or_fresh` Err (corrupted/missing game, world, persona, or character rows) — see `gate.rs:76-80,88,149` + `retry.rs:67`.

---

## T4 — C2 PhaseError Consolidation

**Findings owned:** `code-simplification-pass2-simpler-hexagon-2026-07-09.md` #4 (run_from_input) + #5 (retry_last_response_impl) + `antipattern-fresh-2026-07-09.md` #5 (3 overlapping "save state" paths) + depth analysis M4, M5.

**Practical Benefit: MED** — orchestrators shrink ~60%, error handling becomes type-driven, tests can target individual phases instead of all error paths.

**Certainty: MED** — needs G3 decided (where PhaseError lives, who converts to ActionOutcome).

### Scope

1. **Introduce `PhaseError` enum** (`application/action_pipeline/phase_error.rs`)
   - Variants: `Cancelled`, `NarratorFailed(String)`, `PersistFailed { label: &'static str, source: EngineError }`, `TriggerMissing`, `SnapshotMissing`
   - All phase methods return `Result<PhaseOutput, PhaseError>` instead of mixed `(GameState, String, String, String)` + early-return + bool

2. **Migrate `run_from_input`** (`pipeline.rs:47-141`, 95 → ~40 lines)
   - Linear orchestration: `step → step → step → map PhaseError`
   - All 5 error styles collapse to 1 `match` on `PhaseError`

3. **Migrate `retry_last_response_impl`** (`retry.rs:14-105`, 92 → ~50 lines)
   - 5 early returns collapse to `collect_retry_anchor()?` then dispatch
   - Single `RetryError` type subsumes the ad-hoc `tracing::error!` + `save_retry_error` paths

4. **Reconcile the 3 "save state" paths** (antipattern #5)
   - `save_message_and_snapshot` (application_service) vs `save_message_and_snapshot` (arrival_service) vs `pipeline.persist` — pick one canonical name; document the difference

### Grilling Decision

- **G3: PhaseError → ActionOutcome bridge location.** Either `run_from_input` itself converts (current pattern, ok), or move into `PipelineRun::run` (cleaner but bigger diff). Recommend PipelineRun.

### Out of scope

- Storage seam (T2 PersistenceGate) — orthogonal
- Test factory changes (T5)

### Blast Radius

- `application/action_pipeline/{pipeline,retry,phases}.rs` — 3 files
- ~150 lines changed

### Honest Tradeoff

T4 is the cleanest "worth exploring" candidate in the report. Risk: changing phase method signatures touches every caller in the orchestration layer. Mitigation: introduce PhaseError as a type alias first (no behavior change), migrate one phase at a time, leave orchestrators for last. Sub-plan can land in ~3 PRs.

---

## T5 — C3 TestApp Builder Collapse

**Findings owned:** `antipattern-fresh-2026-07-09.md` #7, #12, #14, #15, #22, #25, #29 + `simpler-hexagon-review.md` blocker #2 + `code-simplification-pass2-simpler-hexagon-2026-07-09.md` #6 + depth analysis M6, M7.

**Practical Benefit: MED** — tests become shorter (estimated -150 lines across retry_main + lifecycle + game_service); future tests get easier to write.

**Certainty: MED** — needs choice between 2 design paths (G4 below).

### Scope

1. **Extract `seed_narrative_into_storage`** (`src/test_support/seed.rs`, ~10 lines)
   - The 6-line insert loop currently duplicated in 7 sites
   - Caller-side: 7 lines become 1 line

2. **`TestApp::make` API** (`src/test_support/test_app.rs`)
   ```rust
   TestApp::make(state, BackendSpec, StorageSpec).build()
   ```
   where `BackendSpec = Mock(MockBackend) | NarratorOnly(MockBackend) | Separate{n, q} | Custom(Arc<GameService>)` and `StorageSpec = InMemory | Sqlite`

3. **4 sub-builders** (`src/test_support/builders/`)
   - `WorldFixture { world, map, player, npcs, room_npcs }`
   - `NarrativeFixture { logs, last_trigger, generation_status, generation_phase }`
   - `GenerationFixture { is_generating, cancel_token }`
   - `InfraFixture { settings, storage, game_service }`

4. **Delete dead code**
   - `make_test_app_with_default_preset` (`tests/helpers/fixtures.rs:425-471`, `#[allow(dead_code)]`)
   - `let _ = world_snapshot;` (`tests/helpers/fixtures.rs:397`)
   - `_created_storage` tuple bool (`test_app_builder.rs:199`)
   - `arrival_service.rs::new_for_test` test-only constructor → migrate to `test_support::builders::arrival_context`

5. **Migrate test files** (sub-tasks)
   - 5a. `tests/integration/flow/retry_main.rs` (-99 lines)
   - 5b. `tests/integration/application/*` (-100 lines combined)
   - 5c. Remaining flow + game_service tests

### Grilling Decision

- **G4: sub-builder composition vs single builder.** Sub-builders add 4 imports but compose cleanly. Single builder with 4 namespaced field groups (`app.world.world`, `app.world.player`) saves imports. Recommend sub-builders — clearer API.

### Out of scope

- T2 god-class split (T5 can land first; T2 lands cleaner after)
- T4 PhaseError (orthogonal)

### Blast Radius

- `src/test_support/{context.rs, test_app_builder.rs}` — collapse 2 files to ~5
- `tests/helpers/fixtures.rs` — lose ~100 lines
- `tests/integration/{flow,application}/*.rs` — lose ~250 lines combined

### Honest Tradeoff

T5 is the lowest-priority structural item — tests work today, just clumsily. Payoff is "future tests are easier" more than "current tests are correct." Can ship as one PR OR parallel T2 (recommended: ship T5 last, after T2 settles the production shape).

---

## T6 — Plan/ADR Honesty

**Findings owned:** `code-review-simpler-hexagon-2026-07-09.md` "Spec (a) Missing / Partial" + `simpler-hexagon-review.md` "Plan validation gates silently failed" + doubt verdict D3.

**Practical Benefit: MED** — future agents reading the plan won't be misled. Avoids the "the plan says X but the code does Y" loop.

**Certainty: HIGH** — pure documentation work.

### Scope

1. **Mark plan §T2.2 / §T2.3 VOID** (`chronicler_engine/docs/plans/opcontext-kill-plan.md`)
   - T2.2 (OpContext FromRequestParts extractor): replaced by direct `state.application_service.X()` access. Mark void with rationale: "OpContext-kill commit 20cacf9 made the extractor moot."
   - T2.3 (GameState::from_snapshot WorldSnapshot variant): the WorldSnapshot variant was never wired; constructors still take 4-5 args. Mark void with rationale: "Cosmetic; deferred until T2 lands."

2. **Mark plan §A6.4 validation gate VOID with rationale**
   - Gate was: `grep -rn 'WorldSnapshot' src/ tests/ returns 0`. Actual: 7 hits. The struct remains in use. Either fix the gate (rename targets in T3.2) or remove the gate.

3. **Mark plan §B1.3 validation gate VOID with rationale**
   - Gate was: `process_action` body ≤30 lines. Actual: 47 lines. The gate was aspirational; the body is doing real work. Update the gate to ≤50 lines or split into 2 functions.

4. **Write ADR-031** (`chronicler_engine/docs/adr/adr-031-opcontext-absorption-tradeoffs.md`)
   - Documents the decision: "OpContext deletion preserved hexagonal boundary but concentrated complexity into DefaultApplicationService; this is acceptable trade-off pending T2 (god-class split)."
   - Cites depth analysis M1 + antipattern #11, #17 + doubt verdicts D1, D2.
   - If T2 lands, this ADR serves as historical record; if T2 is rejected, this ADR is the rationale for the absorption decision.

5. **Amend ADR-030** (`chronicler_engine/docs/adr/adr-030-is-generating-invariant.md`)
   - Add an "Access pattern" section noting `pub(crate)` widening on `DefaultApplicationService` is deliberate and expected to be re-tightened by T2 GenerationGate.

### Out of scope

- Plan §T2.2 resurrection (T2's PersistenceGate + GenerationGate is the actual replacement, not the extractor)
- Plan §T2.3 resurrection (T3's WorldContext rename is the actual replacement)

### Blast Radius

- 1 plan file + 2 ADR files (1 new, 1 amended)

### Honest Tradeoff

T6 is required for plan-honesty but **not for code correctness**. Could ship with T1 (blockers) as the "trust the plan again" PR. Or defer until after T2 lands so the ADR-031 captures the eventual fix shape.

---

## T7 — Mechanical Antipattern Polish (~25 Findings)

**Findings owned:** `antipattern-fresh-2026-07-09.md` #3, #8, #10, #16, #18, #20, #23-28, #30 + `antipattern-check-simpler-hexagon-2026-07-09.md` #2, #4 (partial), #5 (partial), #7, #8, #10, #11, #12, #13, #16 + `code-simplification-pass2-simpler-hexagon-2026-07-09.md` #1, #2, #3, #7, #8.

**Practical Benefit: LOW-MED** — many are cosmetic; some have runtime impact (e.g. #9 from prior antipattern: SERVER TRACE debug spam in polled endpoint).

**Certainty: HIGH** — every finding has file:line evidence and a one-line fix.

### Scope (clustered)

1. **Debug-noise cleanup** (~1 SP)
   - Strip `SERVER TRACE:` from `endpoints.rs` (8+ calls in `generating_status_handler`)
   - Drop unused `State(_state)` in `status_ready_handler`
   - Fix `wait_until_idle` unreachable assert in `is_generating_invariant_tests.rs`

2. **Dead code removal** (~1 SP)
   - `_map` / `_player` / `_npcs` in `spawn_arrival_task_if_needed` (`init_game.rs:101`)
   - `_player_name` in `claim_generation_slot` (`application_service.rs:447,506`)
   - `_sender` ignored twice in `retry.rs:85,128`
   - `unreachable!()` arms in 4 sites (`history.rs:23,36`, `misc/swipe.rs:17,30`)
   - Dead `ProcessActionResult::ShuttingDown` arm at `application_service.rs:464`
   - Dead `tests/infrastructure/guardrails/error.rs::_e2` test fixture
   - `make_test_app_with_default_preset` (`tests/helpers/fixtures.rs:425-471`)
   - `let _ = world_snapshot;` (`tests/helpers/fixtures.rs:397`)

3. **Doc anchor fixes** (~0.5 SP)
   - Add `//! [DOC: ...]` to `src/adapters/driving/http/error_tests.rs:1`
   - Verify `src/adapters/driving/http/error.rs:1` ADR anchor is valid (already verified — file exists)

4. **Move `sanitize_for_prompt`** (~0.5 SP)
   - From `application/narrative_prompt/assembler.rs:424` to `application/input_sanitizer.rs`
   - Module doc stops lying about contents

5. **Reconcile `html_escape`** (~0.5 SP)
   - Delete the private copy in `error.rs:12`; import from `fragments/renderers/response.rs`

6. **Collapse single-field form DTOs** (~0.5 SP)
   - `EditHistoryForm { text }` → `Form<String>` directly
   - `ActionForm` stays (multi-field, semantic names)
   - `CreateGameForm` stays

7. **Move `ServerConfig`** (~0.5 SP)
   - From `app_state.rs` (where it sits next to AppState + ServerResources) to its own `config/server.rs`

8. **Reconcile `ApplicationError::is_user_displayable`** (~0.5 SP)
   - Currently matches `Engine(EngineError::WorldHasGames { .. })` — fragile post-A3a migration. Verify the variant still exists; document the contract.

9. **Inline `persist_initial_state_with_swipes`** (~0.5 SP)
   - 1 method, 2 callers (`create_game`, `reset`). Inline both.

10. **Drop `BackendKind::Test` invariant bypass** (~0.5 SP)
    - Antipattern #10: `BackendKind::Test` admits invariant breakage silently in release. Add `#[cfg(any(test, debug_assertions))]` gate.

### Out of scope

- Storage seam fix (T2)
- Test builder consolidation (T5)

### Blast Radius

- ~15 files; mostly 1-3 line changes
- ~200 lines removable

### Honest Tradeoff

T7 is the "small wins" bucket. Each item ≤0.5 SP individually. Bundle 3-5 per PR. Sub-plan should not propose T7 as one giant PR — split into T7.1 (debug noise), T7.2 (dead code), T7.3 (doc anchors), T7.4 (DTO cleanup).

**The total benefit is real but spread thin.** Several items here are pure cosmetic. If T2/T5 are already huge, T7 may not justify its own sprint — opportunistic bundle during T2/T5 refactor work.

---

## T8 — Workflow Gates (Prevention)

**Findings owned:** doubt verdict D3 + D4 (test-factory sprawl) + `code-review-simpler-hexagon-2026-07-09.md` "Plan validation gates silently failed" + the meta-finding that "40 SP branch → 40 SP review → 40 SP cleanup" repeats without prevention.

**Practical Benefit: MED** — addresses the recursion. Without this, T1-T7 just pay interest on the same debt next branch.

**Certainty: MED** — needs scope decision (G5 below). Hard to verify effectiveness in advance.

### Scope

1. **Architecture-update-before-implement gate** (AGENTS.md enforcement)
   - Add CI check: any PR touching `src/application/`, `src/domain/`, `src/adapters/driving/`, or `src/bootstrap/` must reference a doc path under `chronicler_engine/docs/` in PR description
   - Existing AGENTS.md rule is value, not enforcement; this adds the gate

2. **Plan-adherence audit** (sub-plan-time)
   - For plans with >5 tasks, sub-plan must include "Plan Adherence Audit" section listing each original task + its actual outcome
   - Already implicit in AGENTS.md; formalize via `docs/plans/_template.md`

3. **Antipattern healthcheck wiring**
   - The `antipattern-checker` skill exists but isn't gated on PR
   - Wire `antipattern-checker` as a soft gate: run on PR title prefix `[arch]` or `[refactor]`

4. **Validation gate honesty**
   - Every plan that says "validate: `grep X` returns 0" must include a CI-runnable check
   - Add `scripts/check_plan_gates.sh` that runs each plan's grep/wc/line-count gates

### Grilling Decision

- **G5: scope of gate enforcement.** Soft (PR template hint) vs hard (CI block). Recommend soft — hard gates on subjective checks (antipattern density) tend to be gamed.

### Out of scope

- Process discipline changes (retrospectives, planning ceremonies) — separate HR-process concern

### Blast Radius

- 1 CI workflow + 1 plan template + 1 PR template

### Honest Tradeoff

T8 is the meta-track. **Most likely to be deferred.** The benefit is real but the work is procedural and hard to attribute. Recommend: ship T8 after T1-T6 land, with a 1-month retrospective to measure effectiveness.

---

## T9 — WorldSnapshot Removal (Immutable World Data out of GameState)

**Findings owned:** `domain-reconciliation-2026-07-09.md` Snapshot row (moved out of T3 on 2026-07-10) + `simpler-hexagon-review.md` R6 (re-interpreted) + CONTEXT.md `Snapshot` entry honesty gap.

**Practical Benefit: HIGH** — removes a real architectural smell (intermediate load-bundle exists only because GameState ctor requires 4 immutable args), fixes CONTEXT.md lie (claims "immutable world data cached on AppState as Arcs" — actually loaded from storage every call), aligns with OpContext-kill principle (most call sites of a bundle should not be forced to take all bundled fields).

**Certainty: MED** — touches every engine call reading `state.world` / `state.map` / `state.npcs` / `state.player`; needs own ADR; depends on AppState cache shape (interacts with T2 arch-3 / Ticket 05 phantom-storage).

### Scope (decision A-Deep, locked 2026-07-10)

1. **Remove immutable world fields from `GameState`** (`domain/model/state/game_state.rs:20-23`)
   - Drop `pub world: Arc<WorldCard>`, `pub map: Arc<MapDef>`, `pub player: Arc<PlayerCard>`, `pub npcs: HashMap<String, NpcCard>`
   - `GameState::from_snapshot(&GameStateSnapshot)` takes 1 arg (currently 5)
   - All engine callsites reading `state.world.*` / `state.map.*` / `state.npcs.*` / `state.player.*` must be reworked to take world data from the AppState cache (threaded as parameter) or from a new `WorldContext` handle

2. **Add AppState-level Arc cache for world data**
   - CONTEXT.md `Snapshot` entry already claims this cache exists (`src/adapters/driving/http/app_state.rs:41` AppState does NOT have it). This step makes CONTEXT.md honest.
   - Cache keyed by `world_key` + `persona_key` (mirrors storage layer keys; matches ADR-026 persona relocation)
   - Cache populated at boot (seed-time) + invalidated on world/persona CRUD
   - Concrete shape TBD in T9 sub-plan grilling: `Arc<RwLock<HashMap<WorldKey, CachedWorld>>>` vs dedicated `WorldCache` struct in its own module

3. **Delete `WorldSnapshot` struct** (`src/application/persistence_gate/dto.rs:13`) AND `load_world_snapshot()` AND `world_snapshot_or_empty()` AND `WorldSnapshot::empty()`
   - 4 call sites collapse: `gate.rs:76-80,88,149` + `retry.rs:67`
   - File `src/application/persistence_gate/dto.rs` deleted entirely

4. **Fallback decision (D1b) — SEPARATE, NOT YET LOCKED**
   - Currently `world_snapshot_or_empty()` swallows `load_world_snapshot()` Err and returns `WorldSnapshot::empty()` (a Default-zeroed struct)
   - After T9 removes the struct, the fallback must be re-decided: keep defensive empty-on-Err recovery (current behavior, masks corrupted DB rows) or propagate Err to caller (surfaces corruption loudly)
   - This is a real defensive-code-vs-fail-loud tradeoff. Open question for T9 sub-plan grilling.

5. **New ADR** (probably ADR-033, since ADR-031/ADR-032 are claimed by T6/T2 Ticket 06)
   - Documents: GameState scope shrink (immutable vs mutable split); AppState world cache addition; CONTEXT.md honesty fix; fallback decision (D1b)

### Out of scope

- T3 glossary renames (PersonaCard, TurnResult, Action Pipeline phase, avoid-aliases) — stay in T3 (~4 SP mechanical)
- T2 arch-3 AppState phantom-storage (Ticket 05) — T9 INTERACTS with it (same AppState cache seam) but is scoped to world-data not token/storage phantom; sub-plan grilling must decide merge order

### Blast Radius

- `domain/model/state/game_state.rs` — struct + ctor + builder
- Every engine file reading `state.world` / `state.map` / `state.npcs` / `state.player` (grep needed for exact count — likely 30+ sites)
- `src/application/persistence_gate/dto.rs` — deleted
- `src/application/persistence_gate/gate.rs` — 3 sites refactored
- `src/application/action_pipeline/retry.rs` — 1 site refactored
- `src/adapters/driving/http/app_state.rs` — AppState gains world cache field(s)
- 1 new ADR
- 1 CONTEXT.md correction (Snapshot entry: cache is now real, not aspirational)

### Honest Tradeoff

T9 is the architecturally correct "delete WorldSnapshot" — applies the OpContext-kill principle to a struct that bundles 4 fields every consumer was forced to take even when only needing 1-2 (after the world-data access pattern is changed). NOT a T3-class mechanical rename. Risk: touches every engine read of world/map/npc/player fields; high blast radius. Must be its own sub-plan with its own grilling (D1b fallback, AppState cache shape, ADR number). Recommend: do NOT ship in Wave 2 with T3; ship in Wave 3 alongside or after T2 arch-3 (AppState phantom-storage Ticket 05) — same seam, should be coordinated.

---

## Finding State

| ID | Title | Owner Track | Status |
|----|-------|-------------|--------|
| blocker-1 | cancel_token drop in `make_test_app_with_storage_and_service` | T1 | pending |
| blocker-2 | `load_or_fresh` type lie | T1 | pending |
| blocker-3 | stale guardrail lists deleted `application/context.rs` | T1 | pending |
| blocker-4 | `arrival_service::run` silent return | T1 | pending |
| arch-1 | 49-method god-object + 2 phantom Arc<Storage> | T2 | **resolved** (tickets 02–04; 4 modules carved, god-object gone, `self.storage` grep = 0, `application_service.rs` 723→275 LOC) |
| arch-2 | ProcessActionResult + ApplicationError dual enums | T2 | **resolved** (ticket 04; both enums moved to `application/errors.rs`) |
| arch-3 | AppState + ServerResources parallel-field ghost seam | T2 (C4 free follow-up) | **ticketed** — [Ticket 05](../../../.scratch/t2-god-class-split/issues/05-appstate-token-phantom-storage.md) (grilling; covers phantom `Arc<Storage>` on AppState + dual cancel_token stale-on-replace) |
| arch-4 | process_action 47 lines (plan said ≤30) | T6 (gate amend) + T2 (fix) | **partial** (T2 ticket 03 moved body to `GenerationGate::start_action`, preserved verbatim at 47 LOC; T6 gate amend pending to update ≤30 → ≤50, or split into 2 functions) |
| arch-5 | AppState dual cancel_token stale-on-replace latent concern | T2 follow-up | **ticketed** — [Ticket 05](../../../.scratch/t2-god-class-split/issues/05-appstate-token-phantom-storage.md) (latent: after `replace_cancel_token`, `GenerationGate`'s field is the OLD cancelled clone — generation would be permanently shut down on same-service subsequent action; unexercised by current tests) |
| glossary-1 | PlayerCard → PersonaCard (~60 sites) | T3 | **resolved** (2026-07-10; renames + GameState::player→persona + PromptContext::player→persona + LayerRenderer::player→persona, ~100+ sites, all 1241 tests green) |
| glossary-2 | WorldSnapshot removal (immutable world data out of GameState) | T9 (moved from T3, 2026-07-10) | pending — A-Deep locked; D1b fallback open |
| glossary-3 | TurnResult → ActionResult (~6 sites) | T3 | **resolved** (2026-07-10; 4 sites in action_processing.rs + phases.rs, all tests green) |
| glossary-4 | Action Pipeline phase_trigger_evaluation merged | T3 (amend glossary) | **resolved** (2026-07-10; CONTEXT.md Action Pipeline entry corrected per D3, no code split) |
| glossary-5 | Avoid-alias leakage (10 sites) | T3 | **resolved** (2026-07-10; StoryLogTemplate→NarrativeLogTemplate + parse_command→parse_action; doc-comment sweep on 5 flagged files = 0 general-English uses needed rename; `story-log` UI surface deferred — scope creep, separate track) |
| polish-1 | SERVER TRACE debug noise in polled endpoint | T7.1 | pending |
| polish-2 | Dead `_map`/`_player`/`_npcs` params | T7.2 | pending |
| polish-3 | Dead `_player_name` param | T7.2 | pending |
| polish-4 | Dead `_sender` ignored 2× | T7.2 | pending |
| polish-5 | Dead `unreachable!()` arms (4 sites) | T7.2 | pending |
| polish-6 | Dead `ProcessActionResult::ShuttingDown` arm | T7.2 | pending |
| polish-7 | Dead `make_test_app_with_default_preset` | T7.2 + T5 | pending |
| polish-8 | Dead `let _ = world_snapshot;` | T7.2 + T5 | pending |
| polish-9 | Dead `_created_storage` tuple bool | T7.2 | pending |
| polish-10 | Missing `[DOC: ...]` anchor in error_tests.rs | T7.3 | pending |
| polish-11 | `sanitize_for_prompt` in wrong module | T7.4 | pending |
| polish-12 | `html_escape` defined twice | T7.5 | pending |
| polish-13 | Single-field form DTOs (`EditHistoryForm`) | T7.6 | pending |
| polish-14 | `ServerConfig` lives in `app_state.rs` | T7.7 | pending |
| polish-15 | `is_user_displayable` matches EngineError variant | T7.8 | pending |
| polish-16 | `persist_initial_state_with_swipes` inlinable | T7.9 | pending |
| polish-17 | `BackendKind::Test` bypasses invariants in release | T7.10 | pending |
| polish-18 | `error_return` name lies about return shape | T4 | pending |
| polish-19 | `InMemoryData` 13 fields, no shared behavior | T2 (per-entity stores) | pending (not touched by T2 wave — separate effort) |
| polish-20 | `arrival_service::new_for_test` test-only constructor | T5 | pending |
| polish-21 | `load_messages_with_swipes` defined twice | T2 (persistence consolidation) | **resolved** (ticket 02; free fn stays in `application_service.rs`, `PersistenceGate::load_messages_with_swipes` + `load_messages` both delegate to it — single definition, no duplication) |
| polish-22 | `run_from_input` 5 error styles | T4 | pending |
| polish-23 | `retry_last_response_impl` 5 early returns | T4 | pending |
| polish-24 | `seed_messages` loop duplicated 7× | T5 (`seed_narrative_into_storage`) | pending |
| polish-25 | `map_cancelled` ad-hoc pattern | T4 (subsumed by PhaseError) | pending |
| workflow-1 | Architecture update not gated on PR | T8 | pending |
| workflow-2 | Plan-adherence audit not formalized | T8 | pending |
| workflow-3 | Antipattern healthcheck not gated | T8 | pending |
| workflow-4 | Plan validation gates not CI-runnable | T8 | pending |
| plan-1 | §T2.2 / §T2.3 silently invalidated | T6 (mark VOID) | pending |
| plan-2 | §A6.4 validation gate failed silently | T6 (mark VOID) | pending |
| plan-3 | §B1.3 validation gate failed silently | T6 (mark VOID) | pending |
| plan-4 | No ADR-031 documenting OpContext absorption | T6 | pending |
| plan-5 | ADR-030 access pattern not documented | T6 | pending |
| plan-6 | No ADR documenting T2 modular split (ADR-032?) | T2 follow-up / T6 | **ticketed** — [Ticket 06](../../../.scratch/t2-god-class-split/issues/06-adr-032-t2-modular-split-record.md) (grilling; decide: new ADR-032 vs amend ADR-027 vs fold into ADR-031 vs arch-doc suffices) |

**Total: 45 findings across 9 tracks.**

---

## Recommended Sequencing

**Wave 1 (P0, ship before merge):** T1 (4 bugs) + T6 (plan honesty). ~9 SP. ~1 week. These are non-negotiable defects + the doc anchors to make the rest tractable.

**Wave 2 (P1, ship as cleanup branch):** T2 (god-class split, ~16 SP) + T3 (glossary drift, ~4 SP). ~20 SP. ~2-3 weeks. T2 is the load-bearing fix; T3 is mechanical and can land in parallel.

**Wave 3 (P2, ship as separate PRs from clean main):** T4 (PhaseError, ~5 SP) + T5 (test builder collapse, ~16 SP). ~21 SP. ~2-3 weeks. T5 benefits from T2 being settled; T4 is independent.

**Wave 4 (P3, opportunistic):** T7 (~10 SP across 4 sub-PRs) + T8 (~3 SP). Pick up during other refactors.

**Total elapsed:** ~6-8 weeks if single-threaded; ~3-4 weeks if T2/T5 land in parallel via subagents.

**T9 (WorldSnapshot removal, ~13 SP)** is P2 architecture work, not mechanical. Ship in Wave 3 alongside or after T2 arch-3 (AppState phantom-storage, Ticket 05) — same seam, must be coordinated. Has its own grilling (fallback behavior D1b, AppState cache shape, ADR number).

## Decision Required

Three tiering questions before splitting sub-plans:

1. **Should T2 land before merge or as immediate post-merge follow-up?**
   - Before merge: stricter; matches "reviewers found blockers" reading
   - Post-merge: faster; T2 is "structural improvement" not "defect"
   - Branch's 49-method god-object is a real defect per doubt verdict D2

2. **Should T5 land before T2 (test surface drives production shape) or after (production shape drives test surface)?**
   - Before: tests get fixed first; T2 carves production to match what tests assumed
   - After: T2 settles production; T5 collapses tests to match
   - The branch's tests already grew longer (retry_main +99); T5-first argues for "stop the bleed"

3. **Should T8 (workflow gates) ship before or after the cleanup itself?**
   - Before: prevents recurrence during the cleanup work
   - After: validates the gates against actual drift patterns observed during cleanup
   - Realistic middle: ship a soft version of T8.1 (architecture-update gate) before Wave 2; full T8 after Wave 3.

**User decision on these three will determine which sub-plans to spawn first.**

---

# Appendix A — Raw Findings Inventory

Source priority: `/tmp/antipattern-fresh-2026-07-09.md` (30 findings, 19 files) is the freshest; prior pass `/home/moridin84/projects/mrn-general/tmp/antipattern-check-simpler-hexagon-2026-07-09.md` (16 findings) overlaps; reviews `code-review-*` / `code-simplification-pass2-*` / `simpler-hexagon-review` provide additional grounding.

## A1. Antipattern Fresh Pass (30 Findings)

| # | Sev | Cat | Title | File:Line |
|---|-----|-----|-------|-----------|
| 1 | H | FD | `WorldSnapshot` and `GameState::new` model the same 4-tuple | `application_service.rs:35-49,280-298`; `retry.rs:66-79`; `query_handlers.rs:5-29` |
| 2 | H | R-b-D | `load_or_fresh` returns `Result` but never errors | `application_service.rs:260-279`; `query_handlers.rs:5-29`; `message_editing.rs:85,98,117,144` |
| 3 | H | CC | `is_generating_invariant_tests.rs` is a unit test that does integration testing | `application/is_generating_invariant_tests.rs` (entire file, 225 lines) |
| 4 | H | CC | `ApplicationError` and `ProcessActionResult` are two enums for one concept | `application_service.rs:99-130,130-134`; `message_editing.rs:117-149` |
| 5 | H | FD | Three "save state" code paths with overlapping but distinct semantics | `application_service.rs:306-310,311-345,470-477`; `retry.rs:99-103,107-115,170-175`; `query_handlers.rs:42-48` |
| 6 | H | CC | `app_state.rs` defines both `ServerResources` and `AppState` with parallel fields | `app_state.rs:28-78` |
| 7 | M | FD | `add_input_and_save` duplicated across two test layers | `retry_tests.rs:39-58`; `tests/helpers/pipeline_helpers.rs:124-141` |
| 8 | M | FD | `wait_for_generation_complete` vs `wait_until_idle` model the same poll | `pipeline_helpers.rs:81-94`; `is_generating_invariant_tests.rs:46-71` |
| 9 | H | R-b-D | `process_action` body is 47 lines, plan gate said ≤30 | `application_service.rs:443-498` |
| 10 | M | R-b-D | `BackendKind::Test` admits invariant breakage silently in release | `storage/backend/core.rs:140-165` |
| 11 | H | CC | `application_service.rs` houses 7 unrelated types alongside the service struct | `application_service.rs:35-678` (678 lines) |
| 12 | M | CC | `tests/helpers/fixtures.rs` has 4 near-identical "build a test state" functions | `fixtures.rs:114-205` |
| 13 | M | CC | `arrival_service.rs` and `bootstrap/init_game.rs` both touch arrival narration | `arrival_service.rs`; `bootstrap/init_game.rs:101-159` |
| 14 | H | R-b-D | `make_test_app_with_storage_and_service` silently mints fresh cancel token + is_generating | `test_support/context.rs:184-189,218-234` |
| 15 | L | FD | `_sender` ignored twice in `retry.rs` destructure | `retry.rs:85,128` |
| 16 | M | CC | `assembler.rs` houses `sanitize_for_prompt` outside the prompt-building concern | `narrative_prompt/assembler.rs:419-446` |
| 17 | H | CC | `DefaultApplicationService::new` takes 6 args, parallel to deleted `OpContext` | `application_service.rs:178-195` |
| 18 | M | CC | `fragments/renderers/fragment_renderers.rs` is 6 "query + render template" functions | `fragment_renderers.rs:1-119` |
| 19 | H | R-b-D | `app_state.rs` has `cancel_token: Arc<RwLock<CancellationToken>>` AND `DefaultApplicationService` has its own | `app_state.rs:53,65-76`; `application_service.rs:174,184` |
| 20 | M | FD | `ApplicationError::is_user_displayable` matches a private `EngineError` variant | `application_service.rs:104-110` |
| 21 | M | R-b-D | `app.save_state` used for "fix the status" in `retry.rs` but skips message persistence | `retry.rs:99-103,107-115,170-175` |
| 22 | M | R-b-D | `finalize_app` in test_support vs inline `DefaultApplicationService::new` in tests/helpers/fixtures.rs | `test_support/context.rs:218-234`; `tests/helpers/fixtures.rs:393-401,423-432` |
| 23 | M | CC | `fragment_renderers::render_error` and `error::error_div` both render an error div | `fragment_renderers.rs:21-25`; `error.rs:14-18` |
| 24 | L | FD | `html_escape` defined in two files with identical bodies | `response.rs:45-51`; `error.rs:9-16` |
| 25 | L | R-b-D | `_created_storage` tuple bool never read | `test_app_builder.rs:199-203` |
| 26 | L | CC | `ServerConfig` lives in `app_state.rs` next to `AppState` | `app_state.rs:17-26` |
| 27 | L | R-b-D | `status_ready_handler` takes `State(_state)` and ignores it | `fragments/endpoints.rs:58` |
| 28 | L | R-b-D | `let _ = world_snapshot; // suppress unused if save_snapshot path diverges` | `tests/helpers/fixtures.rs:382-393` |
| 29 | M | R-b-D | `make_test_app_with_default_preset` is dead code with `_world` and `_player` unused params | `tests/helpers/fixtures.rs:434-477` |
| 30 | M | CC | `assembler.rs` defines 5 layers of indirection over the same operation | `narrative_prompt/assembler.rs:18-47,92-104,107-128,163-201,251-280,421-446` |

**Severity legend:** H = high (defect or major smell), M = medium (real smell, mechanical fix), L = low (cosmetic). **Cat legend:** FD = False Deduplication, CC = Coincidental Cohesion, R-b-D = Refactor-be-damned.

## A2. Antipattern Prior Pass (16 Findings) — Overlap Map

| Pass-1 # | Title | Re-confirmed by Fresh Pass? |
|----------|-------|----------------------------|
| 1 | `Storage` overloading (2 phantom `Arc<Storage>`) | **Amplified** (fresh #6, #19) |
| 2 | `load_messages_with_swipes` defined twice | Confirmed (fresh #11) |
| 3 | `sanitize_for_prompt` in `narrative_prompt` | Confirmed (fresh #16) |
| 4 | `InMemoryData` 13 unrelated fields | Confirmed (no re-list) |
| 5 | `application_service.rs` god file | **Amplified** (fresh #11: 7 unrelated types) |
| 6 | `TestAppBuilder` 14 fields | Confirmed (no re-list) |
| 7 | `new_for_test` on `ArrivalTaskContext` | Updated (fresh #13: split with init_game is the smell) |
| 8 | `_sender` ignored in `retry.rs` | Confirmed (fresh #15: fix is `last_input_text` return shape) |
| 9 | `SERVER TRACE` debug noise | Confirmed (no re-list) |
| 10 | `html_escape` defined twice | Confirmed (fresh #24) |
| 11 | `State(_state)` unused | Confirmed (fresh #27) |
| 12 | Single-field form DTOs | Confirmed (no re-list) |
| 13 | `PlayerCardWithKey` / `CharacterSeed` DTOs | Confirmed (no re-list) |
| 14 | `drop(state)` in `build_test_app` | Updated (fresh #28: dead `WorldSnapshot` in fixtures is worse) |
| 15 | `seed_messages` loop duplicated 7× | Confirmed + amplified (fresh #7: also dup'd `add_input_and_save`) |
| 16 | `_created_storage` tuple bool | Confirmed (fresh #25) |

## A3. Code Simplification Pass 2 (9 Items)

| # | SP | Title |
|---|----|-------|
| 1 | 1 | Dead params in `spawn_arrival_task_if_needed` (`init_game.rs:101`) |
| 2 | 1 | Dead `ProcessActionResult::ShuttingDown` arm (`application_service.rs:464`) |
| 3 | 1 | `error_return` name lies about return shape (`phases.rs:63`) |
| 4 | 3 | Long `run_from_input` with 5 divergent error paths (`pipeline.rs:47-141`) |
| 5 | 3 | `retry_last_response_impl` 5 early returns (`retry.rs:14-93`) |
| 6 | 3 | `seed_messages` loop duplicated 3× (`test_support/context.rs:86-91,100-105,121-126`) |
| 7 | 1 | `WorldSnapshot::empty()` is `pub` but only used internally |
| 8 | 1 | `_player_name` dead param (`application_service.rs:447,506`) |
| 9 | 0 | Verify `src/adapters/driving/http/error.rs:1` doc anchor (verified — ADR-027 exists, moot) |

## A4. Code Review (Standards + Spec, 9 Standards + 9 Spec)

### Standards (HARD + Smell)

| ID | Sev | Item |
|----|-----|------|
| S1 | HARD | `claim_generation_slot` self-releases on save failure (plan §T2.4 violation) |
| S2 | HARD | `pub(crate)` fields widen ADR-030 "Read-Only Elsewhere" rule |
| S3 | SMELL | 723-line god class with 6-field + 49-method shape (mirror of deleted `OpContext`) |
| S4 | HARD | `GameState::from_snapshot` still takes 4-5 individual args (plan §T2.3 violation) |
| S5 | HARD | Missing `//! [DOC: ...]` anchor in `error_tests.rs` |
| S6 | HARD | `APPLICATION_STORAGE_GRANDFATHERED` includes deleted `application/context.rs` |
| S7 | SMELL | `arrival_service::run` silent return on `load_expecting_valid_state` Err |
| S8 | SMELL | 5 test factory fns with overlapping shapes (dead `drop(state)`, `let _ = world_snapshot`) |
| S9 | HARD | `make_test_app_with_default_preset(_world, _player, storage)` — wrong signature + dead code |

### Spec (Missing / Partial / Scope Creep / Wrong Implementation)

| ID | SP | Status | Item |
|----|----|----|------|
| T2.2 | 5 | MISSING | `impl FromRequestParts<AppState> for OpContext` never implemented |
| T2.3 | 4 | MISSING | `GameState::new` + `from_snapshot` still take 4-5 individual args |
| T3.2 | 2 | HELPER UNUSED | `make_test_app_with_default_preset` exists but no callsites |
| T2.4 | 3 | BODY LENGTH FAIL | `process_action` body ≤30 specified, actual 45 |
| T3.1 | — | SCOPE CREEP | Replaced `LayeredPromptAssembler::new(...).assemble(...)` with `build_narration_prompt` (not in spec) |
| T3.2 | — | SCOPE CREEP | Seeds both `system_default` AND `quantifier_default` (spec said system_default only) |
| T1.1 | — | SCOPE CREEP | "Amend ADR-030" — actually created new file (term inaccurate) |
| T2.4 | — | WRONG IMPL | Helper self-releases on Err (spec said caller MUST release) |

## A5. Detailed Code Quality Review (simpler-hexagon-review.md)

| ID | Title |
|----|-------|
| R1 | Extract `game_management` from `application_service.rs` (12 methods, ~180 lines) |
| R2 | Make `load_or_fresh` infallible (delete `Result` wrapper + dead `?` + `load_state_lossy`) |
| R3 | Collapse 11 test-app factories into one `TestApp` (promote `app_with_storage_from` to test_support) |
| R4 | Inline `persist_initial_state_with_swipes` (2 callers, 1 method) |
| R5 | Drop `pub(crate)` accessors OR tighten via facade (the 5 accessors + 6 fields = same shape as deleted OpContext) |
| R6 | Delete `WorldSnapshot` entirely (only retry.rs user, fallback chain) |
| R7 | `tests/integration/flow/retry_main.rs:55,109,180...` build-then-discard-then-rebuild pattern |
| R8 | `process_action` has inline shutdown-check duplicating `claim_generation_slot`'s release path |
| R9 | `process_action` body 47 lines (not ≤30 promised by B1.3) |
| R10 | `query_handlers.rs:13-29` `load_state_lossy` `Err` arm unreachable |
| R11 | `init_game.rs:101-148` `spawn_arrival_task_if_needed` 5 unused args |
| R12 | `is_generating_invariant_tests.rs:46-71` `wait_until_idle` unreachable assert after `panic!` |
| R13 | `app_state.rs:31-46` `Storage` overloading |
| R14 | `DefaultApplicationService` 5 singletons + 5 accessors = `OpContext` re-inlined |
| R15 | `make_test_app_with_storage_and_service` silently resets cancel_token/is_generating/settings |
| R16 | `fixtures.rs:382-393` constructs `WorldSnapshot` then drops it |
| R17 | `fixtures.rs:427` `make_test_app_with_default_preset` `#[allow(dead_code)]` |
| R18 | `src/test_support/context.rs` misnamed (246 LOC factory sprawl) |
| R19 | `ApplicationError::is_user_displayable` fragile match against `Engine(EngineError::WorldHasGames { .. })` |
| R20 | `tests/infrastructure/guardrails/layers.rs:5-9` stale `application/context.rs` |
| R21 | Plan §B1.3 validation gate failed silently (process_action 47 lines, gate said ≤30) |
| R22 | Plan §A6.4 validation gate failed silently (`grep WorldSnapshot` returns 7+ hits) |
| R23 | Plan §A6.1 implicit promise about consolidating builders unmet (11 builders up from 4) |

## A6. File-Size Decomposition

| File | main | branch | Δ | Concern |
|------|------|--------|---|---------|
| `src/application/application_service.rs` | 394 | **723** | **+329** | Pushed near 750, god-object risk |
| `src/application/action_pipeline/pipeline.rs` | 265 | 267 | +2 | OK |
| `src/application/action_pipeline/retry.rs` | 358 | 177 | **-181** | **Win** |
| `src/application/arrival_service.rs` | 160 | 139 | -21 | **Win** |
| `src/test_support/context.rs` | 93 | 246 | +153 | New 8-method factory sprawl, misnamed |
| `src/test_support/test_app_builder.rs` | 328 | 336 | +8 | OK (but 14 fields) |
| `tests/helpers/fixtures.rs` | 379 | 479 | +100 | 2 new builders, 1 dead |
| `tests/integration/flow/retry_main.rs` | 599 | 698 | +99 | Tests got LONGER |
| `tests/integration/application/action_pipeline/retry.rs` | 571 | 374 | **-197** | **Win** |
| `tests/integration/application/game_service.rs` | 547 | 502 | -45 | **Win** |
| `tests/integration/application/lifecycle.rs` | 373 | 450 | +77 | Bigger because `make_test_app_with_storage` rebuilds per-test |

---

# Appendix B — Glossary Drift (5 Drift Terms + 9 Gaps)

Source: `/tmp/domain-reconciliation-2026-07-09.md`.

## B1. Drift Fixes (T3)

| Term | Expected (CONTEXT.md) | Actual | File:Line | Fix |
|------|----------------------|--------|-----------|-----|
| **Persona** | Persona value type, world-independent, keyed by persona_key | `PlayerCard`, `PlayerCardWithKey`, `TestPlayer`, `PromptLayer::Player`, `PromptContext::player` (~60 sites) | `domain/model/character.rs:51`; `adapters/driven/storage/backend/core.rs:51`; `test_support/fixtures.rs:42`; `application/narrative_prompt/types.rs:15,32` | Rename value type to `PersonaCard`; bindings already fixed by ADR-026 |
| **Snapshot** | `GameStateSnapshot` only; immutable world data is NOT a snapshot | `WorldSnapshot { world, map, player, npcs }` — a load bundle | `application_service.rs:35,223,253`; `retry.rs`; `query_handlers.rs` | Rename to `WorldContext` (or `WorldBundle`/`WorldLoad`) |
| **Turn** | "Don't use for current architecture. Use Message + Swipe" | `TurnResult` | `domain/engine/action_processing.rs:27`; consumed in `pipeline.rs:90-100` | Rename to `ActionResult` (or `EngineCommitResult`) |
| **Action Pipeline** | 8 distinct phases incl. "trigger evaluation" | Trigger evaluation folded into `phase_engine_commit` via `execute_freeaction_impl` → `evaluate_triggers` | `domain/engine/action_processing.rs:159-170`; `application/action_pipeline/phases.rs:380` | Split `phase_trigger_evaluation` out OR amend glossary |
| **Avoid aliases** | Avoid: `session`, `command`, `text`, `output`, `story`, `setting`, `player`, `npc`, `campaign`, `event`, `scorer`, `bot`, `line`, `variant`, `save` | `StoryLogTemplate`, `parse_command`, doc comments using `session`/`command`/`text`/`output` | `templates.rs:28`; `engine/parser.rs:6`; `game.rs:2`; `logic.rs:37`; `narrative_state.rs:25`; `action.rs:2`; `quantifier.rs:11,109` | `StoryLogTemplate → NarrativeLogTemplate`; `parse_command → parse_action`; doc-comment sweep |

## B2. Glossary Gaps (Terms in Code NOT in CONTEXT.md)

| Code Symbol | Location | Should Add? |
|-------------|----------|-------------|
| `NarrativeSnapshot` | `domain/model/state/game_state_snapshot.rs:22` | Yes — sub-component of Snapshot |
| `NarrativeState` | `domain/model/state/narrative_state.rs:14` | Yes — mutable sub-state (history + input_buffer) |
| `SceneState` | `domain/model/state/scene_state.rs:7` | Yes — holds `npcs_in_area` + `quantifier_confidence` |
| `MovementState` | (referenced `state/movement.rs`) | Yes — mutable sub-state (current_room + dynamic_rooms) |
| `InputBuffer` | `domain/model/state/generation_status.rs:54` | Yes — player input + status + phase, persisted |
| `GenerationStatus` / `GenerationPhase` | `domain/model/state/generation_status.rs` | Yes — generation gate state |
| `StartingScenario` | `domain/model/scenario.rs:14` | Yes — implements "Scenario" but symbol is prefix-extension |
| `PromptLayer` | `application/narrative_prompt/types.rs:14` | No — UI implementation detail |
| `MessageType` | `domain/model/state/message_types.rs:8` | Yes — Narration/Dialogue/System/Input discriminator |
| `MessageEntry` | `domain/model/state/message_types.rs:16` | No — internal view/storage shape |
| `MessageHistory` | `domain/model/message_history.rs:15` | Borderline — thin wrapper |
| `NarrativePromptBuilder` / `assemble_prompt_text` | `application/narrative_prompt/` | No — implementation details |

## B3. Behavior Drift (Not Naming)

- **`switch_swipe` doesn't restore snapshot** (`application/message_editing.rs:35-77`). Glossary says "switching swipes restores the corresponding snapshot"; code only updates `active_swipe_index`. Snapshot restoration capability is unwired.
- **`arrival_service::run` silent return** (`arrival_service.rs:52-58`). On `load_expecting_valid_state` Err, function returns silently. Previously fell through to fresh state + scenario injection. Behavior change, not flagged in plan §T3.1.

---

# Appendix C — Cross-Cutting Observations

Source: `/tmp/depth-analysis-2026-07-09.md` § Cross-cutting + `/tmp/antipattern-fresh-2026-07-09.md` § Cross-cutting notes.

## C1. Recurring Patterns (Future-Work Prevention)

1. **`Storage` type-system hole recurs 5× across the codebase.**
   - `application_service.rs:172-173` (2 fields)
   - `app_state.rs:31-46` (2 structs × 2 fields = 4 instances)
   - `test_app_builder.rs:228-235` (3 fields)
   - `test_support/context.rs` (3 references)
   - **One fix (PresetStore newtype in T2) propagates everywhere.**

2. **`Arc<DefaultApplicationService>` is the most-replicated test signature** — 13+ inline constructions of `DefaultApplicationService::new(storage, preset_storage, settings, token, is_generating, game_service)`. The 6-arg constructor is the most-replicated signature in the codebase. T2 cuts to 1 façade + 4 module-Arcs.

3. **Persistence save-loop duplicated 5+ times across 4 modules.** Same 6-line "save snapshot, insert message, insert swipes" sequence in `application_service`, `arrival_service`, `retry`, `phases`. Every persistence fix touches 4 files.

4. **Internal seams leaked as public surface.** `find_retry_anchor` (`application_service.rs:412`), `load_world_snapshot` (`application_service.rs:227`), `world_snapshot_or_empty` (`application_service.rs:253`), `persist_initial_state_with_swipes` (`application_service.rs:697`) are all `pub`/`pub(crate)` with single caller each.

5. **Two-tier `cancel_token` (AppState vs DefaultApplicationService).** `app_state.rs:53` has `Arc<RwLock<CancellationToken>>` (mutable, lock-protected); `application_service.rs:174` has plain `CancellationToken` (immutable, owned). They are NOT the same token — `bootstrap/run.rs:156` builds the service with a fresh token, then `start_server` constructs `AppState` with a different fresh token. `AppState::replace_cancel_token` mutates AppState's RwLock, leaving service's token unaffected. Two sources of truth for cancellation.

6. **`LlmCallRecorder` is a seam that doesn't exist as a port.** Every LLM caller takes concrete `Arc<LlmCallRecorder>` (`phases.rs:65`, `arrival_service.rs:24`, `retry.rs:115`, `pipeline.rs:18`). The `LlmProvider` port trait *does* exist in `application/ports/llm_provider.rs` but is unused at call sites — phantom-port pattern ADR-027 warns against.

7. **Seam vs adapter asymmetry.** `DefaultApplicationService` has 2 `Arc<Storage>` (phantom seam — same type, different role), 1 `Arc<GameService>`, 1 `Arc<RwLock<AppSettings>>`, 1 `Arc<AtomicBool>`, 1 `CancellationToken`. None have a port trait. `LlmCallRecorder` + `LlmProvider` is the only place the codebase follows ports-and-adapters correctly.

8. **The OpContext kill did not reduce surface area; it relocated it.** Pass-1 #1 (Storage overloading) is unchanged. Fresh pass findings 6, 11, 17, 19 all show the same 5-6 Arc-field + 5-accessor shape re-instantiated under different names. Net result: same shape, different name.

9. **The `load_or_fresh` / `load_expecting_valid_state` / `load_world_snapshot` triplet is the highest-cost duplication.** Findings 1, 2, 5 all derive from the same root: the boundary between "load state with fallback" and "load state, fail on error" is encoded in 3 different functions with overlapping semantics. Untangling this would unlock findings 9 and 21 as natural follow-ons.

## C2. Test Factory Zoo (12 Builders Identified)

1. `make_test_app` (`src/test_support/context.rs`)
2. `make_test_app_without_snapshot` (`src/test_support/context.rs`)
3. `make_test_app_with_sqlite` (`src/test_support/context.rs`)
4. `make_test_app_with_mock_backend` (`src/test_support/context.rs`)
5. `make_test_app_with_backends` (`src/test_support/context.rs`)
6. `make_test_app_with_separate_backends` (`src/test_support/context.rs`)
7. `make_test_app_with_game_service` (`src/test_support/context.rs`)
8. `make_test_app_with_storage_and_service` (`src/test_support/context.rs`) — **BROKEN: drops token**
9. `TestAppBuilder::default_test` (`src/test_support/test_app_builder.rs`) — 14 fields
10. `app_with_storage_from` (`tests/helpers/fixtures.rs:401-418`) — **correctly preserves token**; lives in wrong layer
11. `make_test_app_with_storage` (`tests/helpers/fixtures.rs:383-397`) — constructs dead `WorldSnapshot`
12. `make_test_app_with_default_preset` (`tests/helpers/fixtures.rs:425-471`) — `#[allow(dead_code)]`, 0 callsites

Plus 4 helper state-builders (`create_test_state_with_npcs`, `create_test_state`, `create_basic_test_state`, `create_basic_test_state_no_scenario`, `create_test_game_state`) at `tests/helpers/fixtures.rs:114-205`.

---

# Appendix D — Doubt-Decision Verdicts (Full Text)

Source: doubt-driven-development skill (degraded self-questioning mode in plan-mode constraints). Each decision: CLAIM → ARTIFACT+CONTRACT → DOUBT → RECONCILE → verdict.

## D1. "Deleting OpContext improved the architecture."

- **CLAIM:** The OpContext-kill (commit 20cacf9) simplified the architecture.
- **ARTIFACT:** `application/context.rs` (284 lines) deleted. Dependencies formerly wrapped by OpContext (storage, settings, cancel_token, is_generating, game_service) now accessed directly on `DefaultApplicationService` via 5 accessor methods + `pub(crate)` fields.
- **CONTRACT:** Architecture improves when complexity concentrates, tests shrink, seams narrow.
- **DOUBT findings:**
  - Deletion **moved** complexity, didn't **delete** it. 50+ test fixture callsites now build `make_test_app` factories; 4 free helpers (`map_llm_error`, `load_messages_with_swipes`, `WorldSnapshot`, `DebugStateView`) relocated into `application_service.rs`.
  - Seam **widened**: 49-method `DefaultApplicationService` interface vs 1-context-struct OpContext seam.
  - `application_service.rs` +329 lines. Test app builders +8 methods. Tests got **longer** (`retry_main.rs` +99 lines), not shorter.
  - **No ADR** documents this decision. Plan §T2.2 contingent on OpContext continuation was silently invalidated.
- **RECONCILE:** The deletion preserved the hexagonal seam (verified — 0 OpContext refs in `src/`). Architectural improvement is real but overstated.
- **Verdict: FAIL** — improvement real but trade-off (D2) cancelled most of the leverage.

## D2. "Absorbing OpContext into god class was acceptable trade-off."

- **CLAIM:** 49-method `DefaultApplicationService` is acceptable post-OpContext-kill.
- **ARTIFACT:** `application_service.rs` (723 lines) hosts: 6 `pub(crate)` fields, 5 accessors, `ApplicationError` (enum), `ProcessActionResult` (enum), `WorldSnapshot` (struct), `DebugStateView` (struct), `map_llm_error` (free fn), `load_messages_with_swipes` (free fn), 49 methods.
- **CONTRACT:** "Acceptable" means each concern has a clear home; module shape earns its keep.
- **DOUBT findings:**
  - 4+ distinct concerns mixed (game lifecycle, world/persona CRUD, pipeline helpers, snapshot persistence, orchestration, error types, DTOs).
  - Deletion test fails: removing any concern would either shrink the file or force scattered callsite updates. **Neither is happening** — file is on a growth trajectory.
  - Alternatives demonstrably better: extract `game_management` (12 methods, ~180 lines) is mechanical refactor.
  - ApplicationError ~50 lines of impls sits in `application_service.rs` despite being trivially ownable at `application/errors.rs`.
  - Not acknowledged in any ADR (D3 problem amplifies).
- **RECONCILE:** Module-shape evidence contradicts the claim.
- **Verdict: FAIL** — T2 (god-class split) is the corrective action.

## D3. "Silently abandoning plan §T2.2/§T2.3 was acceptable."

- **CLAIM:** Plan tasks that didn't land can be skipped without ceremony.
- **ARTIFACT:**
  - §T2.2 (`impl FromRequestParts<AppState> for OpContext` extractor in `op_context_loader.rs`) never implemented. Handlers use `State<AppState>` + `state.application_service.X()` directly.
  - §T2.3 (`GameState::new` + `from_snapshot` WorldSnapshot variant) never landed. Constructors still take `(world, map, player, npcs)` at `domain/model/state/game_state.rs:104-110`.
- **CONTRACT:** Plan adherence is required per AGENTS.md "Plan Adherence" rule. Silent deviations forbidden.
- **DOUBT findings:**
  - AGENTS.md mandates: "STOP. Report to user with options A/B. Wait for direction before proceeding."
  - No report was filed. Plan file unchanged.
  - The OpContext-kill (commit 20cacf9) ran *before* plan §T2.2 implementation but the plan was not redrafted. **Two commits invalidating tasks; no follow-up.**
  - §T2.3 work would have collapsed 4-5 args → 1 struct, eliminating the antipattern-fresh #1 (`WorldSnapshot` as passthrough). The architectural claim of the branch was never delivered.
- **RECONCILE:** Process violation regardless of intent.
- **Verdict: FAIL** — T6 (mark §T2.2/§T2.3 VOID or resurrect as part of T2/T3).

## D4. "Test-factory sprawl (11 builders) is unavoidable."

- **CLAIM:** 11 test factories are needed because of orthogonal axes.
- **ARTIFACT:** `test_support/context.rs` (246 LOC, 12 fns), `test_support/test_app_builder.rs` (336 LOC, 14 fields), `tests/helpers/fixtures.rs` (479 LOC, 2 new builders + 1 dead). 13+ inline `DefaultApplicationService::new` constructions.
- **CONTRACT:** Sprawl is unavoidable when the orthogonal axes don't share a generator.
- **DOUBT findings:**
  - 3-4 orthogonal axes (backend, GameService variant, seed present/absent, storage sharing) × ~12 crosses maps imperfectly to 11 builders (no factory covers sqlite + without_snapshot cross).
  - Builder alternative `TestApp::new(state).with_sqlite().with_separate_backends().build()` covers all crosses at ~50 lines of code with ~150 line reduction in call patterns.
  - Tests got **longer** (`retry_main.rs` +99, `lifecycle.rs` +77), not shorter. Sprawl degrades test readability.
  - The deeper problem: factories encode production-shape complexity (`InMemoryData`, `TestAppBuilder`) instead of composing smaller fixtures. Refactoring production types requires parallel test refactoring.
- **RECONCILE:** The builder-collapse alternative is strictly better.
- **Verdict: FAIL** — T5 (TestApp builder family) is the corrective action.

---

# Appendix E — Methodology Notes (Process Document)

## E1. Plan-Mode Constraints (Affected This Investigation)

During this investigation, the primary session was in **plan mode**. Constraints:
- Bash restricted to read-only/non-mutating commands (no `>`, `>>`, `tee`, heredoc-to-file)
- No `write` / `edit` file tool exposed to the primary
- Primary cannot write to `/tmp/` during plan mode
- **Subagents write files; primary does chat-only synthesis**

This means:
- All `/tmp/*.md` artifacts were written by subagents
- The HTML report at `/tmp/architecture-review-*.html` was written by the synthesis subagent
- The doubt-decision verdicts (D1-D4) were produced inline via "degraded self-questioning" mode per doubt-driven-development skill (acceptable fallback when secondary personas cannot be invoked from plan-mode)
- The super-plan itself could not be written until the user exited plan mode

## E2. Source Files (Where Each Finding Came From)

| Source | Findings | Path |
|--------|----------|------|
| Antipattern fresh pass | 30 (8H/14M/8L) | `/tmp/antipattern-fresh-2026-07-09.md` |
| Antipattern prior pass | 16 (3H/6M/7L) | `/home/moridin84/projects/mrn-general/tmp/antipattern-check-simpler-hexagon-2026-07-09.md` |
| Code review (Standards+Spec) | 9 + 9 | `/home/moridin84/projects/mrn-general/tmp/code-review-simpler-hexagon-2026-07-09.md` |
| Code simplification pass 2 | 9 (5H+4M+urgent) | `/home/moridin84/projects/mrn-general/tmp/code-simplification-pass2-simpler-hexagon-2026-07-09.md` |
| Code quality review (thermo-nuclear) | 23 + 4 file-size | `/home/moridin84/projects/mrn-general/tmp/simpler-hexagon-review.md` |
| Depth analysis (codebase-design) | 7 modules + cross-cutting | `/tmp/depth-analysis-2026-07-09.md` |
| Domain reconciliation | 14 terms + glossary gaps | `/tmp/domain-reconciliation-2026-07-09.md` |
| Doubt decisions | 4 verdicts | Inline in conversation (this session) |
| Architecture review HTML | 4 candidates + drift + verdicts | `/tmp/architecture-review-1783626107.html` |

## E3. Investigation Method

1. Read all prior reviews + the original umbrella plan
2. Read SKILL.md for 7 user-named skills (antipattern-checker, codebase-design, domain-modeling, doubt-driven-development, improve-codebase-architecture, improve-ai-plan, wayfinder, thermo-nuclear-code-quality-review)
3. Read `chronicler_engine/CONTEXT.md` (glossary) + `CONTEXT-MAP.md` (workspace context)
4. Read `chronicler_engine/docs/architecture/system.md` (architecture description)
5. Launched parallel subagents for depth analysis (M1-M7) + domain reconciliation (14 terms)
6. Antipattern subagent stalled at synthesis step (76 lines, no write); manual fallback used prior 16-finding scan + fresh 30-finding scan
7. Doubt loop executed inline (4 decisions, all FAIL)
8. Synthesis subagent consolidated 4 inputs into HTML report (47KB, 684 lines)
9. User reviewed HTML, pivoted to holistic investigation → super-plan written

## E4. Methodological Gaps Acknowledged

- **Antipattern fresh pass stalled** — subagent read 76 lines worth of context but did not synthesize. Result: synthesis subagent used the prior 16-finding scan. The 30-finding fresh pass was delivered AFTER synthesis completed; not integrated into the HTML. Could be re-integrated via a follow-up synthesis pass (~10-15 min).
- **No fresh build / cargo test / python build.py runs** — primary was in plan mode; subagents read-only by instruction. All "this is a defect" claims are evidence-based from file:line reads, not runtime confirmation. Sub-plans should re-verify before merge.
- **Vocabulary discipline** — synthesis subagent self-corrected 2 vocab slips (hexagonal boundary → seam; HTTP layer → HTTP side). codebase-design vocabulary (module, interface, implementation, depth, seam, adapter, leverage, locality) and CONTEXT.md glossary (Game, World, Persona, Character, Scenario, Action, Action Pipeline, Trigger, Narrative, Quantifier, Agent, Message, Swipe, Snapshot) used throughout.

## E5. What's NOT Covered (For Future Work)

- **New glossary terms gap analysis** (Appendix B2) — 9 terms in code but not in CONTEXT.md. Sub-plan for adding them to glossary is out of scope.
- **Existing ADRs not re-litigated** — ADR-026 (Persona relocation) is partly superseded by fresh §B1 drift; ADR-027 (hexagonal architecture) is preserved; ADR-030 (is_generating invariant) needs amendment per T6.
- **Performance / Core Web Vitals** — not investigated. chronicler_engine is a Rust backend; "performance" here means latency under load, not frontend metrics. Out of scope for this cleanup.
- **Security audit** — not in scope. No FFI / unsafe code review done.
- **Cross-context implications** (Docker, Scripts) — the super-plan is engine-scoped only. Containerization changes (if any) belong in a separate workspace-level plan.

---

# Appendix F — References

## F1. Plans in `chronicler_engine/docs/plans/`

| Plan | Status | Notes |
|------|--------|-------|
| `opcontext-kill-plan.md` | **Stale** | §T2.2 and §T2.3 silently invalidated; §A6.4 and §B1.3 validation gates failed silently. T6 marks them VOID. |
| `abstraction-fixes-followup-superplan.md` | Active | Reference template for super-plan structure. |
| `reliability-and-cancellation-plan.md` | Active | Adjacent plan; covers R2 (token registration gap). Coordinate with T1 blocker-1. |
| `abstraction-antipattern-healthcheck-plan.md` | Active | Prevention plan. Coordinate with T8. |
| `tier-1-2-fix-plan.md` / `tier-1-tier-2-fix-plan-from-code-quality-review-simpler-hexa.md` | Active | Older tier-1/2 fix plans; should be reconciled with this super-plan. |
| `t1-error-model-unification.md` | Active | Coordinate with T2 ApplicationError extraction. |
| `t2-arch-narration-deepening.md` | Active | Coordinate with T3 M3 ArrivalTaskContext deepening. |
| `t5-type-collapses.md` | Active | Coordinate with T3 `TurnResult → ActionResult`. |
| `t6-messagehistory-encapsulation.md` | Active | Orthogonal. |
| `t10-low-priority-cleanup-bundle.md` | Active | Adjacent cosmetic bundle. Coordinate with T7. |
| `t11-documentation-hygiene-skill-hardening.md` | Active | Doc hygiene. Coordinate with T7.3 + T8. |
| `steering-and-guided-generation.md` | Active | Orthogonal. |
| `mapless-worlds-plan.md` | Active | Orthogonal. |
| `fix-run-branches-rs-port-allocation-race.md` | Active | Orthogonal. |
| `title-delete-llmmessagerepository-port-closure-substitute-re.md` | Active | Coordinate with T2 ports cleanup. |
| `subplan-b-quantifier-field-split.md` | Active | Orthogonal. |
| `subplan-c-mapless-enablement.md` | Active | Orthogonal. |

## F2. ADRs in `chronicler_engine/docs/adr/`

- ADR-026 (Persona relocation to Game) — binding side done; value type side is T3.
- ADR-027 (hexagonal architecture migration) — boundary preserved; phantom-port warning applies to T2.
- ADR-030 (is_generating invariant) — needs amendment per T6.
- ADR-031 — TO BE WRITTEN per T6 (OpContext absorption trade-offs).

## F3. `chronicler_engine/CONTEXT.md` Terms

Canonical glossary: Game, World, Persona, Character, Scenario, Action, Action Pipeline, Trigger, Narrative, Quantifier, Agent, Message, Swipe, Snapshot.

Avoid aliases: Session, Setting, Player, NPC, Campaign, Command, Event, Story, Scorer, Bot, Line, Variant, Save.

Deprecated: Turn (replaced by Message + Swipe; ADR-013).

## F4. Plan-Vocabulary Discipline

Use exactly: **module, interface, implementation, depth, deep, shallow, seam, adapter, leverage, locality.**

Never substitute: component / service / unit (for module); API / signature (for interface); boundary (for seam); layer / wrapper (for module, when meaning module).

Wins phrasing (in codebase-design vocabulary): "locality: bugs concentrate in one module", "leverage: one interface, N call sites", "interface shrinks; implementation absorbs the wrappers". NOT "easier to maintain" / "cleaner code".

---

# End of super-plan

Total: 8 tracks, ~64 SP, 45 findings, 3 sequencing decisions required before sub-plan split.