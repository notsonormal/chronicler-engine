# The shared prefix of the cache candidates, measured on the last trial

Made in the plan review of this map (2026-11-01). Script: [`prefix_measure.py`](prefix_measure.py). Material: the 400 calls of the last 100-Cycle trial of Prompt flexibility, `tmp/pf20/redmist_julian/trial-1/calls.jsonl`. That trial ran with the cards sorted by id and the history cut in blocks.

## Method

- **known** Each call is paired with the previous successful call of the same kind. The measure is the byte prefix that the two requests share (`system_prompt` + newline + `user_prompt`), divided by the length of the newer request. The table weights each pair by its length.
- **known** A candidate is applied by rewriting the stored user prompts of both calls. The story stays the same, so only the layout changes. No model call is made.
- **inferred** A provider caches only a prefix, in blocks, and on a best-effort basis. So this measure is an upper bound for the cached share, not the provider number. In this trial the provider reported about 61 % cached for the Narrator-role calls between history cuts, against 55 to 61 % here.
- **inferred** A real trial of a candidate plays a different story. The measure does not show an output change.

## Result

The layout of a Narrator-role user message today: `GameState`, `KnownCharacters`, `CharactersPresent`, `Persona`, `WorldLore`, `ConversationHistory`, `writing_style`, `output_format`, `PlayerInput`.

| Candidate | Narration: shared prefix | Narration: pairs that break before the history | Impersonate: shared prefix | Impersonate: pairs that break before the history |
|---|---|---|---|---|
| Today | 55.5 % | 42 of 98 | 60.7 % | 37 of 99 |
| No presence marks | 56.3 % | 42 of 98 | 61.4 % | 37 of 99 |
| `GameState` after the history | 59.2 % | 34 of 98 | 65.2 % | 30 of 99 |
| Both of the above | 60.2 % | 34 of 98 | 66.2 % | 30 of 99 |
| No marks, and `GameState` and `CharactersPresent` after the history | 83.9 % | 0 of 98 | 85.3 % | 0 of 99 |

"After the history" means after `ConversationHistory` and before `writing_style`, so that the style and format sections stay at the end.

## Findings

- **known** Today the cache breaks before the history in 42 of 98 Narration pairs: 27 times in `KnownCharacters` (a presence mark) and 15 times in `GameState` (a Room change). A break before the history loses the whole history, which is the largest part of a long call.
- **known** The presence marks alone gain less than 1 point. When a Character enters or leaves the Room, `<CharactersPresent>` changes too, and it sits just after `<KnownCharacters>`, before the history. So the break moves only from one block to the next.
- **known** `GameState` after the history alone gains about 4 points. A Room change usually changes the Characters present too, so `<CharactersPresent>` still breaks the cache before the history.
- **known** Only the three changes together keep every pair's break inside or after the history: about 84 to 85 %, a gain of about 25 to 28 points.
- **inferred** `<CharactersPresent>` holds the full cards of the Characters in the Room. After the history, these cards sit near the end of the prompt, where the model gives text more weight. So this combination can change the output more than `GameState` alone.

## The Quantifier

- **known** The Quantifier calls share 43.8 % of their bytes with the previous Quantifier call (199 pairs). The median shared prefix is 3749 characters, which is about 940 tokens at the 4.0 characters for each token that the caching asset of Prompt flexibility measured. The median prompt is 2085 tokens.
- **known** The provider reported a cache hit on 29 of the 200 Quantifier calls, with 768, 896 or 1024 tokens.
- **known** The first byte that differs falls in `<RecentHistory>` in 157 of 199 pairs, in `<PreviousCharactersPresent>` in 25 and in `<CurrentRoom>` in 17.
- **inferred** The front of the Quantifier prompt is stable, but it is about as long as one cache block of the provider, so it is often too short for a hit. This is the question of [Find why the Quantifier calls read so little from the cache](../issues/02-quantifier-cache.md).
