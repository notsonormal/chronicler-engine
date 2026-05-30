# ADR-021: State Patch Reducer for Post-Generation Agent Composition

**Status:** Accepted
**Date:** 2026-05-30

## Context

Multiple post-generation agents can contribute to the same `StatePatch::Scene` without
overwriting each other's work. The previous implementation blindly overwrote fields on
each iteration, causing silent data loss when multiple agents were registered.

## Decision

We add a `StatePatch::merge(self, other: StatePatch) -> StatePatch` method that combines
patches from multiple agents using defined semantics:

### Merge Semantics

| Field | Behavior |
|-------|----------|
| `npc_ids` | Union of unique IDs, preserving first-seen order |
| `movement_destination` | Keep first non-None value; log warning on conflict |
| `confidence` | Take minimum (most conservative): High > Medium > Low |

### Implementation

The `run_post_generation_agents` method in `DefaultGameService` now uses `fold` to
collect patches and reduce them via `merge`, initializing with the quantifier's
existing result as the base:

```rust
let patch = agents
    .filter_map(|agent| agent.execute(&ctx).ok())
    .filter_map(|result| match result {
        AgentResult::StatePatch(p) => Some(p),
        _ => None,
    })
    .fold(StatePatch::Scene {
        npc_ids: result.npcs.npc_ids.clone(),
        movement_destination: result.movement.destination.clone(),
        confidence: result.npcs.confidence.clone().into(),
    }, StatePatch::merge);
```

## Consequences

### Positive
- Multiple post-generation agents can safely compose their outputs
- Merge semantics are explicit and predictable
- Conservative confidence handling prevents overconfident aggregation

### Negative
- Order-dependent behavior for `movement_destination` (first wins)
- Requires `Clone` on `Confidence` and `StatePatch` for the fold operation

## References

- Issue: State patch reducer for post-generation agent composition
- Code: `src/model/agent.rs` (`StatePatch::merge`)
- Tests: `tests/components/state_patch_tests.rs`
