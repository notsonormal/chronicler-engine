# Find why the Quantifier calls read so little from the cache

Type: research
Status: open
Blocked by: —

## Question

The front of a Quantifier prompt stays the same for about 940 tokens from one call to the next, but the provider reported a cache hit on only 29 of 200 Quantifier calls. Is the cause that this stable front is about as long as one cache block of the provider? If so, which cheap change makes the stable front clearly longer than one block?

## Context

- Known, from [the shared prefix of the cache candidates](../assets/cache-prefix-measure.md) (plan review, 2026-11-01), on the 100-Cycle trial of [Cut the history in blocks](../../prompt-flexibility/issues/20-cut-the-history-in-blocks.md):
  - The Quantifier made 200 of the 400 calls. Its median prompt is 2085 tokens.
  - Two Quantifier calls in a row share 43.8 % of their bytes, with a median of 3749 characters (about 940 tokens at 4.0 characters for each token). So the front does not change on each call, which is what ticket 20 inferred.
  - The provider reported 768, 896 or 1024 cached tokens on 29 calls, and 0 on the other 171.
  - The first byte that differs falls in `<RecentHistory>` in 157 of 199 pairs, in `<PreviousCharactersPresent>` in 25 and in `<CurrentRoom>` in 17.
- OpenCode Go reports `cached_tokens` in blocks, and its cache rule is not known. Two Narrator-role calls come between two Quantifier calls of one Cycle, so the time between two Quantifier calls may matter too (guessed).
- Every prompt and its usage are in `tmp/pf20/redmist_julian/trial-1/calls.jsonl` (`raw_response_json` holds `usage.prompt_tokens_details.cached_tokens`). [`prefix_measure.py`](../assets/prefix_measure.py) measures the shared prefix.
- The Quantifier prompt is built in `src/application/agents/quantifier/prompt.rs`, and the call in `src/application/agents/quantifier/utils/orchestration.rs`.
- The provider rules are in [Find how prompt caching works for our providers and our layout](../../prompt-flexibility/issues/12-prompt-caching.md) and its [asset](../../prompt-flexibility/assets/prompt-caching.md).
- Read only: no engine change and no trial. The cache is a balance, not a goal, so look for a cheap rule, not the best hit rate.

Done when the answer names the cause with evidence from the calls, and says whether a cheap fix exists. A cheap fix becomes a cache candidate (map, "Not yet specified"). If a fix moves a block, measure it with `prefix_measure.py` before it becomes a candidate.
