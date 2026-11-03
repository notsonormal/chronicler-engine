# Count the memory faults of the newest trial

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

How many of the three memory faults does the newest trial story have? Which facts did the story lose when the history cuts dropped its oldest entries?

## Context

- The three memory faults (map Notes) are sections 1, 5 and 6 of [the inconsistencies of two trial stories](../../layout-trials/assets/story-inconsistencies.md): a fact that had left the history, no story clock, and who knows what.
- **The newest story (known).** `tmp/lt15/new/trial-1` is the trial of the build of [Try the history in the ME format](../../layout-trials/issues/15-history-in-me-format.md). It has 100 Cycles and 201 entries. Five history cuts dropped 114 entries by the end. Its copy for a judge is `tmp/lt15/judge/story-Y.md`, and that copy keeps the "history holds" line of each entry.
- **A first count exists (known).** The judges of ticket 15 used a prompt with 11 fault kinds (`tmp/lt15/judge/prompt.md`). For the three memory faults they counted:
  - story Y (the newest story): 1, 4 and 6 (`tmp/lt15/judge/judge-Y.md`);
  - story X (`tmp/lt15/reference/trial-1`, the code before the ME format): 3, 2 and 3 (`judge-X.md`).

  A prompt with 11 kinds can miss cases of one kind (guessed).
- **The judge method.** [Decide how to read an output trial](../../layout-trials/issues/05-read-an-output-trial.md): a `reviewer` subagent reads the whole story alone, with neutral names. The coordinator checks the findings against the story and the data before the owner reads them.
- The faults of the older stories P and Q (`tmp/lt01`, `tmp/lt03`) in the inconsistencies asset are more examples of the same kinds.

## Steps

1. Write a judge prompt for the three memory faults only. Use the definitions of the inconsistencies asset, and the cause check by the "history holds" lines. Ask for every instance, with the entry numbers of both ends, a short quote, and the history count at the later entry.
2. Run two `reviewer` judges on `story-Y.md`, in two fresh sessions, and one on `story-X.md`. The two runs on one story measure the noise of the judge itself.
3. Check every finding against the story and the pack. Drop the false ones, and merge the runs.
4. **List the facts that the story lost.** From the entries that the history cuts dropped, list 20 to 40 facts that a later entry needs or could need. Examples: names, numbers, objects and where they are, promises, who was present at a talk, and time markers (the day, the time of day, "two weeks ago"). For each fact, give:
   - the entry that states it;
   - the first call whose history no longer holds it;
   - a question and its answer;
   - its kind: a fact, the time, or who knows what. For a who-knows-what question, name the Character and say whether the Character can know the fact.

   Each checked fault of step 3 is also a fact on the list.

## Output

An asset `assets/baseline-faults.md` with:

- the judge prompt;
- the count table for the three memory faults: each run on Y, the merged and checked count on Y, and the count on X;
- how many findings the two runs on Y share;
- each checked fault, with its entries, a quote and the history count;
- the fact list, as a table. [Make the memory test from the stored stories](04-memory-test.md) reads it and chooses its own file format.

If a merged count is below about 5, say that the check trial cannot show a change of that kind on its own (inferred).

## Answer

Resolved on 2026-11-03. Asset: [The memory faults of the newest trial, and the facts it lost](../assets/baseline-faults.md). Work files: `tmp/ltm03/`.

| Memory fault | A, run 1 (raw) | A, run 2 (raw) | A, checked | B (raw) | B, checked |
|---|---|---|---|---|---|
| A fact that had left the history | 1 | 2 | 0 | 2 | 0 |
| No story clock | 12 | 10 | 4 | 3 | 1 |
| Who knows what | 6 | 4 | 5 | 4 | 3 |

- **known** Neither story has a confirmed fact that had left the history. Each kind-1 finding broke a fact that the history had already changed, or broke no fact.
- **known** Eight clock findings on A were date conflicts inside the history (the year of the Draught). They are fault kind 2 of the inconsistencies asset, which is out of scope, so the check moved them. With the wording of the judge prompt, the clock count is 12 for A and 2 for B.
- **known** The two runs on A found the same 4 clock faults, and 3 of the 5 who-knows-what faults. The raw counts (19 and 16) are more than twice the checked count (9), so a trial needs the check step.
- **inferred** The story loses old facts by dropping threads, not by contradicting them. A contradiction count does not see this, but the fact list does.
- **inferred** A memory of the dropped entries fixes none of the confirmed faults. Several come before the first cut. The likely fixes are a story clock, a record of who was present at each talk, and a rule that keeps card facts from a Character who has not learned them. Three who-knows-what faults are card leaks: Julian knows Louise's surname and Lisette's years of service, which only the cards state.
- **known** The fact list has 40 facts from the dropped entries of A: 21 of kind fact, 11 of time and 8 of who knows what. The asset marks five rows to treat with care.
- **inferred** Each checked count is 5 or less, so the check trial of the map Destination cannot show a change of one kind on its own.
