# Decide the form of the scene entries

Type: prototype (HITL)
Status: resolved
Blocked by: 15

## Question

What does one scene entry hold, what does the writer call get and write, and how much space does the list of scene entries get?

The answer is one entry form and one writer prompt, tried on the stored story of the memory test and measured. The owner reads the scene entries of that story and gives a go or a no-go.

## Context

- **The decision** ([Decide how the story memory keeps its size](15-story-memory-size.md), Answer). No text is rewritten. At a history cut, the writer writes one scene entry for each scene in the Messages that leave. An entry never changes. All entries show before the history while they fit their space. Then the oldest entry leaves the prompt and stays stored. The opening Narration stays at the head, word for word.
- **The entry (coordinator draft).** The story time, the place, who took part, what happened, and the range of its Messages. The writer finds the scene ends inside the Messages that leave. A cut inside a scene gives two entries, one for each part.
- **The figures to beat** ([Build the base story memory](14-build-base-story-memory.md), Answer, "Measured"). On the 35 kept questions, over 3 runs: no memory 0 %, one summary of all the Messages that left 67 %, the base 77 %, the whole story 95 %. A gap under about 10 points is noise.
- **The tool** ([the memory test](../assets/memory-test.md), [`memory_test.py`](../assets/memory_test.py)). A candidate file has a schedule, `once` or `each_cut`. With `each_cut`, the answer of the writer replaces the memory. A list needs a schedule that adds the answer of each cut to the memory. The tool must also put the opening at the head. Both are changes to the tool. The tool calls each Message of the stored story an "entry", which is not a scene entry.
- **The stored story.** Story A (`tmp/lt15/new/trial-1`), with 5 cuts, the first at #611 (Cycle 44). The question set has 21 fact, 11 time and 10 who questions. `tmp/` is not in git and can be cleared.
- **The rules of the engine that the measurement must keep** (`docs/specs/story_memory.md` at `a9801e4a`). An answer that the limit cut off is a failed writer call. One writer call holds at most one history space of Messages. The Messages of a failed call go to the next call.
- **The size of one call (inferred).** A cut removes about a quarter of the history. At the 16,000 tokens of a trial, that is about 22 Messages (about 4,000 tokens). At the default 65,536, it is about four times as much. So one call at the default size writes several scene entries, and the ceiling of the Connection limits its answer.
- **Who took part.** The writer writes it. The names come from the cast of the World. The Characters present of the Swipe state are not a source: one story has 14 to 20 wrong State lines (map, Out of scope).
- **Marinara Engine, as a start for the wording** ([asset](../assets/marinara-memory.md), section 1.4). Its scene prompt: "Record what happened and its outcomes without treating past states as current." It leaves out the current situation and the open tensions. Its rule for who took part: "A character merely mentioned, remembered, discussed, or addressed while absent is NOT a participant." It finds a scene end with a separate model call at every 5 Messages, and its changelog records scenes that never ended.
- **The opening (known at `a9801e4a`).** `opening_message` (`src/domain/model/utils/opening.rs`) makes the first Message from the scenario text of the World. No other part of the prompt holds that text, so it leaves at the first history cut. It is 93 words in Redmist.
- **The sizes (guessed).** One scene entry is 150 to 250 tokens. A 100-Cycle story is about 41,000 tokens (known), and its scene entries are 3,000 to 6,000 tokens.
- **The seven lost facts.** Section 1 of [the inconsistencies of two trial stories](../../layout-trials/assets/story-inconsistencies.md) lists seven facts that had left the history: the debt of 740,000, the open desk, the revolver, the first meeting of Julian and Gabriella, and others. The stories are `tmp/lt01/redmist_julian/trial-1` and `tmp/lt03/redmist_julian/trial-1`.
- **The owner's rules** (map Notes). The coordinator chooses the design details, states them, and lets the measurement judge. The owner gets a go or a no-go on the concrete result.

## What the answer needs

- **The entry form and the writer prompt**, as a candidate file in `assets/candidates/`.
- **The scene entries of the stored story**, as one readable file in the assets. The owner reads it.
- **The score.** 3 runs of the memory test, with the lowest and the highest, against the base and the whole story, and for each kind of question.
- **The questions that the scene entries still miss in every run**, sorted by what was lost: an exact value, who learned a fact, a reason, or another kind. This chooses the next list: lost exact values point to the world facts, and lost "who learned what" points to the Character memory.
- **The checks of the form.** The share of writer answers that the tool can read. Whether the ranges of one call cover its Messages with no gap and no overlap. Whether each name of who took part is a name of the cast.
- **The sizes.** The tokens of one entry, the entries of one cut, and the tokens of the list after each cut. From these, a space for the list, and the number of Cycles that it covers at the default history size and at the trial size. A smaller history size gives more cuts on the same story.
- **The failed calls**, counted by the rule of the engine.
- **The cost.** The prompt and answer tokens of the writer calls.
- **The seven lost facts**, when the two stories are still stored: whether the scene entry of each first fact holds it.

## Output

The answer names the entry form, the writer prompt and the space of the list, with the measured scores and sizes against the base. It says which list comes third. It names the tickets that follow: where the scene entries attach in the engine, and the build.

## Answer

Resolved 2026-11-22. The owner gave a go on the scene entries of story A in the chat. The measurement is in [the scene entries, measured](../assets/scene-entries.md). The owner read [the scene entries of story A](../assets/scene-entries/entries-r1.md).

- **The entry form.** One JSON object for each scene: `from` and `to` (its first and last Message), `time` (the day of the story and the time of day), `place`, `took_part` (cast names only, by the Marinara rule), `others` (people outside the cast who took part) and `what_happened` (60 to 150 words, past tense). A cut inside a scene gives two entries.
- **The writer call.** It gets the Messages that leave, the cast names (the Persona and the Characters of the World), and the last scene entry written before, read only for the story time and the place. It rewrites nothing. The prompt is [`candidates/scene_entries.json`](../assets/candidates/scene_entries.json).
- **The narrator block.** `<StoryMemory>` before the history holds `<Opening>` (the first Message, word for word) and `<SceneEntries>`, oldest first, under a line that says a past state may have changed.
- **The score (known).** 92 % (89 to 97 %) on the 35 kept questions over 3 runs. The base scores 77 %, the whole story 95 %, no memory 0 %. The gap to the base is above the noise, the gap to the whole story is inside it.
- **The form checks (known).** 15 of 15 answers read, no gap or overlap in the ranges, 1 name of 188 outside the cast. 29 of 82 entries pass 150 words: a soft fault, left.
- **The sizes.** One entry is about 220 tokens. The list costs about 105 tokens for each Cycle, a third of the Messages it covers (known on story A).
- **The space of the list (coordinator, guessed).** Half the history space of the Connection: 8,000 tokens (about 76 Cycles) at the trial size, 32,768 (about 310 Cycles) at the default. A 100-Cycle trial never removes an entry, so the removal is not measured.
- **The split (inferred).** At the default history size, one cut removes about 16,000 tokens of Messages. One call over that was cut off at the cap of 2,000. So the engine splits the Messages of a cut into writer calls of at most about 4,000 tokens.
- **Failed calls and cost (known).** 0 of 15 failed. The writer costs 5 calls and about 30,000 tokens per run, about 1 % of a trial. The narrator block is up to about 5,800 tokens at the last cut, against at most 2,000 for the base.
- **The seven lost facts (known).** All 7 are in the scene entry of the scene where they first appear.
- **The third list (inferred, thin evidence).** The misses are mostly "who learned what" (3 of 5 questions with a miss), and no exact value is lost. So the Character memory comes before the world facts.
- **Next.** [Build the scene entries](17-build-scene-entries.md), which also finds the seam. The base figure of 77 % is a little high: the old tool kept answers cut off at the limit.
