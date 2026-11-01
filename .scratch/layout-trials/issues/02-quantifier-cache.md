# Find why the Quantifier calls read so little from the cache

Type: research
Status: resolved
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

## Answer

Asset: [The Quantifier cache: the front is not too short, the provider does not persist it](../assets/quantifier-cache.md).

- **The hypothesis is refuted, twice.** The provider reports a cache read only in units of 128 tokens: all 400 cached-token figures of the run are multiples of 128. The Quantifier front is 919 tokens at the median, so it is 6 to 7 units, not one. And when a Quantifier call does hit, it reads the whole front. The 29 hits read 22 × 768, 5 × 896 and 2 × 1024 tokens, which is the front rounded down to whole units, or one or two units short of it. A longer front does not give more.
- **What rules the length out as the cause.** A front that is byte-identical to the front of the call before it hits on 23 of 150 calls (15.3 %). The 11 calls whose shared prefix passes 1024 tokens all read 0, and the longest shared prefix of the whole run, 1421 tokens, is a miss. The gap between two Quantifier calls is 5.9 s at the median on a hit and on a miss alike. A hit can match the front of a call 493 s old, and call 198 hits on a front that no earlier call used.
- **The cause, with evidence.** The provider caches a prefix only when it has **persisted** a "cache prefix unit" that the request fully matches, and its documents call that best-effort ([DeepSeek: Context Caching](https://api-docs.deepseek.com/guides/kv_cache/)). Units appear at the end of a request, at a common prefix that the system detects, and at fixed intervals in a long input. A Narrator-role prompt is 3000 to 18 000 tokens and reads 55 % of its input from the cache; a Quantifier prompt is 1300 to 2500 tokens and reads 5.6 %. Even a front that 32 calls used hit 6 times. The document's own promise, that a common prefix hits from the third request, does not hold on this endpoint.
- **No cheap fix exists.** The Quantifier already keeps its stable part first: `<CurrentRoom>`, then `<PreviousCharactersPresent>`, then the two blocks that change. So the direction of [Try the changing blocks after the history](03-changing-blocks-after-history.md) is already in place, and that candidate does not touch this prompt at all.
- **The measured gain of the shapes the ticket names.** I applied them to the stored prompts with the method of [`prefix_measure.py`](../assets/prefix_measure.py), because a move of a block has to be measured before it becomes a candidate. Today 43.8 % of the Quantifier bytes are shared. The front after the history gives 37.1 %: worse by 6.7 points. The only shape that measures higher, 46.6 %, puts the closing instructions before the story parts and breaks the recency rule. The ceiling of today's shape is 44.8 %, so the layout is 1 point from its own ceiling.
- **A longer front cannot pay for itself.** One added token of stable front costs $0.15 per 1M on every call and saves $0.147 per 1M on a hit, because a cache read costs $0.003 per 1M and an input token $0.15 per 1M. The break-even hit rate is 102 %.
- **The stake, if a fix were free.** A perfect hit on all 200 calls would take the Quantifier's input cost from $0.0588 to $0.0350, which is 9.1 % of the run's $0.2613. The provider gives a hit on 29 of the 200 calls, so this number is not reachable.
- **What would settle it.** Send the same Quantifier prompt twice with nothing between the two calls. If the second call also reads 0, the endpoint does not cache this shape at all. That needs model calls, so it is outside this ticket.
- **Not tested (coordinator, 2026-11-01).** Every call of the trial, Narrator-role and Quantifier, sent the same `x-opencode-session` value: the Connection id. This research did not test that. If the provider scopes or routes its cache by that value, the large Narrator-role prompts can push out the Quantifier front. So "the provider does not persist it" can depend on our session value. [Try a separate OpenCode session for each call kind](10-separate-session-per-call-kind.md) tests it.
