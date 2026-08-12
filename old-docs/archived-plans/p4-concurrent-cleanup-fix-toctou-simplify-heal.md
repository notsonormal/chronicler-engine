# P4-Concurrent Cleanup — Fix TOCTOU + Simplify heal

## Summary

Six findings from the thermo-nuclear review of ticket 07's uncommitted diff. Within the locked decisions (Gap 4=B: keep `is_generating` projection; T2 façade-first: preserve public signatures; user choice: no new types), this plan:

1. Fixes the TOCTOU race (finding #1) by moving projection atomic writes inside the existing registry write-lock scope on claim + release paths.
2. Simplifies `heal_stale_generating` to a single registry-locked check — no dual `if !is_generating.load()` blocks (finding #6).
3. Deletes dead delegates (`replace_shutdown_token`, `claim_generation_slot`/`release_generation_slot` façade) left by ticket 07c's cleanup (finding #3).
4. Amends ADR-030 to record that projection writes now occur inside the registry write lock (closes the pre-fix atomicity gap).
5. Adds a TOCTOU regression test that exercises the exact race window: claim game A → reset → release game A while game B is generating — pre-fix this dropped the projection to `false` with B still active.

Findings #2 (delete projection), #4 (`GenerationClaim` newtype), #5 (`GenerationRegistry` newtype + counter move) explicitly **dropped** per user decision — questionable value, phantom-abstraction risk. Lock-order fix is achievable inline without new types.

Inline cleanup PR on `simpler-hexagon` — no new T2 map ticket.

## Key Changes

- **MODIFIED** `chronicler_engine/src/application/generation_gate/gate.rs` — `is_generating.store(true)` on claim path moves inside the existing `registry.write()` scope; `heal_stale_generating` collapses to a single `if !is_generating.load()` gate with persisted-status heal outside the lock and registry-slot heal inside, where slot state is authoritative.
- **MODIFIED** `chronicler_engine/src/application/generation_gate/slot.rs` — `release_owned_slot` moves the `is_generating.store(false)` inside the existing `registry.write()` scope, after the `any_generating()` scan (still inside the lock).
- **MODIFIED** `chronicler_engine/src/application/application_service.rs` — drop dead `claim_generation_slot` + `release_generation_slot` `#[allow(dead_code)]` façade delegates (zero non-façade callers post-07c).
- **MODIFIED** `chronicler_engine/src/adapters/driving/http/app_state.rs` — drop dead `replace_shutdown_token` (zero callers post-07c reset_handler slim).
- **NEW** TOCTOU regression test in `src/application/application_service_tests.rs` (is_generating invariant section) — observable-behavior test using only public `app.is_generating()` API (no field widening). Exercises the race: claim A → reset/start B → release A → assert projection still `true` while B active.
- **MODIFIED** `chronicler_engine/docs/adr/adr-030-is-generating-invariant.md` — amendment note: projection writes now occur inside the registry write lock on both claim and release paths (closes pre-fix TOCTOU); documents heal's authoritative source-of-truth inside the lock.

No new files. No new types. No `pub(crate)` visibility changes. ~4 SP total.

## Implementation

### Phase 1: Lock-order fix (TOCTOU) + heal simplification

- [ ] #### Task 1.1: Move projection writes inside registry write lock (3 SP)
  - **SubTask 1.1.1: Fix `claim_generation_slot`** (`gate.rs`) — move `self.is_generating.store(true, Ordering::SeqCst)` from after the registry write-lock scope to inside it (after `registry.insert(game_id, GenerationSlot::Generating { generation_id })`, before the lock guard drops). Single lock acquisition per claim; projection + registry mutate atomically.
  - **SubTask 1.1.2: Fix `release_owned_slot`** (`slot.rs`) — move `is_generating.store(false, Ordering::SeqCst)` from after the registry write-lock scope to inside it (after `registry.values().any(|slot| slot.is_generating())` computes `any_other_generating`, store `false` only if `!any_other_generating`, all inside the same write guard). Single lock acquisition per release; projection + registry mutate atomically.
  - **SubTask 1.1.3: Simplify `heal_stale_generating`** (`gate.rs`) — collapse the two `if !self.is_generating.load(Ordering::SeqCst)` blocks into one gated scope (Concern 2 resolution):
    - Outer gate `if !self.is_generating.load(Ordering::SeqCst)` — used only as an optimization to skip the write-lock acquisition when trivially idle. Document this as an optimization, not authoritative.
    - First heal (persisted status: `state.narrative.input_buffer.status == Generating → Idle` + phase reset) stays OUTSIDE the registry write lock — touches `GameState`, unrelated to registry lock, keeps lock hold time minimal.
    - Second heal (registry slot for `current_game_id()`) runs INSIDE the registry write lock, **re-checking `slot.is_generating()` (or `self.is_generating.load()`) INSIDE the lock before clearing** (Concern 3 resolution). The pre-lock atomic read is not trusted as authoritative for the clear decision — slot state inside the lock is the source of truth. Comment: "outer `is_generating.load()` is only an optimization to skip lock acquisition; inside, slot state is authoritative."
    - Net: single `if !is_generating.load()` gate wrapping both heals, single registry write-lock acquisition for the slot heal, inner re-check gates the clear.
  - **Failure mode (Finding D)**: `save_message_and_snapshot` failure post-claim releases via `release_owned_slot` using the same registry write lock — projection update stays atomic with slot release. Documented in the inline comment at the release call site.

### Phase 2: Dead code cleanup

- [ ] #### Task 2.1: Drop dead delegates (1 SP)
  - Delete `replace_shutdown_token` from `AppState` (zero callers; reset_handler was sole user, slimmed in 07c)
  - Delete `claim_generation_slot` + `release_generation_slot` façade delegates on `DefaultApplicationService` (both `#[allow(dead_code)]`, zero non-façade callers post-07c — `GenerationGate` methods called directly within `start_action`)
  - `cargo build` green; address any fallout from removed symbols (expected: none)

### Phase 3: TOCTOU regression test

- [ ] #### Task 3.1: Add TOCTOU invariant test (1 SP)
  - New test in `src/application/application_service_tests.rs` (is_generating invariant section): `test_projection_invariant_under_interleaved_release` (Concern 1 resolution — observable-behavior only, no field widening)
  - **Test shape** (uses existing public API `app.is_generating()` + `app.process_action()` only, no `registry` field access):
    1. Setup app with a `MockBackend` that has a configurable delay (≥200ms) so generation A stays in-flight during the test window.
    2. Start generation A: `app.process_action("look")` → `ProcessActionResult::Started`. Assert `app.is_generating().load() == true`.
    3. Call `reset` (or `create_game`) to create game B — this is the trigger that, pre-fix, would race against A's release.
    4. Start generation B: `app.process_action("go north")` → `Started`. Assert `is_generating().load() == true`.
    5. Wait for A's pipeline to reach its phase boundary and abort (α-check mismatch on `started_for != current_game_id`). Pre-fix: A's `GenerationGuard::drop` calls `release_owned_slot` which stores `false` on the projection OUTSIDE the registry lock — clobbers B's active projection. Post-fix: `release_owned_slot` checks ownership + `any_generating()` INSIDE the registry write lock; A's drop is a no-op (B owns the slot), projection stays `true`.
    6. Assertion (post-A-drop, B still generating): `app.is_generating().load() == true`. Pre-fix this would be `false` (the bug); post-fix it's `true`.
    7. Wait for B to complete + drop. Final assertion: `app.is_generating().load() == false`.
  - Test is non-threaded (single test thread driving sequential `process_action` calls; the `spawn_blocking` tasks run on Tokio's blocking pool and race against the test thread). Deterministic enough — uses existing `pipeline_helpers::wait_for_condition` pattern from `invariant_contract.rs` for synchronization.
  - Verify existing tests in `tests/infrastructure/invariant_contract.rs` + `src/adapters/driving/http/fragments/generation_guard_tests.rs` still pass (no API change, signature preserved, no field widening).

### Phase 4: ADR-030 amendment

- [ ] #### Task 4.1: Amend ADR-030 with lock-order note (1 SP)
  - Add subsection under existing "Amendment: Ticket 07 (2026-07-10)" titled "Lock-order fix (post-07 cleanup)"
  - Record: projection atomic writes now occur inside the `HashMap` registry write lock on both claim and release paths, atomically with the registry mutation
  - Document that this closes the pre-fix TOCTOU window where the previous amendment's "same path mutates both" claim was true only in code-path sense, not in atomicity sense (the `store` happened after the write-lock guard dropped)
  - Note the heal-path authoritative source-of-truth: outer `is_generating.load()` is an optimization to skip the write-lock acquisition; inside the lock, slot state (`slot.is_generating()`) is authoritative for the clear decision

## Test Plan

- **Cargo build green**: `cd chronicler_engine && cargo build` compiles after all phases.
- **Existing tests pass**: `cargo test` — all 1235 tests green (2 LLM skipped baseline). No API signature change, no field visibility change, so no test signature migration.
- **TOCTOU regression test passes**: new test asserts projection stays `true` during overlap (claim A → reset → claim B → A drops mid-flight), then goes `false` after B completes. Pre-fix would fail at step 6.
- **Façade signature preserved**: `rg 'is_generating\(\)' src/ tests/` — same ~15 call sites; no signature migrations, no `pub(crate)` widening.
- **Dead code gone**: `rg 'replace_shutdown_token|claim_generation_slot\(|release_generation_slot\(' src/application/application_service.rs src/adapters/driving/http/app_state.rs` returns 0.
- **Lock-order invariant**: `rg 'is_generating.store' src/application/generation_gate/gate.rs src/application/generation_gate/slot.rs` returns matches only inside `registry.write()` scope (manual code review verification).
- **Grep gates from ticket 07 still pass**:
  - `rg started_for phases.rs` ≥ 3
  - `rg is_cancelled phases.rs gate.rs` = 0
  - `rg is_generating app_state.rs` = 0
- **Full integration**: `python build.py` green (fmt + clippy + tests + coverage).

## Per Task/Sub Task Validation Steps

- **SubTask 1.1.1**: `cargo build` green. Code review: `is_generating.store(true, ...)` line is inside the `registry.write()` block in `claim_generation_slot`, not after it.
- **SubTask 1.1.2**: `cargo build` green. Code review: `is_generating.store(false, ...)` line is inside the `registry.write()` block in `release_owned_slot`, not after it.
- **SubTask 1.1.3**: `cargo build` green. `rg '!self.is_generating.load' src/application/generation_gate/gate.rs` returns 1 (not 2). `heal_stale_generating` has single registry write-lock acquisition; inner re-check of `slot.is_generating()` gates the clear (code review).
- **Task 1.1 (overall, primary verifies)**: `cargo test` green. Primary reviews all 3 modified files; confirms lock-order + heal structure visually before running tests.
- **Task 2.1**: `cargo build` green. `rg 'replace_shutdown_token|claim_generation_slot\(|release_generation_slot\(' src/application/application_service.rs src/adapters/driving/http/app_state.rs` returns 0.
- **Task 3.1**: New test passes. `rg 'test_projection_invariant_under_interleaved_release' src/application/application_service_tests.rs` returns match. `cargo test --test invariant_contract` passes. `cargo test generation_guard_tests` passes.
- **Task 4.1**: `rg 'Lock-order fix|TOCTOU' chronicler_engine/docs/adr/adr-030-is-generating-invariant.md` returns match.

## Assumptions

- Locked decisions honored: Gap 4=B (keep projection atomic as derived view) — does not delete the projection, only fixes its write path to be atomic with the registry mutation.
- Façade-first (G1=A) preserved: `app.is_generating() -> &Arc<AtomicBool>` public signature unchanged. No new types introduced. No `pub(crate)` field widening — regression test (Task 3.1) uses only existing public API (`app.process_action`, `app.is_generating()`, `app.reset`/`create_game`).
- Lock-recovery pattern (`unwrap_or_else(|p| { tracing::warn!("..."); p.into_inner() })`) stays inlined in `gate.rs` + `slot.rs`. The shared helpers in `src/adapters/driving/http/locks.rs` are NOT reused — wrong hexagonal layer (ADR-027: application layer must not import from driving adapter layer). Relocating the helpers is a separate cleanup, out of scope.
- `save_message_and_snapshot` failure post-claim releases via `release_owned_slot` using the same registry write lock (Finding D) — projection update stays atomic with slot release. No partial-state window.
- `next_generation_id: Arc<AtomicU64>` stays as-is — produces unique IDs via `fetch_add`; doesn't need the registry lock because distinct concurrent claims get distinct IDs, then serialize on the registry write lock for the slot insert. Not a race.
- `(u64, u64, ProcessActionResult)` tuple return from `claim_generation_slot` stays as-is — 1 consumer, acceptable. `GenerationClaim` newtype dropped per user decision (phantom-abstraction risk).
- `replace_shutdown_token` deletion confirmed safe: grep found zero callers (reset_handler was sole user, slimmed in 07c).
- `claim_generation_slot` / `release_generation_slot` façade delegates confirmed dead: both marked `#[allow(dead_code)]` with zero non-façade callers post-07c — `start_action` calls the `GenerationGate` methods directly.
- This cleanup is independent of T9 (WorldSnapshot removal) — different seam, no interaction.
- Heal path resolution (Concern 2): persisted-status heal stays OUTSIDE the registry write lock (touches `GameState`, unrelated to registry lock); registry-slot heal runs INSIDE the lock with inner re-check of `slot.is_generating()` as authoritative. Pre-lock `is_generating.load()` is documented as an optimization only, not authoritative.
- Heal path TOCTOU (Concern 3): resolved by Concern 2's resolution — inside the registry write lock, slot state (`slot.is_generating()`) is authoritative, not the pre-lock atomic read. The heal could observe an atomic=false-then-slot=Generating inconsistency (another thread claimed between load and lock), but the inner `if slot.is_generating()` re-check ensures the clear decision is based on authoritative slot state acquired under the lock.

## NOT in scope

- Deleting the `is_generating: Arc<AtomicBool>` projection (locked Gap 4=B keeps it).
- `GenerationRegistry` / `GenerationClaim` newtypes (dropped per user decision — questionable value, phantom-abstraction risk).
- Counter relocation into registry lock (not needed — `Arc<AtomicU64>` produces unique IDs safely).
- T9 (WorldSnapshot removal) — separate architectural track, Wave 3.
- Full caller-site migration (G1-B) — `app.cancel_token()` / `app.is_generating()` → direct module access at ~30 sites. Façade-first preserved.
- T4 PhaseError consolidation; T5 test builder collapse.
- `save_snapshot` re-keying by `started_for_game_id` (F9 race bounded by design per ticket 07).
- Relocating `read_lock_or_recover` / `write_lock_or_recover` from `adapters/driving/http/` to a shared util — separate cleanup, would enable application-layer reuse.
- Widening `GenerationGate::registry` field visibility to `pub(crate)` for test access — regression test uses only public API, no widening needed.
- Merging persisted-status heal into the registry write lock — explicit non-goal; persisted `GameState` mutation has no atomicity requirement with the registry slot clear.

## What already exists (reuse, don't reimplement)

- `current_game_id()` on `DefaultApplicationService` → `GameCatalogue` → `Storage`. Already used by α-check; no change.
- `GenerationSlot` enum + `is_generating()` method (`slot.rs`) — stays as-is.
- Existing `unwrap_or_else(|p| { tracing::warn!("..."); p.into_inner() })` lock-recovery pattern — reused in place (4 existing inline sites stay; no new sites added).
- `tracing::warn!` / `tracing::debug!` patterns — matched in any new log lines.
- `ActionOutcome::Cancelled` — α-abort return type, already in use.
- `GenerationGuard::Drop` ownership-check pattern (07d) — preserved, not rewritten.
- `pipeline_helpers::wait_for_condition` (from `tests/helpers/pipeline_helpers.rs`) — reused in Task 3.1 for synchronization.
- `MockBackend::with_delay` + `with_narrations` (from `adapters/driven/llm/providers/mock.rs`) — reused in Task 3.1 to keep generation A in-flight during the race window.
- Existing test app builders (`make_test_app_with_game_service`, `make_test_app_with_sqlite` from `test_support`) — reused in Task 3.1; no new test fixtures needed.

## Failure modes

- **Registry lock poisoned**: `RwLock` poisoned if a panic during claim/release. Recovery via existing `unwrap_or_else(|p| { tracing::warn!("..."); p.into_inner() })` pattern. Defensive but standard for this codebase. No change.
- **`save_message_and_snapshot` failure post-claim** (Finding D): release path uses the same registry write lock; ownership check (`slot.generation_id == claimed generation_id`) verifies before clearing; projection `store(false)` happens inside the same lock scope after `any_generating()` scan. Atomic with the slot release. No partial-state window.
- **`GenerationGuard::Drop` race with concurrent reset**: ownership check prevents old gen A from clobbering new gen B's slot. Already in 07d; preserved. **Task 3.1 tests this path directly** — A drops after B claimed → A's release is a no-op (ownership check fails) → projection stays `true` (B active).
- **Concurrent claim on same game_id**: registry write lock serializes; second claim sees `Generating` slot, returns `ConcurrentGeneration`. Already in 07c.
- **Projection atomic drift after lock-order fix**: impossible by construction — both writes (registry mutation + atomic store) under the same write-lock scope. Task 3.1 verifies behaviorally.
- **Heal-path TOCTOU (Concern 3)**: pre-lock `is_generating.load()` is a stale snapshot by the time the write lock is acquired; resolved by inner authoritative `slot.is_generating()` re-check inside the lock before clearing. Cannot clear an actively-claimed slot.

## Unresolved decisions

None. All 3 question prompts resolved (inline PR / lock-order fix only / drop `GenerationRegistry` + `GenerationClaim` / reuse inline lock pattern / document partial-claim failure mode). All 3 concerns resolved:
- Concern 1 (test field access): use observable-behavior test via public API only, no `pub(crate)` widening.
- Concern 2 (heal block merge "OR"): persisted-status heal stays OUTSIDE registry lock; registry-slot heal runs INSIDE with inner re-check authoritative.
- Concern 3 (heal TOCTOU): inner `slot.is_generating()` re-check inside the write lock is authoritative; pre-lock load is optimization only.
