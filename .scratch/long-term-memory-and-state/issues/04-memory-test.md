# Make the memory test from the stored stories

Type: task (AFK)
Status: resolved
Blocked by: 02, 03

## Question

Make the memory test: a tool that scores a memory candidate on stored trial stories, with no engine change. It shows which facts a candidate keeps and which facts it loses, and it gives the same result each time it runs.

## Context

- **Why (owner, charting).** A memory candidate first runs the memory test, and only a candidate that passes gets a full trial. The model for this tool is [`prefix_measure.py`](../../layout-trials/assets/prefix_measure.py) of Layout trials. It measured each cache candidate on a stored pack before any trial.
- **Inputs.**
  - The fact list of [Count the memory faults of the newest trial](03-baseline-faults.md).
  - The benchmark methods of [Find what is known about long-term memory for LLMs, and what is still open](02-llm-memory-research.md), point 2.
- **The stored stories (known).** `tmp/lt15/new/trial-1` is the newest. Older ones are `tmp/lt15/reference/trial-1`, `tmp/lt01/redmist_julian/trial-1` and `tmp/lt03/redmist_julian/trial-1`. Each pack holds `story.json`, `calls.jsonl` and the stored request of each call under `calls/`. `scripts/trial.py` reads them (`export`, `compare`).
- **The model.** `deepseek-v4.1-flash` on OpenCode Go, the model of every trial (map Notes).
- **The key (known constraint).** An agent may not read `.env` files, or read or print a key. The probe server loads `.env` itself, and no engine route sends a free prompt to a Connection. Ask the owner how the test reaches the model before you build that part. Offer these options:
  - (a) the owner exports the key into the shell that runs the test;
  - (b) the test loads `.env` at run time and never prints the key.

## Points to decide and build

This ticket decides and builds a tool, not engine code.

1. **The cut point.** A question is asked at a given entry. The test gives the history that the Narrator-role call at that entry held, plus the memory that the candidate wrote from the entries dropped before that call.
2. **The question prompt.** Either the stored request of that call with the memory block and the question as the last user turn, or a short prompt with only the memory and the question. The first is closer to play, and the second is cheaper. Measure both on a few questions, and choose.
3. **The candidate.** How a candidate plugs in: a writer prompt and a schedule (when it writes, and from which entries). The tool runs the writer calls on the stored entries itself.
4. **The score.** Each question has a fixed answer. Check an answer by a rule where possible, and by a grader call where not. Report the score for each kind of question: a fact, the time, and who knows what.
5. **The controls.**
   - No memory, only the history: the test must fail on the facts that left the history.
   - The whole story in the prompt, with no cut: this is the upper limit.

   If the no-memory control answers a question right, the question tests nothing. Drop it.
6. **The noise.** Run each control at least 3 times, and report the spread. Use temperature 0 where the provider allows it.
7. **The cost.** Report the calls and the tokens of each run.

The tool lives in `assets/` of this map, as `prefix_measure.py` does in Layout trials. It does not go in `scripts/`.

## Output

The tool, and an asset `assets/memory-test.md` with:

- how to run the tool, and how to add a candidate;
- the question set;
- the score, the spread and the cost of each control;
- the score of one simple candidate: one summary call over all the dropped entries.

Done when the two controls run on the newest pack, the no-memory control fails on the facts that left the history, the whole-story control answers most of them, and the simple candidate runs from start to end.

## Answer

Resolved 2026-11-03. The tool is [`memory_test.py`](../assets/memory_test.py), and the asset is [the memory test](../assets/memory-test.md).

- **The key (owner).** Option (b): the tool reads only `OPENCODE_GO_API_KEY`, from the environment or else from `.env`, and never prints or stores it.
- **The cut point.** A question goes into the stored request of the first Narration entry at or after its `ask_at` entry. Every entry before the oldest held entry is dropped.
- **The question prompt.** The full shape: the stored request, with the memory block, and the question as the last user turn. The short shape costs about 20 times fewer prompt tokens, but its no-memory control cannot drop a question that the history or a card answers. It stays as a cheap screen.
- **The candidate.** A JSON file: a writer prompt, a schedule (`once` or `each_cut`) and a position (`before_history` or `after_history`).
- **The score.** A pattern rule for each fact and time question. Yes or No for a who-knows-what question. A grader call for an answer that hedges but holds the fact. The report gives the score for each kind.
- **The controls** (35 kept questions of 42, 3 runs each): no memory 0 %, the whole story 95 % (94 to 97 %). The no-memory control dropped every "No" who-knows-what question, so the set tests only the "knows" side.
- **The noise.** Temperature 0 does not give the same answers each time. The simple candidate ranged from 57 to 77 % over 3 runs.
- **The simple candidate** (one summary of at most 600 words over all the dropped entries) scores 67 %. It loses who did a small thing, reasons, small details, and who learned a fact.
- **The cost.** For each run of 42 questions: no memory about 642,000 prompt tokens, the whole story about 1,206,000, and the simple candidate about 733,000 with 5 writer calls. 60 to 90 % of these tokens were cached.
- **Changes to the fact list.** F39 now asks which day of the stay it is. W31 and W35 come from the confirmed faults. W33 and W34 were added and taken out again: they are card leaks, not memory.
