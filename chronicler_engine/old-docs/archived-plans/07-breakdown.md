# Ticket 07 — Subtask breakdown (pre-dispatch)

Scouted 2026-07-10. Code state confirmed vs ticket body.

## Current state

| Item | Location | Notes |
|------|----------|-------|
| `GenerationGate` | `src/application/generation_gate/gate.rs` (131 LOC) | 2 fields: `cancel_token: CancellationToken`, `is_generating: Arc<AtomicBool>`. Owns `start_action`/`heal_stale_generating`/`claim_generation_slot`/`release_generation_slot`. |
| Phase-boundary checks | `phases.rs:125,198,232` | 3 sites `self.app.cancel_token().is_cancelled()` → `Err(ActionOutcome::Cancelled)`. Matches ticket. |
| `GenerationGuard` Drop | `src/application/generation_guard.rs` | Unconditional `self.0.store(false)`. Needs active-for-game check. |
| `reset_handler` | `src/adapters/driving/http/fragments/misc/game_control.rs:12-33` | Blocks on `is_generating`, calls `current_cancel_token().cancel()` + `replace_cancel_token()`. Ticket says stop touching tokens/flags. |
| `AppState` | `app_state.rs:48-49` | `cancel_token: Arc<RwLock<CancellationToken>>` + `is_generating: Arc<AtomicBool>`. |
| Shutdown path | `server_impl.rs:19-40,57-64` | Constructs both, `shutdown_signal` cancels token. Stays per ticket. |
| `current_game_id()` | `application_service.rs:237` → `game_catalogue.current_game_id()` → storage. Available. |
| `DefaultApplicationService::new` 6-arg | `application_service.rs:~60-77` | Takes `cancel_token` + `is_generating`. Façade-first wants signature preserved but ticket changes generation model — tension. |
| Test builder | `test_support/test_app_builder.rs:328-343` | Constructs token + flag. Must update. |
| message_editing.rs | lines 31, 137, 178 | ALSO uses `cancel_token().is_cancelled()` — NOT named in ticket 07. Gap 1. |

## PLAN GAPS — need user decision before dispatch

### Gap 1 — message_editing.rs uses cancel_token at 3 sites (lines 31, 137, 178)

Ticket 07 only names phases.rs:125,198,232 + gate.rs:66,80. But `retry_last_response` + `retrigger_event` are pipeline spawn entry points too — same α-mismatch logic applies. Verification gate says `grep is_cancelled phases.rs gate.rs` returns 0, but message_editing.rs would still hit.

- **A)** Extend ticket 07 to migrate message_editing.rs too (3 extra callsites).
- **B)** Defer message_editing.rs to follow-up ticket; keep its cancel_token reads working somehow.

### Gap 2 — Façade `cancel_token()` accessor breaks if GenerationGate drops field

Ticket says "Shutdown-token stays on AppState" but AppState's token is `RwLock<CancellationToken>` (HTTP-owned), NOT the same field GenerationGate owned. The `app.cancel_token()` accessor currently returns `&CancellationToken` (ref). Phases + message_editing read it during migration.

- **A)** Keep a generation-facing token accessor path during transition, replaced by α-check per callsite.
- **B)** Do α-migration + token removal in one atomic chunk (higher risk, one big subagent task).

### Gap 3 — reset_handler behavior change

Current: blocks with 503 when `is_generating`. Ticket 05/07: reset does NOT block — creates new game, old gen aborts at boundary. Removes `service_unavailable_generating()` response path for reset. User-visible HTTP behavior change.

- **A)** Intended — drop the guard (per ticket 05 resolution summary).
- **B)** Keep a guard but per-game (block only if THIS game is generating).

### Gap 4 — is_generating() facade → per-game ripples beyond façade-first

Ticket 05 decision 6: `is_generating(game_id)`. But AppState exposes `pub is_generating: Arc<AtomicBool>` (field), used directly in reset_handler + message_editing retry/retrigger. Per-game means AppState field shape changes, server_impl construction changes, test_app_builder changes, ~3 direct-field reads migrate to method calls. Tension with ticket 04 "façade-first, preserve signatures".

- **A)** Full per-game migration (ticket as written) — accept ripple.
- **B)** Keep global `is_generating: Arc<AtomicBool>` as derived read-only view (sum over registry: any game generating?) + add per-game registry for α-check. Less faithful to ticket 05 but smaller ripple.

### Gap 5 — Naming collision

Domain already has `GenerationStatus` enum (`src/domain/model/state/generation_status.rs`, with `GenerationPhase` + `GenerationStatus::Generating`). Ticket 07 introduces a NEW `GenerationStatus = Idle | Generating { generation_id }` on GenerationGate. Name clash.

- **A)** Rename new type to `GenerationSlot` or `GenerationRegistryEntry`.
- **B)** Reuse domain `GenerationStatus` (semantically wrong — pipeline phase vs registry slot).

**Locked decisions (user, 2026-07-10): A / A / A / B / A.**
- Gap 1 = **A** (extend ticket to migrate message_editing.rs → 07h included)
- Gap 2 = **A** (transition accessor path during migration)
- Gap 3 = **A** (drop reset_handler generation guard — reset always proceeds)
- Gap 4 = **B** (keep global `is_generating: Arc<AtomicBool>` as derived view + add per-game registry for α-check; smaller ripple than full per-game migration)
- Gap 5 = **A** (new type named `GenerationSlot`, not `GenerationStatus`)

## Subtask breakdown (assuming A/A/A/B/A)

| ID | Scope | Files | SP | Depends | Verify |
|----|-------|-------|----|---------|--------|
| **07a** | Introduce `GenerationSlot` enum + `Arc<RwLock<HashMap<GameId, GenerationSlot>>>` registry on `GenerationGate`. Plumb-only — no behavior change. `claim_generation_slot`/`release_generation_slot`/`heal_stale_generating` operate on registry keyed by `current_game_id()`. Constructor takes registry instead of `is_generating`. **GenerationGate drops `cancel_token` field entirely** (pure generation module now — shutdown concern moved out, see 07b). **Two-source-of-truth invariant (Gap 4=B):** registry = write-side truth; `is_generating: Arc<AtomicBool>` = read-only projection, ADR-030 invariant preserved. Same `claim`/`release` path that mutates registry also asserts/updates the atomic (CAS on registry → atomic must match). No other site writes the atomic. **Constructor blast radius (F4):** `DefaultApplicationService::new` 6-arg shape preserved but 2 args swap meaning: was `(cancel_token, is_generating)`, now `(shutdown_token, is_generating_projection)` — registry constructed internally (from shutdown_token + projection). server_impl + test_app_builder adapt. Arg-swap documented in constructor doc comment. | `generation_gate/{gate,mod}.rs`, `generation_guard.rs`, `application_service.rs` (new 6-arg wiring), `server_impl.rs`, `test_support/test_app_builder.rs` | 5 | — | build green; `grep is_generating gate.rs` ≤ 1; `grep -n 'is_generating.store\|is_generating.compare_exchange' gate.rs` only in claim/release; `grep cancel_token gate.rs` = 0; `python build.py` green (construction sites compile) |
| **07b** | α-mismatch check at 3 phase boundaries in `phases.rs` + gate.rs spawn check. Replace `cancel_token().is_cancelled()` with `storage.current_game_id() != generation_started_for_game_id` → abort (don't persist). Thread `generation_started_for_game_id` into pipeline run. **Also:** remove redundant pre-flight at `gate.rs:58-64`. **Drop `cancel_token` field from GenerationGate** (deferred from 07a per option b — field has `// TODO(07b)` marker). **Shutdown separation (F3):** add `app.is_shutting_down()` accessor on DefaultApplicationService (reads AppState.shutdown_token via new field/forward); gate.rs:96 spawned-task check becomes `if app.is_shutting_down() { return; }`. Shutdown concern fully out of GenerationGate. **Observability (F5):** add `tracing::info!` at each abort point: `"Pipeline aborting: game changed (started={started}, current={current}) — discarding in-flight generation"`. `info!` not `warn!` — expected behavior. **Code cloning (F7):** extract `PipelineRun::check_game_unchanged(&self, started_for: GameId) -> Result<(), ActionOutcome>` helper containing the check + log. 3 phase-boundary sites become `self.check_game_unchanged(started_for)?;`. gate.rs spawn check is shutdown (structurally different) — stays inline. | `action_pipeline/phases.rs`, `generation_gate/gate.rs`, `action_pipeline/pipeline.rs`, `application_service.rs` (is_shutting_down accessor), `app_state.rs` (shutdown_token rename/expose) | 5 | 07a | `grep is_cancelled phases.rs gate.rs` = 0; `grep is_shutting_down gate.rs` = 1; `grep generation_started_for_game_id phases.rs` ≥ 3; `grep -c 'Pipeline aborting: game changed' src/application/action_pipeline/phases.rs` = 1; `grep cancel_token src/application/generation_gate/gate.rs` = 0 (field dropped); `python build.py` green |
| **07c** | `reset()` internalizes registry cleanup; `reset_handler` stops touching tokens/flags. `service.reset()` handles everything. Old gen A keeps running, aborts at next boundary. **Also (F8=A):** drop redundant `is_generating` guards in `GameCatalogue::create_game`/`switch_game`/`delete_game` (gate.rs:27,65,78) — α-check handles in-flight gens mechanically; UX aligns with reset (always proceeds). **Note:** behavior change beyond ticket 05's reset-only decision — user authorized 2026-07-10. **Also (07a refinement):** `claim_generation_slot` CAS-on-atomic must change to registry-only per-game check (atomic was global gate, blocks concurrent gens across games). Atomic becomes write-only projection: `store(true)` on any claim, `store(false)` on release if registry has no Generating slots. Enables concurrent gen across games (game A generating → reset → game B can start). | `application/game_catalogue/gate.rs`, `fragments/misc/game_control.rs`, `app_state.rs`, `generation_gate/gate.rs` (claim/release logic) | 5 | 07a, 07b | `grep is_generating app_state.rs` = 0; `grep -c 'is_generating.load' src/application/game_catalogue/gate.rs` = 0; `grep 'compare_exchange' src/application/generation_gate/gate.rs` in claim path = 0 (registry-only check); `python build.py` green |
| **07d** | `GenerationGuard` Drop checks "am I still active generation for my game?" before touching registry. Old gen A cleanup doesn't clobber new game B's slot. | `generation_guard.rs`, `generation_gate/gate.rs` | 3 | 07a | unit test: A drops after B claimed → A no-op |
| **07e** | Shutdown-token rename on AppState (`cancel_token` → `shutdown_token` or clear comment). Migrate `message_editing.rs` callsites to α-check (Gap 1=A). Construction sites already done in 07a. | `app_state.rs`, `server_impl.rs`, `message_editing.rs` | 3 | 07a, 07c | build green; `grep 'is_generating\b' app_state.rs` = 0; `grep is_cancelled message_editing.rs` = 0 |
| **07f** | ADR-030 amend or ADR-033 (single-writer → per-game). Update "Access Pattern" section. **Also (F8=B):** document create/switch/delete guard-removal asymmetry rationale. **Also (F9):** document α-check race boundary — α-check samples current_game_id at phase boundaries; save_snapshot still keys by storage's current game_id (atomic). Race between α-check pass and save is bounded to one phase's worth of stale work — by design per ticket 05. Not a defect. | `docs/adr/adr-030*.md` | 1 | 07b | doc review |
| **07g** | Regression test covering P4-concurrency: **(1) Happy path** — start gen A → reset → start gen B → A completes → verify A discarded + B unaffected. **(2) Triple-overlap** — A running → reset → B started → another reset → C started → A completes → verify A and B discarded, C fine (proves CAS/Drop logic generalizes past single overlap). Case (3) shutdown-during-α-abort skipped as edge-case-y. | `action_pipeline/pipeline_tests.rs` or new | 3 | 07b, 07c, 07d | both tests pass |
| **07h** | *(Merged into 07e per F4 — message_editing migration moved there.)* | — | — | — | — |

**Dependency order:** 07a → 07b → 07c → 07d → 07e; 07f after 07b; 07g after 07b+07c+07d. (07h merged into 07e.)

**Total (post-review):** ~25 SP across 7 live tasks (07h merged). Each ≤5 SP.

## Dispatch plan

Sequential (dependencies prevent parallelism):
1. 07a (5 SP) — registry plumb + 6-arg constructor rewire
2. 07b (5 SP) — α-check + check_game_unchanged helper + is_shutting_down accessor — primary MUST verify build green after
3. 07c (5 SP) — reset internalization + GameCatalogue guard drops
4. 07d (3 SP) — GenerationGuard Drop active-for-game check
5. 07e (3 SP) — message_editing α-migration + shutdown_token rename
6. 07f (1 SP) — ADR-030 amend
7. 07g (3 SP) — regression tests (happy + triple-overlap)

After each: primary agent runs `python build.py` + grep verification gates.

## Required outputs (per improve-ai-plan skill)

### NOT in scope
- Full caller-site migration (G1-B) — `app.cancel_token()` → `app.is_shutting_down()` at ~30 sites. Façade-first preserved.
- Caller migration of `is_generating()` accessor — stays as façade delegate to projection atomic.
- T5 test_support full builder collapse.
- `ServerResources` parallel-field ghost.
- LLM provider concurrency hardening.
- Case (3) shutdown-during-α-abort test — edge-case-y, deferred.
- `save_snapshot` re-keying by started_for_game_id (mechanical race guarantee) — race bounded by design per F9.

### What already exists (reuse, don't reimplement)
- `current_game_id()` — on `DefaultApplicationService` (facade) → `GameCatalogue::current_game_id()` → `Storage::current_game_id()` (atomic read, in-memory). No new accessor needed for α-check sample.
- `GenerationStatus` domain enum (`src/domain/model/state/generation_status.rs`) — DON'T reuse for registry slot (Gap 5=A names new type `GenerationSlot` to avoid collision).
- `CancellationToken` + `Arc<RwLock<CancellationToken>>` shutdown pattern on AppState — stays, just clarified as shutdown-only.
- `tracing::info!` macro — for F5 abort logs.
- `ActionOutcome::Cancelled` enum variant — reuse as α-abort return type (already returned by existing phase checks).
- `GameId` type — already in use (`u64` alias).
- `read_lock_or_recover`/`write_lock_or_recover` (`http/locks.rs`) — for shutdown_token RwLock access.

### Failure modes
For each new codepath — what happens when it fails:

- **α-check abort (07b):** Pipeline returns `Err(ActionOutcome::Cancelled)`. `execute_action_impl` early-returns. No persistence. Log emitted (F5). In-flight LLM tokens spent but result discarded (known, bounded — F9).
- **Registry lock poisoned (07a):** `RwLock<HashMap>` poisoned if a panic during claim/release. Recovery: `unwrap_or_else(|p| p.into_inner())` — same pattern as `read_lock_or_recover`. Registry continues. Defensive but standard for this codebase.
- **GenerationGuard Drop race (07d):** Old gen A's Drop sees active slot = gen B (different generation_id). A no-ops. B unaffected. Test in 07g case (1).
- **Concurrent reset during claim (07a):** `claim_generation_slot` CAS on registry keyed by `current_game_id()`. If reset changed game_id between `load_or_fresh()` and the CAS, claim goes to new game_id's slot. Acceptable — new generation runs on new game. Old gen aborts at next boundary.
- **Save after reset (07c/F9):** In-flight pipeline's `save_message_and_snapshot` uses storage's current game_id (post-reset). Stale snapshot lands to new game. Bounded to one phase's work (until next α-check catches it). Documented in ADR (07f).
- **Shutdown-during-α-abort:** α-check passes, then shutdown fires, spawned task exits via `is_shutting_down()` check at next iteration. No deadlock. Not tested (case 3 skipped).

### Unresolved decisions
None. All 5 gaps resolved (A/A/A/B/A). All 9 plan-review findings applied (F1, F2, F3, F4, F5, F6, F7, F8, F9).

## Plan review log

Findings applied (user approval in parentheses):
- **F1** — Two-source-of-truth invariant (Gap 4=B): registry write-side, atomic projection. New verify gate on 07a. (a)
- **F2** — Remove redundant pre-flight at gate.rs:58-64 (shutdown detected at gate.rs:71). 07b scope grows. (A)
- **F3** — GenerationGate drops cancel_token entirely; `is_shutting_down()` accessor moves shutdown out. 07b scope grows (5 files). (B)
- **F4** — DefaultApplicationService::new 6-arg preserved, 2 args swap meaning, 07a grows to include server_impl + test_app_builder construction. 07e shrank 5→3 SP, 07h merged into 07e. (A)
- **F5** — 4 abort-point `tracing::info!` logs at α-mismatch sites. 07b verify gate added. (a)
- **F6** — 07g regression test covers happy path + triple-overlap. Case (3) skipped. (A)
- **F7** — Extract `PipelineRun::check_game_unchanged` helper; 3 call sites one-liners. Log reproductions 4→1. 07b verify gate updated. (A)
- **F8** — Drop `is_generating` guards in GameCatalogue::create_game/switch_game/delete_game (gate.rs:27,65,78). Behavior change beyond ticket 05, user-authorized. 07c grows 3→5 SP, 07f documents asymmetry. (A)
- **F9** — Document α-check/save_snapshot race boundary in ADR. Not a defect — bounded by design. 07f scope grows (no SP change). (A)
