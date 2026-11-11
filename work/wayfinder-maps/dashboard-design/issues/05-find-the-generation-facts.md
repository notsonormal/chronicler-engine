# Find what the engine offers for a visible generation

Type: research
Status: open
Blocked by: none

## Question

What would a pending row at the end of the story log need, and what does the Stop of a single Job need on the screen?

## Context

- The critique's P1 issue: a generation shows only as a 12px label in the bottom-right corner, and the story log does not change until the Narration lands. A single Job has no Stop; an Autoplay run has one ([the Game tab critique](../assets/game-tab-critique/report.md)).
- Known from the plan review (map, "What already exists"): the engine can stop a single Job (`GameWriter::stop`, Scenario 39.4 in `docs/specs/dashboard.md`), and only `autoplay_controls_answer` hides the Stop. The status words are in `step_label` and `busy_answer`. `GameKind::BusyState` is a change kind.
- Find, from the code, the specs and the tests (local sources only):
  - Whether each step change of a Job raises `BusyState`, so that a screen load shows the new step (inferred, not checked: `entry.step = step` in `src/application/generation/gate.rs`).
  - Whether the story-log part (`render_story_log_part`) can read the busy state, so that it renders a pending row at the end of `#story-log`, within the rules of `docs/diataxis/reference/frontend/dashboard.md`. What the morph does with a row that is there in one load and gone in the next.
  - What a Stop of a single Job must change on the screen: `#stop-btn` in the Job state, the words of the `/status/reset-busy` route in `answer_words.rs`, and the scenarios of `docs/specs/autoplay.md` and `browser_autoplay.md` that name Stop as an Autoplay control.
  - Which specs a change touches (`docs/specs/story_log.md`, `dashboard.md`, `autoplay.md`, `failure_display.md`, the `browser_*.md` specs) and which browser tests read the status selectors.
- Label each finding known, inferred or guessed.

Done when the answer is in a linked asset (`../assets/generation-facts.md`) with source paths, and the answer names the options and their cost for the pending row and for the Stop of a single Job on the screen.
