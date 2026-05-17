# Plan: State Patch Reducer for Post-Generation Agent Composition

## Problem

`run_post_generation_agents` (in `src/application/game_service/actions.rs`) blindly overwrites the `QuantifierResult` on every agent iteration:

```rust
for agent in service.agent_registry.agents_for_phase(ExecutionPhase::PostGeneration) {
    match agent.execute(&agent_ctx) {
        Ok(AgentResult::StatePatch(StatePatch::Scene { npc_ids, movement_destination, confidence })) => {
            result.npcs.npc_ids = npc_ids;           // overwrites previous agent
            result.movement.destination = movement_destination; // overwrites previous agent
            result.npcs.confidence = QuantifierConfidence::from(confidence); // overwrites previous agent
        }
        ...
    }
}
```

If two post-generation agents are registered (e.g. Quantifier + a future Continuity Checker), the last agent wins. Lists like `npc_ids` are not accumulated, and contradictory `movement_destination` changes are silently discarded.

## Goal

Implement a reducer or conflict-resolution strategy so that multiple post-generation agents can safely contribute to the same `StatePatch::Scene` without overwriting each other.

## Approaches

### Approach A: Minimal Inline Reducer (Smallest Change)

Keep the bridge function but add a small inline accumulation loop.

- Collect `npc_ids` into a `HashSet` to build a union, then convert back to `Vec` preserving insertion order.
- Track the first non-None `movement_destination`; if a later agent returns a different one, log a warning.
- Track the minimum `confidence` seen.
- Apply the merged values to `result` once after the loop.

**Pros:**
- Touches only `actions.rs` (~20 lines changed).
- Zero new public types or API surface.
- Lowest risk of breaking existing tests.

**Cons:**
- Merge logic is hidden inside a temporary bridge function (the overarching spec already plans to delete this bridge in Phase 3).
- Not reusable if new `StatePatch` variants are added later.
- Harder to unit-test in isolation.

---

### Approach B: `StatePatch::merge` Method (Recommended)

Add a first-class merge method to `StatePatch` and refactor the bridge to use it.

- Add `StatePatch::merge(self, other: StatePatch) -> StatePatch` in `src/model/agent.rs`.
- Define documented merge semantics:
  - `npc_ids`: union of unique IDs, preserving first-seen order.
  - `movement_destination`: if all agree, use that value; on contradiction log a `warn!` and keep the first non-None.
  - `confidence`: take the minimum (most conservative) across all contributors.
- Refactor `run_post_generation_agents` to collect patches, reduce them, and apply the result once.
- Add focused unit tests for the merge method.

**Pros:**
- Clean, testable, and documents the composition contract explicitly.
- Survives the Phase 3 bridge removal — `StatePatch::merge` stays useful when agents mutate state directly.
- Easy to extend when new `StatePatch` variants are added.

**Cons:**
- Slightly more files touched (`agent.rs`, `actions.rs`, tests, ADR).
- Still leaves the `QuantifierResult` bridge in place (does not complete Phase 3).

---

### Approach C: Full Phase-3 Pipeline Rewrite (Largest Scope)

Replace the temporary bridge with a formal `AgentPipeline` as originally planned in the overarching spec.

- Create `src/engine/agent_pipeline.rs` with an `AgentPipeline` struct.
- Remove `QuantifierResult` from `action_processing.rs`; have the pipeline apply `StatePatch` values directly to `GameState` (or a turn-scratch state).
- Wire `pre_generate()` into the action flow (currently `NarratorAgent` exists but is not invoked).
- Refactor `actions.rs` to delegate orchestration to the pipeline.

**Pros:**
- Solves the overwrite bug at the root cause (the bridge is the problem).
- Aligns with the existing architectural roadmap (overarching spec Phase 3).
- Enables pre-generation agents and future extensibility immediately.

**Cons:**
- Much larger scope: touches `action_processing.rs`, `actions.rs`, `service.rs`, and all callers.
- Risk of breaking existing integration tests that depend on `QuantifierResult`.
- Effectively subsumes the issue into "finish Phase 3" rather than fixing a focused bug.

## Suggested Fix

**Approach B** is recommended. It is the sweet spot: it solves the composition bug properly with a reusable reducer, but it does not expand scope into a full pipeline rewrite that the overarching spec already tracks separately.

## Files to Change (Approach B)

1. **`src/model/agent.rs`**
   - Add `StatePatch::merge(self, other: StatePatch) -> StatePatch`.
   - Add helper for conservative `Confidence` aggregation.

2. **`src/application/game_service/actions.rs`**
   - Refactor `run_post_generation_agents` to collect patches, reduce via `merge`, apply once.

3. **`src/narrative/agents/registry_tests.rs`** (or new module)
   - Unit tests for disjoint npc_ids → union.
   - Overlapping npc_ids → deduplicated union.
   - Same movement → accepted.
   - Conflicting movement → warning, first wins.
   - Different confidences → minimum.
   - `NoOp` / errored agents are neutral.

4. **`docs/adr/adr-009-agent-trait-registry.md`**
   - Document merge semantics under Consequences.

## Verification

- `cd chronicler_engine && python build.py` passes (fmt + clippy + tests + coverage).
- New unit tests for patch reduction pass.
- Existing game_service integration tests pass (single-quantifier baseline is unchanged).

## Out of Scope

- Replacing the `QuantifierResult` bridge entirely (tracked in overarching spec Phase 3).
- Adding new agent types beyond the existing quantifier.
- Changing `action_processing.rs` signatures.
- Pre-generation agent composition.
