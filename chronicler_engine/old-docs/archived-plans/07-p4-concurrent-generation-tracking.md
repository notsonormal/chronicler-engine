Type: task (AFK-able; implementation)
Status: resolved
Blocked by: (none — ticket 05 resolved)

# Ticket 07 — Implement P4-concurrent per-game generation tracking

Story points: ~8 (break down further if needed — see sub-task suggestions below). Touches: GenerationGate, action_pipeline phases, AppState, reset_handler, bootstrap, ADR-030 amendment, regression test.

Graduated from [Ticket 05](./05-appstate-token-phantom-storage.md) — grilling resolved the design. This ticket implements it.

## Question

Implement the P4-concurrent design locked in ticket 05.

## Resolution summary (from ticket 05)

Replace `cancel_token` (for generation) + global `is_generating: Arc<AtomicBool>` with per-game generation tracking:

1. **Per-game generation registry.** Replace `is_generating: Arc<AtomicBool>` on GenerationGate with `Arc<RwLock<HashMap<GameId, GenerationStatus>>>` where `GenerationStatus = Idle | Generating { generation_id: u64 }`. Exact shape decided during implementation (HashMap vs dedicated GenerationRegistry struct in its own module — grilling deferred this).

2. **α mismatch check at phase boundaries.** At each of the 3 phase boundaries (`phases.rs:125,198,232` + `gate.rs:66,80`), replace `self.app.cancel_token().is_cancelled()` with `storage.current_game_id() != self.generation_started_for_game_id` → abort pipeline, do not call `save_message_and_snapshot`.

3. **`service.reset()` handles reset internally.** `reset_handler` (HTTP) stops touching tokens/flags directly — calls `state.application_service.reset()` which:
   - Deletes old game, creates new game (current behavior)
   - Old in-flight generation A keeps running its LLM call (no interruption)
   - At A's next phase boundary, α check fails (game_id mismatch) → A aborts, does not persist
   - New game B is immediately available

4. **Generation cleanup on Drop.** `GenerationGuard` Drop checks "am I still the active generation for my game?" before touching the registry. Old generation A's cleanup does NOT clobber new game B's flag (closes the P4-serialize race).

5. **Shutdown-token separate.** `AppState.cancel_token` stays for `server_impl.rs:40,60-64` shutdown path only. Rename or comment to make its server-lifecycle role explicit ("NOT generation token — shutdown drain only"). Generation-token is now per-game on GenerationGate, NOT AppState.

6. **`is_generating()` facade becomes per-game.** `state.application_service.is_generating()` → `is_generating(game_id)`. HTTP handlers reading "am I generating?" pass the current game_id.

7. **Keep AppState.storage + AppState.preset_storage** (ticket 05 decision 5b). Not phantom — driving-side handlers use them for direct adapter ops (settings, presets). Unchanged by this ticket.

## AHDR / doc update

Amend **ADR-030** (`is_generating Dual-Source Invariant`) or write **ADR-033** — its "global AtomicBool single-writer" premise changes to per-game tracking. Decision at implementation time: amend vs new ADR. The "Access Pattern" section added to ADR-030 by T6 (2026-07-10) needs revisiting — the `pub(crate)` widening note stays accurate, but the single-writer claim is now per-game, not global.

## Sub-task suggestions (if 8 SP feels too large)

- 07a: Introduce `GenerationStatus` enum + per-game registry shape on GenerationGate (no behavior change — plumb only).
- 07b: Replace 3 cancel_token phase-boundary checks with α mismatch check.
- 07c: Update `reset()` to handle cancel/registry internally; `reset_handler` stops touching tokens/flags.
- 07d: `GenerationGuard` Drop checks active-for-game before touching registry.
- 07e: Rename/clarify AppState.cancel_token as shutdown-only.
- 07f: Update `is_generating()` facade to per-game signature; migrate HTTP handler callsites.
- 07g: ADR-030 amendment or ADR-033.
- 07h: Regression test (start gen A → reset → start gen B → A completes → verify A discarded + B unaffected).

## Verification gates (from ticket 05)

- `grep -rn 'generation_started_for_game_id\|current_game_id.*started_for' src/application/action_pipeline/phases.rs` returns 3+ hits.
- `grep -rn 'is_cancelled' src/application/action_pipeline/phases.rs src/application/generation_gate/gate.rs` returns 0 (generation no longer checks cancel_token; shutdown path keeps its own token).
- `grep -rn 'is_generating' src/adapters/driving/http/app_state.rs` returns 0.
- Regression test exists + passes.
- `python build.py` green.

## Out of scope

- T5 test_support full builder collapse.
- Full caller-site migration (G1-B).
- `ServerResources` parallel-field ghost.
- LLM provider concurrency hardening beyond ticket 05's verification.

## Answer

**Implemented 2026-07-10.** All 5 ticket 05 verification gates pass:
- `grep started_for phases.rs` = 9 (≥3) ✓
- `grep is_cancelled phases.rs gate.rs` = 0 ✓
- `grep is_generating app_state.rs` = 0 ✓
- 2 regression tests (happy path + triple-overlap) pass ✓
- `python build.py` green (1235 tests, 2 LLM skipped) ✓

**Design landed (per ticket 05 resolution, plan review A/A/A/B/A + 9 findings F1-F9):**

1. **Per-game registry** on `GenerationGate`: `Arc<RwLock<HashMap<u64, GenerationSlot>>>` where `GenerationSlot = Idle | Generating { generation_id }`. Named `GenerationSlot` not `GenerationStatus` (Gap 5=A — avoids collision with existing domain enum). Registry = write-side truth.

2. **`is_generating: Arc<AtomicBool>` kept as read-only projection** (Gap 4=B). Same claim/release path updates both. ADR-030 single-writer invariant preserved (F1). Scan-on-release: only `store(false)` when no other game is Generating.

3. **α-mismatch check at 3 phase boundaries** (`phases.rs`) + redundant pre-flight at `gate.rs:58-64` removed (F2). Extracted `PipelineRun::check_game_unchanged(started_for)` helper (F7). Log at abort point: `tracing::info!("Pipeline aborting: game changed ...")`.

4. **`GenerationGate.cancel_token` field dropped** (07b). Shutdown concern separated: new `is_shutting_down()` accessor on `DefaultApplicationService`. `AppState.cancel_token` renamed → `shutdown_token` (07e) — server-lifecycle only. `message_editing.rs`'s 3 `cancel_token().is_cancelled()` callsites migrated to `is_shutting_down()` (07e, Gap 1=A).

5. **`reset_handler` slimmed** (07c, Gap 3=A): no more `is_generating` guard (503), no more `current_cancel_token().cancel()` / `replace_cancel_token()`. Reset always proceeds. Old in-flight gen A keeps running LLM call, aborts at next phase boundary via α-check.

6. **`GameCatalogue` guards dropped** (07c, F8=A): `create_game`/`switch_game`/`delete_game` no longer block on `is_generating`. UX aligns with reset — always proceeds. α-check handles in-flight gens mechanically.

7. **`claim_generation_slot` registry-only per-game check** (07c refinement): no more global `compare_exchange` CAS. Atomic becomes unconditional `store(true)` on any claim. Enables concurrent gens across different games (game A generating → reset → game B can start its own gen).

8. **`GenerationGuard` Drop active-for-game check** (07d): holds game_id + generation_id + registry ref + atomic ref. Drop checks "am I still the active generation for my game?" before touching registry/atomic. Old gen A's Drop is a no-op if superseded by gen B. Closes the P4-serialize race.

9. **`build_fresh_initial_state` map consistency fix**: test fixtures' `create_test_state_with_npcs` updated to include a `default_scenario` with `starting_room_id="room1"` — without it, `world.starting_room_id()` falls back to `"start"` (default) which doesn't match the seeded map's `room1`. Surfaced during 07g test writing (not a production bug — production worlds always set starting_room_id via scenarios).

**ADR-030 amended** (07f): per-game tracking + projection invariant documented. F8 (create/switch/delete guard asymmetry) + F9 (α-check/save race boundary — bounded by design) recorded.

**Behavior changes (user-authorized):**
- Reset always proceeds during generation (Gap 3=A — user 2026-07-10)
- Create/switch/delete always proceed during generation (F8=A — user 2026-07-10)
- Old in-flight generation A's LLM cost is silently discarded (bounded by α-check — F5 observability log added)

**Tests deleted (testing removed old-guard behavior, 07c):**
- `test_action_concurrent_rejection` (global flag guard — replaced by per-game registry)
- `test_delete_game_handler_generating` (503 on delete during gen — removed)
- `test_reset_handler_generating` (503 on reset during gen — removed)
- `test_create_game_concurrent_generation_rejected` (create rejected during gen — removed)
- `test_pipeline_cancels_mid_run` + `test_phase_trigger_continuation_cancels_at_start` (cancel_token tests — 07b, pointer to 07g for new α-check coverage)
- `test_inv004_cancellable_at_boundaries` migrated in-place (cancel_token.cancel → set_game_id mismatch)

**New tests (07g):**
- `test_p4_concurrent_happy_path` — gen A → create_game → gen B → A discarded, B persisted
- `test_p4_concurrent_triple_overlap` — A → reset → B → reset → C → A+B discarded, C persisted

**Subtask breakdown:** `.scratch/t2-god-class-split/issues/07-breakdown.md` (7 live subtasks 07a-07g, 07h merged into 07e, ~25 SP).

**Build:** `python build.py` green (build_20260710_204824.log). 1235 tests, 0 failures.
