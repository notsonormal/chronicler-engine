# R6: the preset module

Type: task (AFK)
Status: resolved
Blocked by: 33

## Question

Build §8 of [the build plan of the response](../../../../docs/plans/rust-rebuild-response-build-plan.md) on `wf/rebuild-response`. Read §1, §2 and §8 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §8 pass. Each commit passed the commit gate, under the standing approval in the map's Notes. A gap that needs a decision stops the slice and goes to the owner as a new ticket.

## Answer

Built on `wf/rebuild-response` in one commit, after a green full gate (`logs/build_20261021_031110.log`: 189 guardrail, 1347 integration, 29 browser tests pass).

- `PromptPresetService` is the preset module. It gives the preset of a Game for the narrator (`for_narration`) and for each hook (`hook_text`); the Job reads the Game row and passes it in, so the module reads no Game. It holds the edit and delete rules; the handlers only parse forms and call it.
- A Game-row read failure ends the Job: `active_quantifier_preset_id`, `active_options_preset_id` and `preset_id_or` are gone, with their registry fallback (finding 15). Scenario 38.13, test `preset_game_read_fails`; until R7 the line is the generic missing-data line.
- A preset delete is refused while a Game uses it: "Cannot delete preset used by N game(s)" (400), scenario 21.40. `EngineError::count_of_games` is the one plural rule for this refusal and `WorldHasGames`. A failed delete answers 500 (scenario 21.41).
- The Job's missing-preset policy is unchanged (G3 is R7's).
- Known limit, accepted (lead): the Game count and the delete are two storage calls, so a Game could take the preset between them. The server has one player, and after R7 a missing preset shows the `Record::Preset` line. A one-statement `DELETE … WHERE NOT EXISTS` is the fix if it matters later.
- A failed preset read in an edit or a delete now shows `Update failed:` or `Delete failed:` (was `Load failed:` for a delete).
