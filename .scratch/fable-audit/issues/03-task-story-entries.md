# The notice for a cut Narration, and the Retry of the opening Narration

Type: task
Status: open
Blocked by: —

## Question

Put two changes in place (owner, 2026-11-10, D8 a and D9 of [the decisions](../decisions.md)). Both change what a story entry is, so both change a spec. Use a `strong-implementer`: each change leaves one design point open.

### 1. A Narration that the model cut at its token limit says so (D8 a)

- **The fault (ran it).** The model answered `…there is one thing you must` with `finish_reason: "length"`. The story log showed the cut text as a finished Narration, and the status read "Ready".
- **Read it.** `parse_chat_response` accepts every answer that has text, and reads `finish_reason` only when the text is empty. The test `test_length_finish_reason_with_content_is_an_answer` pins that, and it stays. Nothing carries the fact to the Job or to the record.
- **The change.** Carry the fact out of the transport in the result of the call. The Narration keeps its text. The story log shows a small line under that Swipe, such as "The model stopped at its token limit."
- **The open design point.** The line must still be there after a reload, and it belongs to one Swipe. So the fact needs a home in the Swipe record. Choose the home and write the reason in `## Answer`. It is not a failure line of the Game: the status stays "Ready" (D3).
- **The failing test first.** An HTTP test with a scripted answer that has text and `finish_reason: "length"`: the story log has the line on that entry, and a second Swipe with a complete answer has none.

### 2. A Retry of the opening Narration keeps the start of the Scenario (D9)

- **The fault (ran it).** A Retry of the opening called the narrator with no authored text, saved the answer as Swipe 2, and analysed it. The state moved from `front_gates` to `entrance_hall`, and the Trigger of Gabriella fired.
- **Read it.** `JobPlan::for_request` maps a Retry of the opening to a plain Narration, and `is_analysed` accepts it (`src/domain/model/generation.rs:86`). No model call writes the opening itself, so nothing analyses it.
- **The decision.** The opening is a Narration like any other, with one guard:
  - Retry stays on the opening.
  - The model gets the authored opening text of the Scenario as its direction.
  - The engine does not analyse the result. No Quantifier call and no Trigger runs. The new Swipe has the starting state of the Scenario.
- **The open design point.** How the authored opening reaches the prompt: as the direction of the call, in the shape that the narrator prompt already has for a direction. Do not add a new prompt block.
- **The failing test first.** A new Game, then a Retry of the opening: the request to the model holds the authored opening text, the new Swipe has the Room and the Characters present of the Scenario, and the story has one Message.

## Rules for every change

- Write the failing test first. Then make the change.
- Line numbers are from commit `e8d6512a`. Find the site again by its name before you edit.
- One change that turns out not to be local stops. Say so in `## Answer`, and do the other changes.
- Use `python build.py clippy` and `python build.py test-pattern` during the work. Run `python build.py` once at the end.
- Do not commit.

## Not in scope

- A cut answer of the Quantifier or of the Options Agent. The report recommends to treat it as a failed attempt. No decision accepted that.
- A message after a hard stop (D8 b).
- The prompt of an event Narration (finding 6 of the engine report).

## What already exists

- Finding 4 of [the engine report](../assets/engine-defects-and-risks.md) and finding 8 of [the story report](../assets/story-damage-and-false-screen.md).
- `docs/specs/swipe_new.md` has the Retry scenarios. The Retry of the opening has no scenario and no test today.
- A closed design note says that the opening "cannot be retried" (`.scratch/_closed/turn-state-model/assets/clean-design-glm.md:362`). D9 replaces that sentence.
