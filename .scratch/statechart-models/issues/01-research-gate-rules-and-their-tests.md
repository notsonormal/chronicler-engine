# Research: which rule sentences of the gate the tests already pin

Type: research
Status: resolved
Blocked by: —

## Question

Which rules of the gate does a test already pin, and which rules have no test?

The gate here is the busy state and its holder, the writer rule, the job id check, stop, reset, delete, and the Autoplay holder. The rules with no test are the candidate invariants of the trial TLA+ model in [Prototype: one TLA+ model of the gate, built from the spec only](03-prototype-spec-only-gate-model.md).

Produce a table with one row for each rule sentence:

| Rule sentence (quoted) | Source (doc and section) | Test that pins it (file and test name, or SCENARIO / CASE tag) | Pinned, partly pinned, or not pinned | Why (one line) |

Then give:

1. The list of rules that no test pins, or that a test pins for one order of events only.
2. A count: rules in total, pinned, partly pinned, not pinned.
3. One line on the result for ticket 03: is there enough unpinned ground to justify the trial?

## Context

- **The rule sentences.** `docs/diataxis/reference/game_flow.md`: *The Job*, *The Busy State and Its Holder*, *The Autoplay Loop*, *The Writer Rule*, *Stop and Reset*, and *The State of a Game*. Also the gate scenarios in `docs/specs/` (for example `actions.md`, `dashboard.md`, `reset.md`, `story_log.md`, `swipe_switch.md`, `games.md`, `autoplay.md`, `narrator_mode.md`).
- **The tests (known).** `tests/http/races.rs` holds the race scenarios: an Action while a Job runs (actions 3.4), two Actions at the same moment (3.5), `/options` while a Job runs (options 24.21), a stopped Job writes nothing (dashboard 39.5), a stop at each model call (39.4), reset while a Job runs (reset 7.3), edit and delete while busy (story_log 8.7), swipe switch while busy (swipe_switch 36.3), a Job on a non-active Game (games 18.4). Other tier-1 files and the gate's own tests may pin more. `tests/AGENTS.md` lists the test files.
- **Prior rulings.** [Research: race tests for the guard and the job id check](../../_closed/platform-choice/issues/06-research-race-tests-for-guard.md) listed six interleavings; [Decide the race tests and guardrail rule 18](../../_closed/platform-choice/issues/12-decide-race-tests-and-rule-18.md) placed them. Check that each one is in the code today.
- **"Partly pinned"** means the test checks the rule for one order of events, and the rule claims it for every order. Name the orders the test does not cover.
- Read the tests, not only their names. A test whose name matches a rule may assert something else.

## Answer

Asset: [The gate rules and the tests that pin them](../assets/01-gate-rules-and-their-tests.md) (read-only research; no build and no test run).

**Counts.** 39 rule sentences of the gate: 20 pinned, 16 partly pinned, 3 not pinned.

**No test at all.**
- "`GameWriter` owns the busy-state registry. The registry maps each busy Game to its holder."
- "No saved record holds the holder, the step or the progress."
- "A restart loses every holder, so every Game is idle after a start." The browser stub tests restart a stub server, not the engine with a running Job (coordinator check).

**Partly pinned, with the orders no test covers.**
- A delete of the Game while a plain Job runs, and the late save of that Job. Only the delete during an Autoplay run is tested (autoplay 41.16). `tests/http/games_delete.rs` holds no Job (coordinator check).
- A Narrator Mode write and a posture write while a plain Job runs. Only the preset write is tested (actions 6.4).
- The busy check before another refusal: an edit, a delete or a swipe switch with a body that another check would also refuse.
- A stop on a Game with a failure line. Only the clear of the Options failure is tested (options 24.5, 24.18).
- The accept's read and its count in one hold; the reset's two writes in one hold (no failure injected into either).
- The job id check at the Input save, and after a delete of the Job's Game.
- The Swipe state rules: the restart half of "a stop or a restart between the two writes"; the delete half of "change which Swipe state is current"; that an edit and a delete write no Swipe state; that an edit changes the text only.

**Platform-choice interleavings.** I1 to I6 are all present today. The Retry 12.1 and Retrigger 15.1 tests hold a real Job, not `try_claim`.

**Verdict for the trial.** Yes: 19 of 39 sentences are partly or not pinned, and the gaps are the gate's own orders. Caution (inferred): the busy rule makes many of these orders unreachable through the routes, so a counterexample is a candidate tier-1 test only where a route can reach it. The three unpinned sentences are ownership and restart claims; a TLA+ model can state them, but only a restart test or a storage read can pin them.
