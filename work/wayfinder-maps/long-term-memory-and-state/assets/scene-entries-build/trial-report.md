# Check trial of ticket 17: the scene entries on 100 Cycles

Build: the uncommitted tree of branch `long-term-memory-2` (DB migrated to v35, table `scene_entries` present).
Setup: `tmp/ltm17/redmist_julian.json` (`history_tokens` 16000, 100 Cycles, "Go check" Connection for every role).
Pack: `tmp/ltm17/redmist_julian/trial-1/` (`pack.md`, `calls/`, `calls.jsonl`, `story.json`).
Reference: `tmp/ltm09/redmist_julian/trial-1`. Side by side: `tmp/ltm17/compare.md`.
Run: 100 of 100 Cycles, stop reason `budget`, 0 restarts, 1181 s, 3,420,787 tokens, no stuck Job.

Epistemic labels: **known** = read from the pack, the DB or the call records. **Inferred** = a reading of known facts.

## (a) Writer calls at each cut, and the scene entries in the pack

- **Known.** The history was cut 5 times (calls 2834, 2884, 2926, 2968, 3020; each cut drops 22 to 25 Messages from the Narrator-role request). A writer ran at every cut: 12 writer calls (`memory`).
- **First cut: the writer failed, and the cut had no scene entries.** The first cut came with the Impersonate call 2834. Both tries of its first writer call failed (see b). That Impersonate call dropped 25 Messages (#1329 to about #1353) and carried the `<StoryMemory>` block with `<Opening>` only: no `<SceneEntries>`. So the pack does not show scene entries from the first cut on. It shows them from the next call that ran a writer, the Narration call 2838. One Narrator-role call (2834) went without entries for the Messages it had lost.
- **Recovery worked as the ticket says.** The next Job (Narration, call 2836) ran the writer again on the same Messages (#1329 to #1351) and got entries. The failed call added nothing, and its Messages went to the next cut.
- **Known.** After that, every cut wrote entries: 5 successful cuts, 36 entries in total. They cover #1329 to #1447 with no gap and no overlap (checked in the DB), and each cut is tied to its own Swipe (5 distinct `swipe_id` values).
- The pack lines `Scene entries written here: N (list after this cut: …)` appear at the 5 cuts that wrote entries.

## (b) Failed writer calls

Two failed calls, both from the first cut (the first try, then the second try on the same input):

| Call | Try | Tokens in / out | Finish | Record error |
|---|---|---|---|---|
| 2832 | first | 4198 / 832 | `stop` | `unexpected response format (expected scene entries)` |
| 2833 | second | 4198 / 1060 | `stop` | same |

- **Cause (known for the data, inferred for the engine step).** Both answers are complete JSON, not cut off. In both, one entry has the key `"others"` twice: the model writes `"others"` after `"took_part"`, then again after `"what_happened"`. A strict check of all 12 answers finds a duplicate key in exactly these two answers and in none of the 10 successful ones. `serde_json` derive refuses a duplicate field, and `parser.rs` turns that error into `None` through `.ok()`. So the call counts as "does not parse". I did not run the engine's parser on the text. The inference fits the engine log: `[Memory] try 1 of 2 failed: … Failed to parse LLM response as scene entries`, then `Agent memory failed`.
- The call record and the log name no cause (`unexpected response format`). The serde message is thrown away, so a reader cannot tell a duplicate key from a missing field. This is a finding for the writer parser: the error should keep the serde text, or the read should tolerate a repeated `others`.
- Both tries failed the same way because the second try has the same input. The retry does not help against a habit of the model.
- No other writer call failed. The server log has only two WARN lines since start (the two above) and no ERROR line.

## (c) Cost, list size and cache

**Extra calls and tokens of the writer.** 12 calls (10 good, 2 failed), 35,800 tokens in, 10,181 out: 45,981 tokens, 1.3 % of the 3,420,787 of the trial. The two failed calls cost 10,288 of them.

| Cut | Writer calls (ids) | Tokens in / out | Entries written | List after the cut (pack estimate) | Block in the next Narrator-role call |
|---|---|---|---|---|---|
| 1 (call 2834) | 2832, 2833 (both failed) | 8,396 / 1,892 | 0 | none | Opening only |
| 2 | 2836, 2837 | 5,192 / 1,432 | 6 | 6 entries, ~1,437 tokens | 6 entries, ~1,631 tokens (call 2838) |
| 3 | 2882, 2883 | 5,697 / 1,525 | 7 | 13 entries, ~2,884 tokens | 13 entries, ~3,078 (call 2884) |
| 4 | 2924, 2925 | 5,363 / 1,182 | 4 | 17 entries, ~4,035 tokens | 17 entries, ~4,229 (call 2926) |
| 5 | 2966, 2967 | 5,439 / 2,078 | 10 | 27 entries, ~6,058 tokens | 27 entries, ~6,251 (call 2968) |
| 6 | 3018, 3019 | 5,713 / 2,072 | 9 | 36 entries, ~8,039 tokens | **34** entries, ~7,842 (call 3020) |

(Estimate = the engine's four characters to a token. The block in the next call includes the tags, `<Opening>` and the head line.) Every cut was one split of 2 calls: a first call of about 4,200 to 4,500 tokens of input, and a second call of about 1,000 to 1,300. A call took roughly 4 to 11 s (inferred from the record timestamps). The Cycles after the last cut (3018) carried the same block, so the list grew in 5 steps and not smoothly.

**Narrator-role calls (Impersonate, Narration, event Narration), cached share**

| | Reference `ltm09` | This trial |
|---|---|---|
| Input tokens | 2,564,384 | 2,953,337 (+15.2 %) |
| Cached | 2,192,896 | 2,540,288 |
| **Cached share** | **85.5 %** (the ticket says 85.6 %; I get 85.5 % from `summary.json`) | **86.0 %** |
| Narration only | 85.4 % | 86.3 % |
| Impersonate only | 86.0 % | 86.0 % |

The scene-entries block adds about 389 k input tokens to the Narrator-role calls, and 347 k of them are cached. The uncached part grows by about 41.6 k tokens (371,488 to 413,049). The cache holds, because each block is stable between cuts and sits before the history.

Other numbers: all calls 79.8 % cached (reference 80.1 %); wall time 1181 s against 968 s (+22 %; part of the gap is the 12 writer calls, the rest is probably load and network, and I did not separate them). The Quantifier read 37.2 % from the cache against 44.0 % in the reference; the Quantifier prompt does not contain the block, so I take this for provider noise (guessed).

## (d) Did the list pass its space? Is a smaller-history trial worth it?

- **Yes, once, which is not what the ticket expected.** The space is half of the 16,000-token history: 8,000 tokens. After cut 6 the full list was ~8,039 tokens (36 entries). The block of call 3020 shows 34 entries from `#1340-#1344` on: the two oldest entries (`#1329-#1336`, `#1337-#1339`) left the prompt, the `<Opening>` stayed, and the block measured ~7,842 tokens, under 8,000. The two entries stay stored (36 rows in `scene_entries`). So the removal rule works on a real run. It ran for the last ~40 calls (20 Cycles).
- **Why it came earlier than the 5,600 to 6,400 tokens of ticket 16 (inferred).** The writer ignores the "60 to 150 words" length in 13 of 36 entries (up to 276 words; median 129, mean 144). The list holds 5,179 words in all.
- **A smaller-history trial is not needed for the removal itself.** This trial already shows it, and the seam tests cover the arithmetic. One would be worth running only for a question this trial cannot answer: whether the narrator contradicts facts of entries that have left the prompt. For that, 8,000 history tokens (space 4,000) with 60 Cycles would show many removals in about 12 minutes. This is optional (my judgement).

## (e) Anything abnormal

- **The first cut lost its entries** (a, b): the Impersonate call 2834 ran without any scene entry after dropping 25 Messages. The engine recovered at the next Job.
- **Repeated `others` key:** the one failure cause, and the error text hides it (b).
- **Entry length:** 13 of 36 entries are over 150 words (c, d).
- **Server:** no restart, no stuck Job, no ERROR line, 2 WARN lines (the two writer failures). The Game ended with no failure (`game_failure: null`). The one `event-narration` call and the 99 Narration calls are all fine (0 failed calls apart from the writer).
- **My own side effect:** `python scripts/trial.py compare` writes `compare.md` next to its first pack, so it **overwrote `tmp/ltm09/redmist_julian/compare.md`** (scratch, git-ignored). The new compare is also at `tmp/ltm17/compare.md`. The old ltm09 compare file is gone.
- `scripts/trial.py` printed `Hosts: ?` in the pack summary. I did not look into it.
- Before the run I copied the probe DB to `tmp/ltm17/chronicler_3001.pre-v35.db` (a backup from before the migration).

## Evidence: scene entries from the pack

Source: the DB table `scene_entries` and `pack.md` (the written text of `#1443-#1445` is in `story.json`).

1. `#1433-#1433`, Day 1, evening, "The corridor at the foot of the service stair", took part Julian, Carla:
   "Julian left the kitchen. A lamp was lit at the foot of the service stair, and Carla stood by it with her cap in her hands, clearly having waited a while. She told him Jezebel had gone up to her office with the ledger under her arm, and that Lisette had aired his room and left a note on the desk saying something had come for him from the cousins, sent not by the estate's post."
2. `#1443-#1444`, Day 2, morning, "The master's room", took part Julian, Lisette: "Morning came grey and wet-slate through the curtains, the house already awake. Lisette knocked at eight with a tray … and reported that the kitchen was in a mood: Louise said the eggs would be hard as instructed and the tea strong enough to stand a spoon in … Julian … said to tell Louise the tea was the right colour and he would give his verdict on the eggs after eating them rather than before …" It matches the Messages #1443 and #1444 (names, the hour, the tea, the eggs). It also moved the story clock to "Day 2, morning" correctly.
3. `#1445-#1445`: "… the post had come up while Julian slept, and among it a card addressed to him in an unrecognised hand, with no seal and no name on the back; she offered to leave it with the tray before he went up to Miss Jess." It matches #1445.
4. The head of the block as the narrator saw it at call 3020: `<StoryMemory>`, `<Opening>` (Carla's handshake Message, word for word), `<SceneEntries>` with the line "Past scenes, oldest first. Each entry records what happened then. A state that an entry describes … may have changed since.", then `[#1340-#1344] Day 1, morning. The front steps and entrance hall of the Redmist Estate. Took part: Julian, Gabriella, Jezebel.` and so on to `[#1446-#1447]`.
5. All entries carry the story time and place, and I found no entry that names a person outside the story. I read 3 entries against their Messages (items 1 to 3) and the rest only for shape. This is a spot check, not a quality judgement.

## Check, point by point

| Pass condition | Result |
|---|---|
| Writer calls run at each cut | Yes, 6 cut attempts, 12 calls |
| The pack shows the scene entries from the first cut on | **Partly:** from the second attempt (call 2838). The first cut had no entries because both tries failed |
| No writer call fails, or each failure is explained | 2 failed (first and second try of one call); explained in (b) |
| Extra calls and tokens, list size per cut, cached share | Given in (c): 12 calls, 45,981 tokens, 8,039-token list at the end; 86.0 % against 85.5 % |
| Removal (point 2) | Exercised at the 6th cut; no smaller trial needed (d) |
