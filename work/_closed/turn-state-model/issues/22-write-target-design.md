# Write the target design of the game-state lifecycle in an open session

Type: grilling
Status: resolved
Blocked by: 21

## Question

Which design of the game-state lifecycle does this effort take as its target? The design covers the states of a generation and what each holds, what binds to a Swipe, what each Retry form runs again, who may write game state and when, and where notices and the location live. It does not depend on a language. It is a reference for fixes in the current code and a possible target for a rewrite (ticket 17, Q3). This grilling is step 5 of the clean-room route. It has no wall.

## Inputs

- The corrected `../assets/clean-spec.md` (version 2) and `../assets/clean-spec-rulings.md` from [Answer the clean agents' questions and close the spec gaps](21-close-clean-spec-gaps.md). Read the "For ticket 22" part of its Answer: R-39 is the central invariant, and a stop undoes saves that a restart keeps. The three designs cite version 1 (`../assets/clean-spec-v1.md`).
- The three clean designs and `../assets/clean-run-gaps.md` from ticket 20.
- **Prior art:** the [SillyTavern asset](../assets/research-13-sillytavern-swipe-state.md) and the [Marinara asset](../assets/research-14-marinara-swipe-state.md). The design states which source each choice follows, if any.
- **Checklist:** the 36 known problems in the [research 16 asset](../assets/research-16-clean-room-collapse.md) §2, and row W.
- The facts of the current code in the research assets 04–12. The session may read them. It copies no code shape without a reason.
- The owner's lessons about building an AI-native project. The fog of the [platform choice](../../platform-choice/map.md) map lists what that question covers.

## Context

- **Skills:** `/grilling`, `/domain-modeling` and `/codebase-design`. Use `/prototype` if a state model needs a concrete artifact to react to.
- **Terms:** record new or changed terms in the design. `CONTEXT.md` changes only when the plan after this map takes them up, because `CONTEXT.md` describes the current engine.

## Output

`../assets/target-design.md`. It holds no code and no language types. Each choice names its source: a spec requirement, a clean draft, a premise source, an owner ruling or a lesson. A table records how the design treats each of the 36 problems: settled, moot, or open.

## Done when

- The design exists, and the owner has accepted it.
- The table of the 36 problems is complete.

## Answer

**The target design exists, and the owner accepted it** (round 4, Q2): [`target-design.md`](../assets/target-design.md). It holds no code and no language types. Its decision log has 26 choices (D-01 to D-26), and each one names its source. Its §10 table treats each of the 36 known problems of research 16 §2: 17 settled, 4 moot and 15 open. Of the open rows, 12 are outside the lifecycle, and 3 are code shape that the owner left to the plan. Row W is settled.

**The core of the design.**

- **The game state is on the Swipe.** Every Swipe holds a full Swipe state. The current state, the start state of a Retry, the status and whether Retrigger is allowed are derived, not stored (D-01, D-02). A swipe switch, a delete, a failed Retry and a stop write no game state.
- **The text shows at once.** A generation saves its text with the pre-analysis state. The analysis then updates the same Swipe. A stop or a restart before the update keeps the text in the pre-analysis state (D-04, OR-53). Only the work that made a Swipe writes its Swipe state (D-03).
- **The status.** The busy mark, the step name and the cancel handle are in memory only. The failure line and the dock record (the offered set and one notice field) are saved on the Game (D-05, D-06). So no start-up pass and no "heal" step exist.
- **One writer at a time.** A request reads the active-Game pointer once, and every write names that Game (D-09). A per-Game guard is held for one check and one save unit, never across an LLM call (D-07). Each save unit of a work checks the work id, so a stop or a reset refuses the work's later saves (D-08).
- **The plans and the steps.** The order of steps of each request and the six steps of a generation are behaviour. The code shape is the plan's choice (D-16).

**The owner's answers** (four rounds):

| Question | Answer |
|---|---|
| R1 Q1: where the current state lives | Derived from the Swipe (recommended). |
| R1 Q2, R2 Q2: what a stop undoes | Only the step that runs. The owner: "the generation and the input are two separate operations." Round 1's undo list in memory was dropped. OR-51 and OR-52. |
| R2 Q3: where the status lives | The busy mark in memory; the failure line saved (recommended). |
| R2 Q4: the code shape of the steps | Not decided here (Option C). Against the recommendation of one step runner. |
| R2 Q5: the terms | Swipe state, start state, current state (recommended). |
| R3 Q1: when the text shows | None of the three options. The text shows at once, and the analysis updates it. A stop before the analysis keeps the pre-analysis state. OR-53. |
| R3 Q2: the failure lines | Accepted. |
| R4 Q1: which generations follow OR-53 | Every analysed generation (recommended). |
| R4 Q2: the design | Accepted. |

**The free points of the spec** (§7): the analysis of an event Narration has its movement taken out before the world rules, as all three clean designs do (D-12). Each LLM call reads as much of the story as today (D-17). The failure lines are in §4.7 (D-18). The status, the busy mark and the records are D-05 and D-06.

### `clean-spec.md` version 3

- **New:** R-11a, the text shows at once and the state follows (OR-53).
- **Changed:** R-81, the Stop term and the stop row of §2 (OR-51, OR-52). The text of R-81 in version 2 is the text of OR-31 and OR-48.
- **Rulings:** OR-51 to OR-53, in a new ticket-22 section of [`clean-spec-rulings.md`](../assets/clean-spec-rulings.md). OR-51 replaces the undo part and the examples of OR-31. OR-52 withdraws OR-48.
- **Checks.** The leak check finds 0 identifier hits and 0 `src/` paths in version 3 (tested). No independent reviewer read version 3 or the design.

### For ticket 23

- Each D-row of the decision log is one element to compare.
- Differences that the comparison will meet (inferred from research 04, 05 and 12): Snapshot pointers in place of Swipe states; the status inside every Snapshot (F-6); one process-wide Game id (research 05 §0); arrival (S-3); `System` notices in the story; a pointer that is set once before the Quantifier and never moves (S-1), where the design updates the Swipe after the analysis; `/status/reset-generating` frees the slot and lets the work go on.
- Design §8 has one spec observation: the opening can never be deleted under R-76.
- The failure lines: three lines change their words, and five are new. `tests/http/failure_display.rs` and `tests/browser/stub/announcements.rs` contain today's words (known). The plan changes them.

### For the plan

- New terms: Swipe state, start state, current state, pre-analysis state, work entry, dock record, event record. `CONTEXT.md` takes them up when the code changes. Its Snapshot entry describes today's code. Ticket 21's Answer lists the other glossary changes.
- Tickets 01–03: the design is a candidate model for [Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md) (T-01 settled). The dock record answers [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md) (T-02). The room report on the Swipe answers [Decide where the room a generation arrives in lives](03-decide-where-the-room-lives.md) (T-03). Ticket 23 recommends for each one: re-state it, close it, or rule it out of scope.
