# The memory test

Source: [Make the memory test from the stored stories](../issues/04-memory-test.md), 2026-11-03.

The memory test scores a memory candidate on a stored trial story, with no engine change. It asks questions whose answers left the history, at the entry where they left. Each question goes into the stored Narration request of that entry. Only the memory part of the request changes from arm to arm.

- The tool: [`memory_test.py`](memory_test.py).
- The question set: [`memory_test_questions.json`](memory_test_questions.json), 42 questions.
- The simple candidate: [`candidates/one_summary.json`](candidates/one_summary.json).
- The raw results: `tmp/ltm04/` (`runs/` for the answers and the cost, `memory/` for the memory that each candidate run wrote, and `report.md`). `tmp/` is not in git.

## Summary

- **known** The two controls behave as the ticket requires. On the 35 kept questions, the no-memory control scores 0 % in all 3 runs. The whole-story control scores 95 % (94 to 97 %).
- **known** The simple candidate (one summary of at most 600 words over all the dropped entries) scores 67 %, with a spread of 57 to 77 % over 3 runs. It keeps the plot facts and most times. It loses who did a small thing (who took a call, who lit a fire), reasons, small details (a dish), and who learned a fact.
- **known** Temperature 0 does not make the answers the same each time. The no-memory control changed its answer on 1 question in 3 runs. The candidate changes more, because each run writes its own memory. A difference of less than about 10 points between two candidates is inside the noise of 3 runs (inferred from the 20-point range of the candidate).
- **known** The full shape is the score. The short shape (only the memory and the question) costs about 20 times fewer prompt tokens. It cannot find the questions that the history or the cards already answer, so it is a cheap screen of a memory text, not a score.
- **inferred** The set tests only the "knows" side of who knows what. With no memory, the model answers "No" to every "Can X know …?" question, so the control drops all of them.

## How to run it

Run from the repo root. The tool reads `OPENCODE_GO_API_KEY` from the environment, or else from the one line of `.env` that sets it. It never prints or stores the key. It sends `reasoning_effort: none` and temperature 0, as in the trials, to `deepseek-v4.1-flash` on OpenCode Go.

```bash
T=.scratch/long-term-memory-and-state/assets/memory_test.py
python3 $T check                       # the call, the cut and the oldest held entry of each question
python3 $T run none --runs 3           # the no-memory control
python3 $T run whole --runs 3          # the whole-story control
python3 $T run .scratch/long-term-memory-and-state/assets/candidates/one_summary.json --runs 3
python3 $T report --detail             # the score, the spread and the cost of every arm
python3 $T regrade                     # score the stored answers again after a rule change
```

Other flags of `run`: `--shape short`, `--only F01,F07`, `--pack <trial pack>`, `--out <folder>`, `--workers 6`, `--first-run 4` (to add runs 4 and up). A run skips the questions that it already answered. So a run that stopped can start again.

One run of 42 questions takes about 3 minutes with 6 workers. The three arms can run at the same time.

## How it works

1. **The cut point.** A question has an `ask_at` entry. The tool takes the stored request of the first Narration entry at or after it. That request holds the history that the engine sent. The tool finds the oldest entry that the history held. Every entry before it is dropped. `check` prints both, and it warns when an entry that states the fact is still in the history.
2. **The arms.**
   - `none`: the stored request as it is.
   - `whole`: the dropped entries go back at the front of the history, one message each, in the roles of the ME format (an Impersonate entry is `user`, a Narration entry is `assistant`).
   - A candidate: the stored request with the candidate's memory in a `<StoryMemory>` block. `before_history` puts it at the end of the user message before the history. `after_history` adds it as a user message after the history.
3. **The question.** The tool adds one user message at the end: an out-of-character mark, "use only what the story so far says", "if it does not say, answer I don't know", and the question. A who-knows-what question asks for Yes or No first.
4. **The score.** A fact or time question has `accept` patterns. The answer is right if a pattern matches. An answer with "I don't know" and no match is wrong. An answer with "I don't know" and a match (for example "I don't know — the drawer was empty") goes to a grader call, which replies RIGHT or WRONG. A who-knows-what question is right when the first word (Yes or No) matches. A question with no `accept` goes to the grader. In these runs the grader decided 7 answers.
5. **The kept questions.** A question is kept only when the no-memory control answers it wrong in every run. The other questions test nothing, and the report shows them as "(dropped)".

## How to add a candidate

A candidate is a JSON file with these fields:

| Field | Meaning |
|---|---|
| `name` | The name of the result files. |
| `schedule` | `once`: one writer call over all the dropped entries at each ask point. `each_cut`: one writer call at each history cut, with the previous memory and the entries of that cut only (a rolling memory). |
| `position` | `before_history` or `after_history`. |
| `writer_system`, `writer_user` | The writer prompt. `writer_user` can use `{entries}` (the entries as `[#id Narration]` or `[#id Julian]` and their text), `{previous}` (the previous memory, `each_cut` only), `{first}` and `{last}`. |
| `max_tokens` | The cap on the writer answer. The default is 2048. |

The tool keeps each written memory in `memory/<name>-r<run>/`. The questions of one run that share a dropped range share one memory. A new run writes a new memory, so the spread includes the noise of the writer.

A candidate that needs more than a prompt and a schedule (for example, a memory for each Character, or a list of who was present) needs a new schedule in `Memories.for_call`. Keep the result one text for the `<StoryMemory>` block, or add a position.

## The question set

The source is the fact list of story A in [the baseline asset](baseline-faults.md): 40 facts, made from the dropped entries. Changes:

- **F39** asked "How many nights has Julian slept at the estate?". The whole-story control answered "I don't know" 3 times: the story never counts nights. The question is now "Which day of Julian's stay is it?", and the answer is "the second day".
- **Four access questions from the confirmed faults** were added as `W31`, `W33`, `W34` and `W35`. W33 and W34 were taken out again. They are card leaks, not memory: "Petit" first appears at #617 in Julian's own line, and the call at #618 holds that line. W31 and W35 stay in the set, but the control drops them, because their facts were still in the history.
- **F14** stays. The whole-story control answers "No" 3 times, but the story supports "Yes": at #581 Julian says "And this, under his cushion, with your name on it." The upper limit is below 100 % because the model misreads.

The whole-story control answers 21 of 21 kept fact questions in every run. So the fact rows are supported by the story (inferred: a right answer from the whole story is a check of the row).

| Kind | Questions | Kept | Dropped by the no-memory control |
|---|---|---|---|
| fact | 21 | 21 | — |
| time | 11 | 10 | F10 ("twelve years" is still in the history at #634) |
| who knows what | 10 | 4 | F11, F15, F28, F38, W31, W35: every "No" question |
| all | 42 | 35 | 7 |

## The result

Shape `full`, the newest pack (`tmp/lt15/new/trial-1`), 3 runs for each arm. "Kept" columns give the right answers of each run.

| Arm | Kept: fact | Kept: time | Kept: who | Kept: all, mean (min to max) | Prompt tokens per run | Calls per run | Memory words |
|---|---|---|---|---|---|---|---|
| none | 0/21, 0/21, 0/21 | 0/10, 0/10, 0/10 | 0/4, 0/4, 0/4 | 0 % (0 to 0) | 642,000 | 45 | 0 |
| whole | 21/21, 21/21, 21/21 | 10/10, 9/10, 9/10 | 3/4, 3/4, 3/4 | 95 % (94 to 97) | 1,206,000 | 45 | 0 |
| one_summary | 12/21, 15/21, 11/21 | 9/10, 10/10, 7/10 | 2/4, 2/4, 2/4 | 67 % (57 to 77) | 733,000 | 50 | 503 |

The cost columns:

- They count every call of a run, with the 2 questions that were later taken out and the F39 rerun. So a run of the 42 questions costs a little less.
- About 60 to 90 % of the prompt tokens were cached. The stored requests of one ask point share their prefix.
- `one_summary` adds 5 writer calls for each run (5 dropped ranges) and about 90,000 prompt tokens (inferred: the writer calls and the memory block).
- The completion tokens are small: about 1,000 for each run of a control, and about 4,300 for a candidate run with its writer calls.

What the simple candidate lost in every run (from the answers):

| Question | What it lost |
|---|---|
| F19 | Why Tomas refused a morning meeting (a reason). |
| F22 | Who took Campion's call. The memory says only "Campion rang". |
| F24 | Who had the sitting room prepared. |
| F27 | Where Campion keeps the codicil. |
| F34 | Louise's dish. The memory says only "Louise is coming up about supper". |
| F23, F36 | That Lisette knows about the call, and that Tomas knows the codicil. The memory records the act, not who learned what. |

The writer prompt asked for "who was present at each conversation, and who learned which fact". The memory of 500 words did not keep it.

### The two prompt shapes

12 kept questions, 1 run, with the memory of run 1 of `one_summary`:

| Arm | Full shape | Short shape | Prompt tokens, short |
|---|---|---|---|
| none | 0 of 12 | 0 of 12 | 1,300 |
| whole | 12 of 12 | 11 of 12 | 301,000 |
| one_summary | 5 of 12 | 7 of 12 | 9,300 |

The full shape of the candidate costs about 210,000 prompt tokens for the same 12 questions (inferred from the run cost). The short shape is kinder to the candidate: with no history around it, a vague memory line answers F06 and F16. In the short shape the no-memory control has nothing to read, so it cannot drop a question that the history or a card answers. **Choice: the full shape is the score.** The short shape can screen many writer prompts cheaply before a full run.

## Limits

- **One story.** All questions come from story A. Story B and the older packs have no question set. A question set for a second story needs a fact list like the one in the baseline asset.
- **The who-knows-what side "No" is not tested.** A plain Yes or No question cannot test it, because the model with no memory says No. FANToM asks one fact in several forms ("list who knows X") and counts a fact right only when every form is right. The Curse paper uses twin scenes. Either one is a later change to the set.
- **The set tests recall, not use.** A right answer to a question does not show that the narrator uses the fact in play. The check trial measures use.
- **The questions are not blind.** I wrote the `accept` patterns after I read the answer column of the fact list. Some rules are wide. For example, F26 accepts any "three" or "3" in the answer. So a rule fault can still exist. `report --detail` and the `given` field of each row show every answer.
- **The fact rows were not checked one by one.** The whole-story control answers every kept fact question, which supports them (inferred).
