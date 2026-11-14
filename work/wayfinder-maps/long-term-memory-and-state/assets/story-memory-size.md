# Evidence: the size of the story memory

Collected on 2026-11-12 from the checks of [Build the base story memory](../issues/14-build-base-story-memory.md). The code was the uncommitted build on `long-term-memory-2` (parent `4baba0a4`). The ticket that uses this evidence is [Decide how the story memory keeps its size](../issues/15-story-memory-size.md).

Each statement is marked known, inferred or guessed. The files are in [`story-memory-size/`](story-memory-size/).

## The problem

The story memory must hold a story of any length in a fixed space. The base rewrites one text at each history cut, from the previous text and the Messages that leave. So the text stays inside its space only if the model shortens the old part each time it adds a new part.

The model, `deepseek-v4.1-flash`, does not do this reliably (known). In some runs the text reaches the size limit at the fourth cut. In other runs it stays under half of the limit.

## The limits in the engine (known)

- `STORY_MEMORY_TOKENS` is 2,000 (`src/application/prompting/token_budget.rs`). It is the answer limit of the writer call and the space that the history cut reserves.
- `STORY_MEMORY_CHARS` is 8,000 (4 characters for each token). The texts that reached the limit had about 1,445 words and 8,440 characters.
- The writer prompt asks for at most 600 words (`STORY_MEMORY_WORDS` in `src/application/agents/memory/prompt.rs`).
- An answer that the provider stopped at the answer limit, or that is longer than 8,000 characters, is a failed writer call (`StoryMemoryWriter::check_answer` in `src/application/agents/memory/agent.rs`). The Job then keeps the previous story memory and its marker. The role health shows "Story memory failed".
- After a failed writer call, the Messages of that cut are in neither the history nor the story memory. At the next cut, the writer gets those Messages again together with the new ones.

## The sizes in two trials in the engine (known)

Both trials are 100 Cycles of `redmist_julian` at 16,000 history tokens. Each number is the whole story memory after one cut.

| Trial | Writer prompt | Words after each cut | Answer tokens after each cut |
|---|---|---|---|
| 1 ([`trial-1/`](story-memory-size/trial-1/)) | 600 words, no rule to shorten | 218, 504, 648, 870, 760, 1,105 | 321, 729, 923, 1,233, 1,126, 1,651 |
| 2 ([`trial-2/`](story-memory-size/trial-2/)) | 600 words and a rule to shorten (the prompt in the engine) | 288, 448, 531, 656, 513 | 413, 643, 761, 937, 724 |

- Trial 1 had 6 cuts (Cycles 41, 51, 61, 72, 82, 92). Trial 2 had 5 cuts (Cycles 45, 56, 68, 78, 90). No writer call failed, and the provider stopped no answer at the limit.
- Trial 1 ran before the review round. At that time the engine cut a long text at 8,000 characters and saved it. No text reached that size.
- In both trials the text became smaller once. So the model does shorten sometimes.
- No run in the engine has reached the limit. So only tests prove the failed-call path, not a trial.

## The sizes in the memory test (known)

The [memory test](memory-test.md) rewrites the memory at the 5 cuts of a stored story (`tmp/lt15/new/trial-1`, local only). It ran 3 times for each wording, at temperature 0. The words are the memory after each cut.

| Wording (candidate file) | Run | Words after each cut | Questions answered |
|---|---|---|---|
| 600 words ([`base_rolling_summary_v1`](candidates/base_rolling_summary_v1.json)) | 1 | 432, 344, 911, 563, 671 | 31 of 42 |
| | 2 | 356, 505, 1,021, 1,443, 1,443 | 34 of 42 |
| | 3 | 392, 278, 627, 604, 496 | 30 of 42 |
| 1,000 words and a rule to shorten ([`base_1000_shorten`](candidates/base_1000_shorten.json)) | 1 | 447, 822, 1,024, 1,394, 1,394 | 32 of 42 |
| | 2 | 450, 896, 1,356, 1,471, 1,469 | 32 of 42 |
| | 3 | the same text as run 1 | 32 of 42 |
| 600 words and a rule to shorten ([`base_rolling_summary`](candidates/base_rolling_summary.json), the prompt in the engine) | 1 | 423, 527, 765, 1,171, 1,447 | 34 of 42 |
| | 2 | the same text as run 1 | 34 of 42 |
| | 3 | 400, 444, 502, 753, 860 | 34 of 42 |
| Two sections, Earlier and Latest ([`base_two_part`](candidates/base_two_part.json), not kept) | 1 | 325, 591, 591, 591, 588 | 17 of 42 |
| | 2 | 348, 583, 573, 573, 573 | 19 of 42 |
| | 3 | 349, 662, 662, 662, 662 | 18 of 42 |

- The mean scores are 70 % (600 words), 71 % (1,000 words and the rule), 77 % (600 words and the rule) and 33 % (two sections). The controls are 0 % with no memory, 67 % with one summary of all cut Messages, and 95 % with the whole story. The percent counts only the 35 questions that the no-memory control answers wrong in every run. The rows of each arm are in [`memory-test/report.md`](story-memory-size/memory-test/report.md), which the tool wrote on 2026-11-12.
- **The tool does not fail at the limit.** It keeps a text that the provider stopped in the middle of a sentence, and it continues. The engine would count that answer as a failed writer call. So the scores of the runs that reached the limit are not the scores of the engine.
- **Five of the nine runs of the first three wordings reached the limit**, at the fourth or the fifth cut: run 2 of the first wording, all three runs of the second, and runs 1 and 2 of the third. Two of those five are copies of another run.
- **Runs repeat.** Two of three runs gave the same text for two of the wordings. The cause is not known. The temperature is 0, so the spread between runs is understated.
- **The wording with two sections held its size and stopped working.** From the second or third cut on, the model returned the previous memory unchanged and ignored the new Messages. The files of these runs were deleted. The sizes above come from the output of the run. The candidate file was written again from the session record with the same prompt text.

## What the text keeps and what it loses (inferred)

This is one reading of two texts of trial 2, not a measurement: [`02421-memory.md`](story-memory-size/trial-2/02421-memory.md) (first cut, 288 words, 23 Messages) and [`02605-memory.md`](story-memory-size/trial-2/02605-memory.md) (fifth cut, 513 words, 115 Messages).

- The whole first text became the first paragraph of the fifth. The list of suspects is gone (Marcus, the courier's link to Adler, Lisette, Louise, Jezebel). The plan to read the payment binders is gone.
- The time became less exact. "Day 1, morning (Julian arrived a few hours ago)" became "Day 1, late morning to afternoon". The text does not say when each scene happened.
- The text names who was present for some scenes and not for others.
- The newest part has the most detail. The last three paragraphs cover about 23 Messages. The first two cover about 90.

This agrees with the memory test, where the base loses points on small facts and on who learned what.

## What is not known

- How the size moves after the sixth cut. No run is longer than 100 Cycles.
- How often a run in the engine reaches the limit. Two trials did not.
- What the engine does in play after the first failed writer call. The next call gets more Messages, so a second failure is likely (guessed).
- Whether another model keeps the size. Every run used `deepseek-v4.1-flash` with no reasoning.

## The files

| Folder | Content |
|---|---|
| [`story-memory-size/trial-1/`](story-memory-size/trial-1/) | The 6 writer calls of trial 1 (prompt and answer of each) and the summary of the trial. |
| [`story-memory-size/trial-2/`](story-memory-size/trial-2/) | The 5 writer calls of trial 2 and the summary of the trial. |
| [`story-memory-size/memory-test/`](story-memory-size/memory-test/) | The memory text after each cut for 9 runs, the answers of each run (`runs/`), and the report. |

The full packs of the two trials are local only (`tmp/ltm14/redmist_julian/trial-1` and `tmp/ltm14/run2/trial-1`, 30 MB each).
