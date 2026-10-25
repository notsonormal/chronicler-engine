# Decide what test machinery the production build carries

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

`testing` is a default Cargo feature, so test machinery ships in the production build. Decide:

1. Whether `testing` stays a default feature (findings 8, 9): `test_support::tx_token()` reachable from any `src/` file, `JobTasks` in production, and the Contradiction state check only under `diagnostics`.
2. The test-only reads in production code: `GameViewQuery::story_messages` (used by the `story_of` test helper), `Narrator::recorder`, the `MockBackend` knobs.
3. One composition root, with the LLM port as the only test input (candidate S4, finding 14); `app_with_production_graph` already exists.
4. `AppState.job_deps.storage` is `pub`, so the HTTP layer can reach `Storage` and two HTTP sites start a Job.

## Context

- Inputs: [synthesis.md](../assets/architecture-review/synthesis.md) row S4; [FINDINGS.md](../assets/whole-rebuild-review/FINDINGS.md) findings 8, 9, 14 and the `AppState` row.
- Skills: `/grilling`, `/domain-modeling`, `/codebase-design` (its design-it-twice pattern for the interfaces). Terms: `CONTEXT.md`. Design: `.scratch/_closed/turn-state-model/assets/target-design.md`.

## Answer

Resolved 2026-10-20 with the owner. **No code change.** The owner's premise: for this application it does not matter much that test machinery ships in the production build.

1. **`testing` stays a default Cargo feature** (findings 8 and 9 closed on this point). `tx_token()`, `JobTasks` and the test-only constructors keep shipping.
2. **Test-only reads stay as they are:** `GameViewQuery::story_messages`, `Narrator::recorder` and the `MockBackend` knobs.
3. **Candidate S4 is closed.** The two graphs (`build_app_graph`, `build_app_graph_for_tests`) stay. Fact recorded: tier 1 has two test inputs today, the LLM port and the storage failure layer (`Storage::with_failure`, 9 files in `tests/http/`).
4. **Finding 14 is closed.** `AppState.job_deps` stays `pub`; two HTTP handlers that start a Job are acceptable.
5. **Finding 9 (a Job panic is invisible to tier 1) is closed.** The lints in `src/lib.rs` deny `unwrap_used`, `expect_used` and `panic`, and `ScriptedModel` answers an unscripted call with an error and panics only on drop, on the test thread. Remaining gap (inferred, accepted): implicit panics (index out of bounds, string slicing, overflow in debug, `unreachable!`, `assert!`, a dependency) still end a Job silently.
6. **`diagnostics` stays (option C).** It selects `assert_state_consistency` in `game_rules.rs` (room exists, scene Characters and encounter log in the World). Because `testing` is a default feature, the check runs in every build, so production is strict. **Recorded mismatch:** `docs/plans/subplan-c-mapless-enablement.md:458` says production runs without the check and self-heals; that text describes the intent, not the build. Inferred risk, not checked: a World edit that removes a Character named in a Game's encounter log would fail each analysis of that Game.

Nothing from this ticket goes into the build plan of the response.
