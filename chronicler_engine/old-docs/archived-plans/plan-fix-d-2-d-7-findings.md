# Plan: Fix D.2-D.7 Findings

**Date:** 2026-07-06
**Status:** Planning — ready (revised per `improve-ai-plan` review + D.5 dropped)
**Scope:** `chronicler_engine/src/`
**Source:** `chronicler_engine/tmp/holistic-hexagon-investigation-second-opinion.md` §D.2-D.7
**Depends on:** none
**Total:** 8 SP across 5 atomic tasks

## Summary

Five findings: one docs annotation, two pure-docs changes, one unused-re-export cleanup, one real duplication collapse. D.5 (settings helper) dropped — symptom-level fix with marginal payoff, would be discarded if a real SettingsView facade ever lands. All shippable independently, no task >5 SP.

## Already exists — reuse, don't write

| Need | Existing code |
|---|---|
| `build_fresh_initial_state` (live impl) | `GameServiceContext::build_fresh_initial_state` at `application/context.rs:44` — used by `application_service.rs:369` |
| Scenario NPC init | `GameState::init_scenario_npcs` at `domain/model/state/game_state.rs:135` |
| Scenario log injector | `inject_scenario_logs` at `application/scenario.rs:23` — unchanged |
| Build/test runner | `python build.py` |

## Key Changes

| Task | Finding | SP | Type |
|---|---|---|---|
| 1 | D.4 | 0 | Doc annotation |
| 2 | D.2 | 1 | Doc comments on 3 fields |
| 3 | D.6 | 1 | Module-level doc (conservative) |
| 4 | D.3 | 1 | Delete unused re-export |
| 5 | D.7 | 5 | Extract `apply_scenario_injection` free fn; collapse 3 → 1 impl |

## Implementation

### Task 1: D.4 — Mark audit finding non-issue (0 SP)

**Files:** `chronicler_engine/tmp/holistic-hexagon-investigation-second-opinion.md`

Change `### D.4 — MockBackend signals ⚠️ partially` header to `✅ NON-ISSUE (verified 2026-07-06)`. Body unchanged. Update final-verdict Top-3 list.

Rationale: both `narration_started` and `trigger_started` exist (mock.rs:30,32). Both have `/// do not mutate externally` doc comment. Audit claim of "comment not found" is wrong.

**Verification:** `rg "do not mutate externally" chronicler_engine/src/adapters/driven/llm/providers/mock.rs` → 2 hits.

### Task 2: D.2 — Doc comments on persisted-transient fields (1 SP)

**Files:** `src/domain/model/state/narrative_state.rs`

Add `///` doc comment to each of:
- `last_trigger: Option<StoredTriggerContext>` (L16)
- `pending_location: Option<String>` (L18)
- `pending_event: Option<String>` (L20)

Text: `/// Transient pipeline scratch — set during request handling, consumed via `.take()`. Persisted in NarrativeSnapshot for audit/debug only; not relied upon across reloads.`

No code change. `retry_target` already correctly `#[serde(skip)]` — leave alone.

**Verification:** `rg "Transient pipeline scratch" src/domain/model/state/narrative_state.rs` → 3 hits.

### Task 3: D.6 — Module-level doc on phase/status (1 SP)

**Files:** `src/domain/model/state/generation_status.rs`

Replace module doc with conservative wording:

```
/// Generation status and pipeline phase.
///
/// `GenerationPhase` indicates the current pipeline step (narrating /
/// quantifying / generating-event). `GenerationStatus` indicates LLM-call
/// state (idle / generating / error). The two are independent axes; for
/// the live state machine and valid transitions see
/// `application/action_pipeline/pipeline.rs`.
```

No code change. Pure docs. Spot-check `generation_status_tests.rs` for invariant assertions before merge.

**Verification:** `python build.py` (no logic change).

### Task 4: D.3 — Drop unused transport re-export (1 SP)

**Files:** `src/adapters/driven/llm/transport/mod.rs` (1 line delete)

Delete `pub use request::{build_request_payload, configure_request, DEFAULT_MAX_TOKENS};` (L13).

**Verification:**
- `rg "transport::build_request_payload|transport::configure_request|transport::DEFAULT_MAX_TOKENS" chronicler_engine/src/ chronicler_engine/tests/` → 0 hits
- `rg "build_request_payload" chronicler_engine/src/` → only `request.rs:25` def + `client.rs:16,47` (`super::request::`) + `request_tests.rs:1` (direct path)
- `python build.py`

### Task 5: D.7 — Extract `apply_scenario_injection` free fn + collapse 3 → 1 impl (5 SP)

**Files:**
- `src/application/scenario_injection.rs` (new, free fn)
- `src/application/mod.rs` (export new module)
- `src/application/context.rs` (refactor `build_fresh_initial_state` to call free fn)
- `src/bootstrap/state.rs` (delete dead free fn)
- `src/bootstrap/mod.rs` (delete re-export)
- `src/bootstrap/init_game.rs` (refactor `load_game_state` else-branch)
- `tests/integration/` (2 new integration tests)

#### Step 1 — New free fn `src/application/scenario_injection.rs`:

```rust
//! Scenario injection shared between fresh-state init and runtime load.
//!
//! Replaces duplication between:
//!   - `GameServiceContext::build_fresh_initial_state` (`context.rs:44`)
//!   - `bootstrap::state::build_fresh_initial_state` (dead code)
//!   - `bootstrap::init_game::load_game_state` else-branch
//!
//! Note: does NOT call `inject_scenario_logs` — caller composes.
//! Note: `text.is_empty()` gates ONLY the message-add. `init_scenario_npcs`
//! fires for any `Some(scenario)` regardless of text emptiness.

use crate::domain::model::character::PlayerCard;
use crate::domain::model::map::MapDef;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::template::{render_template, TemplateVars};
use crate::domain::model::world::WorldCard;

pub fn apply_scenario_injection(
    state: &mut GameState,
    world: &WorldCard,
    map: &MapDef,
    player: &PlayerCard,
) {
    let Some(scenario) = world.default_scenario() else {
        return;
    };
    let starting_room_id = state.movement.current_room_id();
    let room_name = map
        .get_room_by_id(&starting_room_id)
        .map(|r| r.name.clone())
        .unwrap_or_else(|| starting_room_id);
    state.narrative.pending_location = Some(room_name);

    let text = render_template(&scenario.text, &TemplateVars::new(&player.sheet.name));
    if !text.is_empty() {
        state.add_message(text, None, MessageType::Narration);
    }
    state.init_scenario_npcs(scenario);
}
```

#### Step 2 — Refactor `GameServiceContext::build_fresh_initial_state` (`context.rs:44`)

Replace body (lines 44-72) with:
```rust
pub fn build_fresh_initial_state(&self) -> GameState {
    let starting_room_id = self.world.starting_room_id();
    let mut initial_state = GameState::new(
        Arc::clone(&self.world),
        Arc::clone(&self.map),
        Arc::clone(&self.player),
        (*self.npcs).values().cloned().collect(),
        starting_room_id,
    );
    crate::application::scenario_injection::apply_scenario_injection(
        &mut initial_state,
        &self.world,
        &self.map,
        &self.player,
    );
    initial_state
}
```

#### Step 3 — Delete dead code

- `src/bootstrap/state.rs`: delete `pub fn build_fresh_initial_state` (lines 9-37). File stays open for other fns.
- `src/bootstrap/mod.rs`: delete `pub use state::build_fresh_initial_state;` (L17).

#### Step 4 — Refactor `bootstrap/init_game.rs::load_game_state` else-branch (L52-90)

Replace body with:
```rust
let starting_room_id = world_arc.starting_room_id();
let mut new_state = GameState::new(
    Arc::clone(world_arc),
    Arc::clone(map_arc),
    Arc::clone(player_arc),
    npcs_map.values().cloned().collect(),
    starting_room_id,
);
// Order: scenario injection FIRST, then scenario logs.
// See Issue 2 verification step before merge — swap if scenario.rs
// assumes NPCs are NOT initialized.
crate::application::scenario_injection::apply_scenario_injection(
    &mut new_state,
    world_arc,
    map_arc,
    player_arc,
);
inject_scenario_logs(&mut new_state, world_arc, player_arc);
tracing::info!("No existing snapshot; initialised fresh game state");
let initial_snapshot = GameStateSnapshot::from_game_state(&new_state);
let snapshot_id = storage.save_snapshot(&initial_snapshot)?;
// Per-message persist logic unchanged (L80-91)
```

**Issue 2 verification (implementation step before merge):**
Read `application/scenario.rs::inject_scenario_logs`. Confirm:
- If assumes `init_scenario_npcs` has run → current plan order correct.
- If assumes NPCs NOT initialized → swap to `inject_scenario_logs` THEN `apply_scenario_injection`.

If verification fails, reorder the two calls. Document final order in a comment at the call site.

#### Step 5 — Tests

**Unit tests in `src/application/scenario_injection.rs` (`#[cfg(test)] mod tests`):**
- `no_scenario_leaves_state_untouched` — world without `default_scenario`, assert no messages, no `pending_location`
- `empty_text_does_not_add_message_but_initializes_npcs` — scenario with `text = ""` + non-empty NPCs, assert no message + NPCs initialized

**Integration tests in `tests/integration/`:**
- (a) Fresh-init with non-empty scenario text — startup path → snapshot saved + first message in `messages` + swipe in `swipes` + `pending_location` set
- (b) Fresh-init with empty text + NPCs — startup path → snapshot saved + NO message in `messages` + NPCs initialized

**Test fixture concern:** Existing scenario-aware builder may not exist. Implementation step: scan `test_support/` for one. If none, inline minimal fixture in test module.

#### Step 6 — `arch-lint` check

- `application/scenario_injection.rs` imports from `domain::model::*` only. `application → domain` allowed. ✓
- `bootstrap/init_game.rs` calls `crate::application::scenario_injection` — `bootstrap → application` allowed. ✓

**Verification:**
- `rg "fn build_fresh_initial_state" chronicler_engine/src/` → exactly 1 hit (`context.rs`)
- `rg "fn apply_scenario_injection" chronicler_engine/src/` → exactly 1 hit (`scenario_injection.rs`)
- `python build.py`

## Test Plan

| Task | Tests |
|---|---|
| 1 | None (doc only) |
| 2 | None (doc only) |
| 3 | None (doc only); `python build.py` |
| 4 | None (deletion); `python build.py` |
| 5 | 2 new unit tests in `scenario_injection.rs` + 2 new integration tests |

## Failure modes

| Codepath | Failure | Handled? |
|---|---|---|
| D.6 doc | (n/a, docs only) | ✓ |
| D.7 free fn extraction | Scenario order breaks `inject_scenario_logs` assumption | Issue 2 verification step; integration tests catch drift |
| D.7 fresh-init persistence | DB write fails | Pre-existing `load_game_state` error propagation unchanged |

## Assumptions

1. `GameServiceContext::build_fresh_initial_state` does not read `self.settings` — verified by reading context.rs:44-72.
2. `inject_scenario_logs` is idempotent — verified by reading `application/scenario.rs:23`.
3. `transport::build_request_payload` re-export has no external consumers — verified by grep.
4. `bootstrap::state::build_fresh_initial_state` has no callers — verified by grep.
5. `python build.py` covers all integration tests touching initial state.
6. Scenario-aware test builder exists in `test_support/` or inline minimal fixture is acceptable — implementation step.

## Reversibility

- Tasks 1-4: trivial revert.
- Task 5: revert by restoring deleted `bootstrap/state.rs` fn + `bootstrap/mod.rs` re-export + inlining `load_game_state` else-branch + deleting `application/scenario_injection.rs`.

## NOT in scope

- B1 (fabricated presets claim — leave to audit author)
- C1, C5, C8, C10-C19 (separate plans per audit Fix Status table)
- D.1, D.4 (non-issue)
- D.5 (settings helper — **explicitly dropped per user review 2026-07-06**; not worth the indirection)
- E.1-E.6 (out of scope per audit)
- ADR revisions (separate plan: ADR-027 stale)

## Open items from review

- **Issue 2** (D.7 call order): Implementation step — read `scenario.rs::inject_scenario_logs` to verify final order.
- **Issue 4** (Path A vs Path B equivalence): Resolved — free fn makes both paths literally identical. Audit's "is_some_and text-not-empty" claim was incorrect.
- **Issue 6** (D.6 doc depth): Conservative wording chosen over speculative state-machine table.
