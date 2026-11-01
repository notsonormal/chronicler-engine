# Run the baseline trial

Type: task (AFK)
Status: open
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
