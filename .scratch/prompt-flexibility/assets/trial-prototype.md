# The trial prototype: two trials of one test setup

Made for [Shape the test setup and the trial](../issues/08-test-setup-and-trial.md) on 2026-10-31.

## What was made

- **Script.** [`PROTOTYPE_trial.py`](../prototype/PROTOTYPE_trial.py). It is throwaway. It has three commands:
  - `run` creates a new Game on a probe server and starts Autoplay. While the run plays, it copies each new `llm_messages` row out of the DB. At the end it saves the setup, the story and the run facts, then calls `export`.
  - `export` writes the reading pack: `pack.md` and one file for each call in `calls/`.
  - `compare` writes the summary of two or more trials side by side, then the story of each trial, entry by entry.
- **Probe server.** `target/ui-investigator`, port 3001. The DB is a copy of `tmp/db-backup-20261031/chronicler_3150.db`. Both role connections point at "Go check": OpenCode Go, `deepseek-v4.1-flash`, `reasoning_effort: none`.
- **Packs.** These are in `tmp/pf08/` and are not in git. They are 40 MB for each long trial, because each call file holds the full prompt.
  - `tmp/pf08/trial-a/pack.md`: 34 cycles.
  - `tmp/pf08/trial-b/pack.md`: 88 cycles.
  - `tmp/pf08/compare.md`: the two trials side by side.
  - `tmp/pf08/smoke/pack.md`: 2 cycles.

## The two trials

Same test setup: `redmist_estate`, `julian`, the default presets, novel mode, third person past tense, the default `response_length`, Budget 100 cycles, token limit 6,000,000.

| | Trial A | Trial B |
|---|---|---|
| Cycles played | 34 of 100 | 88 of 100 |
| Why it stopped | Impersonate call, `content_filter` | Impersonate call, `content_filter` |
| Calls | 139 | 355 |
| Tokens in / out | 693,694 / 13,460 | 2,908,606 / 41,000 |
| Cached input | 1 % | 1 % |
| Failed calls | 1 | 5 (4 Quantifier, 1 Impersonate) |
| First history cut | none | cycle 39 (call 304) |
| Wall time | 6.5 min | 18 min |
| Median words, Narration / Input | 160 / 113 | 174 / 124 |

## Findings

Known unless marked otherwise.

1. **One cycle is 4 calls, not 3.** The Quantifier runs after every story entry, after the Impersonate Input too. Options does not run, because Options always on is off in this setup.
2. **Only the story is kept for a long trial.** `messages` and `message_swipes` keep every story entry and its Swipe state. `llm_messages` is a debug record (owner, 2026-10-31): it keeps only the newest 50 rows and has no `game_id`. The prompts, the tokens, the finish reasons and the failed calls exist only there. Trial B made 355 calls, so the trial runner must copy each row while the trial runs. The prototype does this. It finds the Game of a call by time and its story entry by text.
3. **A trial stops at its first failed story call.** One `content_filter` answer on an Impersonate call ended each trial: trial A at cycle 34, trial B at cycle 88. The length of a trial is thus random. Two trials of one setup are equal only up to the shorter one.
3a. **A restart passes the filter after some tries.** After the stop of trial B, Autoplay was started again 3 times on the same Game, with a Budget of 1 cycle. Impersonate calls 507 and 508 were filtered again. Call 509 passed, and the cycle completed. The filter is thus not stable for one story point, but it can repeat.
4. **The engine hides the cause.** OpenCode Go answers a filtered call with `finish_reason: content_filter`, empty content and 0 tokens. The engine stores and shows "LLM returned an empty response". The pack reads the finish reason from the raw answer.
5. **A failed Quantifier call does not stop Autoplay.** The Quantifier tries once more, and the run continues. In trial B, 4 Quantifier calls failed (2 entries, 2 attempts each). The Swipe states of these entries still show the Characters present of the entry before (observed, and checked against the code: the `Low` confidence fallback keeps them). [Decide what the state does when a state writer fails](../issues/15-failed-state-writer.md) has the rule.
6. **OpenCode Go does cache, a little.** 1 % of the input tokens were read from the cache (`cache_read` in `usage.opencode.tokens`). This agrees with [Find how prompt caching works for our providers and our layout](../issues/12-prompt-caching.md): the layout breaks the shared prefix.
7. **OpenCode Go reports `allowance`, not `cost`.** `usage.cost` is 0. `usage.opencode.allowance` and `list_cost` are the price numbers. The response has no `provider` field, so the upstream host is unknown.
8. **The first history cut came at cycle 39.** This matches the estimate of [Find how ST, ME and other tools keep long-term memory](../issues/13-long-term-memory.md). At the end of trial B, an Impersonate call held 71 entries of history and had dropped 106.
9. **The Quantifier labels a Narration as the player.** `<LatestNarration>` always starts with the player name (`Julian: …`), also when the entry is a Narration (`src/application/agents/quantifier/prompt.rs`, the `<LatestNarration>` format string). This belongs to [Decide which parts of a Quantifier or Options call an author can change](../issues/10-quantifier-options-layout.md).
10. **The cost is small.** 88 cycles used about 2.95 million tokens. At this rate, three trials of two setups use about 18 million tokens. The owner set the limit at millions to tens of millions of tokens a day.
11. **Two trials of one setup do not send the same prompt.** The card order is random (a `HashMap`), and the engine sends no seed and no sampler. A test setup cannot fix these parts on the current engine.

12. **Both known Impersonate defects show.** In trial B, Impersonate call 152 holds the global `response_length` ("under 150 words"), and its user prompt ends with an empty `<PlayerInput>` block. These are the two defects that the map Notes name.

## What the reading pack shows

- The test setup: World, Persona, mode, posture, both connections, the four preset ids, `response_length`, Budget, token limit.
- A summary: cycles played, why the run stopped, tokens in, out and cached, the OpenCode Go allowance, a table of calls by kind with failures, the list of failed calls with the finish reason, and the first history cut.
- The story: each entry with its kind and word count, the Room and the Characters present of its Swipe state (Δ marks a change), and the calls of its Job. Each call line shows tokens, cached tokens, how many entries its history holds and drops, the Quantifier answer and any failure. Each call links to a file with the full system prompt, user prompt and answer.
