# Hexagonal Architecture Gap Closure — Chronicler Engine

**Status:** Complete.
**Supersedes:** `docs/plans/abstraction-fixes-followup-superplan.md` (per user direction 2026-07-04). Useful items from that plan's T2-ARCH track (G2-G5 grilling decisions, Narration deepening constraints 1-7) absorbed into Phase F below. Other T-tracks from the former super-plan (T1, T3, T4, T5, T6, T9, T10) are out of scope here — they concern abstraction/cleanup debt unrelated to hexagonal invariant enforcement.

**Scope:** `chronicler_engine/`. Total ~36 SP across 5 phases.

## Summary

Close verified hexagonal-architecture gaps in Chronicler Engine. Scope locked by user decisions:
- All 12 gaps in scope, EXCEPT #2/#3 (Repository ports) — **dropped**. `Storage` stays concrete per ADR-027; user decided the multi-impl argument is insufficient and `Storage::new_in_memory()` + `LayeredBackend` Test variant already provide test substitution.
- #11 (ArrivalTaskContext extraction) **absorbs** T2-ARCH grilling decisions G2-G5 + constraints 1-7 from the former super-plan (now superseded).
- #9 (debug.rs) **accepted** as documented exemption — no code change beyond doc.
- #12 (AppState holds TextCheckService) **accepted** as Deviation T1 — no code change.

**Gates (revised 2026-07-04):**
- Original plan gated Phase C (#1, #6) on T2 reliability plan landing first. **Gate removed.** Reason: T2/R2 plan dated 2026-06-28 is itself still in "Planning / Sub-plans pending" status after 2 weeks with no movement. Re-evaluation showed Phase C touches `LlmCallRecorder` constructor wiring (`game_service.rs:45`, `agent.rs:56`, `init_game.rs:307`, `application_service.rs:308`, `server_impl.rs:22`) while R2's N21 touches `CancellationToken` registration at `init_game.rs:299`. Different fields, different concerns. No real merge conflict.
- **Phase F (#11) gate removed (2026-07-04):** Originally gated on R2/N21 (cancel-token registration on same struct). User removed gate: T2 plan 2+ weeks stale, no movement. F.1 lands preserving current `cancel_token` behavior in `GameServiceContext`; R2 adds checks + token registration later.
- #11 also coordinates with R2 cancellation work in `docs/plans/reliability-and-cancellation-plan.md` (G4 cancellation shape decision).

Verified findings drive scope:
- #8 reframed: field on `Swipe` (not `Message`), type `u64` (not `i64`). **E.2 DROPPED as YAGNI 2026-07-04** — `Swipe::snapshot_id` stays on domain entity per ADR-027 Deviation 3. `Message::id` is also DB-assigned; full DTO split would force duplication across message aggregate.
- #6 prod wiring in `server_impl.rs:28-35` (not `init_game.rs` as review implied).
- #9 actually accesses `state.game_service.backend_info()`, not `ApplicationService`.
- #7 `LlmProvider` port already exists at `application/ports/llm_provider.rs`. `Connection` is constructor input only, not runtime arg. E.1 downgraded to placement fix (3 SP).

## Implementation order

Phase A → B → C.2 → C.1 → E.1 → (E.2 DROPPED) → F (gated). See each phase below.

Rationale for C before E: C is the substantive hex-invariant improvement (composition root cleanup, review's #1 finding). E is small placement fixes. Work the bigger lever first.

## Key Changes

### Phase A — Quick Wins (~2 SP) ✅ COMPLETE 2026-07-04

**A.1 — Gap #10: Fix CheckResult import path (1 SP)** ✅
- `src/adapters/driving/http/templates.rs:8` → `use crate::application::ports::text_checker::CheckResult;`
- `src/adapters/driving/http/view_models.rs:9` → same
- Sweep confirmed: no other mis-targeted adapter imports of port types
- Verify: `grep -rn "adapters::driven::text_check::CheckResult" src/` returns 0 hits ✅

**A.2 — Gap #9: Formalize debug.rs exemption (1 SP, accept path)** ✅
- ADR-027 §3.2 reframed: actual pattern is `state.game_service.backend_info()`, not `ApplicationService` as review claimed
- `// arch-lint: debug-direct` comment added at `src/adapters/driving/http/debug.rs:35`
- Precedent rule documented: new diagnostic endpoints must add explicit ADR exemption
- Verify: ADR-027 §3.2 reflects actual access pattern ✅

### Phase B — Lint Enforcement (~8 SP) ✅ COMPLETE 2026-07-04

**B.1 — Gap #4: Split arch-lint narrative scope (5 SP)** ✅
- `arch-lint.toml` `narrative` scope split into 4 distinct scopes:
  - `ports` → `src/application/ports/**`
  - `driven-llm` → `src/adapters/driven/llm/**`
  - `driven-text-check` → `src/adapters/driven/text_check/**`
  - `narrative` (residual) → `src/application/agents/**`, `src/application/narrative_prompt/**`
- New deny rules: `ports → {driven-llm, driven-text-check, narrative, server, storage, storage-models, bootstrap, test-support, engine}`; bidirectional `driven-llm ↔ driven-text-check`
- Subset trap documented: `ports` is subset of `application/**`; do NOT add `application` to deny list or ports-internal imports (e.g. `LlmMessage` from `llm_message_repository` used by `llm_provider`) false-trigger as self-deps
- Note: arch-lint 0.4.3 lacks scoped-exemption support — ADR-027 Storage-exemption rule stays deferred (documented in ADR)
- Verify: arch-lint test passes; deny rule fires on backward-dep violations ✅

**B.2 — Gap #5: Test-file exclusion — KEPT** ✅ (scope revised)
- `**/*_tests.rs` exclusion REMAINS in `arch-lint.toml`
- Rationale documented in `arch-lint.toml`: arch-lint 0.4.3 cannot distinguish test fakes (`Storage::new_in_memory()`, `MockBackend`) from production leaks
- Investigation surfaced 8 violations when exclusion was lifted — 4 legitimate test fakes, 1 real leak (`prompt_preset_tests.rs:2` domain test → application), 3 ports→test-support test fake imports
- Decision: test files reviewed at PR time, not lint-enforced. The 1 real leak is a Tier-2 concern, out of scope for this plan.

### Phase C — Constructor Cleanup (~18 SP, NOT GATED)

**Gate removed 2026-07-04.** Original T2 gate was self-imposed hedge against merge conflicts. R2's N21 (cancel-token registration at `init_game.rs:299`) and Phase C (LlmCallRecorder wiring at `game_service.rs:45`, `agent.rs:56`, `init_game.rs:307`) touch different fields in different files. No real conflict.

**C.2 — Gap #6: Promote `with_backends` to prod constructor (5 SP)** [DO FIRST]
- Update `src/adapters/driving/http/server_impl.rs:28-35` to use `GameService::with_backends(recorder, registry)` instead of `with_storage(storage, preset_storage, settings)`
- Demote `with_storage` to test-support-only (move to `test_support/` or feature-gate)
- Update `bootstrap/init_game.rs:249-261` to wire recorder + registry into `GameService` rather than building `GameServiceContext` directly
- Verify: `with_storage` has no non-test callers; `with_backends` is sole prod path

**C.1 — Gap #1: Remove bootstrap service-locator calls (13 SP)** [DO SECOND]
Broken down:

- (a) **Inject recorders via constructors (5 SP)**: `QuantifierAgent`, `GameService` gain `Arc<LlmCallRecorder>` constructor params; remove `bootstrap::llm_factory::get_llm_recorder_for` calls from:
  - `src/application/game_service.rs:45`
  - `src/application/agents/quantifier/agent.rs:56`
  - `src/bootstrap/init_game.rs:307` (this becomes a call site that injects, not fetches)
- (b) **Update init_game.rs arrival wiring (3 SP)**: `ArrivalTaskContext` receives recorder via constructor (already has field — just change caller); `spawn_arrival_task_if_needed` receives recorder as param
- (c) **Move `build_fresh_initial_state` access (3 SP)**: Move from `application_service.rs:15` (use stmt) + `:308` (call) to a `GameStateFactory` port in `application/ports/`; bootstrap injects impl at construction. Application no longer reaches into `crate::bootstrap::`
- (d) **Move `create_text_check_service` to bootstrap proper (2 SP)**: `src/adapters/driving/http/server_impl.rs:22` call should already be in bootstrap wiring path, not driven adapter startup — relocate call to `bootstrap/` and pass `TextCheckService` as constructor arg
- Verify: `grep -rn "crate::bootstrap::" src/application/ src/adapters/` returns 0 hits in non-test, non-bootstrap files

### Phase E — Domain Cleanup (~8 SP, parallel to C)

**E.1 — Gap #7: Move Connection to port config (3 SP, downgraded from 8)** [DO THIRD]

Verified finding: `LlmProvider` port already exists at `application/ports/llm_provider.rs` (transport-only trait, 4 impls, used as `Arc<dyn LlmProvider>` inside `LlmCallRecorder`). `Connection` is constructor input via `XxxBackend::from_connection(&Connection)`, NOT a runtime arg. Application layer never sees `Connection` at runtime. The port IS the seam; only placement is wrong.

- Rename `Connection` → `LlmProviderConfig`
- Move struct from `domain/model/settings.rs:47-57` to `application/ports/llm_provider.rs`
- Update `bootstrap/llm_factory.rs:15-26`: signature becomes `get_llm_recorder_for(config: &LlmProviderConfig, storage: Arc<Storage>)`. Dispatch on `config.provider` instead of `connection.provider`
- Rename `from_connection` → `from_config` on 4 backends (DeepSeekBackend, OpenRouterBackend, OllamaBackend, MockBackend if applicable). Signature rename only.
- Keep ALL fields: `id`, `name`, `provider`, `model`, `api_key`, `base_url`, `single_user_message`, `max_tokens`, `max_context_tokens`. `id`/`name` are user-facing identity (settings UI), not pure infra.
- Update ~20 consumer sites (settings UI handlers, bootstrap)
- Verify: `grep -rn "domain::model::settings::Connection" src/` returns 0; `LlmProviderConfig` lives in ports

**E.2 — Gap #8: Move `snapshot_id` off `Swipe` (DROPPED as YAGNI — Deviation 3)**
- Review was wrong: field on `Swipe` (line 12), not `Message`; type `u64`, not `i64`
- Dropping: `snapshot_id` is a DB-assigned FK, structurally identical to `Message::id` (also DB-assigned). Pure DTO split would force `MessageRow` duplication across the entire message aggregate — more complexity than it solves.
- Hexagonal principle is dependency *direction* (domain must not depend on adapter types); `Option<u64>` is a primitive, no direction violation.
- 6 application-layer sites read the value legitimately; moving to mapper only would force N+1 `storage.fetch_snapshot_id_for_swipe()` queries.
- ADR-027 §Deviation 3 records the decision.

### Phase F — Arrival Extraction (~5 SP, GATED on R2 + N21)

**F.1 — Gap #11: Extract arrival narration to application use case (5 SP)**

**Gate removed (2026-07-04):** Originally gated on R2/N21 (cancel-token registration on same struct). User removed gate: T2 plan 2+ weeks stale, no movement. F.1 lands with `cancel_token: Option<CancellationToken>` placeholder field; R2 fills it in when it lands. Same pattern as the removed Phase C gate.

Must resolve G2-G5 grilling decisions first (per former super-plan T2-ARCH §"Grilling decisions deferred to sub-plan"):

- **G2 (naming)**: `Narrator` (conflicts with `AGENT_NARRATOR`?), `NarrationDriver`, `SceneNarration`, or two modules `ArrivalNarration` + `InputNarration` (rejects deepening). Decide via `improve-codebase-architecture` skill grilling loop.
- **G3 (state ownership shape)**: (a) always receive `GameState`; (b) always own storage + load internally; (c) two entry points sharing internals.
- **G4 (cancellation)**: (a) require `cancel_token` param; (b) `Option<&CancellationToken>`; (c) always require token with spawner supplying it. **Coordinate with R2** — N21 token registration fix is owned there.
- **G5 (ADR-018 conflict)**: `architecture/system.md:208-209` currently documents `ArrivalTaskContext` as deliberate; must revisit + update if extraction proceeds.

Constraints the extracted module must satisfy (from former T2-ARCH):
1. State ownership — pipeline receives pre-loaded `GameState`; arrival owns `Arc<Storage>` and loads snapshot inside `run()`
2. Preset acquisition — pipeline calls `service.load_preset_and_response_length()` at call time; arrival receives `arrival_preset` + `response_length` baked into struct at spawn time
3. Persistence policy — both route through `save_message_and_snapshot`
4. Cancellation — pipeline checks `cancel_token` pre+post LLM call; arrival task's token registration + checks owned by R2
5. Status reporting — pipeline writes `GenerationStatus::Generating` and transitions through phases; arrival only sets `Generating` then `Idle`/`Error`
6. Backwards data flow — pipeline returns `(text, backend_name, model_name)` for caller to persist `LlmMessage` forensics; arrival discards `backend_name`/`model_name`
7. Domain role — pipeline = "narrate user input"; arrival = "narrate arrival scene". Different domain meaning.

Side concerns:
- `ArrivalTaskContext` has 12 fields (coupling smell)
- `phase_narrate` returns `backend_name`/`model_name` for forensics (extracted module must expose them or persist `LlmMessage` itself)
- `spawn_blocking` is caller's concern; extracted module stays sync

Scope:
- Create `src/application/arrival_service.rs` (or per G2 naming) with trait + impl
- Move `ArrivalTaskContext` (init_game.rs:106-117) + `run()` (147-193) + `spawn_arrival_task_if_needed` (170) from bootstrap to application layer
- `ArrivalTaskContext` field `recorder: Arc<LlmCallRecorder>` injected via constructor (depends on C.1 — recorder injection pattern)
- Bootstrap `init_game.rs` retains only composition + spawn invocation
- Update ADR-018 + `system.md:208-209` per G5
- Verify: `grep -n "ArrivalTaskContext" src/bootstrap/` returns 0 hits; struct lives in `src/application/arrival_service.rs` (or G2-decided name)

### Phase G — ADR + Documentation (1 SP, woven through phases)

**G.1 — ADR-027 + system.md updates**
- Phase A: debug.rs exemption framed (#9) ✅
- Phase B: arch-lint scope split documented; test-file exclusion kept with rationale ✅
- Phase C: bootstrap service-locator removed; `with_backends` promoted
- Phase E.1: `Connection` renamed to `LlmProviderConfig`; stays in domain per Deviation 2
- Phase E.2: `Swipe::snapshot_id` kept on domain entity per Deviation 3 (YAGNI)
- Phase F: `ArrivalTaskContext` extracted; ADR-018 + `system.md:208-209` updated per G5
- #12: Deviation T1 (AppState holds `TextCheckService` directly) formalized as accepted exemption — no code change
- `system.md` "Storage Exception" section unchanged (D dropped, Storage stays concrete)
- `docs/plans/hexagonal-deferred-arch-lint-rules.md` — rules now enforced (after Phase B)

## Test Plan

**Per-phase validation gates** (`cd chronicler_engine && python build.py`):

- **Phase A**: `grep -rn "adapters::driven::text_check::CheckResult" src/` returns 0; ADR-027 §3.2 reflects actual `debug.rs` access pattern ✅
- **Phase B**: `arch-lint` runs with split scopes; cross-scope port↔adapter import fails; test files excluded per rationale ✅
- **Phase C**: `grep -rn "crate::bootstrap::" src/application/ src/adapters/` returns 0 hits in non-test, non-bootstrap files; `with_backends` sole prod constructor; `with_storage` test-only
- **Phase E.1**: `grep -rn "domain::model::settings::Connection" src/` returns 0; `LlmProviderConfig` lives in domain/model/settings.rs per Deviation 2
- **Phase E.2**: DROPPED. `Swipe::snapshot_id` stays per Deviation 3.
- **Phase F**: `ArrivalTaskContext` not in `src/bootstrap/`; arrival narration spawned via application-layer port; ADR-018 + `system.md:208-209` updated

**Acceptance criteria (whole plan)**:
1. Pure-hex invariant machine-enforced via `arch-lint` (no scope lumping; test files excluded with documented rationale)
2. `bootstrap/` is sole composition root — no application-layer or driving-adapter code reaches into `crate::bootstrap::` at runtime
3. `LlmProviderConfig` stays in domain per Deviation 2 (moving would violate `model → application` arch-lint rule)
4. `Swipe::snapshot_id` stays in domain per Deviation 3 (YAGNI — full DTO split would force duplication across message aggregate)
5. `ArrivalTaskContext` lives in application layer
6. `debug.rs` exemption documented in ADR-027 with actual access pattern
7. `AppState holds TextCheckService` (Deviation T1) documented as accepted exemption
8. `python build.py` passes with full test suite green at each phase boundary

## Assumptions

- **arch-lint 0.4.3** lacks scoped-exemption support — confirmed 2026-07-04 via crate source inspection. ADR-027 Storage-exemption rule stays deferred.
- **`Storage::new_in_memory()`** exists and is usable as test fake (Phase B.2 — scout confirmed exists)
- **`MockBackend`** is already a fake LLM provider (scout confirmed) — used in test imports
- **`LlmCallRecorder`** remains a concrete struct (not a port) after Phase C — it's an application orchestrator, not an adapter concern; injecting it via constructor is sufficient pure-hex compliance
- **`LlmProvider` port already exists** at `application/ports/llm_provider.rs` (verified). E.1 is a placement rename, not new port introduction.
- **`LlmBackendType` enum stays in `domain/`** (Deviation 1, already decided) — used by both port config and domain; purely a value enum, not infra
- **#12 (AppState holds TextCheckService)** stays as Deviation T1 — formalized in ADR-027, no code change. Review agreed this is acceptable.
- **#9 (debug.rs)** accepted as documented exemption per user answer
- **`LlmProviderConfig`** keeps ALL `Connection` fields including `id`, `name` (user-facing identity, not pure infra — settings UI displays them)
- **`Swipe::snapshot_id`** stays on domain entity (Deviation 3, decided 2026-07-04). Full DTO split rejected as YAGNI — `Message::id` and other DB-assigned values already live in domain. `Option<u64>` is a primitive, no dependency-direction violation.
- **ADR-018 + `system.md:208-209`** must be updated as part of Phase F per G5 — not deferred
- **Build validation** at each phase boundary via `python build.py` (fmt + clippy + tests + coverage) per AGENTS.md
- **Sub-agent delegation**: Phase C mechanical constructor migrations suitable for `worker` subagent (mid-tier model). Primary agent verifies + runs `build.py` after each sub-task. Phase F grilling (G2-G5) requires `oracle` or `planner` (top-tier) — not delegated to worker.
- **Supersession scope**: This plan supersedes `abstraction-fixes-followup-superplan.md` ONLY for items it owns (#11 / ArrivalTaskContext extraction, formerly T2-ARCH). Other T-tracks (T1, T4, T5, T6, T9, T10) and R-tracks (R1, R2) from that super-plan are **not** superseded — they remain independently scheduled. User may consolidate later if desired.

## Progress log

- **2026-07-04**: Phase A complete (A.1 + A.2). `build.py` PASS (1247 tests).
- **2026-07-04**: Phase B complete (B.1 scope split + new deny rules; B.2 test-exclusion kept with rationale). `build.py` PASS (1247 tests).
- **2026-07-04**: Gate on Phase C removed. Original gate was self-imposed hedge against merge conflicts that don't actually exist. Only Phase F remains gated (genuine R2/N21 overlap on `ArrivalTaskContext` struct).
- **2026-07-04**: Phase C complete (C.2 with_storage promoted to prod constructor; C.1 bootstrap service-locator removed; build_fresh_initial_state moved to GameServiceContext method; create_text_check_service relocated to bootstrap::wiring; ServerResources expanded). `build.py` PASS (1247 tests). Storage exemption reduced 5→3 files.
- **2026-07-04**: Phase E.1 complete (Connection → LlmProviderConfig rename across all .rs files; from_connection → from_config on all 4 backends; rusqlite::Connection preserved as DB handle). `build.py` PASS (1247 tests). Renamed-in-place per Deviation 2.
- **2026-07-04**: Phase E.2 DROPPED as YAGNI. `Swipe::snapshot_id` stays on domain entity per ADR-027 Deviation 3 (new). Reviewer's specific claim was wrong (field on Swipe not Message, u64 not i64); `Message::id` is also DB-assigned; full DTO split would force duplication across message aggregate for zero architectural benefit.
- **2026-07-04**: Phase F gate removed. Originally gated on R2/N21 (cancel-token registration on ArrivalTaskContext struct). User removed gate: T2 plan 2+ weeks stale. F.1 preserves current cancel_token behavior (in GameServiceContext, run() doesn't check it); R2 adds checks + token registration later.
- **2026-07-04**: Phase F.1 complete. `ArrivalTaskContext` struct + `run()` + `run_sync()` + `new_for_test` moved from `bootstrap/init_game.rs` to `application/arrival_service.rs`. `inject_scenario_logs` also moved from `bootstrap/scenario.rs` to `application/scenario.rs` (pure application-domain logic). `spawn_arrival_task_if_needed` KEPT in bootstrap per plan deviation — it's composition root wiring (builds GameServiceContext + LlmCallRecorder, spawns blocking task). G2-G5 decisions: keep struct name (no rename), preserve state ownership shape, preserve cancellation behavior, system.md references updated. Plan Deviation note added to plan doc.
