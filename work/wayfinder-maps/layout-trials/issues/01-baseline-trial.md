# Run the baseline trial

Type: task (AFK)
Status: resolved
Blocked by: the merge of `prompt-flexibility` into `main` (owner approval), and the branch `layout-trials`

## Question

Run the test setup once on the merged code, so that each comparison of this map has its reference.

## Context

- Run on `layout-trials` right after it is made from `main`, before any candidate change.
- Copy `scripts/trial_setups/redmist_julian.json` to `tmp/lt01/redmist_julian.json`, and set its `out` to `tmp/lt01/redmist_julian`. Keep every other value: 100 Cycles, token limit 6,000,000, restart limit 5, 1 repeat.
- The `connections` id `conn-1793449507061-3a1cb15f` is the "Go check" Connection of the main probe DB, `target/ui-investigator/debug/chronicler_3001.db` (known, 2026-11-01: OpenCode Go, `deepseek-v4.1-flash`, `reasoning_effort` none, DB version 34). A worktree of a candidate copies this DB, so both sides use the same Connection (map Notes).
- This trial is also the check that Prompt flexibility did not run. Its last trial ran at [Cut the history in blocks](../../prompt-flexibility/issues/20-cut-the-history-in-blocks.md). [Add the connection slots and remove the global response length](../../prompt-flexibility/issues/21-connection-slots-and-response-length.md) and [Put the call parameters on the presets](../../prompt-flexibility/issues/22-call-parameters-on-presets.md) came after it, and were checked by hand only.
- Run `python scripts/trial.py compare` with this pack and the pack of that last trial (`tmp/pf20/redmist_julian/trial-1`). Expected (inferred): the same shape, with 100 Cycles, a history start that moves only at a cut, and about 50 % cached. A difference is a finding, not a failure: record it.
- Run [`prefix_measure.py`](../assets/prefix_measure.py) on the new pack. Its numbers are the reference for the cache candidates.
- No engine change in this ticket.

Done when the pack and `compare.md` exist, and the answer records: the Cycles played, why the run stopped, the tokens in and out, the cached share (in total and for each call kind), the failed calls with their `finish_reason`, the first history cut, and the differences from the earlier trial.

## Answer

Resolved on 2026-11-01. The baseline trial played its whole Budget with no failure.

### How it ran

- **known** The run used HEAD `1a0c9162` from the main checkout, because `main` and `layout-trials` are at the same commit. No worktree was made and no branch was switched.
- **known** The probe server was stopped first and started again on a fresh build of that commit. The binary that it replaced was built at 14:08, before the merge. So a run on the old server would not have measured this code.
- **known** The setup file is `tmp/lt01/redmist_julian.json`. It takes every value of `scripts/trial_setups/redmist_julian.json` and sets `out` to `tmp/lt01/redmist_julian`.
- **known** The Connection `conn-1793449507061-3a1cb15f` exists in the probe DB. Its name is "Go check", its provider is OpenCodeGo, its model is `deepseek-v4.1-flash`, and its `reasoning_effort` is none. The DB is at schema version 34.
- **known** No other client used the probe server during the run, so every counted token belongs to this trial.

### The run

| Item | Value |
|---|---|
| Cycles played | 100 of 100 |
| Story entries | 201 |
| Wall time | 1280 s |
| Tokens in / out | 3,067,165 / 52,637 (3,119,802 in total) |
| Cached input | 1,459,072 = 47.6 % of input |
| Restarts | 0 of 5 |
| Failed calls | 0 of 400 |

**Why it stopped (known).** The run played its whole Budget of 100 Cycles. `stopped_by_limit` is false, the Game reported no failure at the end, and the newest call has no failure.

### Cached share for each call kind

| Call kind | Calls | Failed | Tokens in | Tokens out | Cached | Cached share |
|---|---|---|---|---|---|---|
| narration | 100 | 0 | 1,373,233 | 25,720 | 724,608 | 52.8 % |
| impersonate | 100 | 0 | 1,239,215 | 22,702 | 717,056 | 57.9 % |
| quantifier | 200 | 0 | 454,717 | 4,215 | 17,408 | 3.8 % |

**Failed calls (known).** There are none. So there is no `finish_reason` to report: no `content_filter`, no length stop and no retry.

### First history cut

- **known** The first cut is at call 786, an Impersonate call. It drops 24 entries.
- **known** Call 786 is the 153rd call of this pack. The Game of this trial starts at call id 634, so the raw ids of this pack and of the pf20 pack are not comparable.
- **known** 124 of the 400 calls carry a cut. After call 786, nearly every Narrator call cuts in blocks: 24 entries first, then 43, 60, 79, 95, 109, 125, 141 and 156 entries near the end.

### Pack checks (the layout failure mode)

- **known** The pack holds 100 Narration calls, 100 Impersonate calls and 200 Quantifier calls. That is the expected shape.
- **known** `<ConversationHistory>` is in all 200 Narrator-role user prompts and in no Quantifier prompt. No history block is empty.
- **known** "player character" is in all 100 Impersonate system prompts. So `scripts/trial.py` still tells the two Narrator kinds apart by their text.
- **known** `pack.md` shows the Room and the Characters present of all 201 entries. So the history columns are present.
- **known** No browser was used. The trial drove the server through its HTTP routes.

### Differences from the pf20 trial

`tmp/pf20/redmist_julian/trial-1` is the last trial of Prompt flexibility.

| Item | Baseline | pf20 |
|---|---|---|
| First cut, pack call | 153 | 157 |
| Calls with a cut | 124 | 122 |
| Tokens in / out | 3,067,165 / 52,637 | 3,042,919 / 43,900 |
| Cached input | 47.6 % | 49.5 % |
| Narration cached | 52.8 % | 55.3 % |
| Impersonate cached | 57.9 % | 58.0 % |
| Quantifier cached | 3.8 % | 5.6 % |
| Narration tokens out, median | 1016 | 913 |
| Impersonate tokens out, median | 853 | 650 |
| Narration words, median | 191 | 175 |
| Input words, median | 160 | 126 |
| Wall time | 1280 s | 1241 s |

- **known** pf20 appended the "Response Length" paragraph to the Impersonate prompt, inside `<output_format>`. The baseline does not. In the baseline the paragraph is on the system preset only, so only the Narration prompt carries it. Its text is the same in both packs.
- **inferred** The missing constraint lets Impersonate write longer. Its output grows by 31 % (650 to 853 tokens), and the Input entry grows from a median of 126 to 160 words. The Narration output grows by 11 %, and its prompt keeps the paragraph.
- **known** The cached share falls by about 2 points in total. The Narration cached share falls by 2.5 points and the Quantifier share by 1.8 points. The Impersonate share does not change.
- **known** pf20 recorded one `event-narration` call. The baseline recorded none.
- **guessed** No Trigger fired an event narration in this story. This is a story difference and not a code difference.
- **known** Both runs have the expected shape: 100 Cycles, 201 entries, and a cut that moves in blocks. The expectation of the ticket holds.

### The reference numbers for the cache candidates

`prefix_measure.py` on the new pack (`tmp/lt01/prefix-measure.md`), baseline against pf20:

| Candidate | Narration | Impersonate |
|---|---|---|
| Today | 56.3 % (pf20 55.5 %) | 57.4 % (pf20 60.7 %) |
| No presence marks | 56.8 % (56.3 %) | 57.9 % (61.4 %) |
| `GameState` after the history | 66.8 % (59.2 %) | 63.4 % (65.2 %) |
| Both of those | 67.4 % (60.2 %) | 64.2 % (66.2 %) |
| No marks, `GameState` and `CharactersPresent` after the history | 82.0 % (83.9 %) | 84.0 % (85.3 %) |

- **known** The pairs that break before the history are 37 of 99 for Narration and 35 of 99 for Impersonate today. In pf20 they were 42 and 37.
- **known** The combined candidate still keeps every pair's break inside or after the history. Its gain is 25.7 points for Narration and 26.6 for Impersonate.
- **known** `GameState` after the history alone now gains 10.5 points for Narration, against 3.7 in pf20. It gains 6.0 for Impersonate, against 4.5 in pf20.
- **known** The Quantifier calls share 39.0 % of their bytes with the previous Quantifier call, over 199 pairs. In pf20 it was 43.8 %.
- **known** The provider reported 47.6 % cached against 56.3 % shared prefix. So the measure stays an upper bound, as the asset says.

### Paths

- Pack: `tmp/lt01/redmist_julian/trial-1`
- Comparison: `tmp/lt01/redmist_julian/compare.md`
- Prefix measure: `tmp/lt01/prefix-measure.md`
- Setup file that the run used: `tmp/lt01/redmist_julian.json`

No engine change was made. The probe server is left running on port 3001 on the build of `1a0c9162`.
