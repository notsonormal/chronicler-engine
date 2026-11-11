# The Quantifier cache: the front is not too short, the provider does not persist it

Written for [Find why the Quantifier calls read so little from the cache](../issues/02-quantifier-cache.md)
(research, 2026-11-01). Read only: no engine change, no trial, no model call.

Every claim is **known** (read in the code, read in a stored row, or read in a document),
**inferred** (reasoned from a measurement) or **guessed**.

## 1. The answer in four lines

- **known** The stable front of a Quantifier prompt is 6 to 7 cache units of the provider, not 1.
  The provider reports a cache read only in units of 128 tokens.
- **known** When a Quantifier call hits, the provider reads the whole front: 768, 896 or 1024 tokens.
  So the front does not need to be longer.
- **known** A front that is byte-identical to the front of the call before it still hits on only
  23 of 150 calls (15.3 %). Repetition does not make it reliable either: a front seen 32 times
  hits 6 times.
- **inferred** The cause is provider-side. The provider caches a prefix only when it has
  **persisted** a "cache prefix unit" that the request fully matches, and that persistence is
  best-effort. No layout of the Quantifier prompt changes it. No cheap fix exists.

## 2. Material and method

- **known** The material is `tmp/pf20/redmist_julian/trial-1/calls.jsonl`: the 400 calls of the
  last 100-Cycle trial of Prompt flexibility. 200 of them are Quantifier calls, and none failed.
  One model served them all: OpenCode Go `deepseek-v4.1-flash`.
- **known** The figures for the cache read are `raw_response_json.usage.prompt_tokens_details.cached_tokens`.
  The provider repeats the same number in `usage.opencode.tokens.cache_read`, and
  `usage.opencode.tokens.input` is always `prompt_tokens` minus that number.
- **known** The request is `system_prompt + "\n" + user_prompt`, as in
  [`prefix_measure.py`](prefix_measure.py).
- **known** A "front" is everything in the request before `<RecentHistory>`. That is the system
  message plus `<CurrentRoom>` plus `<PreviousCharactersPresent>`
  (`src/application/agents/quantifier/prompt.rs`).
- **inferred** The token count of a part is `characters × prompt_tokens / characters_of_the_request`.
  The estimate is good to about 2 %, because the front is mostly XML and identifiers.

The scripts are throwaway and live in `tmp/lt02/` (`where.py`, `pairs.py`, `front.py`,
`occurrence.py`, `frontmatch.py`, `front_measure.py`, `summary.py`).

## 3. The size of the front, and the size of one cache unit

| Measure | Value | Label |
|---|---|---|
| Front in tokens | min 804, median 919, max 1254 | known |
| Prompt in tokens | min 1304, median 2085, max 2466 | known |
| Front as a share of the prompt | median 44.6 %, ceiling 44.7 % of the tokens | known |
| The first differing byte, over 199 pairs | `<RecentHistory>` 157, `<PreviousCharactersPresent>` 25, `<CurrentRoom>` 17 | known |
| Distinct Quantifier system messages in the whole run | 1 (the same SHA-1 on all 200 calls) | known |

- **known** All 400 cached-token figures of the run are multiples of 128: the Quantifier shows
  768, 896 and 1024, and the Narrator role shows 1152, 2048, 4096, 5120, 6144, 7168, 8192,
  10240, 12032, 12288, 14336, 14976, 15360, 15616, 16000, 16384 and 16512.
- **inferred** So one cache unit of this provider is 128 tokens, and the reported hit is a whole
  number of units.
- **known** The front is therefore 6 to 7 units, not one. The premise of the ticket, "the stable
  front is about as long as one cache block", needs a block of about 940 tokens. The measured
  block is 128.

## 4. What one hit reads

- **known** The 29 hits read 22 × 768, 5 × 896 and 2 × 1024 tokens: 23 424 tokens, which is
  5.6 % of the 415 193 Quantifier input tokens.

The table compares the reported hit with the front, rounded down to whole 128-token units.

| Reported hit minus the front rounded down | Hits |
|---|---|
| 0 (the whole front) | 12 |
| −128 (one unit short) | 12 |
| −256 or −384 | 3 |
| +128 | 2 |

- **known** So a hit reads the whole front, or misses it by one or two units. A hit does not need
  a longer front.
- **known** A brand-new front can hit: call 198 has a front that no earlier call used, and it
  read 768 tokens. So the read does not need the previous call of the same kind.

## 5. What does not separate a hit from a miss

| Test | Result | What it rules out |
|---|---|---|
| The front is byte-identical to the front of the call before | 23 hits of 150 (15.3 %) | The provider has all the bytes it needs and still misses 85 % of the time |
| The front changed since the call before | 6 hits of 49 (12.2 %) | A changed front is not the cause |
| Gap to the previous Quantifier call, median | 5.9 s on a hit, 5.9 s on a miss | Time between two Quantifier calls is not the cause |
| Gap to the previous Quantifier call, maximum | 7.4 s on a hit, 18.6 s on a miss | A long gap does not lose the cache |
| Gap to the last call of any kind with the same front | median 5.7 s on a hit, 6.1 s on a miss; maximum 493 s on a hit | A lifetime of minutes is not the limit |
| Front 3300 to 3600 characters | 7 hits of 41 | No band of front sizes is better |
| Front 4000 to 4100 characters | 0 hits of 22 | A bigger front is not better |
| Shared prefix of 1024 tokens or more (11 calls) | 0 hits of 11 | The longest shared prefixes of the run are all misses |
| The most repeated front of the run, which 32 calls used | 6 hits of 32 | Repetition does not make a hit certain |
| The request before this one hit | 21 of 29 hits, against a base rate of 69 % | The two call kinds do not share a cache state |
| Hits per 20 calls | 1, 2, 2, 1, 4, 6, 0, 3, 6, 4 | The hits drift up over the run, and they do not cluster |

- **known** The 11 misses with a shared prefix of 1024 tokens or more are the decisive test of
  the ticket's hypothesis. They share more than one unit, some share 1421 tokens, and every one
  of them read 0.
- **known** The longest shared prefix over all 19 900 pairs of Quantifier calls is 1421 tokens,
  against a median prompt of 2085. So no pair of Quantifier prompts ever has the append-only
  shape that the provider rewards (its Examples 1 and 2, section 6).

## 6. The provider's rule, from its own documents

- **known** DeepSeek, [Context Caching](https://api-docs.deepseek.com/guides/kv_cache/), read
  2026-11-01: "Each cached prefix is an independent, complete unit. A subsequent request can only
  hit the cache if it **fully matches** a **cache prefix unit**."
- **known** Same page: the provider persists a cache prefix unit in three cases.
  1. At the end position of the user input and the end position of the model output, for each
     request.
  2. When the system detects a common prefix across multiple requests.
  3. At fixed token intervals, for long inputs or long outputs.
- **known** Same page: "The cache system works on a 'best-effort' basis and does not guarantee a
  100 % cache hit rate." "Cache construction takes seconds." A cache that is not used "will be
  automatically cleared, usually within a few hours to a few days".
- **known** DeepSeek, [the news page of 2024-08-02](https://www.deepseek.com/news/context-caching/):
  only a prefix that is identical "from the 0th token" counts, and a match in the middle of the
  input does not hit.
- **known** Measured on this pack, the reported hit is a whole number of 128-token units, and
  the hit never exceeds the shared prefix that the script measures. Four of the 29 hits report up
  to 3 % more, which is the error of the token estimate itself.
- **inferred** This endpoint is not DeepSeek direct. OpenCode Go publishes no cache rule, so I use
  the DeepSeek rule as the best available model of its behaviour. The usage shape fits: the
  endpoint reports `cache_read`, `cache_write_5m` and `cache_write_1h`, and the prices that the
  stored rows give ($0.15 per 1M input, $0.003 per 1M cache read, $0.60 per 1M output) are the
  [DeepSeek V4.1 Flash prices of the OpenCode console](https://dev.opencode.ai/console/guides/budgets).
- **inferred** Rule 1 is why a long prompt hits so much better than a short one. A Narrator-role
  prompt is 3000 to 18 000 tokens, so rule 3 carves units inside it, and rule 1 leaves a unit at
  the end of each earlier prompt. A Quantifier prompt is 1300 to 2500 tokens, and the only stable
  part of it is a 900-token front, which rules 1 and 3 never reach.
- **guessed** Which of the three rules produced each of the 29 hits.
- **known** The document's Example 2 says that a common prefix is detected "after the first two
  requests", so a third request hits. This does not hold here. A front that 32 calls used hit 6
  times, and call 38 missed with a front that call 36 had just hit.
- **known**, for a comparison and not as our rule: other providers with an implicit cache publish
  a minimum input length, such as 4096 tokens for the [Gemini 3.8 Flash implicit cache](https://ai.google.dev/gemini-api/docs/generate-content/caching).
  A minimum length of that kind would also leave a 2 000-token Quantifier prompt uncached.

## 7. The cheap-fix test

[`prefix_measure.py`](prefix_measure.py) answers 43.8 % for the Quantifier today, which matches
the earlier measurement. I copied its method into `tmp/lt02/front_measure.py` and applied the
candidates that the ticket names. Each candidate rewrites the stored user prompt, so the story
stays the same and no model is called.

| Candidate | Shared prefix |
|---|---|
| Today: front, history, latest message, instructions | 43.8 % |
| The same order written out again (a check on the script) | 43.8 % |
| The front after the history | 37.1 % |
| The front after the history and the latest message | 37.1 % |
| The instructions first, then the whole prompt | 46.6 % |
| **Ceiling**: today's front, read on every call | 44.8 % of the bytes |
| **Ceiling**: the system message alone | 36.1 % of the bytes |

- **known** The Quantifier prompt already has its stable part first. Its changing blocks,
  `<RecentHistory>` and `<LatestMessage>`, come after it. So the direction of
  [Try the changing blocks after the history](../issues/03-changing-blocks-after-history.md) is
  already in place here, and that candidate does not touch the Quantifier at all.
- **known** Moving the front later loses 6.7 points. The only shape that measures higher puts the
  closing instructions before the story parts, which breaks the recency rule that the Narrator-role
  layout keeps ([prompt_system_design.md](../../../../docs/diataxis/explanation/prompt_system_design.md)).
  It gains 2.8 points, and it is not a candidate.
- **known** Today's layout is 1.0 point below its own ceiling, 44.8 %. No order of the existing
  blocks reaches the ceiling, because the bytes after the front change on every call.
- **inferred** A longer stable front cannot pay for itself either. One added token of stable front
  costs $0.15 per 1M on every call and saves $0.15 − $0.003 = $0.147 per 1M when the call hits. The
  break-even hit rate is 102 %, so even a guaranteed hit does not recover the added token. The
  balance only turns on a provider whose cache read is much cheaper than its input.

## 8. The stake, in money

- **known** The prices that the stored rows give for this endpoint are $0.15 per 1M input tokens,
  $0.003 per 1M cache-read tokens and $0.60 per 1M output tokens. Three calls solve the three
  unknowns exactly, on both the Quantifier and the Narrator role.
- **known** The run's `usage.opencode.list_cost` is $0.2613 in total: Quantifier $0.0614 (23.5 %),
  Narration $0.1081 (41.4 %), Impersonate $0.0906 (34.7 %), Event Narration $0.0012 (0.5 %).

| Case | Cached share of the Quantifier input | Quantifier input cost | Saving |
|---|---|---|---|
| Today | 5.6 % | $0.0588 | — |
| Every call hits the whole front | 44.7 % | $0.0350 | $0.0239, which is 9.1 % of the run price |

- **known** The second row is a ceiling that no engine change reaches, because it needs a hit on
  all 200 calls and the provider gives one on 29.
- **known** The two Narrator-role kinds already read 55 % and 58 % of their input from the cache,
  so the cache works for them. The Quantifier is the one call kind that the provider does not serve.

## 9. Findings

- **known** The hypothesis of the ticket is refuted twice. The front is 6 to 7 units, not one,
  and a hit reads the whole front.
- **known** The cover for the misses is the provider, not the layout. The Quantifier prompt is
  stable where it can be, and it already matches the ceiling of its own shape to within one point.
- **known** Nothing in the engine raises the Quantifier's cached share:
  - A longer stable front costs more than it saves (section 7).
  - A reorder of the blocks cannot pass the ceiling of 44.8 % without moving the recency text.
  - The provider persists a cache unit on a best-effort basis and does not report why it misses.
- **inferred** The measurement that would settle it is a provider-side one: send the same
  Quantifier prompt twice with nothing between the two calls. If the second call also reads 0,
  the endpoint does not cache this shape of prompt at all. That is outside this ticket, because it
  needs model calls.
- **inferred** A related shape may be worth a measurement later: the Narrator role hits because its
  prompt is long. If a future ticket wants the Quantifier's answer cached too, the lever is a
  prompt above the provider's interval for long inputs, and that lever costs tokens. It is not a
  layout candidate.

## Sources

| Date | Source | What it gives |
|---|---|---|
| read 2026-11-01 | [DeepSeek: Context Caching](https://api-docs.deepseek.com/guides/kv_cache/) | The unit rule, the three persistence rules, best-effort, the lifetime |
| 2024-08-02 | [DeepSeek: Context Caching on Disk](https://www.deepseek.com/news/context-caching/) | Only a prefix from the 0th token hits |
| read 2026-11-01 | [OpenCode console: budgets](https://dev.opencode.ai/console/guides/budgets) | The prices of DeepSeek V4.1 Flash, which match the stored rows |
| read 2026-11-01 | [Gemini API: context caching](https://ai.google.dev/gemini-api/docs/generate-content/caching) | A comparison: implicit caches publish a minimum input length |
| — | `tmp/pf20/redmist_julian/trial-1/calls.jsonl` | Every measurement above |
