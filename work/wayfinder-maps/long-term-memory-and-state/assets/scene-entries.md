# The scene entries, measured

Source: [Decide the form of the scene entries](../issues/16-scene-entries.md), 2026-11-22. This asset holds the measurement. The ticket's Answer and the owner's go or no-go are not here.

## Summary

- **known** The scene entries score 92 % (89 to 97 %) on the 35 kept questions over 3 runs. The base scores 77 %, and the whole story 95 %. The gap to the base (15 points) is above the noise of about 10 points. The gap to the whole story (3 points) is inside it.
- **known** Every time question is right in every run (10 of 10). 20 of the 21 fact questions are right in at least 2 of 3 runs. F19 (a reason) is right in 1 of 3.
- **known** Only one question is wrong in every run: F14, "Does Marguerite Hollis know that the key was found under a cushion?". The whole-story control also gets it wrong in every run.
- **known** The form holds. The tool read 15 of 15 writer answers. No call left a gap or an overlap in its ranges. One name of 188 was not a cast name ("Tomas", once).
- **known** No writer call failed in the 3 runs. Each call held about 3,800 to 4,100 tokens of Messages and answered 970 to 1,680 tokens, under the cap of 2,000.
- **known** The list is large. After the fifth cut it holds 5,600 to 6,400 tokens for 57 Cycles of story, about a third of the Messages it covers. One scene entry is about 220 tokens. The writer splits a cut of about 22 Messages into 4 to 10 scene entries.
- **inferred** At the default history size (65,536), one writer call cannot hold a whole cut under a cap of 2,000. A probe call over 15,600 tokens of Messages was cut off at 2,000 tokens. With a cap of 8,000 it answered 3,120 tokens. So the engine must split a large cut into several writer calls, or raise the cap.
- **known** All seven lost facts of the two pilot stories are in the scene entry of the scene where they first appear (one run, with cuts made by the probe, not by the engine).
- **inferred** The questions that the scene entries still lose are mostly "who learned what" (3 of 5 questions with a miss). Exact values are kept. This points to the Character memory as the third list, but the evidence is thin: one question lost in every run, four lost in one or two runs.

## What I ran

From the repo root. The tool reads the key itself.

```bash
T=work/wayfinder-maps/long-term-memory-and-state/assets/memory_test.py
C=work/wayfinder-maps/long-term-memory-and-state/assets/candidates/scene_entries.json
python3 $T write $C tmp/ltm16/probe-a                     # one form probe, 5 writer calls
# the controls, the base and one_summary: copied from tmp/ltm04/runs/ into tmp/ltm16/runs/ (not rerun)
for r in 1 2 3; do python3 $T run $C --first-run $r --runs 1 --out tmp/ltm16 & done; wait
python3 $T report --out tmp/ltm16 --detail                # -> scene-entries/report.md
python3 $T scenes tmp/ltm16/memory/scene_entries-r{1,2,3} --markdown <assets>/scene-entries/entries-r1.md
python3 tmp/ltm16/big_call.py                             # the large-call probe (2 calls)
python3 tmp/ltm16/lost_facts.py tmp/lt01/redmist_julian/trial-1 tmp/ltm16/lt01 400
python3 tmp/ltm16/lost_facts.py tmp/lt03/redmist_julian/trial-1 tmp/ltm16/lt03 600
```

The first run of the three failed on a fault of my tool change (the `each_cut` branch ran first). I fixed it, deleted those results and ran again. The numbers here are from the second start.

## The entry form and the writer prompt

The candidate: [`candidates/scene_entries.json`](candidates/scene_entries.json). The tool schedule: `append_cut` ([the memory test](memory-test.md)).

**What the writer gets.**

- The Messages of the cut, as `[#id Julian]` or `[#id Narration]` and their text.
- The names of the cast: the Persona, then the Characters of the World, read from `data/`. Why: the ticket rules out the Characters present of the Swipe state as a source.
- The last scene entry written before, as one line, marked "only for the story time and the place, do not repeat it". Why: a cut can start inside a scene, and the Messages often do not say the day. Without the earlier entry, the writer cannot count the days. The entry is read, not rewritten, so the rule "no text is rewritten" holds.
- Nothing else. No earlier memory, no cards, no state.

**What one scene entry holds.** One JSON object for each scene, in `{"entries": [...]}`:

| Field | Meaning | Why |
|---|---|---|
| `from`, `to` | The first and the last Message of the scene | The draft of the ticket. It also makes the gap and overlap check possible. |
| `time` | The day of the story and the time of day, for example "Day 1, evening" | The draft. Time questions were the weak point of one_summary. |
| `place` | Where the scene happens | The draft. |
| `took_part` | Cast names only | The draft and the Marinara rule: "A character merely mentioned, remembered, discussed, or addressed while absent is NOT a participant." I added "or spoke in it by phone", because a phone call is a scene of story A. |
| `others` | People outside the cast who took part, by the story's name | Story A has people outside the cast who act (Tomas, Campion, Marguerite). Without this field the writer must put them in `took_part`, and the name check fails. |
| `what_happened` | 60 to 150 words, past tense | The draft and the Marinara wording: "Record what happened and its outcomes without treating past states as current." The list of what to keep (names, numbers, objects and where they are, promises and times, reasons, who learned what) comes from the losses of one_summary and the base. |

**Why JSON.** The form checks need the ranges and the names as fields. A JSON answer that the limit cut off does not parse, so a cut-off answer cannot pass as a short good one.

**What the narrator sees.** One `<StoryMemory>` block at the end of the user message before the history, as the base has it:

```
<Opening>
(the first Message of the story, word for word)
</Opening>

<SceneEntries>
Past scenes, oldest first. Each entry records what happened then. A state that an entry describes (a place, a mood, an object, a plan) may have changed since.

[#524-#532] Day 1, morning. The gravel drive outside the Redmist Estate gate. Took part: Julian, Carla.
(what happened)
...
</SceneEntries>
```

The head line is the "past states, not current" rule for the narrator.

**The rules of the engine in the tool.** A finish reason of `length` is a failed call. An answer that does not parse is also a failed call: the engine could not store it (my choice; the spec names only the cut-off case). A failed call adds nothing, and its Messages go to the next call. One call holds at most one history space of Messages (16,000 tokens). No call failed in the test, so these rules were not exercised by the score.

**No revised prompt.** The brief allowed one revision for a clear form failure. There was none: all answers parsed, every range was whole, one name was wrong. One soft fault remains: 29 of 82 entries pass 150 words (the longest has 362). I did not try a revision for it.

## The score

Shape `full`, story A (`tmp/lt15/new/trial-1`), 35 kept questions. The controls, the base and one_summary are the stored runs of [Build the base story memory](../issues/14-build-base-story-memory.md) and the memory test.

| Arm | Fact (21) | Time (10) | Who (4) | All kept, mean (min to max) |
|---|---|---|---|---|
| none | 0, 0, 0 | 0, 0, 0 | 0, 0, 0 | 0 % (0 to 0) |
| one_summary | 12, 15, 11 | 9, 10, 7 | 2, 2, 2 | 67 % (57 to 77) |
| base_rolling_summary | 16, 16, 15 | 9, 9, 10 | 2, 2, 2 | 77 % (77 to 77) |
| **scene_entries** | **21, 19, 20** | **10, 10, 10** | **3, 2, 2** | **92 % (89 to 97)** |
| whole | 21, 21, 21 | 10, 9, 9 | 3, 3, 3 | 95 % (94 to 97) |

**inferred** The base figure flatters the base. Its runs used the old tool, which kept a text that the limit cut off. Two of its three runs reached the 2,000-token limit at the fifth cut ([Evidence: the size of the story memory](story-memory-size.md)). Under the engine rule those calls fail.

**known** The scene entries win where the base lost small acts and places: F22 (who took Campion's call), F24 (who had the sitting room prepared), F27 (where Campion keeps the codicil), F23 (that Lisette knows about the call). The base got these wrong in every run, and the scene entries got them right in every run.

The full table of every question is in [`scene-entries/report.md`](scene-entries/report.md).

## The questions that the scene entries still miss

Sorted by what was lost. "Writer" means the scene entries do not hold the fact. "Narrator" means they hold it and the answer still failed.

| Kind of loss | Question | Runs wrong | Where it was lost |
|---|---|---|---|
| Who learned a fact | F14: Does Marguerite know the key was under a cushion? | 3 of 3 | Writer. No entry says that Julian showed her the key "under his cushion" (#581). The whole story also fails it, so the question is weak. |
| Who learned a fact | F12: Does Jezebel know about the odd sign-in? | 1 of 3 | Narrator. The entries say both that Julian and Carla would keep it from her for a day and that Julian told her. The narrator took the first. |
| Who learned a fact | F36: Does Tomas know what the codicil says? | 1 of 3 | Narrator. The entries say that Carla would take the codicil to Tomas, not that he read it. |
| A reason | F19: Why did Tomas refuse a morning meeting? | 2 of 3 | Writer. Runs 2 and 3 kept "mornings were no good" and dropped the queue of shopkeepers. |
| An exact value | F05: How long was Marguerite Bernard's nurse? | 1 of 3 | Narrator. The entry holds "his last two years". The answer hedged. |

**inferred** No exact value was lost by the writer. The one loss in every run is "who learned what". This points to the Character memory as the third list, not to the world facts. The evidence is one question in every run and four in some runs, so it is weak.

**inferred** The world facts matter only after the oldest scene entries leave the prompt. In this test nothing leaves (see the sizes). So this test cannot show a need for the world facts.

## The form checks

From `python3 $T scenes` ([`scene-entries/scenes.md`](scene-entries/scenes.md)), 3 runs, 15 writer calls, 82 scene entries.

| Check | Result |
|---|---|
| Answers the tool can read | 15 of 15 |
| Calls whose ranges cover their Messages with no gap | 15 of 15 |
| Calls with an overlap, or a range outside the call | 0 |
| Names in `took_part` that are not cast names | 1 of 188 ("Tomas", run 3, cut at #638) |
| Entries over 150 words | 29 of 82 (mean 150 words, 72 to 362) |

**known** The writer splits finely. A scene entry covers about 4 Messages (2 Cycles) on average. One call in run 3 gave 10 scene entries for 20 Messages. The splits follow changes of place: a walk from the study to the library is its own scene. I did not judge whether this is too fine. The owner reads [`scene-entries/entries-r1.md`](scene-entries/entries-r1.md).

## The sizes

Token counts: a writer answer is its usage count. A rendered scene entry is counted at 4.2 characters per token (known: the stored Narration requests of story A have 4.18 to 4.22 characters for each prompt token).

| | Run 1 | Run 2 | Run 3 |
|---|---|---|---|
| Scene entries per cut | 4, 6, 5, 6, 5 | 4, 5, 5, 6, 5 | 5, 10, 5, 6, 5 |
| Answer tokens per cut | 1,040 to 1,423 | 996 to 1,668 | 967 to 1,675 |
| List tokens after each cut | 985, 2,076, 3,195, 4,547, 5,633 | 988, 2,027, 3,214, 4,815, 5,996 | 1,129, 2,603, 3,505, 4,787, 6,413 |

- One scene entry: mean 220 tokens, median 207, from 115 to 471 (known). The guess of the ticket was 150 to 250.
- The opening: 126 tokens (known).
- One cut: about 22 Messages, 3,800 to 4,100 tokens of Messages, about 1,200 tokens of scene entries (known).
- The list after the fifth cut: 5,600 to 6,400 tokens for Messages #524 to #637, which are 57 Cycles and 19,500 tokens (known). So the list costs about 105 tokens for each Cycle, a third of the Messages (inferred from these figures). The base memory at the same point was 1,200 to 2,000 tokens.

**The large-call probe** ([`scene-entries/probes/large-call/`](scene-entries/probes/large-call/)). One writer call over the Messages of the first four cuts (#524 to #614, 15,600 tokens), as at a larger history size:

- Cap 2,000: cut off at the limit. A failed call under the engine rule (known).
- Cap 8,000: 3,120 answer tokens, 17 scene entries, no gap or overlap (known). That is about 69 tokens for each Cycle, finer than in small calls (inferred from one call).

**inferred** At the default history size, a cut removes about 16,000 tokens of Messages. Its scene entries need 3,000 to 5,000 answer tokens. Under a cap of 2,000 every such call fails, and its Messages pile onto the next call. So the engine must split the Messages of a cut into writer calls of at most about 4,000 tokens. The largest answer measured for 4,000 tokens of Messages is 1,675 tokens.

**The space of the list (a proposal, guessed).** Half the history space of the Connection. The list then grows with the history, and the narrator prompt grows by at most a half.

| History size | Space of the list | Cycles in the list (at 105 tokens per Cycle) | Cycles in the history (inferred) |
|---|---|---|---|
| 16,000 (the trial size) | 8,000 tokens | about 76 | about 32 to 44 |
| 65,536 (the default) | 32,768 tokens | about 310 (about 470 at the 69 tokens of the large call) | about 130 to 180 |

**inferred** At the trial size, the list reaches 8,000 tokens after about 76 Cycles have left the history, near Cycle 115 of a story. So a 100-Cycle trial never removes a scene entry. A test of the removal needs a longer story or a smaller space.

## The failed calls

Counted by the engine rule: a cut-off answer, or an answer the tool cannot read.

- The 3 runs: 0 of 15 (known).
- The large-call probe: 1 of 1 at the cap of 2,000, 0 of 1 at 8,000 (known).
- The lost-facts probe: 0 of 9 (known).

## The cost

| | Writer calls per run | Writer prompt tokens per run | Writer answer tokens per run |
|---|---|---|---|
| scene_entries | 5 | 23,500 | 6,000 to 6,800 |

- **known** One writer call: about 4,700 prompt tokens and 1,250 answer tokens.
- **inferred** A 100-Cycle trial spends about 2.9 million tokens. Five writer calls are about 1 % of it, as with the base.
- **known** The narrator side costs more. The test runs of the scene entries send 774,000 to 790,000 prompt tokens, against 658,000 to 677,000 for the base and 642,000 for no memory. At the last cut, the memory block is about 5,800 tokens, against at most 2,000 for the base.
- **known** The cached share of the test runs is 55 to 59 %, against 40 to 51 % for the base. The memory test does not run the narrator as the engine does, so this is not the engine's cached share. **inferred** The block changes only at a cut, as the base does, so the engine's cache keeps working between cuts.

## The seven lost facts

**How I checked it.** The two pilot stories (`tmp/lt01/redmist_julian/trial-1` is story P, `tmp/lt03/redmist_julian/trial-1` is story Q) are older packs. Their stored requests have no `<ConversationHistory>` block, so the tool cannot find their history cuts. The probe [`lost_facts.py`](scene-entries/probes/lost-facts/lost_facts.py) cuts each story into parts of about 4,000 tokens from the first Message, as the cuts of story A, and chains the writer calls as `append_cut` does. I read the scene entries myself. One run.

| Story | Fact (first Message) | Scene entry | Holds it |
|---|---|---|---|
| P | Julian and Gabriella met (#338) | #338-#343: "Gabriella opened the doors and introduced herself as Bernard's wife" | yes |
| P | Bernard stopped paying eight weeks before he died (#369) | #361-#365: "interest paid regularly until eight weeks before he died"; again in #366-#373 | yes |
| P | The debt is 740,000 (#363) | #361-#365: "about seven hundred and forty thousand" | yes |
| P | Pell has served 30 years (#367) | #366-#373: "Ambrose Pell, who had handled the Redmist account thirty years" | yes |
| Q | The desk was opened on day one, an oak desk (#541) | #538-#541 and #547-#556: "a heavy oak desk", "the third drawer slid open" | yes |
| Q | The revolver (#547) | #547-#556: "a loaded old revolver … Julian cleared the revolver's chambers, set it on the blotter" | yes |
| Q | The first "If you are reading this" letter (#543) | #542-#546: "said that if he was reading it the writer was dead" | yes |

**known** 7 of 7 are held, on the day and with the value. **inferred** This shows the writer keeps them. It does not show that the narrator uses them in play.

## Limits

- **One story for the score.** All questions come from story A.
- **No scene entry left the prompt.** The test shows every entry. The removal of the oldest is not measured.
- **No failed call in the score.** The engine rules are in the tool, but no run hit them.
- **The large-call figures come from one probe call.**
- **The lost-facts cuts are the probe's, not the engine's.** The real first cut of story P was at #400 and of story Q at #595.
- **The test scores recall, not use in play.** A trial measures use.

## Files

- The candidate: [`candidates/scene_entries.json`](candidates/scene_entries.json).
- The tool: [`memory_test.py`](memory_test.py), schedule `append_cut`, commands `write` and `scenes`. The guide: [`memory-test.md`](memory-test.md).
- The scene entries of run 1, for the owner: [`scene-entries/entries-r1.md`](scene-entries/entries-r1.md).
- The writer records of the 3 runs: [`scene-entries/memory/`](scene-entries/memory/). The answers and the cost: [`scene-entries/runs/`](scene-entries/runs/). The score report: [`scene-entries/report.md`](scene-entries/report.md). The form checks and sizes: [`scene-entries/scenes.md`](scene-entries/scenes.md).
- The probes: [`scene-entries/probes/large-call/`](scene-entries/probes/large-call/) and [`scene-entries/probes/lost-facts/`](scene-entries/probes/lost-facts/). Their scripts were run from `tmp/ltm16/` and write there.
