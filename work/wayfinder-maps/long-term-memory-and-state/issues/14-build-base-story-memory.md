# Build the base story memory

Type: task (AFK)
Status: resolved
Blocked by: 13

## Question

Build the base story memory of [Choose the memory candidates to try](05-memory-candidates.md), test first: at each history cut a writer call summarizes the Messages that leave, and every call with a history space shows the summary before the history.

Done when the full `python build.py` passes and the check below passes.

## Decisions (from ticket 05)

- **Summarization at the cut.** When Messages leave the history, a writer call first summarizes them into the story memory. The first form is one summary text, rewritten at each cut from the previous text and the Messages that leave.
- **In the prompt.** The Narration, the Impersonate Input and the event Narration show the story memory before the history.
- **Storage.** With the Game, tied to the Swipe that caused the cut. A Retry or a delete removes it.
- **Failure.** A failed writer call keeps the previous story memory.
- **The player's story does not change.** The story memory is not part of the story log. A person reads it in the stored call record and in the trial pack. The base adds no dashboard view.
- **The trial pack** shows the story memory of each Cycle.

## Facts and design

The Answer of [Find where the base story memory attaches in the engine](13-base-memory-seam.md) and [its asset](../assets/base-memory-seam.md). The coordinator spot-checked six of its line references on 2026-11-11, and they were right. Read the asset in full before you plan.

**The choices of the asset's section 9 (coordinator, 2026-11-11).**

1. **Storage.** An optional `SwipeState` key. Known cost: every Swipe after the first cut holds a copy of the memory text. Accepted for the base. The change "a list of scene entries" must look at it again.
2. **The marker.** The key holds the text and the id of the last Message that the text covers. The writer reads the previous text and only the Messages that left since.
3. **The cap.** One constant in tokens, used for the reserve in the cut arithmetic, the answer limit of the writer and the save. First value: 2,000 tokens (guessed, about 10 % of a 16,000-token history plus the block). The trial reports the real sizes.
4. **The role.** A new `Role::Memory`, so a failed summary never shows as a failed Narration.
5. **No switch.** The base is always on. It does not come from the agent config.
6. **An Input Message** keeps the memory from before the cut in its Swipe state. Left as it is.
7. **The block.** `<StoryMemory>`, with no lead line, as the last part before the history. No block while the memory is empty.
8. **The first cut** is one writer call.

**The two refactors of the asset's section 10 come first**, each with its tests green before the memory step is added: expose the cut plan of one call, then the before-narration hook and its step.

**The writer prompt lives in the code (owner, 2026-11-11).** The coordinator recommended a fifth preset type with its own Connection slot. The owner chose the code: the instructions of the writer are code, the writer uses the Connection of the Narration slot, and the build adds no preset type, no settings field and no migration. The first prompt text is the one of [`one_summary.json`](../assets/candidates/one_summary.json), changed to a rewrite: it gets the previous story memory and the Messages that left since.

## Steps

1. Read `tests/STRATEGY.md` ("The one rule") before you add a test. Write the seam cases first (`/tdd`).
2. Build the base on the branch `long-term-memory-2`.
3. **Glossary.** "Story memory" enters `CONTEXT.md` when its code lands. The category name of a writer ("state writer") is decided here too, because this is the second writer (`/domain-modeling`).
4. **Docs.** Change what the change makes wrong (`docs/AGENTS.md`, "Direction of edits").
5. `/simple-after-plan`: the comment pass, the doc review, `/code-review`, the full `python build.py`, and the retro.

## Check

1. **The memory test.** Run the writer prompt of the base as a candidate of [the memory test](../assets/memory-test.md) (`schedule: each_cut`, `position: before_history`), 3 runs. Report the score beside the three known figures: 0 % with no memory, 67 % with one summary of all the dropped entries, and 95 % with the whole story. This is a report, not a gate: the base is the ground for the changes on top.
2. **The trial.** One 100-Cycle trial of `scripts/trial_setups/redmist_julian.json` (`history_tokens: 16000`) on the built change. Pass when:
   - a writer call runs at each cut, and the pack shows the story memory from the first cut on;
   - the cuts fall as in the reference trial (`tmp/ltm09/redmist_julian/trial-1`), or the report explains the difference by the size of the memory block;
   - the report gives the extra calls and tokens of the writer, and the cached share of the Narrator-role calls beside the 85.6 % of the reference trial.

## Not in scope

- The form as a list of scene entries, the Character memory, and loading by a key. They are the changes on top (Answer of ticket 05).
- The view and the edit in the dashboard.
- The story clock, the card changes and the lore entries.

## Answer

The base story memory is built and committed: `a9801e4a` on `long-term-memory-2` (2026-11-14). This section gives the final state. It replaces the build notes, the two review rounds and the check rounds of 2026-11-11 to 2026-11-13. Their full text is in git: `git show a9801e4a:work/wayfinder-maps/long-term-memory-and-state/issues/14-build-base-story-memory.md`.

### What the engine does (known, read in the code at `a9801e4a`)

- **The writer call.** When a narrator call would cut Messages from the history that the story memory does not cover, the Job first calls the story memory writer. The writer gets the previous story memory and those Messages, and it rewrites the story memory. This holds for every narrator call: Narration, Impersonate, Retry and event Narration.
- **The prompt.** Every narrator call shows the story memory in a `<StoryMemory>` block, after World Lore and before the history. A call with no story memory has no block. The Quantifier call does not show it.
- **Storage.** The story memory is the optional key `story_memory` of the Swipe state: the text, and the id of the newest Message that the text covers. So a Retry, a swipe switch and a delete of the last Message give the older story memory back with no code of their own. No migration.
- **The player's story.** It does not change. The story log keeps every Message, and the dashboard has no view of the story memory. A person reads it in the stored call record and in the trial pack.
- **The size.** `STORY_MEMORY_TOKENS` is 2,000. The history cut reserves this space, but only when a story memory exists or the history already passes its space. The response length of the writer call is 2,000 tokens, or the ceiling of the Connection when that is lower. The prompt asks for three tenths of the response length in words, so 600 words at 2,000 tokens. It tells the writer to shorten the oldest parts first.
- **One writer call holds at most one history space of Messages**, oldest first, and at least one Message. The other Messages wait for the next cut. This keeps the writer prompt inside the context window for a Game with a long story and no story memory yet.
- **Failure.** A writer call fails when the provider fails, when the answer is empty, when the provider stopped the answer at the response length, or when the answer is longer than 8,000 characters. The writer never cuts a text. After a failure, the Narration is still saved, the previous story memory and its marker stay, and the role health shows "Story memory failed — keeping the previous story memory". The Messages that the failed call missed go to the next writer call.
- **The status.** The dashboard shows "Writing the story memory..." during the writer call. A stop during the writer call ends the Job before the narrator call, with only the Input saved.
- **The trial pack.** `scripts/trial.py` counts `memory` as its own call kind, and it is no story call. The pack shows the story memory of each entry, in full where it changes and "unchanged" elsewhere.

### Where the code is

| Part | Place |
|---|---|
| The story memory and its marker | `src/domain/model/story_memory.rs`, the `story_memory` key in `src/domain/model/state/swipe_state.rs` |
| The writer and its prompt (the prompt is code) | `src/application/agents/memory/agent.rs` and `prompt.rs` |
| The hook trait | `BeforeNarrationAgent` in `src/application/agents/trait_def.rs` |
| The Job: plan the cut, call the writer, call the narrator | `Job::narrate` and `Job::write_story_memory` in `src/application/pipeline/job.rs` |
| The cut plan and the reserve | `PromptAssembler::plan_cut` in `src/application/prompting/assembler.rs`, `HistoryCut::dropped_within_one_space` in `src/domain/model/history_cut.rs` |
| The answer check before the save | `LlmCallRecorder::complete_checked`, `LlmFailure::AnswerTooLong` |
| The Role and the Step | `Role::StoryMemory` (stored name `memory`), `Step::WritingStoryMemory` (name `WritingMemory` on `/debug/state`) |
| The spec | `docs/specs/story_memory.md`, scenarios 46.1 to 46.13, each with a test in `tests/http/story_memory.rs`. The stop during the writer call is a row of scenario 39.4 in `docs/specs/dashboard.md` |
| The glossary | **Story memory** and **State writer** in `CONTEXT.md` |

### Where the build differs from the brief above

1. **The writer call is inside the narration step, not a step of its own** (asset section 10 asked for a step). The narration step loads its inputs once, plans the cut, calls the writer and calls the narrator from the same values. A step of its own would load the inputs twice, or hand records from one step to the next, which design rule 7 of `job.rs` forbids. The status label still changes during the writer call.
2. **The writer is a field of the Job, not a slot of the `AgentRunner`.** It does not come from the agent configuration (choice 5), and its failure rule belongs to the Job.
3. **The reserve is conditional** (choice 3 gave one reserve). A reserve that is always on would move the cut of every story before its first cut.
4. **The save never cuts the text** (choice 3 gave the cap for the save). A text that does not fit is a failed call. A cut text would lose its end, which holds the newest events.
5. **The response length obeys the ceiling of the Connection** (choice 3 gave one constant). A Connection with a ceiling under 2,000 tokens gets a shorter story memory.
6. **The first cut can be more than one writer call** (choice 8). This happens only when more than one history space of Messages has no cover, for example in a Game saved before the story memory existed. Each Action then covers one more history space.
7. **The Role is `Role::StoryMemory`**, not `Role::Memory` (choice 4). Its stored name is still `memory`.
8. **The HTTP scenarios were not written test first.** The seam and unit cases failed first. Most HTTP scenarios were written after the code.

### Measured

Both checks of this ticket ran on 2026-11-11 and 2026-11-12. The last code changes came after them, on 2026-11-12 and 2026-11-13: the ceiling of the Connection, and the limit of one history space for each writer call. Neither changes a 100-Cycle trial that starts from an empty story and has no ceiling (inferred). No trial ran after them.

**The memory test (known).** 35 kept questions, 3 runs for each row. The row in bold is the prompt in the engine.

| Memory in the prompt | Right answers, mean (min-max) |
|---|---|
| None | 0 % |
| One summary of all the dropped entries | 67 % (57-77) |
| Base, 600 words | 70 % (66-77) |
| Base, 1,000 words and a rule to shorten | 71 % (71-71) |
| **Base, 600 words and a rule to shorten** | **77 % (77-77)** |
| Base, two sections (Earlier, Latest) | 33 % (31-37) |
| The whole story | 95 % (94-97) |

A gap under about 10 points is noise, so the base is not better than one summary. This check was a report and not a gate. With the two-section wording, the model returned the previous memory unchanged.

**The trial (known).** 100 Cycles of `redmist_julian` at 16,000 history tokens, with the prompt in the engine, against the reference trial of [Make the history size a Connection field](09-history-size-field.md).

| | This build | Reference |
|---|---|---|
| History cuts | 5 (Cycles 45, 56, 68, 78, 90) | 6 |
| Writer calls | 5, none failed, none stopped at the limit | none |
| Writer tokens | 23,444 prompt, 3,478 answer (under 1 % of the run) | none |
| Story memory after each cut, in answer tokens | 413, 643, 761, 937, 724 | none |
| Entries in the history after a cut | 64 to 67 | 61 to 67 |
| Cached share of the Narrator-role calls | 85.5 % | 85.6 % |

All three pass conditions of the Check hold. An earlier trial, with a prompt that had no rule to shorten, grew to 1,651 answer tokens at its sixth cut. The sizes of every run and the writer calls of both trials are in [Evidence: the size of the story memory](../assets/story-memory-size.md).

### The gate

- The full `python build.py`, browser tier included, passed on 2026-11-13 on the uncommitted build, after the last code change (`logs/build_20261113_000838_4184055.log`: 309 guardrail tests, 1,708 integration tests with 3 skipped, 44 browser tests).
- The formal model of the gate has the writer call. On 2026-11-14, `python scripts/verify_coverage.py check` reports that every listed function matches its recorded hash.
- No full gate ran on the commit itself, so it is inferred that the commit holds the tree of that run. A `--no-browser` run on 2026-11-14 passed every step before the guardrail tests. There, one guardrail failed on an untracked file of other work (`src/application/prompting/assembler_tests.rs`), and the integration tier did not run.
- The comment pass, the doc review, `/code-review` (two rounds) and the retro are done. The retro of 2026-11-13 gave its findings to the [retro issue bucket](../../retro-issue-bucket/map.md).

### Open

- **The size of the story memory is not under control.** In five of nine runs of the memory test, the text reached the response length at the fourth or fifth cut. In both trials it stayed under it. In the engine, such an answer is a failed writer call, and the story memory stays at its last good text. The ticket is [Decide how the story memory keeps its size](15-story-memory-size.md).
- **Every Swipe after the first cut holds a copy of the story memory text** (choice 1, accepted for the base).
- **The outside check of the map did not run on the base.** No judge counted the three memory faults. This ticket did not ask for it.
