# Share the retry of the Agent calls and keep the cause of a refused answer

Type: task (AFK)
Status: open
Blocked by: 17

## Question

Build two fixes that [Build the scene entries](17-build-scene-entries.md) left open, test first:

1. **One retry frame.** The Agents that try a failed model call a second time share one frame. They stop each holding their own loop.
2. **The cause of a refused answer.** When the engine refuses a model answer it cannot read, the call record says why.
3. **Replay the measured answers.** A seam case reads every raw writer answer that the memory test recorded under `assets/scene-entries/` through the engine's reader, and each one reads. The retro of ticket 17 (session `e91f4262`) measured 2 of 26 answers with a key written twice, which the reader refused until the fix of 2026-11-22. This makes the map Note "A build replays the measured answers" a test.

Done when the full `python build.py` passes.

## Context

- **The retry (known, the uncommitted tree on `long-term-memory-2` after `4832b0fa`, 2026-11-22).** The standards review of ticket 17 (session `55ee2d8e`) found the writer's second try to be the third hand-written loop with two tries:
  - `src/application/agents/memory/agent.rs`: `StoryMemoryWriter`, through `LlmCallRecorder::complete_read`;
  - `src/application/agents/options/utils/orchestration.rs`: `MAX_OPTIONS_ATTEMPTS`;
  - `src/application/agents/quantifier/utils/orchestration.rs`: `max_attempts = 2`.

  The three differ a little in when they try again. Every try is recorded through `LlmCallRecorder`.
- **The cause (known, same tree).** The check trial of ticket 17 (session `39e53f54`) had two failed writer tries. The record said only "unexpected response format". The trial agent read the raw answers and guessed the cause, a key written twice, but could not confirm it from the record. `LlmFailure::ParseError` (`src/error.rs`) carries `expected_format: &'static str` and the raw response, not the reader's error. `memory/parser.rs` drops the `serde_json` error with `.ok()`. The Quantifier and Options parsers do the same (inferred, from a grep).
- **Why it blocks the memory view.** The memory view adds a reader of the writer's output. Its build must not copy the retry loop a fourth time, and a person must see why an entry is missing.

## Open

- Whether the differences between the three retry rules are intended, or drift. Keep the behaviour of each Agent unless a rule is plainly a slip, and report each difference.
- Where the frame lives: a method of `LlmCallRecorder`, or a small helper next to the Agents.
- Whether the reader's error goes into `ParseError` as an owned string, or only into the log line of the failed call. It must reach the stored call record.
