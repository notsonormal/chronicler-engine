# Plan: Reliability and Cancellation

**Date:** 2026-06-28
**Status:** Planning / Sub-plans pending
**Scope:** `chronicler_engine/`

## Related

- Extracted from: `docs/plans/abstraction-fixes-followup-superplan.md` (2026-06-28 split)
- Original investigation: `docs/reviews/abstraction-antipatterns-summary.md` + `docs/reviews/zone-{a,b}-*.md`
- ADR-018 (application-service, deliberate-defer of `ArrivalTaskContext`): `docs/adr/adr-018-application-service.md`

## Objective

Organise reliability and concurrency debt surfaced during the 2026-06-27 holistic re-review (originally filed under the abstraction-fixes super-plan). These are NOT abstraction debt — they are persistence-error-propagation + cancellation-contract concerns. Split out for scope clarity.

Item classes:
- **R1: Persistence Reliability** — swallowed save errors on `reset` + pipeline warn-on-save paths.
- **R2: Arrival Task Cancellation** — `ArrivalTaskContext`'s `CancellationToken` is never registered with `AppState.cancel_token`, so production cancel paths (`reset_handler`, ctrl-C shutdown) don't cancel in-flight arrival narration. Plus the missing `is_cancelled()` checks inside `ArrivalTaskContext::run()`.

## Why split out

The 2026-06-27 holistic re-review mixed abstraction findings with reliability + concurrency findings. N17 in particular was framed as "add cancellation checks" — but the checks read a token that production never cancels, because the token is created fresh at `bootstrap/init_game.rs:299` and never registered with `AppState`. The super-plan absorbed the bug under "abstraction-fixes" framing, which obscured the real concurrency issue. T8 (persistence reliability) is similarly a behaviour decision, not module shape — its architecture-lens note in the super-plan said "Belongs in error-model discussion (T1)" (now relocated here as a sequence dependency).

---

## R1 — Persistence Reliability

**Findings owned:** N11, N20/M7, M7 (duplicate).

**State:**

- **N11:** `DefaultApplicationService::reset` (`application_service.rs:285-296`) does `let _ = Self::persist_initial_state_with_swipes(&ctx);` — swallows all per-message/swipe persistence failures (only `tracing::error!` inside helper). Behaviour change from `?` propagation to silent swallow. Documented in thermo-nuclear commit body.
- **N20/M7:** `ActionPipeline::run_from_input:108` — `if let Err(e) = save_message_and_snapshot(...) { tracing::warn!("Failed to save post-quantifier metadata: {e}"); }` — pipeline continues after warn. Tests that don't assert on tracing output pass despite persistence failure.

**Risk:** silent partial-persistent state on disk failure. Game appears to work; later load fails on missing starting message. Worse for production: no alert.

**Sub-plan scope:**

1. **Design decision** — how should partial-persistence failure surface? Options:
   - (a) propagate to caller (`Result<(), ApplicationError>`) — currently partially reverted by thermo-nuclear
   - (b) mark `Game` row as "needs repair" + retry on next access
   - (c) keep silent + log + add a healthcheck rule that fails on `tracing::error!` containing "Failed to save"
2. Verify the per-phase `save_message_and_snapshot` warn paths — list all 3+ sites in `pipeline.rs`. Decide each: propagate vs. continue.
3. Healthcheck: add check that greps tracing `warn!`/`error!` on save paths during test runs — surfaces silent failures.

**Blast radius:** small surface (`application_service.rs`, `pipeline.rs`), but behaviour-change risk.

**Risk profile:** touches user-facing reset behaviour. Integration tests need new coverage for partial-failure scenarios.

**Dependency:** sequence after T1 (error model unification in abstraction-fixes super-plan). Once the error seam is unified, swallow-vs-propagate becomes a single-place policy, not a per-call-site decision.

### Architecture-lens note

Not a module-shape issue — a behaviour decision. Sequence after T1 so T8's swallow-vs-propagate policy lands in a unified error seam.

---

## R2 — Arrival Task Cancellation

**Findings owned:** N17 (cancellation checks), N21 (NEW — token registration gap).

**State:**

- **N21 (NEW, P0):** `ArrivalTaskContext`'s `CancellationToken` is created fresh at `bootstrap/init_game.rs:299` and passed into the `GameServiceContext` at spawn time. It is NEVER registered with `AppState.cancel_token` (the server-held token that production cancel paths actually cancel). Production cancel paths:
  - `server/fragments/misc/game_control.rs:17` — `state.current_cancel_token().cancel()` on reset
  - `server/server_impl.rs:55-62` — ctrl-C shutdown signal cancels `app_state.cancel_token`

  Both cancel the server-held token. Neither touches the arrival task's isolated token.

  **Race window:** bootstrap (`bootstrap/run.rs:126` calls `spawn_arrival_task_if_needed`) → arrival LLM completes. Arrival runs in background on the runtime; server starts AFTER spawn. If `reset_handler` or ctrl-C fires while the arrival LLM call is in flight, the arrival task continues. `ArrivalTaskContext::run()` proceeds to `save_message_and_snapshot` at `init_game.rs:246` — writing pre-reset state on top of the reset, resurrecting the old game state with a new narration message.

- **N17 (was "P0 runtime risk", reframed):** `ArrivalTaskContext::run()` has zero `is_cancelled()` checks inside its body. The token IS plumbed to the struct (`self.ctx.cancel_token: CancellationToken`), just never read. Three checks needed (mirroring the pipeline pattern):
  - Check A: at start of `run()`, before snapshot load — mirrors `application_service.rs:186` (spawn_blocking entry).
  - Check B: between `make_prompt_context(...)` resolve and the `assembler.assemble(...).and_then(|assembled| backend.complete(...))` chain — mirrors `phases.rs:204` (pre-LLM check in `phase_trigger_continuation_raw`).
  - Check C: between `let narration = match {...}` and the `match narration` state-mutation block — mirrors `phases.rs:113` (post-LLM check in `phase_narrate`) + `phases.rs:238` (post-LLM check in `phase_trigger_continuation_raw`).

  But: N17 alone is decorative — the token is never cancelled in production. N17 checks only become load-bearing once N21 is fixed (token registration).

**Why this is P0:** real production race condition. Bootstrap-time arrival spawn is single-fire per process start, but window per-process is bounded only by LLM latency (typical 2-30s for arrival narration). Over many restarts, eventually bites. Costs a stuck-shutdown OR reset-resurrection bug.

**Sub-plan scope (the chunk R2 ships):**

1. **Token registration (N21 fix):** thread the server's cancel token from `run_server_with_config` into `spawn_arrival_task_if_needed`. Two design options:
   - **(i) Move spawn site:** relocate `spawn_arrival_task_if_needed` call from `bootstrap::run` into `run_server_with_config` after `AppState` construction (`server_impl.rs:33`ish). Arrival task gets `app_state.current_cancel_token()` directly. Breaks the "bootstrap module owns arrival spawn" boundary.
   - **(ii) Plumb token through `ServerResources`:** create cancel token at bootstrap time, pass to both `spawn_arrival_task_if_needed` and `ServerResources`; `AppState` adopts the existing token instead of creating its own. Cleaner module separation. `ServerResources` gains one new field.
   
   Recommend (ii) — preserves module boundaries, mirrors how `storage`/`settings` already thread through `ServerResources`.

2. **Cancellation checks (N17):** add three `is_cancelled()` checks in `ArrivalTaskContext::run()` per the A/B/C placement above. Unit-returning fn → early `return;` on cancel. Log style: sentence-form no prefix, matches arrival file's existing log convention (`"No snapshot found in spawn, starting fresh"`, `"Failed to save arrival message and snapshot: {e}"`). Suggested messages:
   - `"Arrival task cancelled before start"`
   - `"Arrival task cancelled before LLM call"`
   - `"Arrival task cancelled after LLM call"`

3. **Test plan:** `ArrivalTaskContext` has an 11-field constructor plus `Connection` config wiring + `MockBackend` resolution via `get_llm_backend_for`. A direct unit test asserting "no LLM call on cancellation" would require >40 lines of fixture setup, duplicating scaffolding that does not yet exist. Existing pipeline cancel tests cover the same `is_cancelled()` pattern at a layer with clean test injection (`src/application/action_pipeline/actions_tests.rs:133`, `pipeline_tests.rs:183,252`). For R2's chunk, defer arrival-specific coverage until the T2-ARCH sub-plan lands shared `NarrationDriver` test scaffolding. For N21 registration: integration-test-level — start server, trigger arrival, reset mid-flight, assert no resurrection write. Likely feasible with existing `test_app_builder` scaffolding.

4. **Verification:**
   - `python build.py` — fmt + clippy + tests + coverage must pass clean. No new clippy warnings (`#[cfg(test)]` panic exemption unaffected; no `unwrap`/`expect` added; `?` not introduced in unit-returning fn).
   - Manual diff review of the ~12 lines of checks against the pattern at `phases.rs:204`, `phases.rs:113/238`, `application_service.rs:186`.
   - Manual review of the token-registration threading diff.
   - Integration test (if feasible with existing scaffolding) for the reset-during-arrival race.

**Out of scope:**

- T2-ARCH deepening (`NarrationDriver` module extraction) — stays in abstraction-fixes super-plan.
- Helper extraction from `phase_narrate` + `ArrivalTaskContext::run` — T2-ARCH.
- Error channel refactor (`GenerationStatus::Error` vs `Err(...)`) — T1 in abstraction-fixes super-plan.
- Persistence policy on `save_message_and_snapshot` warn — R1 in this plan.

**Blast radius:**
- N21: `server/server_impl.rs` ( AppState construction adopts token), `server/app_state.rs` (`ServerResources` gains field, `current_cancel_token` reads existing), `bootstrap/run.rs` (creates token, passes to both spawn + resources), `bootstrap/init_game.rs` (`spawn_arrival_task_if_needed` signature gains `cancel_token` param).
- N17: `bootstrap/init_game.rs` `ArrivalTaskContext::run` (~12 lines).

**Dependencies:** none. R2 can proceed independently. Coordinate with the abstraction-fixes super-plan's T2-ARCH (which will benefit from the registered token for its G4 cancellation-plumbing grilling).

---

## Finding State (this plan's scope only)

| ID | Class | Owner | Note |
|----|-------|-------|------|
| N11 `reset` swallows persistence errors | active | R1 | `application_service.rs:298` `let _ = ...`; R1 decides propagate vs mark-needs-repair vs silent+healthcheck |
| N17 `ArrivalTaskContext::run()` no cancellation check | active | R2 | Three checks (pre-start, pre-LLM, post-LLM); decorative until N21 lands |
| N20 = M7 pipeline warn-on-save | active | R1 | `pipeline.rs:108`; pipeline continues after warn |
| N21 (NEW) Arrival task token not registered with AppState | active | R2 | `bootstrap/init_game.rs:299` creates fresh token, never registered. Real P0 race |

For abstraction-fixes findings, see `abstraction-fixes-followup-superplan.md` Finding State table.

**Re-flag rule:** any future audit referencing N11/N17/N20/N21 belongs in this plan, not the abstraction-fixes super-plan. Re-flag against the relevant R1/R2 sub-plan, not the raw finding.

---

## Decisions to Lock Before Sub-Plans

- **R1:** Partial-persistence failure: propagate, mark-needs-repair, or silent+healthcheck?
- **R2 (N21):** Move spawn site to `run_server_with_config` (option i), or plumb token through `ServerResources` (option ii)?

---

## Verification Strategy

- Each sub-plan defines its own test guarantees + verification commands.
- Project-level: `python build.py` (fmt + clippy + tests + coverage) is the gold standard.
- For R1 (behaviour change): require integration test coverage of partial-failure scenarios before merge.
- For R2 (concurrency): require integration test covering reset-during-arrival + ctrl-C-during-arrival races before merge.

## Sub-plan Creation Order (recommended)

1. **R1 (persistence reliability)** — P1; behaviour risk, sequence after T1 (abstraction-fixes super-plan) so the error seam is unified.
2. **R2 (arrival task cancellation)** — P0; real production race. Can proceed independently; N21 registration fix is the load-bearing part, N17 checks ride along.

## Plan Adherence

This plan **does not dictate implementation**. It enumerates scope + blast radius + dependencies. Where a sub-plan deviates, stop and report per `AGENTS.md` "Plan Adherence" rule.
