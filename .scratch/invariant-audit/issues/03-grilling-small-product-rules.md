# Decide the small product rules

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Five findings depend on a product rule that only the owner sets. What is the rule for each one?

1. **A config change during a plain Job (gate-06).** `save_config` refuses only an Autoplay holder. A change of mode, posture or preset during a plain Job is accepted. The Job reads the Game row again for each step, so the narration and the Quantifier can use different presets.
   Choices: refuse the write while any holder holds the Game; have the Job read its config once at the start and keep it; or accept the current rule and state the reason.
2. **The Games list order (gate-08).** A comment in `changes.rs:180` says that the list follows the newest Message. The SQL orders by `updated_at`, and no Message write touches that column.
   Choices: order the list by the newest Message (the code changes), or by the last change of the Game row (the comment changes). Read `docs/specs/games.md` first: it may already decide this.
3. **An empty Autoplay budget (http-07).** `Budget::parse` returns `Budget::DEFAULT` (10 Cycles) for an empty field, on purpose (`src/application/autoplay/budget.rs:16-17`). The coordinator thinks that this is a product choice, not a gap.
   Choices: accept (state the reason), or refuse an empty field.
4. **An empty Message text (llm-04).** The History editor saves an empty text, and some providers then refuse every later generation of the Game.
   Choices: refuse the save with a message, or treat an empty save as a delete of the Message. Does text with only spaces count as empty?
5. **A Job that panics (gate-05).** A panic releases the hold, but writes no failure line. The accept already cleared the old line, so the status says Ready and the player's Action is gone with no word.
   Choice: what the player sees after a panic: the text of the failure line, and whether it names the lost Action.

Rules 1 to 3 graduate their own task tickets. Rules 4 and 5 unblock [Make an empty Message text follow ticket 03](09-task-empty-message-text.md) and [Write a failure line when a Job panics](10-task-failure-line-after-a-panic.md).

## Context

- Findings in full: gate-05, gate-06 and gate-08 in [the gate report](../assets/gate.md), http-07 in [the http report](../assets/http.md), llm-04 in [the llm report](../assets/llm.md). The coordinator confirmed gate-06 (`src/application/generation/gate.rs:501-508`), http-07 and llm-04 (`gate.rs:560-571`) in the code.
- Rule 1 changes a gate rule. The spec `docs/specs/autoplay.md` 41.12 refuses the write only for an Autoplay run. A change runs `/verify-gate` (map Notes) and touches the open diagram and `game_flow.md` tickets of [statechart-models](../../statechart-models/map.md).
- Use `/grilling`. Five rules fit one round. Read the findings in the code again first (map Notes).

## Answer
