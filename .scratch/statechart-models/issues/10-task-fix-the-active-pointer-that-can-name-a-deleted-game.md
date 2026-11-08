# Fix the active pointer that can name a deleted Game

Type: task
Status: open
Blocked by: —

## Question

Close the three orders that leave the active pointer (`ActiveGame`) on a Game that no longer exists. Decide the design, change the code, bring the TLA+ model along, and add a test where a seam can force an order.

The harm is low (owner, ticket 09, Q4). But ticket 14 waits for the model fix of this ticket, because the diagrams must show a model that follows the code.

## The three orders (known: TLC on a scratch variant of the model)

1. A delete of Game 2 passes its check, a switch moves the pointer to Game 2, and the delete removes Game 2. Run 1 found this order (`DeleteNotActive`).
2. A switch reads Game 2, a delete of Game 2 passes its check and removes it, and the switch sets the pointer to Game 2.
3. A reset creates Game 3 in its hold. A delete of Game 3 passes its check, because the pointer still names the old Game, and removes it. Then `ActiveGame::set_new` moves the pointer to Game 3.

Detail: the Comments of [ticket 09](09-grilling-decide-the-gaps-and-the-verify-skill.md). The scratch variant (a switch in two steps, the rule `ActiveExists`) and its configs: [assets/09-switch-in-two-steps/](../assets/09-switch-in-two-steps/).

## Context

- **Two locks (known).** `ActiveGame` has its own mutex. `BusyRegistry::hold` locks only the `busy` mutex. `GameCatalogue::switch_game` calls `storage.get_game` and then `ActiveGame::set`, outside every hold. `GameCatalogue::delete_game` reads `ActiveGame::get` outside the hold of `GameWriter::delete`. `GameCatalogue::reset` calls `GameWriter::replace` and then `ActiveGame::set_new`.
- **A silent set (known).** `ActiveGame::set` calls `Storage::touch_game_updated_at`, an `UPDATE` that answers `Ok` when no row matches.
- **The harm (inferred).** Each later request names a Game that does not exist, until the person switches Games. A restart repairs the pointer (`resolve_game_id` picks the newest Game of the World). Only a thread preemption reaches each order.
- **The fix mutant of run 1 closes order 1 only.** A check inside the hold of the delete is not enough.
- **A likely design (guessed).** One lock covers the pointer and the Game rows: for example, the pointer check and the pointer set happen inside a hold of the registry. This ticket decides. If the fix needs a refactor beyond `ActiveGame`, `GameCatalogue` and `GameWriter`, stop and report it (`AGENTS.md`).

## The work

- The code fix.
- The TLA+ model: the switch in two steps, the rule `ActiveExists` (games.md 18.2 and 19.2), and the fix. `delete-race` then passes. Update `coverage.toml` (the new sync commit), the mutants and the *Known findings* of `SKILL.md`.
- A tier-1 test only if a seam can force an order. Today `ActiveGame` and `GameCatalogue` have no pause seam. Do not add a seam only for this test without a reason in the Answer.
- One row in [the trial log](../assets/trial-log.md).

## Checks

- `python .agents/skills/verify-gate/check.py` passes, the mutants included.
- `python build.py` passes.
- Do not commit.
