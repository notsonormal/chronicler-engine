# Decide if a stop cancels a running model call

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

Target design D-11 says a stop cancels the model call "when it can". Today it cannot. The Job runs in `spawn_blocking`, and it checks its cancel handle only before each part of a generation (`JobWriter::cancelled`, `src/application/pipeline/job.rs`). `LlmProvider::complete` is synchronous, and the transport is `reqwest::blocking` with a 180 s timeout. After a stop, the player sees the Game idle at once, and the Job's late answer is discarded. But the provider call runs to its end, its tokens are paid, and the Job's task lives until the call returns.

- **Option A: cancel the call.** Pass a cancel handle through `Narrator::narrate`, the Agent calls and the `LlmProvider` port to the transport. Because the transport is blocking, this needs an async client, or a call on another thread that the Job stops waiting for. Add seam cases for the Narrator and the port.
- **Option B: stop waiting only.** The Job stops waiting for the answer on a stop, and the call ends in the background. This frees the Job sooner, but the tokens are still paid.
- **Option C: leave it.** "When it can" allows it. Record it as a known limit in `docs/diataxis/reference/game_flow.md`.

## Context

- Source: core step C3 ([The core step](07-core-step.md)): "a stop does not cancel a running model call because `Narrator::narrate` has no cancel handle (D-11 'when it can')". The C7 doc review confirmed that the docs state the code as it is.
- Design: `.scratch/turn-state-model/assets/target-design.md`, D-11.

## Answer

Option C (owner, 2026-10-19): leave it. A stop does not cancel a model call that runs. D-11 "when it can" allows this.

- **Facts checked (known):** `Job::require_alive` checks the cancel handle before each step. (2026-10-19, whole-rebuild review: the cancel handle and `require_alive` were removed as redundant; the id check in `JobWriter::step` now stops the Job before its next step, with the same behaviour.) After a stop, at most one model call that already runs goes on to its end, and the Job starts no further call. The stop removes the job entry, so the Game is idle at once and a new Job can start. The id check refuses the late answer.
- **Reason:** every request is non-streaming. Not every provider stops the model when the client disconnects (guessed: Ollama stops; OpenRouter documents a cancel for streamed requests only). Option A changes the `LlmProvider` port, 4 providers, `LlmRecorder`, the Narrator, both Agents and the test fakes, and it may still save no tokens.
- **Done:** `docs/diataxis/reference/game_flow.md`, section "Stop and Reset", states the limit: "A model call that runs when the stop comes runs to its end, and the id check refuses its answer."
- **Out of scope:** a streaming transport with a real cancel of the call. It is an effort after the build, and it is recorded on the map.
