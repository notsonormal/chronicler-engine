# ADR-033: Dissolve `domain/engine/` Subfolder

**Date:** 2026-07-27
**Status:** Accepted
**Supersedes:** `### \`domain/engine/\` Subfolder Kept` block in [ADR-027](./adr-027-hexagonal-architecture-migration.md) (lines 88–90)

## Context

ADR-027 recorded the decision to keep `src/domain/engine/` as a 7-file subfolder holding "types (`model/`) vs rules (`engine`)" at zero cost. The subfolder was aspirationally a rules module, but its 11 free fns were all parked methods — every one of them had `GameState` or `Room` as its single behavioral owner, with `map`/`npcs` as read-only context.

That made `domain/engine/` a misc-folder in practice: a category-less dumping ground for "engine stuff" that belonged on owning types. The free-fn scanner effort (prior map `.scratch/free-fn-scanner-rules/`, ticket 06) relocated all 11 free fns to their owning types and deleted the folder.

ADR-027's `### \`domain/engine/\` Subfolder Kept` block is now factually stale — the folder does not exist. `grep -rn "domain::engine\|domain/engine" src/` returns zero hits and the directory is absent from the tree.

A separate staleness exists on ADR-027's lines 72–74: the 3 grandfathered `*/gate.rs` paths (`persistence_gate/gate.rs`, `game_catalogue/gate.rs`, `world_catalogue/gate.rs`) were flattened by this effort's ticket 03 into single-file modules (`persistence_gate.rs`, `game_catalogue.rs`, `world_catalogue.rs`). That is pure path drift — no decision change — recorded here so ADR-027's grandfathered list and `tests/infrastructure/guardrails/layers.rs` agree.

## Decision

### Dissolve `domain/engine/`; do not replace it

`src/domain/engine/` is dissolved. No generic "rules" subfolder replaces it. Pure-rule functions live as `impl` methods on their owning domain type, in the type's module file or a sibling `_*_tests.rs` block.

The 11 relocations from prior ticket 06:

- `Action` enum + `parse_action` → `src/domain/model/action.rs`, collapsed to `impl Action { pub fn parse(input: &str) -> Self }`.
- 7 fns from `action_processing.rs` (movement, NPC encounters, trigger narration, freeaction impl) → `impl GameState` in `src/domain/model/state/game_state.rs`.
- `attempt_semantic_walk` → `impl GameState` (same file).
- `create_dynamic_room` → `impl Room::new_dynamic` in `src/domain/model/map.rs`.
- `assert_state_consistency` + private helpers → `impl GameState` in `game_state.rs`, kept behind `#[cfg(feature = "diagnostics")]`.
- `evaluate_triggers` → `impl GameState::evaluate_triggers` (GameState owns the state transitions; `NpcCard` is read-only context).
- `NpcEncounterLog::check_condition` → `src/domain/model/trigger.rs`.

`domain/mod.rs` carries `pub mod model;` only — no `engine` module.

### Why dissolution over retention

- Chose dissolution because the subfolder had no cohesive category — every resident had a single behavioral owner elsewhere. Keeping it would preserve a misc-folder whose ongoing role is "things that don't fit" — the anti-pattern that motivated the free-fn doctrine.
- Rejected "keep, just rename" because the rename would still be a misc-folder; the correct home for each resident was its owning type, not a renamed bucket.
- Rejected "flatten into `domain/` root" because the residents were not top-level domain concepts — they belonged to existing types in `domain/model/`. A flat `domain/*.rs` would orphan them from their owners.

### Path correction for ADR-027's grandfathered `gate.rs` list

ADR-027 lines 72–74 list 3 grandfathered `*/gate.rs` paths. Ticket 03 of this effort flattened each `<name>/{mod.rs,gate.rs}` pair into a single-file module at the parent level. The corrected paths are:

- `src/application/persistence_gate.rs` (was `persistence_gate/gate.rs`)
- `src/application/game_catalogue.rs` (was `game_catalogue/gate.rs`)
- `src/application/world_catalogue.rs` (was `world_catalogue/gate.rs`)

`tests/infrastructure/guardrails/layers.rs` already carries the corrected paths. Pure path drift — the 3 files remain on the storage-direct grandfathered list, only the file shape changed.

### Free-fn doctrine implication

The free-fn allowlist in `tests/infrastructure/guardrails/free_fn.rs` has no entry for `domain/engine/` and never will — the folder does not exist. No category-folder rule applies to `domain/model/` subdirectories; free fns there remain subject to the standard scanner triage (genuine free-fn category vs parked method).

## Consequences

### Positive

- ✅ ADR-027 no longer documents a folder that does not exist. `domain/model/` is the only domain subfolder; "types vs rules" communicates through `impl` methods on owning types, not through folder partitioning.
- ✅ Free-fn doctrine has no `engine/` carve-out — the misc-folder pattern that motivated this ADR is structurally prevented from recurring in `domain/`.
- ✅ ADR-027's grandfathered-path list now agrees with `tests/infrastructure/guardrails/layers.rs` (both reflect post-flatten paths).

### Negative

- ⚠️ `GameState` method count grew (5 → ~15). Large `impl` blocks are the tradeoff of behavioral cohesion over file-size orthodoxy — acceptable because the methods are the GameState's behavior, not a separate "engine" concern.

### Trade-offs

- Chose `impl GameState` for `evaluate_triggers` over `impl NpcCard` because GameState owns the state transitions (room position, encounter log); NpcCard is read-only context. The behavioral owner wins over the data-flavour match.
- Chose single-file modules (`persistence_gate.rs` etc.) over the `folder + mod.rs + gate.rs` pattern because the folder collapse (ticket 03) satisfied `guardrails_mod_purity` with fewer files; the gate concept survives in the single-file module's name.

## Related ADRs

- [ADR-027: Hexagonal Architecture Migration](./adr-027-hexagonal-architecture-migration.md) — superseded `domain/engine/ Subfolder Kept` block; corrected `gate.rs` paths in the grandfathered list.
- [ADR-028: Test Module Header Convention](./adr-028-test-module-header-convention.md) — test siblings moved alongside their relocated `impl` blocks per this convention.

## History

- **2026-07-27**: Initial decision. Records prior effort ticket 06's dissolution + this effort's ticket 03 path correction in one superseding ADR.
