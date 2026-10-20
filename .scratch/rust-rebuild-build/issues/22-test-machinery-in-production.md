# Decide what test machinery the production build carries

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`testing` is a default Cargo feature, so test machinery ships in the production build. Decide:

1. Whether `testing` stays a default feature (findings 8, 9): `test_support::tx_token()` reachable from any `src/` file, `JobTasks` in production, and the Contradiction state check only under `diagnostics`.
2. The test-only reads in production code: `GameViewQuery::story_messages` (used by the `story_of` test helper), `Narrator::recorder`, the `MockBackend` knobs.
3. One composition root, with the LLM port as the only test input (candidate S4, finding 14); `app_with_production_graph` already exists.
4. `AppState.job_deps.storage` is `pub`, so the HTTP layer can reach `Storage` and two HTTP sites start a Job.

## Context

- Inputs: [synthesis.md](../assets/architecture-review/synthesis.md) row S4; [FINDINGS.md](../assets/whole-rebuild-review/FINDINGS.md) findings 8, 9, 14 and the `AppState` row.
- Skills: `/grilling`, `/domain-modeling`, `/codebase-design` (its design-it-twice pattern for the interfaces). Terms: `CONTEXT.md`. Design: `.scratch/turn-state-model/assets/target-design.md`.

## Answer
