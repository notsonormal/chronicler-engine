# Shape the test setup and the trial

Type: prototype
Status: resolved
Blocked by: 01

## Question

What does a test setup fix, how does a trial start, how many times does it repeat, what does it save, and what does the reading pack look like?

## Context

- Autoplay plays N cycles on one Game (`POST /autoplay` with a budget). The `llm_messages` table already stores every prompt, request and answer.
- A first rough reading pack exists: [story transcript](../assets/autoplay-story-transcript.md) and [the stored Impersonate prompt](../assets/impersonate-prompt-157.txt), made by hand from the probe DB.
- Points to settle: what is fixed (World, Persona, opening, presets, connections, cycles); how a trial starts (a script against a server, a CLI subcommand, or a test harness); repeats, because the same setup gives different output each time; how failed calls show (the Quantifier failed every cycle with `content_filter`, which left the game state stale with no visible sign); how two trials sit side by side. Do not define criteria for good output: that is not yet specified (see the map).
- A trial calls a paid model many times. Settle a cost limit for one trial.
- Reuse what exists: the LLM messages panel (`/fragment/llm-messages`) and the probe server of `/chronicler-ui-investigator` (`target/ui-investigator/`, port 3001), which the first Autoplay run used.
- Long play is where memory and state fail. The reading pack may need to show what fell out of the history window and which state changed. See [Find how ST, ME and other tools keep long-term memory](13-long-term-memory.md) and [Find how ST, ME and other tools keep long-term state](14-long-term-state.md). They do not block this ticket: the first prototype runs on the current engine.
- From [Find how prompt caching works for our providers and our layout](12-prompt-caching.md): record per call the cached tokens (`usage.prompt_tokens_details.cached_tokens`, already in `raw_response_json`), the upstream host and the cost (for OpenCode Go, `usage.opencode.allowance`, because its `usage.cost` is 0). The first trial gives the first pair of calls of one kind on OpenCode Go, so it answers whether that connection caches. The random card order (a `HashMap`) means two trials of one setup do not send the same prompt.
- From [Find how ST, ME and other tools keep long-term memory](13-long-term-memory.md): a trial needs about 39 cycles of this World's prose to show the first history cut, or a connection with an 8192-token context to show it at about cycle 10. The cycle depends on how long the model writes, so a test setup must fix `response_length` and the connection context, or two setups differ in a second variable. The reading pack can report the loss from the stored request alone: the Messages that `<ConversationHistory>` does not hold, and the offset of the front cut.
- Use `/prototype`: a script that exports a trial from the database into a reading pack is a cheap first artifact.

## Prototype

[The trial prototype: two trials of one test setup](../assets/trial-prototype.md). Two 100-cycle Autoplay trials ran on OpenCode Go (owner, 2026-10-31: OpenCode Go is the trial connection; millions to tens of millions of tokens a day is fine). They stopped at cycle 34 and cycle 88, each on a `content_filter` Impersonate call. The owner's decisions are below and in the answer.

### Decisions so far (owner, 2026-10-31)

- **The connection of a trial.** OpenCode Go. The cost limit of a trial is a token limit. Millions to tens of millions of tokens a day is fine. Hundreds of millions is not.
- **A filtered story call.** The trial runner starts Autoplay again on the same Game. The pack marks the gap. The runner needs a limit on restarts, because one story point can be filtered again (see finding 3a).
- **The call record.** `llm_messages` is a debug record. The story lives in `messages` and `message_swipes`. The trial runner copies the `llm_messages` rows while the trial runs. The engine record does not change for trials.
- **The trial runner.** The prototype script becomes a kept script in `scripts/`. It runs against a probe server. It reads a test setup file (World, Persona, connections, the four presets, `response_length`, mode, posture, Budget, token limit, restart limit) and applies it through the HTTP routes before the run. No engine command. Default restart limit: 5 restarts for each trial.
- **The card order.** The build plan fixes the card order: sort the Characters by id, in the Narrator-role calls and in the Quantifier. Today a `HashMap` gives a new order on each call (`src/application/pipeline/job.rs`, `src/application/agents/quantifier/utils/orchestration.rs`). The first calls of trial A and trial B listed the cards in different orders.
- **The sampling.** A trial uses the same sampling as play. It sends no temperature 0 and no seed for trials only, so a trial tests what a player gets. [Decide the parameters of each call](07-call-parameters.md) decides the sampling of play. Two trials of one setup therefore still differ.
- **Repeats.** The setup file has a `repeats` field, default 1. Now the trials find faults: one 100-cycle trial for each setup. A comparison of two setups uses 2 trials for each setup, after the criteria and the judge exist (map, "Not yet specified").

## Answer

A trial is one run of a fixed test setup through Autoplay on a probe server, driven by a script. The engine does not change for trials. The prototype answered each point (owner, 2026-10-31):

- **The test setup.** One JSON file: World, Persona, connections, the four presets, `response_length`, mode, posture, Budget, token limit, restart limit and `repeats`. The script applies it through the HTTP routes before the run. The prototype did not apply a setup, so this is the one new part of the kept script.
- **How a trial starts.** `python scripts/trial.py run <setup.json>` against a probe server (see `/chronicler-ui-investigator`). The prototype `prototype/PROTOTYPE_trial.py` becomes this script in the build plan, and the prototype is then deleted.
- **What it saves.** The story comes from `messages` and `message_swipes`. The prompts, tokens, finish reasons and failed calls come from `llm_messages`, which is a debug record of 50 rows. So the script copies each new row while the trial runs. The engine record does not change.
- **The connection and the cost limit.** OpenCode Go. A trial has a token limit, default 6,000,000. Millions to tens of millions of tokens a day is fine. One 100-cycle trial uses about 3 million tokens.
- **Failed calls.** A filtered story call (`content_filter`) ends an Autoplay run. The script starts Autoplay again on the same Game, at most 5 times for each trial, and the pack marks the gap. The pack shows every failed call with its `finish_reason`.
- **Repeats.** Default 1. Now a trial finds faults: one 100-cycle trial for each setup. A comparison of two setups uses 2 trials for each setup, after the criteria and the judge exist.
- **Randomness.** A trial uses the same sampling as play. The build plan fixes the card order (sort by Character id), because a `HashMap` order has no purpose and breaks the cache.
- **The reading pack.** As the prototype makes it, with no change: the setup table, a summary, the story with the state of each entry and its calls, one file for each call, and `compare.md` for trials side by side. The owner may ask for a change after reading real packs.

Engine items for the build plan, from the trials: the card order; a failure text that names the `finish_reason` (today "LLM returned an empty response" hides `content_filter`). The Quantifier `<LatestNarration>` label went to [Decide which parts of a Quantifier or Options call an author can change](10-quantifier-options-layout.md). The Quantifier state after a failure went to [Decide what the state does when a state writer fails](15-failed-state-writer.md).

Assets: [the trial prototype](../assets/trial-prototype.md), [the prototype script](../prototype/PROTOTYPE_trial.py).
