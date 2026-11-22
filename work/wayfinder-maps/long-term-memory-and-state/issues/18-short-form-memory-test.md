# Measure a short form of the story memory

Type: prototype (AFK)
Status: open
Blocked by: none

## Question

Does a short summary form keep enough of the story to compete with the scene entries? Each summary has an event progression of 5 bullet points and key facts of 2 bullet points.

The answer is a memory test score for the short form, at two frequencies, against the scene entries, together with the size of each form. With it, the owner can decide whether the engine offers more than one type of story memory.

## Context

- **Origin (owner, 2026-11-22).** During the plan review of [Build the scene entries](17-build-scene-entries.md), the owner described a summary form from another online application. In that application, a person sets the summary frequency to none, low, medium or high. Each summary has two sections: "event progression" (5 bullet points) and "key facts" (2 bullet points). The owner asked whether the engine should support several types of summary instead of narrowing down on one. The coordinator proposed measuring this form first, and the owner agreed. A setting for several types gets its own ticket only when two forms have measured scores.
- **Why it matters for the space of the list.** The owner thinks that half the history space for the scene entries may be too much (plan review of ticket 17). A short form that scores near the scene entries would make a small space, such as 10 %, realistic.
- **The memory in principle** (map Notes). No text is rewritten from its own earlier version. So each short summary is written once, from the Messages of one writer call, and added to a list. A version that combines or shortens old summaries is out of scope.
- **The figures to beat** ([Decide the form of the scene entries](16-scene-entries.md), Answer). On the 35 kept questions, over 3 runs: no memory 0 %, one 600-word summary 67 %, the base 77 %, the scene entries 92 % (89 to 97 %), the whole story 95 %. A gap under about 10 points is noise.
- **What a short summary may lose (inferred).** The 600-word summary lost who did small things, the reasons people gave, and who learned what ([the memory test](../assets/memory-test.md)). Seven bullet points are much shorter than 600 words.
- **The tool** ([`memory_test.py`](../assets/memory_test.py)). The `append_cut` schedule adds the answer of each cut to a list, puts the opening at the head, and applies the engine rule for a failed call. It reads only the JSON of the scene entries (`parse_scene_answer`, `render_scene_memory`). So the short form needs a parse step and a render step in the tool. One writer call holds at most one history space of Messages (`HISTORY_SPACE`). The frequency needs a smaller part per call, so the tool needs a setting for the size of one part.
- **The stored story.** Story A (`tmp/lt15/new/trial-1`), with 5 cuts, the first at #611 (Cycle 44). It was still stored on 2026-11-22. `tmp/` is not in git and can be cleared. If it is gone, stop and report.
- **The model and the cost** (map Notes, "The memory test comes first"). The model is `deepseek-v4.1-flash` on OpenCode Go. One run of 42 questions costs about 640,000 to 1,200,000 prompt tokens for each arm, and 60 to 90 % of them are cached. An agent may not read `.env` files.

## Design (coordinator, 2026-11-22)

- **The form.** One summary for each writer call, as JSON: `from` and `to` (its first and last Message), `events` (5 bullet points, in order) and `key_facts` (2 bullet points). The writer gets the Messages and the cast names, as the scene entry writer does. Keep the bullets as the owner described them, with no added fields such as the story time or who took part. The test asks whether the plain form is enough.
- **The block.** `<StoryMemory>` before the history, with `<Opening>` word for word and the summaries oldest first, as with the scene entries. Only the form of the list differs.
- **Two frequencies.** "Medium" is one summary for about 4,000 tokens of Messages, about 12 Cycles: the size of one writer call of the scene entries. "High" is one summary for about 2,000 tokens, about 6 Cycles. A frequency is the number of Messages that one summary covers.
- **No space limit in the test.** Every summary stays in the block, as in the scene entries measurement. The answer reports the sizes, so the space can be chosen afterwards.

## What the answer needs

- **The candidate files**, one for each frequency, in `assets/candidates/`.
- **The summaries of story A**, as one readable file in the assets, so the owner can compare them with [the scene entries of story A](../assets/scene-entries/entries-r1.md).
- **The score.** 3 runs of each frequency, with the lowest and the highest. Compare them with the scene entries and the whole story, for each kind of question (fact, time, who).
- **The questions that the short form misses in every run but the scene entries answer.** Sort them by what was lost: an exact value, who learned a fact, a reason, a time, or another kind.
- **The sizes.** The tokens of one summary, the tokens per Cycle, and the size of the list after each cut, beside the 105 tokens per Cycle of the scene entries. From these, the number of Cycles that 10 %, 25 % and 50 % of the history space would hold at the trial size and at the default size.
- **The failed calls**, counted by the rule of the engine.
- **The cost.** The prompt and answer tokens of the writer calls.
- **The seven lost facts** of [the inconsistencies of two trial stories](../../layout-trials/assets/story-inconsistencies.md), section 1: whether a summary holds each one.

## Output

An asset, `assets/short-form.md`, and an Answer section. The Answer says whether the short form is within noise of the scene entries, at which frequency, and at what size. If it is, the Answer names the follow-up ticket: a story memory type that the owner can choose, with its place in the engine beside the scene entries. If it is not, the Answer says what the short form loses, so the owner can judge the trade.

## Not in scope

- Any engine change. The tool and the candidate files change only under `assets/`.
- A form that combines or shortens old summaries.
- The "low" frequency. Ticket 16 measured that one writer call over a whole cut at the default size cut off its answer. A short form over a whole cut is the 600-word summary case, which is already measured at 67 %.
- A setting in the dashboard.
