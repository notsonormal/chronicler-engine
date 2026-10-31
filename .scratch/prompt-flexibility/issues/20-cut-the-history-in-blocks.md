# Cut the history in blocks

Type: task (AFK)
Status: open
Blocked by: 18, 19

## Question

Replace the two text cuts of the Narrator history with one history cut. The new cut drops whole Messages from the front, in blocks, and knows what it dropped.

Sources:
- [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md), fix 2 ("the stepped history cut");
- [Decide whether this map designs memory and state, or only keeps room for them](16-memory-and-state-scope.md), room 2;
- cheap rule 4 of [Find how prompt caching works for our providers and our layout](12-prompt-caching.md).

This ticket says **block**, not step, because `CONTEXT.md` gives **Step** to a part of a Job. It says **history space**, not budget, because **Budget** is the Cycles of an Autoplay run.

It waits for [Fix the prompt text](19-fix-the-prompt-text.md) for two reasons. Both tickets change the assembler. And the cut helps the cache only when the prompt before the history stays the same, which needs the card order (inferred).

Done when `HistoryCut` is the only history cut, the 100-Cycle check trial shows the history start moving only at a cut, and `python build.py` passes.

## Decisions

Choices made when the work was ticketed ([Write the build plan for the engine changes](09-build-plan.md)), corrected by the plan review of 2026-10-31:

- **The rule.** A pure type `HistoryCut` in `src/domain/model/history_cut.rs` walks the history from its first Message.
  - When the kept Messages pass the history space, it drops the oldest Messages until the rest fills at most three quarters of the space.
  - It holds `kept_from` and `blocks: Vec<HistoryBlock>`, where `HistoryBlock { dropped: Range<usize>, arrival: Option<usize> }`. `arrival` is the index of the Message whose arrival made the cut drop. Ticket 16 says the cut knows "on which cycle" it dropped. A pure function has no Cycle, so the Message index stands for it.
  - No production code reads `blocks` yet. They are the room that ticket 16 asks for.
  - The cut saves nothing, logs nothing and fires no hook. It replaces both text cuts.
  - It is a type, not a free function, because of the free-function guardrail.
- **The history space stays the same between calls of one kind.**
  - Space = `min(MAX_HISTORY_TOKENS, input limit − system half − fixed user half − reserve)`, floored at 0 with saturating arithmetic.
  - The **input limit** is the number that `fit_messages_to_context` computes today: the window, less `SAFETY_MARGIN_TOKENS`, less the effective `max_tokens` (`requested.unwrap_or(MAX_RESPONSE_TOKENS)`), floored at `MIN_INPUT_BUDGET_TOKENS`. One function computes it for both places, so the two cannot drift.
  - The **fixed user half** is every user-half part except `<PlayerInput>` and `<Guide>`, with the history slot empty. The post-history splice is in it.
  - So the space changes only when a part before the history changes. Such a change breaks the provider cache anyway.
  - The space covers the history entries, not the history tags or the marker (a few tokens). The overflow guard catches the rest.
- **The overflow guard.** If the real prompt still passes the input limit (for example, a Player Input above the reserve), the assembler computes how many more of the oldest kept Messages must go. It calls `drop_front(count)` once. That records one block with `arrival: None`.
- **The block size is one quarter of the history space.**
  - At a 32768-token window (the default of OpenRouter and OpenCode Go) the space is the full 16000 tokens. At this World's prose (about 225 tokens a Message, inferred from trial B), a cut drops about 18 Messages. So a cut comes about every 9 Cycles.
  - After a cut, the kept history fills at most three quarters of the space. It can be lower by at most the size of the last Message dropped.
  - On an 8192-token window the space is about 1600 tokens (inferred), so a block is one or two Messages. The rule does not empty the history.
  - The check trial confirms the size.
- **The reserve** for `<PlayerInput>` and `<Guide>` is `min(1024, window / 8)` tokens, one constant and one rule in `token_budget.rs`. An Event Narration frame holds the previous Narration (inferred: under 600 tokens at this World's prose). A flat 1024 would empty the history at the Mock window of 4096.

## Facts

Known on 2026-10-31 (inferred where marked).

- Two cuts run today, and both cut text, not Messages:
  1. `render_history_layer` keeps the last `MAX_HISTORY_TOKENS * 4` = 64000 characters of the entry text (`truncate_to_budget`, `src/application/prompting/token_budget.rs`, one caller). It can start in the middle of an entry and drop its sender label.
  2. `fit_messages_to_context` calls `trim_history_to_budget` (`src/application/prompting/utils/context.rs`, one caller). It drops whole lines from the front until the user half fits. An entry with newlines can lose only its first line, which holds the sender label.
- No test calls either cut directly. Both cuts recompute from scratch on every call. No code returns or logs what it dropped. The marker "(History truncated to fit context window)" shows only when nothing survives.
- `fit_messages_to_context` keeps its input limit private (`max_input_tokens`). It raises `ContextOverflow` when the system half plus the safety margin plus `MIN_INPUT_BUDGET_TOKENS` passes the window. It clamps `max_tokens` to the room left, at least 1. Its one production caller is `render_and_fit`.
- `token_budget.rs` constants: `MAX_CONTEXT_TOKENS` 32768, `MAX_HISTORY_TOKENS` 16000, `MAX_RESPONSE_TOKENS` 2048, `SAFETY_MARGIN_TOKENS` 256, `MIN_INPUT_BUDGET_TOKENS` 512. `estimate_tokens` is `chars().count().div_ceil(4)`.
- Default windows (`LlmBackendType::default_max_context_tokens`): OpenRouter and OpenCode Go 32768, Ollama 8192, Mock 4096.
- An entry renders as one string: the sender prefix, the text and a newline. The text is model prose and can hold newlines.
- The user half after the history is: the post-history splice, `<PlayerInput>`, `<Guide>`.
- The Narrator reads the whole story: `STORY_LIMIT` is `i32::MAX` (`storage/stories.rs`). So a rule that walks the history from its first Message gives the same answer on every call for the same history and the same space.
- Three cut tests are in the Narrator seam (`narrator_tests.rs`):
  - `test_a_small_budget_drops_the_oldest_history` (`CASE narrator-history-small-budget`). It uses a 2048-token window and asserts that "Entry 199" stays. Under the new rule the space at 2048 is 0, so this case breaks;
  - `test_the_default_window_carries_the_whole_history` (`CASE narrator-history-large-budget`);
  - `test_a_history_past_the_cap_keeps_its_newest_text`.
- There is no `context_tests.rs` or `token_budget_tests.rs`.
- Specs 20.25 and 20.26 (`tests/http/settings_connection_form.rs`, `docs/specs/settings.md`) compare the entry count of a tight window with a wide one. They stay true under any cut that drops more in a tighter window.
- A new `*_tests.rs` must be registered with `#[cfg(test)] mod …;` in its `mod.rs` (`scripts/check_test_structure.py`). A domain test file has a plain `//!` summary, no DOC anchor and no `CASE` tags (`guardrails/structure.rs`, `guardrails/unit_test_allowlist.rs`). A seam test carries `// CASE <id>: <what it pins>` on the line above it.
- (inferred) In trial B the 64000-character cap bound first, at cycle 39. On an 8192-token Ollama window the window binds first, at about cycle 10.

## What already exists

| Need | Reuse |
|---|---|
| Token cost of a text | `estimate_tokens` (`token_budget.rs`) |
| The input limit | The formula inside `fit_messages_to_context`. Move it into one function, and call it from both places |
| A history of many entries in a seam test | `Fixture::history_of` (`narrator_tests.rs`) |
| The one Narrator entry point | `Job::narrate` → `Narrator::narrate` → `PromptAssembler::assemble` → `LayerRenderer::render_and_fit`. Narration, Event Narration, the Retry and Impersonate all pass through it |

No code does a front cut of whole Messages today. `StoryWindow` is a read shape, not a cut.

## Steps

1. **The rule, test first.** Write `src/domain/model/history_cut.rs` and `src/domain/model/history_cut_tests.rs`.
   - Line 1 of `history_cut.rs` is `//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]`, line 2 a one-line summary. The test file has a plain `//!` summary and no `CASE` tags. Register it in `src/domain/model/mod.rs`.
   - `HistoryCut::of(entry_tokens: &[usize], history_space: usize) -> HistoryCut`.
   - `HistoryCut::drop_front(&mut self, count: usize)` drops `count` more whole Messages and records them as one block with `arrival: None`.
   - One constant: the share of the space that the kept history may fill after a cut (75 %).
   - Cases:
     - a history that fits drops nothing;
     - a history whose total equals the space drops nothing;
     - the first pass of the space drops the oldest Messages to at most 75 %;
     - one more Message after a cut keeps the same `kept_from` until the space is passed again;
     - the same input gives the same result;
     - one Message larger than the space drops all;
     - a space of 0 drops all;
     - the blocks cover `0..kept_from` with no gap, also after `drop_front`.
2. **The input limit and the reserve.** Move the input limit out of `fit_messages_to_context` into one function in `context.rs`, and make `fit_messages_to_context` call it. Add the reserve rule to `token_budget.rs`. Use saturating arithmetic, because a small window (the 512-token `narrator-context-overflow` case) must still reach the `ContextOverflow` error, not panic.
3. **The Narrator seam.** Add to `narrator_tests.rs`, each with its `// CASE` line:
   - `narrator-history-cut-drops-whole-messages`: no kept entry starts in the middle of its text, and each kept entry has its sender label;
   - `narrator-history-cut-holds-between-blocks`: two calls past the space, with one more Message on the second, start the history at the same entry;
   - `narrator-history-cut-ignores-the-player-input`: two calls with the same history and Player Inputs of different lengths (both under the reserve) start the history at the same entry;
   - `narrator-history-cut-keeps-max-tokens`: when the history is cut, the requested `max_tokens` is sent unchanged. Use a window with room after the space (8192 or more) and `max_tokens: Some(500)`;
   - `narrator-history-cut-marker-when-nothing-kept`: when Messages exist and the space keeps none, the history holds the marker.

   Change the three old cut tests to whole Messages. `narrator-history-small-budget` needs a window where the space keeps some but not all of its 200 entries. If a rewritten test no longer uses `MAX_HISTORY_TOKENS`, remove its import, or clippy fails.
4. **The assembler.** In `render_and_fit`:
   - Render every layer in the one array, with the history slot holding its tags and no entries.
   - Work out the history space (see Decisions).
   - Cost each rendered entry (sender prefix, text and newline) with `estimate_tokens`. Rounding up each entry keeps the sum above the cost of the joined text.
   - Call `HistoryCut::of`. Render the history from `kept_from` into its slot. The marker shows when Messages exist and none is kept.
   - The overflow guard (see Decisions): compute the count, call `drop_front(count)` once, and render once more.
   - `fit_messages_to_context` keeps the overflow error and the `max_tokens` clamp, and no longer trims.
   - Delete `trim_history_to_budget` and `truncate_to_budget`.
5. **Docs.**
   - `prompt_system.md`: a "History" paragraph: the history cut drops whole Messages from the front, in blocks, and knows the Messages it dropped. No numbers: they stay in the code. The Layer 5 row points at the paragraph.
   - `prompt_system_design.md`, the reasons:
     - a history start that moves on every call ends the provider cache at the history, and a cut in blocks holds the start still between cuts;
     - a cut of whole Messages never gives the model a Message with no sender;
     - the cut is the natural point for a later memory to save what leaves the prompt (ticket 16).
6. **Check.** Run a 100-Cycle trial: `python scripts/trial.py run scripts/trial_setups/redmist_julian.json` on OpenCode Go (the file holds 100 Cycles). Compare it with trial B: `python scripts/trial.py compare tmp/pf08/trial-b <new pack>`. Read:
   - the first kept history entry changes only at a cut: about 7 values over Cycles 39 to 100 (inferred), not one for each call;
   - no call's history starts in the middle of a Message;
   - the cached input share against trial B (1 %), as a report. The OpenCode Go cache rule is unknown;
   - total tokens under 6,000,000.
7. Run `python build.py test-pattern "context_window" --test-target http` (specs 20.25 and 20.26), then `python build.py`.

## Failure modes

| Code path | Failure | Result | Handled by |
|---|---|---|---|
| History space | A Player Input or Guide passes the reserve | The overflow guard drops more whole Messages as one block, and the history start moves on that call | `narrator-history-cut-ignores-the-player-input` pins the normal case |
| History space | A part before the history changes (a Room change, a new Character present) | The space can change, and the history start can move | Accepted. The same change already breaks the cache before the history |
| History space | The window is smaller than the parts it must subtract | The space is 0, and saturating arithmetic stops a panic. `ContextOverflow` still fires for a system half that cannot fit | Step 2 |
| History space | The Mock window (4096) with a real World | Little or no history. Mock-backed integration tests see a shorter history than today | The scaled reserve. Specs 20.25 and 20.26 still compare a tight and a wide window |
| Overflow guard | The fixed part alone passes the input limit, and no entry remains | The prompt passes the window, and the clamp sets `max_tokens` to 1, as today | Accepted. Same as today's behaviour |
| History cut | One Message is larger than the space | It is dropped whole. The marker shows if nothing remains | Domain case and `narrator-history-cut-marker-when-nothing-kept` |

## Not in scope

- Saving the dropped Messages to a memory. The long-term memory effort decides that.
- A setting for the block size or the reserve.
- The history of the Quantifier and Options calls (`AGENT_HISTORY_WINDOW` stays 4).
- The history as chat messages, and any other layout trial.
- A log line for each cut. The trial reads the prompts, not the logs.

## Output

Resolved when the check trial confirms the block size, or the answer names a better size with the evidence, and `python build.py` passes. The answer records the check results, the cached share and the pack path.
