` block when you exit via the ready menu. Emitting the final v4 plan now with all 3 audit decisions baked in. This replaces the "audit-first" gates with concrete decisions.

<proposed_plan>
# Super-Plan: Holistic Hexagon Fixes (final — scout audits applied)

## Summary

Organise remaining A/B/C findings from 2026-07-05 holistic hexagon investigation (`tmp/holistic-hexagon-investigation-second-opinion.md`) into 9 tracks. Source verified 2026-07-06: 3 fixed (C2/C3/C17), 1 fabricated (B1), several YAGNI-deferred.

**Amendments vs v3 (scout audits 2026-07-06):**
- **H1 SubTask 1.1.4 decision locked:** scout audited `application_service_tests.rs` → Characterization A (axum imports used for assertions only, not setup). Action = relocate file to `adapters/driving/http/error_tests.rs`, no import replacement. Gate removed.
- **H6 decision locked:** scout audited 3 `phase_trigger_continuation` sites → Decision (c) semantically-distinct 3-layer chain (Site 1 `phases.rs:183` raw logic, Site 2 `pipeline.rs:145` `pub(crate)` entry + `PipelineRun::new` construction, Site 3 `pipeline.rs:214` private `map_cancelled` wrapper). No collapse. Add 3 doc comments. **Track drops 4 SP → 2 SP.**
- **H8 SubTask 4.1.1 audit closed:** scout audited `LayeredPromptAssembler` (4 callers in 4 files, has behavior via `assemble`) → Decision (3) rename-only, no inline-and-delete. `LayeredBackend` (9 usages all in `core.rs`) → rename to `BackendKind`. Vestigial-struct concern dismissed; struct has real behavior.

**Carried over from v2/v3:**
- H3 dropped — phantom abstraction
- H0 split into 2 ADRs (ADR-028 `HttpError` boundary, ADR-029 `is_generating` invariant)
- H2 rewritten as Option D (lifetime separation): service singletons → `AppState` (axum extractor); domain data → per-op `WorldView<'a>`; `GameServiceContext` deleted or shrunk to `WorldView`
- H1 SubTask 1.1.2 default = explicit impl (not blanket)
- H5 narrowed: extract pure-fn `build_narration_prompt` only; no shared `narrate_into_state`. Depends on H4 dropped; moved to Phase 2
- All parallelism removed — tasks sequential, `python build.py` verifies each

**Numerical corrections baked in:** settings reads = 9; storage exemption = 5 files; GameServiceContext = 7 methods + 7 helpers; `run.rs` = 295 LOC.

**Total:** ~35 SP across 9 tracks (down from 37 after H6 scope reduction).

## Key Changes

- **H0 (2 SP) — 2 ADRs:** ADR-028 (`HttpError` trait boundary: trait in `application/`, `IntoResponse` impl in `adapters/driving/http/`, explicit impl not blanket) + ADR-029 (`is_generating` dual-source invariant: AtomicBool = cached view, single-writer rule). Blocks H1, H4.
- **H1 (5 SP):** Move `IntoResponse for ApplicationError` from `application/` to `adapters/driving/http/` via `HttpError` trait. SubTask 1.1.4 = relocate `application_service_tests.rs` → `adapters/driving/http/error_tests.rs` (audit decision: Characterization A, no import replacement needed).
- **H2 (8 SP, 4 sub-tasks):** Split `GameServiceContext` by lifetime: service singletons (`Storage`, `LlmCallRecorder`, `CancellationToken`, `AtomicBool`, `RwLock<AppSettings>`) → `AppState` axum extractor; domain data (`World`, `Map`, `Player`, `Npcs`) → per-op `WorldView<'a>`. Production struct drops `load_state_for_test`.
- **H4 (5 SP):** Document `is_generating` dual-source invariant on `app_state.rs:50-58` + `generation_status.rs`. Audit mutation sites. Property test asserts AtomicBool never diverges from persisted `GenerationStatus` post-mutation. Don't collapse to single source (AtomicBool avoids DB hit on hot poll path).
- **H5 (3 SP):** Extract pure-fn `build_narration_prompt(ctx, history, persisted_state) -> AssembledPrompt` from `phase_narrate` (`phases.rs:73`) + `ArrivalTaskContext::run` (`arrival_service.rs:88`). Pure = `make_prompt_context` build + `LayeredPromptAssembler` setup only. Leave control flow, cancellation, `ActionOutcome::Cancelled` mapping, and persistence in each caller.
- **H6 (2 SP, down from 4):** Scout-audited: 3 sites semantically distinct, no collapse. Add 3 doc comments per scout report:
  - `phases.rs:183` `phase_trigger_continuation_raw` — "Raw trigger continuation. Does NOT handle cancellation. Caller must wrap with `map_cancelled`."
  - `pipeline.rs:145` `pub(crate) phase_trigger_continuation` — "External entry point. Constructs `PipelineRun` from `ctx`, delegates to private continuation. Used by `retry.rs` + tests."
  - `pipeline.rs:214` private `phase_trigger_continuation` — "Cancellation-wrapped continuation. Applies `map_cancelled` around `_raw`."
- **H7 (3 SP):** Arch-lint storage-direct enforcement (5-file exemption list per ADR-027 changelog corrected from 3).
- **H8 (3 SP):** SubTask 4.1.1 audit closed (decision 3 = rename-only). SubTask 4.1.2: rename `LayeredPromptAssembler` → `PromptAssembler` (~10 call sites across 4 files: `narrative_prompt/mod.rs`, `game_service.rs`, `arrival_service.rs`, `action_pipeline/pipeline.rs` + 2 doc refs) and `LayeredBackend` → `BackendKind` (9 usages all in `adapters/driven/storage/backend/core.rs`). No struct deletion.
- **H9 (5 SP):** Low-priority docs/comments bundle (C.4 self-heal, C.13 drop, D.6 Phase+Status, C.19 Message triple, D.3 build_request chain).

## Implementation

### Phase 0: Architectural Lock

- [ ] #### Task 0.1: H0 — Two ADRs (2 SP)
  - [ ] ##### SubTask 0.1a: Draft ADR-028 `HttpError` trait boundary (1 SP) — scope: trait lives in `application/`; `IntoResponse` impl lives in `adapters/driving/http/`; explicit impl over ApplicationError (not blanket). ADR-conflict check vs ADR-018 + ADR-027. Verify: ADR file exists at `chronicler_engine/docs/architecture/adr-028-http-error-boundary.md`, no conflicts logged.
  - [ ] ##### SubTask 0.1b: Draft ADR-029 `is_generating` dual-source invariant (1 SP) — scope: AtomicBool = cached view of persisted `GenerationStatus`; single-writer rule = only `ApplicationService` mutates both in same critical section. ADR-conflict check vs ADR-018 + reliability plan R2. Verify: ADR file exists at `chronicler_engine/docs/architecture/adr-029-is-generating-invariant.md`, no conflicts.

### Phase 1: Structural fixes

- [ ] #### Task 1.1: H1 — Move `IntoResponse` to adapter (5 SP)
  - [ ] ##### SubTask 1.1.1: Define `HttpError` trait in `application/error.rs` (trait method `status_code() -> StatusCode` + `error_body() -> ErrorResponse`). Verify: trait compiles, no methods reference axum types outside adapter. (1 SP)
  - [ ] ##### SubTask 1.1.2a: Lock decision in ADR-028 (already in Task 0.1a): explicit impl `impl IntoResponse for ApplicationError` in adapter (not blanket). Default. (0 SP — covered by H0)
  - [ ] ##### SubTask 1.1.2b: Implement `impl IntoResponse for ApplicationError` in `adapters/driving/http/error.rs` referencing `HttpError` trait methods. Delete old impl from `application/application_service.rs:9-10,95`. (1 SP)
  - [ ] ##### SubTask 1.1.3: Update `application/application_service.rs` to depend on `HttpError` trait, not axum. Remove `use axum::response::IntoResponse` from application_service.rs imports. Verify: `cargo build` clean, application layer has no `axum::*` imports. (1 SP)
  - [ ] ##### SubTask 1.1.4: Relocate `application_service_tests.rs` → `adapters/driving/http/error_tests.rs` (audit decision: Characterization A, axum imports are assertions-only, no replacement needed). Update module declaration in `adapters/driving/http/mod.rs`. Verify: tests relocate cleanly, `cargo test --test application_service_tests` runs at new location. (2 SP)
  - [ ] ##### Validate: `python build.py` passes end of task.
- [ ] #### Task 1.2: H2 — GameServiceContext lifetime split (8 SP)
  - [ ] ##### SubTask 1.2.1: Define `AppState` struct in `bootstrap/wiring.rs` holding singletons: `Arc<Storage>`, `Arc<LlmCallRecorder>`, `CancellationToken`, `Arc<AtomicBool>`, `Arc<RwLock<AppSettings>>`, `Arc<PresetStorage>`. Implement axum `FromRef` extractors for each. Verify: compiles, no domain data fields present. (2 SP)
  - [ ] ##### SubTask 1.2.2: Define `WorldView<'a>` struct in `application/world_view.rs` holding borrowed domain data: `&'a World`, `&'a Map`, `&'a Player`, `&'a Npcs`. Constructor loads from storage per-op. Verify: lifetime annotations compile, no `Arc` clones needed for domain data. (2 SP)
  - [ ] ##### SubTask 1.2.3: Rewrite handler signatures in `adapters/driving/http/` to extract `State<AppState>` + construct `WorldView` per-op. Delete `GameServiceContext` (or shrink to `WorldView` wrapper if tests still need it). Verify: `cargo build` clean, no handler takes `GameServiceContext` directly. (2 SP)
  - [ ] ##### SubTask 1.2.4: Update integration tests in `tests/integration/application/` to use new extraction pattern. Delete `GameServiceContext::load_state_for_test` from production struct (move to test helper if still needed). Verify: `cargo test` passes, no `load_state_for_test` on production struct. (2 SP)
  - [ ] ##### Validate: `python build.py` passes end of task.
- [ ] #### Task 1.3: H4 — `is_generating` invariant docs + property test (5 SP)
  - [ ] ##### SubTask 1.3.1: Document invariant on `app_state.rs:50-58` + `generation_status.rs` per ADR-029. Add `// SAFETY: AtomicBool is cached view of persisted GenerationStatus; only ApplicationService mutates both` comments at each mutation site. Verify: every mutation site has doc comment referencing ADR-029. (1 SP)
  - [ ] ##### SubTask 1.3.2: Audit all `is_generating` mutation sites (grep `is_generating.store(` + `is_generating.compare_exchange(`). Record each in a findings list. Verify: list matches actual source (no fabricated sites). (1 SP)
  - [ ] ##### SubTask 1.3.3: Add property test asserting: after any `ApplicationService` mutation, `AtomicBool.load()` == `persisted_status == Generating`. Use `proptest` or sequential test. Verify: test passes for all mutation paths; test fails if AtomicBool and persisted status diverge (inject mutation artificially to confirm test catches it). (2 SP)
  - [ ] ##### SubTask 1.3.4: Run property test under concurrent load (spawn 4 threads each calling `ApplicationService::narrate`). Verify: no divergence detected, no race conditions logged. (1 SP)
  - [ ] ##### Validate: `python build.py` passes end of task.

### Phase 2: Debt prune (sequential, no parallelism)

- [ ] #### Task 2.1: H5 — Extract `build_narration_prompt` pure fn (3 SP)
  - [ ] ##### SubTask 2.1.1: Extract `fn build_narration_prompt(ctx, history, persisted_state) -> AssembledPrompt` in `application/narrative_prompt/mod.rs`. Body = `make_prompt_context(...)` construction + `LayeredPromptAssembler::new(...).with_max_tokens(...).assemble(...)` call. Pure — no mutation, no cancellation check, no persistence. Verify: pure function compiles, no `&mut` params, no IO. (1 SP)
  - [ ] ##### SubTask 2.1.2: Replace prompt-construction lines in `phase_narrate` (`phases.rs:73`) with call to `build_narration_prompt`. Keep cancellation check + `ActionOutcome::Cancelled` mapping + pipeline-stage persistence in `phase_narrate`. Verify: behavior identical (golden test snapshot unchanged), no `LayeredPromptAssembler` direct reference in `phases.rs`. (1 SP)
  - [ ] ##### SubTask 2.1.3: Replace prompt-construction lines in `ArrivalTaskContext::run` (`arrival_service.rs:88`) with call to `build_narration_prompt`. Keep inline persistence in arrival path. Verify: behavior identical, no `LayeredPromptAssembler` direct reference in `arrival_service.rs`. (1 SP)
  - [ ] ##### Validate: `python build.py` passes end of task.
- [ ] #### Task 2.2: H6 — Doc comments on 3 continuation sites (2 SP, down from 4)
  - [ ] ##### SubTask 2.2.1: Add doc comment to `phases.rs:183` `phase_trigger_continuation_raw`: "Raw trigger continuation: LLM call, state commit, snapshot persistence. Does NOT handle cancellation semantics; caller must apply `map_cancelled`." (1 SP, includes verifying line still correct)
  - [ ] ##### SubTask 2.2.2: Add doc comments to `pipeline.rs:145` `pub(crate) phase_trigger_continuation` ("External entry point. Constructs `PipelineRun` from `ctx` and delegates to private continuation, which wraps raw logic with cancellation handling. Used by `retry.rs` and integration tests.") and `pipeline.rs:214` private `phase_trigger_continuation` ("Cancellation-wrapped continuation. Applies `map_cancelled` around `phase_trigger_continuation_raw` so `Cancelled` errors propagate as handled cancellations."). (1 SP)
  - [ ] ##### Validate: `python build.py` passes end of task (no code changes, only doc comments; verify clippy + fmt clean).
- [ ] #### Task 2.3: H7 — Arch-lint storage-direct enforcement (3 SP)
  - [ ] ##### SubTask 2.3.1: Update ADR-027 changelog to reflect corrected 5-file exemption list (was 3). Verify: list matches actual source — grep storage-direct access sites and confirm 5 files. (1 SP)
  - [ ] ##### SubTask 2.3.2: Add arch-lint rule (in `scripts/` or as rustdoc test) that fails build if storage-direct access appears outside the 5 exempted files. Verify: rule runs in `python build.py`, fails on injected violation (add temporary storage access in non-exempt file, confirm rule fails, remove). (2 SP)
  - [ ] ##### Validate: `python build.py` passes end of task.

### Phase 3: Cosmetic

- [ ] #### Task 3.1: H8 — Rename `Layered*` (3 SP)
  - [ ] ##### SubTask 3.1.1: Rename `LayeredPromptAssembler` → `PromptAssembler` across 4 files (`narrative_prompt/mod.rs` re-export, `game_service.rs` use + struct field + 2 constructions, `arrival_service.rs` construction + `.assemble()`, `action_pipeline/pipeline.rs` use + struct field + fn param). Verify: `grep -rn LayeredPromptAssembler chronicler_engine/src/` returns 0 hits. (2 SP)
  - [ ] ##### SubTask 3.1.2: Rename `LayeredBackend` → `BackendKind` in `adapters/driven/storage/backend/core.rs` (9 usages: 1 field, 8 pattern matches/constructions). Verify: `grep -rn LayeredBackend chronicler_engine/src/` returns 0 hits. (1 SP)
  - [ ] ##### Validate: `python build.py` passes end of task.
- [ ] #### Task 3.2: H9 — Low-priority docs/comments bundle (5 SP)
  - [ ] ##### SubTask 3.2.1: Comment/docs bundle — C.4 self-heal comment in `application_service.rs`, C.13 drop stale comment, D.6 Phase+Status doc in `application_service.rs`/`wiring.rs`/`generation_status.rs`. (3 SP)
  - [ ] ##### SubTask 3.2.2: Type/chain docs — C.19 Message triple, D.3 build_request chain (multi-file). (2 SP)
  - [ ] ##### Validate: `python build.py` passes end of task.

## Test Plan

Per-task validation contract: each task ends with `python build.py` (fmt + clippy + tests + coverage). Specific tests:

- **H1 (Task 1.1):** relocated `error_tests.rs` passes at new location. Added regression: HTTP error response shape pinned for `ApplicationError::Engine` + `ApplicationError::ConcurrentGeneration` (StatusCode + body).
- **H2 (Task 1.2):** all existing integration tests pass with new `AppState` + `WorldView` pattern. No `GameServiceContext::load_state_for_test` on production struct.
- **H4 (Task 1.3):** property test passes under single-thread + 4-thread concurrent load. Test catches injected divergence (verify by temporarily breaking invariant).
- **H5 (Task 2.1):** golden test snapshots unchanged (prompt output byte-identical pre/post extraction).
- **H6 (Task 2.2):** doc-comment-only, no behavior change. All tests pass unchanged.
- **H7 (Task 2.3):** arch-lint rule passes on clean tree; fails on injected violation.
- **H8 (Task 3.1):** pure rename, tests pass. 0 hits for old names in `src/`.
- **H9 (Task 3.2):** docs-only, no behavior change.

## Assumptions

- Scout audits (3 scouts, 2026-07-06) are authoritative for H1/H6/H8 decisions. Audit reports saved in session memory; regenerated reports would produce identical conclusions.
- `GameServiceContext` lifetime split (H2 Option D) is the agreed fix per improve-ai-plan Issue 4; not revisited here.
- H3 (AppSettingsView facade) remains dropped per improve-ai-plan Issue 1.
- H0 split into 2 ADRs per improve-ai-plan Issue 2.
- All tasks sequential per user direction (no parallel code editing).
- Worker subagents used for implementation per AGENTS.md cost-reduction preference; primary agent verifies 5 SP+ task outputs and runs `build.py`.
- `python build.py` is the per-task validation gate (existing chronicler_engine convention).
- Sub-plan creation order (for when sub-plans are authored): Phase 0 → Phase 1 (H1 → H2 → H4) → Phase 2 (H5 → H6 → H7) → Phase 3 (H8 → H9). Each sub-plan locks its own scope and must stop-and-report deviations per AGENTS.md plan-adherence rule.
- This super-plan does not dictate implementation; sub-plans own implementation detail.
