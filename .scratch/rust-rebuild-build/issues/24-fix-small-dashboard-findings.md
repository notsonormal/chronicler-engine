# Fix the small dashboard findings

Type: task (AFK)
Status: open
Blocked by: —

## Question

Fix UI findings F4 and F5, each with a spec scenario and a tier-1 test:

- **F4:** `/debug/state` reports `step: "Narrating"` for an idle or failed Game (two status types: `view_query::Status` and `GenerationStatus`).
- **F5:** the refusal of a World delete reads "Cannot delete world with 1 games".

Commit after a green commit gate (plan §2), under the standing approval in the map's Notes.

## Context

- Inputs: [the UI findings](../assets/ui-review/findings.md) F4, F5 (steps, shots).
- If the fix of F4 needs a type decision that ticket 19 or 22 owns, stop and record it there.

## Answer
