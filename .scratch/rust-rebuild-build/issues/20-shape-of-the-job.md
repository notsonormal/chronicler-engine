# Decide the shape of the Job: one run loop, its failure lines and the narrator's prompt rules

Type: grilling (HITL)
Status: open
Blocked by: 19

## Question

Given the shape of a generation (ticket 19), what is the shape of the Job and of the narrator behind it? Decide:

1. **`Job::run` as one loop** (finding 6): one fact in place of the three booleans, and whether the Options run follows a `Stopped::SaveFailed`.
2. **The failure lines** (finding 2): what replaces the blanket `impl From<EngineError> for Failure`, so one fault shows one line whether `accept` or the Job reads it, and a read inside `JobWriter` does not show "The new text could not be saved."
3. **The error class of the new storage code**: the 34 rusqlite errors wrapped by hand as `EngineError::Config`, which `GenerationFailure::from_engine_error` cannot classify.
4. **The narration prompt rules behind the narrator** (candidates M3, S5, finding 19): `continuation_user_message` and the hook-input assembly move out of the Job; the `agent-edge` rule (the narrator gets values, not storage) stays.

## Context

- Inputs: [synthesis.md](../assets/architecture-review/synthesis.md) rows M3, S5; [FINDINGS.md](../assets/whole-rebuild-review/FINDINGS.md) findings 2, 6, 19; thermo M7 in `../assets/whole-rebuild-review/findings/review-thermo.md`.
- Skills: `/grilling`, `/domain-modeling`, `/codebase-design` (its design-it-twice pattern for the interfaces). Terms: `CONTEXT.md`. Design: `.scratch/turn-state-model/assets/target-design.md`.

## Answer
