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

## Build notes

Built on `long-term-memory-2`, not committed. The full gate without the browser tier passed (`logs/build_20261111_224124_3807423.log`). The browser tier did not run.

### What was built

- **The two refactors of the asset's section 10.** `PromptAssembler::plan_cut` and `Narrator::plan_cut` return the history cut of one call, from the same renderer that `assemble` uses. The new `BeforeNarrationAgent` trait is the before-narration hook. The Job runs a story memory step between `save_input` and `narrate`, with its own Step, `Step::WritingMemory` ("Writing the story memory...").
- **The story memory.** `StoryMemory` (`src/domain/model/story_memory.rs`) holds the text and `covers_up_to`, the id of the last covered Message. It is the optional `story_memory` key of `SwipeState`, so no migration. `StoryMemory::uncovered` gives the dropped Messages after the marker.
- **The writer.** `StoryMemoryWriter` (`src/application/agents/memory/`) is built in `wiring.rs` on the Narration recorder and records under `Role::Memory` (agent name `memory`). Its instructions are code (`prompt.rs`), from the `one_summary` candidate. `STORY_MEMORY_TOKENS` (2,000) is the answer limit, the size of the saved text (`STORY_MEMORY_CHARS`) and the reserve in the cut.
- **The block.** `<StoryMemory>` ends the first user part, after World Lore and before the history. A call with no story memory has no block. Every kind of generation shows it.
- **Failure.** A failed writer logs a warning, keeps the previous story memory and its marker, and the Job continues. The role-health banner shows "Story memory failed — keeping the previous story memory". No generation failure banner shows.
- **`scripts/trial.py`.** `memory` is its own call kind and no story call. The pack shows the story memory of each entry's active Swipe, or "unchanged".
- **Glossary and docs.** `CONTEXT.md` gains **Story memory** and **State writer** (the Quantifier and the story memory writer), and the Role, Step, Agent, Hook and Swipe state entries name the new parts. The agent system, prompt system and game flow references and the two design explanations describe the hook, the block and the reserve. New spec `docs/specs/story_memory.md`, scenarios 46.1 to 46.6.

### Departures from the asset, with reasons

1. **The reserve is conditional.** The cut first runs with no reserve. It keeps the 2,000-token reserve only when a story memory exists or the history already passes its space. Always reserving would move the cut of every story with a long history, also before its first cut. So the cuts of a trial can fall earlier than in the reference trial from the first cut on, by about the size of the reserve.
2. **The writer is a `JobDeps` field, not an `AgentRunner` slot.** It is not built from the agent configuration, and its failure rule (keep the previous memory) belongs to the Job step.
3. **`narrate` does not return the memory.** The story memory step returns it. `narrate` and `save_text` take it as an argument. This keeps the writer call out of the narrator call.
4. **The story memory step loads the narration inputs a second time.** `Job::with_narration_call` loads them as locals and lends them to a closure. Design rule 7 forbids record fields in `job.rs`, so a struct that holds the loaded records was not possible. The cost is a second set of reads in a generation.
5. **Writer call settings.** `max_tokens` is `STORY_MEMORY_TOKENS` (2,000), not the candidate's 2,048, because one constant was decided. The samplers are the provider defaults. The label of an Input entry is the Persona name, not a fixed "Julian".
6. **The memory cases of the Narrator seam contract are in `src/application/narration/mod_tests.rs`.** With them, `narrator_tests.rs` passed the 2,000-line limit. The file-pairing guardrail allows only a `_tests.rs` file that has a source file of the same name, so a `narrator_story_memory_tests.rs` was not possible. The shared helpers of `narrator_tests.rs` became `pub(super)`.
7. **Scenario 46.2 widens the Connection's window in its Given.** The cut keeps 75 % of the history space as headroom. So the kept history sits between 75 % and 100 % of the space, and two new Messages can cause a second cut. The wider window makes the scenario deterministic.
8. **Scenario 46.6 was added.** A stop at the writer call is a new stop point. The scenario checks the status label, the stop, and that only the Input is saved, with no story memory.
9. **Test-first was partial.** The seam and unit cases failed first. The HTTP scenarios were written after the Job step, so they were not seen failing first.
10. **Changed existing tests.** `long_story_app` scripts an optional writer reply, because a narrow window now calls the writer. `failure_display` expects three "No calls yet" entries, because `Role::Memory` is a fifth Role.

### Not done

- **The formal model of the gate.** The build warns that the TLA+ model may not match `Job::generate`. A model call now sits between two step holds of one Job. The model needs the memory step hold, the memory model call, a mutant, and new words in the diagrams. `/maintain-formal-verification-skill` and `python scripts/verify_coverage.py record` were not run.
- **The browser tier, the memory test, the trial, `/code-review` and the retro** are for the coordinator.
- **A candidate file for the memory test** was not written. The writer prompt to score is `WRITER_SYSTEM` and the user prompt in `src/application/agents/memory/prompt.rs`.

## Check results

Run by the coordinator on 2026-11-11, on the uncommitted build of `long-term-memory-2` (parent `4baba0a4`).

**The memory test (known).** Candidate [`base_rolling_summary.json`](../assets/candidates/base_rolling_summary.json), which holds the writer prompt of `src/application/agents/memory/prompt.rs` with the label "Julian". Shape `full`, 3 runs, 35 kept questions.

| Arm | Fact | Time | Who | All kept, mean (min-max) | Memory words |
|---|---|---|---|---|---|
| none | 0/21 | 0/10 | 0/4 | 0 % | 0 |
| one_summary | 12, 15, 11 of 21 | 9, 10, 7 of 10 | 2, 2, 2 of 4 | 67 % (57-77) | 503 |
| base_rolling_summary | 15, 18, 14 of 21 | 8, 7, 7 of 10 | 1, 2, 2 of 4 | 70 % (66-77) | 709 |
| whole | 21/21 | 10, 9, 9 of 10 | 3/4 | 95 % (94-97) | 0 |

The base and the single summary score the same within the spread of the runs. The memory passed the 600 words that the prompt asks for.

**The trial (known).** Pack `tmp/ltm14/redmist_julian/trial-1` (local only), against `tmp/ltm09/redmist_julian/trial-1`. 100 Cycles, 201 entries, 1,817 s.

- **A writer call at each cut: pass.** 6 cuts and 6 writer calls, none failed (calls 1996, 2039, 2079, 2124, 2165, 2208). The pack shows the story memory from the first cut on, in full where it changes and "unchanged (covers up to #N)" elsewhere. The Narration and Impersonate call files hold the `<StoryMemory>` block. The Quantifier call files do not.
- **The cuts: explained by the reserve.** The first cut is at Cycle 41 on both sides. After it, the base cuts every 10 Cycles (41, 51, 61, 72, 82, 92) and the reference about every 11 (41, 52, 64, 75, 87, 98). The base holds 56 to 58 entries after a cut, the reference 61 to 67. That is the 2,000-token reserve.
- **The cost.** The writer: 6 calls, 29,620 prompt tokens and 5,983 answer tokens, which is under 1 % of the 3,121,873 prompt tokens of the run. The Narrator-role calls: 2,680,981 prompt tokens against 2,560,543 (+4.7 %), and a cached share of 84.2 % against 85.6 %.

**Findings of the trial.**

- **The memory grows at each cut and reaches the cap soon (known).** The writer's answers were 321, 729, 923, 1,233, 1,126 and 1,651 tokens. The prompt asks for at most 600 words, and the model does not obey it: it adds the new entries and shortens little. The cap is 2,000 tokens, and the save cuts the text at the cap, so the end of the text goes first. The end holds the newest events. A story of about 110 Cycles or more at this history size reaches that point (inferred from the growth). This is the first thing that the change "a list of scene entries" must solve.
- **The Quantifier's cached share fell to 19.5 %, from 44.0 % (known). The cause is not known.** The Quantifier prompt holds no story memory. The run was slower than the reference (1,817 s against 968 s) and had one network error (call 2077), so the provider is the likely cause (guessed).
- One Quantifier call failed on a network error (call 2077). It is not a failure of the change.

## Review round

Run by the coordinator on 2026-11-11 over the uncommitted build (`/simple-after-plan`). The reports are local files: `tmp/ltm14/doc-review.md`, `tmp/ltm14/review-standards.md` and `tmp/ltm14/review-spec.md`. The comment pass deleted 24 comments and rewrote 7. The doc fixes (14 findings, 12 applied in a first pass) are not part of the list below.

**The fix list (coordinator decisions). Each item names its finding.**

1. **A story memory that the answer limit cut off is a failed writer call** (standards, point 3). Before the fix, a non-empty answer with the finish reason `length` was a good answer, `LlmCallResult` did not carry the finish reason, and the writer cut the text at `STORY_MEMORY_CHARS`. So the save could hold a memory that stops in the middle of a sentence, with the marker advanced. The decision: the finish reason reaches the writer. An answer that the provider stopped at the limit, or that is longer than the saved size, is a failure of the writer: the previous story memory and its marker stay, and the role health shows it. The writer never cuts a text.
2. **One size, stated once, and a rule to shorten** (standards, "Duplicated knowledge", and the trial finding). The prompt asked for "at most 600 words" while the limit was 2,000 tokens, and the trial answers grew to 1,651 tokens. The decision: the prompt takes its word limit from the constant (half the token count, so 1,000 words for 2,000 tokens). It also tells the writer what to do when the memory would pass the limit: shorten the oldest parts first, keep the open promises, plans and appointments, the names and who knows what, and drop finished small steps.
3. **One load for the plan and the call, and the writer step in `generate`** (standards, points 1 and 2). The two loads of the narration inputs can differ when a Connection or a preset changes during the writer call. The nested `run_step` is the only one outside `generate` and `run`. The decision: the inputs load once, the plan and the call read the same values, and `generate` runs the writer step itself. If both cannot hold cleanly together, one load comes first, and the build notes say why.
4. **The tier-1 tests do not read `Storage`** (standards, hard violation 1, and doc finding 7). The tests assert the `<StoryMemory>` block of the next narrator prompt and the Message ids of the next writer prompt, through `ScriptedModel::calls()`.
5. **The Narrator cases mirror a real source file** (standards, hard violation 2). `narration/mod_tests.rs` goes. The length problem of `narrator_tests.rs` is solved where it is: a case belongs to the test file of the source file that holds its behaviour (the cut arithmetic is in `assembler.rs`).
6. **New scenarios with tier-1 tests** (spec review): the Impersonate call shows the story memory, a Retry starts from the story memory of the state before the last Message, a swipe switch gives the old story memory back, and a delete of the last Message gives the older story memory back.
7. **Scenario 46.6 against 39.4** (doc finding 8): one row for the story memory writer call in the Examples table of scenario 39.4 of `docs/specs/dashboard.md`, with its test, and scenario 46.6 goes. If the test of 39.4 cannot take the row cheaply, 46.6 stays and the intro of 39.4 names it. The Endpoints list of `story_memory.md` then matches its scenarios.
8. **Small points:** the stale comment in `generation.rs` ("The Retry of a typed Input keeps the start state as it is"), the name `plan` beside `plan_cut` in one closure, the before-narration hook takes a `CallSettings` and not an `AgentSettings` with two empty fields, the repeated first lines of `history_cut` and `render_and_fit`, and the second guard `EngineError::Config("…needs a leaving Message")`.

**Rejected.**

- "`BeforeNarrationAgent` has one implementer" (Speculative Generality). The agent system names one trait for each hook position, and this ticket decided the hook. An `Arc<StoryMemoryWriter>` would make the story memory the one Agent with no hook.
- The spec review's note on the new role-health text and the new step label as dashboard changes. Both follow from the decided `Role::Memory` and the new model call.


## Review fixes

Applied on 2026-11-12 over the uncommitted build. Gate: `python build.py --no-browser` passed (`logs/build_20261112_002230_3897756.log`), with the expected verify-gate warning on `Job::generate`. The formal model and its hashes are not updated.

1. **Done.** `LlmCallResult` and the transport's `ChatCompletionResult` carry `stopped_at_limit: bool`. `parse_chat_response` returns `ChatAnswer { text, stopped_at_limit }` and still accepts a non-empty answer with the finish reason `length`, so a Narration stopped at the limit stays a good answer. One new failure, `LlmFailure::AnswerTooLong { raw_response, stopped_at_limit }`, covers both cases. The writer calls a new `LlmCallRecorder::complete_checked`, which runs the caller's check before the save, so a refused answer is recorded as a failed call and the role health shows it. `complete` delegates to it with no check. The writer never cuts a text. Tests: the provider case `chat-completions-answer-stopped-at-limit`, the writer cases `memory-at-size`, `memory-stopped-at-limit` and `memory-past-size` (red first), and scenario 46.6 (`test_a_story_memory_cut_off_at_the_limit_keeps_the_previous_one`). `ScriptedModel` has a `cut_off` reply.
2. **Done.** `STORY_MEMORY_WORDS = STORY_MEMORY_TOKENS / 2` in `memory/prompt.rs` set the word limit (later changed to three tenths, see `## Check results after the review round`), and the prompt has the sentence on how to shorten. Case `memory-prompt-size`.
3. **Done, with one departure.** Rule 7 bans record fields and record parameters in `job.rs` (Game, Message, SwipeState, Tail, DockRecord, WorldBundle). It does not ban locals inside one step. So the narration step `narrate` loads the inputs once, plans the cut, runs the writer and calls the narrator from the same locals. The writer does not run as its own step in `generate`: that would hand the loaded records from one step to the next, which rule 7 forbids, or load them twice. The writer sets its own step name with `JobWriter::step` and gives the narration step name back after it, so no nested `run_step` remains. A stop during the writer call ends the Job before the narrator call.
4. **Done.** `tests/http/story_memory.rs` reads no `Storage`. Its Then lines observe the story memory through the next writer prompt (`<PreviousMemory>`) or the next narrator prompt (`<StoryMemory>`), from `ScriptedModel::calls()`. The spec's intro defines "the next Action shows X as the story memory". These tests passed at once: the behaviour already existed, only the observation changed.
5. **Done, in the narrator's test file.** `narration/mod_tests.rs`, its `mod` line and its seam entry are gone. Its cases are in `narrator_tests.rs`, and the 27 `pub(super)` helpers are private again. The file is at 1,983 non-blank lines, after shorter helpers for the input-filter, cleanup and cut cases (no case removed). The cut cases did not move to an `assembler_tests.rs`: the Prompt Assembler is not a box of `tests/STRATEGY.md`, so a seam file there needs a policy change. The cut is reached through `Narrator::plan_cut` and `narrate`, which are the Narrator's seam.
6. **Done.** Scenarios 46.7 to 46.10 with tests: Impersonate, Retry, swipe switch, delete. They passed at once, as the spec review expected.
7. **Done, the full way.** Scenario 39.4 has a row for the story memory writer call (`JobKind::NarrationAtCut`, held at `Role::Memory`, status "Writing the story memory..."). The old 46.6 is gone, and the number now holds the cut-off case of item 1. The Endpoints list of `story_memory.md` matches its scenarios.
8. **Done.** The `generation.rs` comment says that the Retry of a typed Input takes the start state whole, story memory too. The name `plan` beside `plan_cut` is gone. The hook takes `&CallSettings`. `history_cut` and `render_and_fit` share `cut_prompt`. `MemoryHookInput::new` returns `None` with no leaving Message, so the writer's `EngineError::Config` guard and its test are gone.

## Check results after the review round

Run by the coordinator on 2026-11-12, on the uncommitted build after the review fixes. The coordinator accepted the two departures of `## Review fixes` (the writer call inside the narration step, and the Narrator cases back in `narrator_tests.rs`).

**The writer prompt (known).** Four wordings ran in the memory test, shape `full`, 3 runs each. The words are the memory size at each of the 5 cuts, for the run that grew most.

| Wording | Kept, mean (min-max) | Words at each cut |
|---|---|---|
| 600 words (`base_rolling_summary_v1`) | 70 % (66-77) | 356, 505, 1,021, 1,443, 1,443 |
| 1,000 words and a rule to shorten (`base_1000_shorten`) | 71 % (71-71) | 450, 896, 1,356, 1,471, 1,469 |
| 600 words and a rule to shorten (`base_rolling_summary`) | 77 % (77-77) | 423, 527, 765, 1,171, 1,447 |
| Two sections, Earlier and Latest (not kept) | 33 % (31-37) | 325, 591, 591, 591, 588 |

About 1,450 words is the answer limit of 2,000 tokens, so those answers were cut off. Two of the three runs of a wording often gave the same text, so the spread is understated. The two-section wording held its size, but the model returned the previous memory unchanged and ignored the new entries. The engine holds the third wording: `STORY_MEMORY_WORDS` is three tenths of `STORY_MEMORY_TOKENS`.

**The second trial (known).** Pack `tmp/ltm14/run2/trial-1` (local only). 100 Cycles, 201 entries, 1,274 s, no failed call.

- 5 cuts (Cycles 45, 56, 68, 78, 90) and 5 writer calls, none failed and none cut off. The answers were 413, 643, 761, 937 and 724 tokens. The memory became smaller at the last cut.
- The history holds 64 to 67 entries after a cut, as in the reference trial.
- The writer used 23,444 prompt tokens and 3,478 answer tokens. The cached share of the Narrator-role calls is 85.5 %, against 85.6 % in the reference trial.
- The cached share of the Quantifier calls is 47.5 %. So the 19.5 % of the first trial came from the provider on that run, not from the change (inferred).

**The open finding.** The memory test shows runs where the memory reaches the answer limit at the fourth cut. The trial shows a run where it stays under half of the limit. So the size of a rewritten summary is not under control with this model: it depends on the run. In the engine, an answer at the limit is a failed writer call, and the story memory then stays at its last good text. The base does not solve this. It is the first question of the change on top.

**The formal model.** The `verify-gate` model has the writer call and its two step holds, one new mutant and one new diagram page. All 7 configurations hold, and the check catches all 25 mutants. The hashes are recorded.

## Answer

The base story memory is built on `long-term-memory-2` (not committed on 2026-11-12, parent `4baba0a4`). The full gate passes (`logs/build_20261112_011540_3926626.log`).

- At a history cut, the Job calls the story memory writer before the narrator call. The writer rewrites the story memory from the previous text and the Messages that left since. The text and the id of the last covered Message are an optional key of the Swipe state, so a Retry, a swipe switch and a delete need no code of their own.
- Every narrator call (Narration, Impersonate, Retry, event Narration) shows the text in a `<StoryMemory>` block before the history. The story that the player reads does not change.
- A failed writer call, and an answer that the limit cut off, keep the previous story memory and its marker. The role health shows "Story memory failed".
- The writer prompt is code and uses the Connection of the Narration slot (owner decision). The limit is 2,000 tokens, and the prompt asks for 600 words.
- Measured: the memory test gives 77 % (0 % with no memory, 95 % with the whole story). In a 100-Cycle trial at 16,000 tokens, 5 writer calls ran, none failed, and the cached share of the Narrator-role calls stayed at 85.5 %.
- Open: the size of the rewritten text is not under control. In some runs of the memory test the text reaches the limit at the fourth cut. The first change on top must answer this.

