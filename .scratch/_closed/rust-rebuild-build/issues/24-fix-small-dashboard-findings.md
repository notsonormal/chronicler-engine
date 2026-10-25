# Fix the small dashboard findings

Type: task (AFK)
Status: resolved
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

Built and committed on `wf/rebuild-core` after a green full gate (`logs/build_20261020_194617.log`: 189 guardrail, 1340 integration, 29 browser tests pass).

- **F4:** `DebugStateView.step` is `Option<Step>`. It is `Some` only for `Status::Busy`, and `null` for an idle or failed Game. `Step` no longer derives `Default`, so no code path can give a default step. The two status types stay; the fix needed no type decision from ticket 19 or 22. Spec `dashboard.md` 39.6 (a Step only while a Job runs) and 39.7 (a storage failure answers 500). `GET /debug/state` is now in the endpoint list.
- **F5:** the `WorldHasGames` message reads "1 game" or "N games". Spec `worlds.md` 25.10. The route `POST /worlds/:key/delete` is now in the endpoint list, so the "Complete specs" rule adds 25.11 (a delete, and an unknown key) and 25.12 (a storage failure answers 500).
- Two legacy delete tests in `tests/http/requires_migration/worlds_fragment_handlers.rs` are deleted (25.10 and 25.11 cover them); the quarantine pin goes from 74 to 72. The idle `step` check of `requires_migration/debug.rs` is deleted (39.6 covers it).
- Review findings rejected: (1) one shared plural helper for `src/error.rs` and the Worlds template: the two live in different layers, and a shared helper would couple the error type to the template for a one-word rule. (2) Put the Step on the Generating state of the DTO: that is a type change of the status pair, the area of ticket 22 and the small-items fog; the module summary of `generation_status.rs` is still true of the domain enums.

