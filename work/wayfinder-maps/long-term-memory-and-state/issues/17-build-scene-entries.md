# Build the scene entries

Type: task (AFK)
Status: resolved
Blocked by: 16

## Question

Replace the base story memory with the scene entries of [Decide the form of the scene entries](16-scene-entries.md), test first. At each history cut, writer calls add scene entries to a list that never changes, and every call with a history shows the opening and the scene entries before the history.

Done when the full `python build.py` passes and the check below passes.

## Decisions

From the Answer of ticket 16, the Answer of [Decide how the story memory keeps its size](15-story-memory-size.md) and the map Notes "The memory in principle". The coordinator chose the points marked (coordinator, 2026-11-22).

- **The form and the prompt.** The entry form and the writer prompt of [`candidates/scene_entries.json`](../assets/candidates/scene_entries.json). The prompt stays code, as in the base (owner, 2026-11-11). The writer uses the Connection of the Narration slot (`settings.connection_for(PresetType::System)`, with the ceiling of `max_tokens_for`) and `Role::StoryMemory` (agent name `memory`), as the base writer does in `Job::story_memory_call`. The Messages in the writer prompt are numbered by their Message id, as the base cover mark `covers_up_to` is (plan review, 2026-11-22).
- **What the writer gets.** The Messages that leave, the cast names (the Persona and the Characters of the World, not the Characters present of the Swipe state), and the last scene entry written before. It reads no other memory. In a cut split into several calls, the "last scene entry written before" of a later call is the last entry of the call before it (plan review, 2026-11-22).
- **The split (coordinator).** The Messages of one cut go to writer calls of at most about 4,000 tokens of Messages each, with the answer cap of 2,000 tokens. A small cut is one call. The calls run one after the other, before the narrator call, in the step `WritingStoryMemory`. A single Message over 4,000 tokens goes to a call alone, as `HistoryCut::dropped_within_one_space` already does for one history space. The split reuses the token counts of the `HistoryCut` (plan review, 2026-11-22).
- **A failed call.** A cut-off answer or an answer that does not parse is a failed call. It adds nothing, and its Messages go to the next cut. The entries of the calls before it stay.
- **One second try (owner, 2026-11-22, plan review).** The writer Agent tries a failed call once more, with the same input, before it gives up. This covers a provider error, a cut-off answer, an answer that does not parse and an answer whose `from` numbers fail the check below. The pattern is the one retry of the Quantifier (`quantifier/utils/orchestration.rs`) and of the Options agent (`options/utils/orchestration.rs`). Both tries are recorded through `LlmCallRecorder`. The tries run inside the Agent, so a Stop takes effect at the next step of the Job, as today. Only when both tries fail does the rule above apply.
- **The engine trusts only the scene starts (owner, 2026-11-22, plan review decision 2).** The prompt stays as measured, with `from` and `to`. The engine reads only `from`, the first Message of each scene, and ignores `to`. It sets each `to` itself: the Message before the next scene's start, and the last Message of the call for the last scene. So the entries cover the Messages of the call with no gap and no overlap. The `from` numbers must be Messages of the call and must go up, and the first one is set to the first Message of the call. An answer that fails this check is a failed call. The read step accepts a Markdown code fence and text around the one JSON object, as `parse_scene_answer` of the memory test tool does. The engine keeps its own cover mark: after a good call it is the last Message of that call.
- **The block.** `<StoryMemory>` as the last part before the history, with `<Opening>` (the first Message of the story, word for word) and `<SceneEntries>` (oldest first, under the head line of the candidate). No block before the first cut.
- **The space (coordinator).** Half the history space of the Connection. The cut arithmetic reserves it. When the list passes its space, the oldest entry leaves the prompt and stays stored. The space holds the whole block: the tags, `<Opening>`, the head line and the entries (plan review, 2026-11-22). **The reserve (owner, 2026-11-22, plan review decision 1).** The space is half of the history space that the cut measures with no reserve: the first pass of `LayerRenderer::cut_of`. With a large window, that is half the history size of the Connection (8,000 tokens at the trial size), and the history keeps its full size. When the window leaves less room, the history and the list split it evenly, so the history never drops to zero. The block drops its oldest entries to fit the same number that the cut reserves. The owner kept 50 % after seeing the sizes: at Cycle 100 of story A the list was 5,600 to 6,400 tokens. A `PromptAssembler` seam case pins the split on a small window.
- **Storage (coordinator; the map fog said so on 2026-11-22).** A list that grows cannot be copied into every Swipe state. The entries get their own storage, tied to the Swipe whose cut wrote them. A Retry, a swipe switch or a delete that removes that Swipe from the story also removes its entries from the prompt. This needs a migration (map Notes, "Standing rules"): a new table whose row names its `message_swipes` row with `ON DELETE CASCADE`, so a delete of the last Message removes its entries with no new code. The narrator call reads the entries of the active Swipes of the Messages before the Job's place (for a Retry, before the last Message). The writer runs before the Swipe exists, so its entries are written in the same transaction as the text, in `GameWriter::save_text`, through the write guard (plan review, 2026-11-22).
- **The base goes.** The rewritten summary, its `SwipeState` key and its prompt are removed. Old saved Games that hold the key keep working: `SwipeState` has no `deny_unknown_fields`, so the key is ignored once the field goes. `StoryMemoryHookInput`, the `BeforeNarrationAgent` trait and the agent wiring in `bootstrap/wiring.rs` change their result type and stay.
- **The player's story does not change.** The trial pack shows the scene entries of each Cycle: `scripts/trial.py` reads the SQLite file, so `story_memory_of` changes to a read of the new table, with its Python test. No dashboard view: the memory view is a later ticket.

## Steps

1. **Find the seam first.** Read the Answer and the asset of [Find where the base story memory attaches in the engine](13-base-memory-seam.md), and the code of [Build the base story memory](14-build-base-story-memory.md) (`docs/specs/story_memory.md`). Write a short plan under `tmp/ltm17/plan.md`: the table and its tie to the Swipe, the step of the Job, the split, the reserve in the cut arithmetic, and what of the base is removed. If a decision above does not fit the code, stop and report it before you build.
2. Read `tests/STRATEGY.md` ("The one rule" and "Spec scenarios") before you add a test. Write the tests first (`/tdd`), each at the tier that sees it (plan review, 2026-11-22):
   - **Seam cases.** The writer Agent (`src/application/agents/memory/agent_tests.rs`): a good answer, a cut-off answer, an answer that does not parse, an answer whose `from` numbers fail the check, ranges set from the scene starts when the model's `to` is wrong, a first try that fails and a second that succeeds, and the two call-record cases. The `PromptAssembler` (`assembler_tests.rs`): the block, and the removal of the oldest entry when the list passes its space. The storage contract (`tests/storage/story_storage.rs`): the write with the Swipe, the read of the entries of the active Swipes, and a failure through `Storage::with_failure`.
   - **Scenarios.** Rewrite `docs/specs/story_memory.md` (46.1 to 46.13) for the scene entries, then their tests in `tests/http/story_memory.rs`: the split, a failed call in the middle of a split, Retry, swipe switch, delete, Impersonate and the event Narration. The Job is no box, so the split and the failed call are scenarios, not seam cases.
3. Build on the branch `long-term-memory-2`. The change touches `job.rs` and `gate.rs`, so run `/verify-gate` after it.
4. **Glossary.** "Scene entry" enters `CONTEXT.md` when its code lands (`/domain-modeling`).
5. **Docs.** Change what the change makes wrong, `docs/specs/story_memory.md` first (`docs/AGENTS.md`, "Direction of edits").
6. `/simple-after-plan`: the comment pass, the doc review, `/code-review`, the full `python build.py`, and the retro.

## Check

1. **The trial.** One 100-Cycle trial of `scripts/trial_setups/redmist_julian.json` (`history_tokens: 16000`) on the built change. Pass when:
   - writer calls run at each cut, and the pack shows the scene entries from the first cut on;
   - no writer call fails, or the report explains each failure;
   - the report gives the extra calls and tokens of the writer, the size of the list after each cut, and the cached share of the Narrator-role calls beside the reference trial (`tmp/ltm09/redmist_julian/trial-1`, 85.6 %).
2. **The removal.** A 100-Cycle trial does not fill the space. A seam test covers it. The report says whether a trial with a smaller history size is worth running.

## Not in scope

- The memory view and the edit in the dashboard.
- The Character memory and the world facts.
- The judged check trial of the map (the outside check).
- An edit of an older Message (map, "Not yet specified").
- A new change kind for the counters: no dashboard part shows the entries yet.
- A second memory form or a choice of memory types. [Measure a short form of the story memory](18-short-form-memory-test.md) measures one first. Keep the form behind the writer trait (`BeforeNarrationAgent`), so a second form would be a second writer.

## What already exists (plan review, 2026-11-22)

- `Job::narrate` (`src/application/pipeline/job.rs`) already plans the cut, runs the writer before the narrator call, and keeps the previous memory when the writer fails. The scene entries change what the writer returns, not this flow.
- `HistoryCut::dropped_within_one_space` and its `entry_tokens` (`src/domain/model/history_cut.rs`): the bound of one call and the token counts for the split.
- `StoryMemory::uncovered` (`src/domain/model/story_memory.rs`): the first Message no entry covers. The cover mark becomes the `to` of the newest active entry, which the engine set (decision 2).
- `LayerRenderer::cut_of` (`src/application/prompting/assembler.rs`): the reserve that counts the block at full size only after the first cut, and `story_memory_layout_tokens`. The reserve changes from `STORY_MEMORY_TOKENS` (2,000) to the space of the list.
- `Step::WritingStoryMemory`, `StoryMemoryHookInput`, `BeforeNarrationAgent`, `StoryMemoryWriter` and its wiring.
- `message_swipes ... ON DELETE CASCADE` and the `foreign_keys` pragma (`db.rs`): the delete of the last Message.

## Failure modes (plan review, 2026-11-22)

| Path | Failure | What happens | Handled |
|---|---|---|---|
| One writer call of a split | Cut off, does not parse, provider error, bad `from` numbers | A second try with the same input; if it also fails, the call adds nothing; the entries of the calls before it stay; its Messages and the later ones go to the next cut | Yes, by the Decisions |
| The entries of one call | The model's `to` leaves a gap or an overlap | The engine ignores `to` and sets the ranges from the scene starts (decision 2) | Yes |
| The entries of one call | A `from` is outside the call or does not go up | A failed try: the second try, then the failed-call rule | Yes |
| The narrator call after the writer | Fails or is stopped | The entries were never saved; the next Job writes them again, at the cost of the writer calls | Yes, as in the base |
| Stop during a split | The Job id check in `step` fails between calls | The Job ends; nothing is saved | Yes, as in the base |
| An old saved Game | The first cut after the change | The writer gets the oldest uncovered history space of Messages: 4 calls at 16,000 tokens, up to 16 at the default size, before one Narration. Later cuts catch up the rest | Accepted; the report gives the time |
| The reserve on a small window | The window leaves less room than history plus list | The history and the list split the room evenly (decision 1) | Yes |
| The storage write | The transaction fails | The text and the entries are lost together, as any failed save | Yes |

## Answer

Built on `long-term-memory-2`, uncommitted on top of `4832b0fa` (2026-11-22). After two review rounds, a doc review and a comment pass, the full `python build.py` passed on the final tree (`logs/build_20261122_233539_1300277.log`: 1,751 integration, 309 guardrail and 44 browser tests, 0 failed). The check trial passed.

**What the engine does now.**
- **The writer.** At a history cut, the Job splits the uncovered Messages of one history space into writer calls of at most 4,000 tokens of Messages (`HistoryCut::writer_calls`). The calls run one after the other in the step `WritingStoryMemory`, before the narrator call. Each call gets its Messages numbered by id, the cast (a `Cast` value: the Persona, then the Characters of the World), and the last entry written before. Its prompt is the candidate's text, word for word (`src/application/agents/memory/prompt.rs`).
- **The read.** `src/application/agents/memory/parser.rs` reads the one JSON object, even inside a fence or with text around it. A key written twice keeps its last value, as the memory test tool's reader did. The engine reads only `from`. The first `from` is not checked, and the first scene starts at the first Message of the call (coordinator, 2026-11-22, after both reviews found that the check tested a value the engine throws away). Later starts must be Messages of the call, after the first Message, and must go up. The engine sets every `to`.
- **A failed call.** It is an empty answer, a cut-off answer, an answer that does not read, an answer with bad starts, or a provider error. The Agent tries it once more with the same prompt, and both tries are recorded. If both fail, the run of calls stops: the entries of the earlier calls stay, and the rest go to the next cut.
- **Storage.** A new table `scene_entries` (migration 35) has one row for each entry. The row points at the `message_swipes` row of the new Swipe, with `ON DELETE CASCADE`. `JobWriter::save_text` writes the entries in the transaction of the text. The narrator reads the entries of the active Swipes before the Job's place, so Retry, swipe switch and delete need no code of their own.
- **The block.** `<StoryMemory>` is the last part before the history. It holds `<Opening>` (the first Message, word for word) and `<SceneEntries>` under the candidate's head line. It shows only after the first cut. Its space is half the history space of the first cut pass, and it holds the whole block. One exception: an opening larger than the space is kept whole with no entries, so on a small window the block can pass its space and shorten the narrator's answer (plan, no case covers it). When the room is at least 1.5 times the history size, the history keeps its full size. When the room is at most the history size, the two split it evenly. Between those two, the list keeps its half of the history size and the history gets the rest (plan, accepted by the coordinator). The oldest entries leave the prompt first and stay stored. The block gets the room that the rest of the prompt leaves under the input limit, up to its space, so the history keeps its priority and a long Player Input can shrink the block down to 0. A case sweeps the input length from 0 to 600 repeats. `ReservedCut` and `CutPrompt` carry the shrunk room, so the planned cut and the narrator call agree (coordinator, after the review).
- **The base is gone.** The rewritten summary, its `SwipeState` key, its prompt and `STORY_MEMORY_TOKENS` are removed. An old saved Swipe state with the key still loads (storage case `story-swipe-state-old-story-memory-key`).
- **Spec and docs.** `docs/specs/story_memory.md` holds 46.1 to 46.16, all covered by `tests/http/story_memory.rs`. The docs under `docs/diataxis/` and `CONTEXT.md` ("Scene entry") are updated. `/verify-gate`: the model got one new action for the next writer call. All 7 configs hold, all 25 mutants are caught, and the hashes are recorded.

**The check trial** ([report](../assets/scene-entries-build/trial-report.md), [compare](../assets/scene-entries-build/compare.md), [writer calls](../assets/scene-entries-build/writer-calls.jsonl)). One 100-Cycle trial of `redmist_julian` at 16,000 tokens. The pack is at `tmp/ltm17/redmist_julian/trial-1`.
- **Writer calls.** 5 cuts gave 12 writer calls and 36 entries, with no gap or overlap.
- **One failed call.** Both tries of the first call at the first cut failed, because the model wrote the key `others` twice in one entry. So the Impersonate call of that cut showed only `<Opening>`, and the next Job wrote the entries. The parser fix above accepts that answer now. A seam case covers it (`memory-repeated-key`), but no trial has run with the fix.
- **Cost.** The writer used 45,981 tokens, which is 1.3 % of the trial. 10,288 of those tokens went on the two failed tries.
- **The list after each cut.** 6, 13, 17, 27 and 36 entries, or about 1.4k, 2.9k, 4.0k, 6.1k and 8.0k tokens.
- **The cache.** The Narrator-role calls were 86.0 % cached, against 85.5 % in the reference trial (`summary.json`; the ticket's 85.6 % was a misread).
- **The removal (check 2).** Unlike the forecast, the list passed its space at the last cut: the prompt showed 34 of the 36 entries. So the trial also exercised the removal. A smaller-history trial is not needed for the removal. It would be useful only to see whether the story contradicts the facts of entries that left the prompt, and that belongs to the judged check trial of the map.
- **The entry size.** The median entry has 129 words, and 13 of 36 entries are over the candidate's 150-word guide.

**Left open.**
- A failed read gives only "scene entries" or the starts text as its failure text. The serde error is not kept, so a reader of the call record cannot see the cause.
- The writer's second try is the third copy of the two-try loop.
- Both go to [Share the retry of the Agent calls and keep the cause of a refused answer](20-retry-frame-and-refusal-cause.md) (owner, 2026-11-22), which blocks the memory view.
- After a delete or a wider window, the history can hold Message 1 again while entries exist. The block then still shows, so `<Opening>` repeats the first history entry. It is cosmetic and rare.
- `trial.py compare` writes `compare.md` beside its first pack, so it overwrote `tmp/ltm09/redmist_julian/compare.md`.

The term became Memory entry (owner, 2026-11-22); the table, the types and the `<MemoryEntries>` tag use it.
