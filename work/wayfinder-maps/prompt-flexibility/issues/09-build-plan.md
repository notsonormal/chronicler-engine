# Write the build plan for the engine changes

Type: task
Status: resolved
Blocked by: 06, 07, 08, 10, 15, 16

## Question

Write the build plan that carries out the decisions of the layout model, the call parameters and the test setup, as one plan in `docs/plans/`, ready for an implementer.

## Context

- Follow the decisions of [Decide which parts of a call an author can change](06-the-layout-model.md), [Decide the parameters of each call](07-call-parameters.md) and [Shape the test setup and the trial](08-test-setup-and-trial.md).
- From [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md): the layout stays an engine rule (no block list, no new text fields, no preset variables). Four fixes: sort `KnownCharacters` and `CharactersPresent` by Character id; the stepped history cut as one explicit step (the plan chooses the step size, the check trial confirms it); a layer with no content renders nothing, not even its tags; rename `Context:` to `Scenario:`. The layers stay an ordered list in code with no fixed slot for a new layer. `prompt_system_design.md` gets the reasons of the four fixes and no plans.
- Also follow [Decide which parts of a Quantifier or Options call an author can change](10-quantifier-options-layout.md): rename `<LatestNarration>` to `<LatestMessage>` with no label (closing block, default Quantifier preset, stored rows); render placeholders in the card text of both calls; sort every card list by Character id; add `option_count` to the Options preset (default 3) for `{{option_count}}` and the parser; drop "exactly 3" from the Options closing line; list the answer formats of each parser in `agent_system.md`.
- Blast radius to cover: the tests that assert prompt text (`application/narration/narrator_tests.rs`, the Quantifier and Options prompt tests, the spec scenarios on prompt content), the seed presets in `data/prompt_presets/`, and the preset rows already stored in existing databases if the preset shape changes.
- The owner wants the plan written directly in the repo.
- The plan includes the spec changes, the tests, the doc changes (`docs/diataxis/reference/narrative/prompt_system.md` and others), and one trial on the current data as the check that the changes work.
- From [Shape the test setup and the trial](08-test-setup-and-trial.md): move `prototype/PROTOTYPE_trial.py` to `scripts/trial.py`, add the setup file and the restart limit, and delete the prototype. Two engine items: sort the cards by Character id (`src/application/pipeline/job.rs`, `src/application/agents/quantifier/utils/orchestration.rs`), and make the failure text name the `finish_reason`. The check trial uses OpenCode Go and one 100-cycle trial.
- From [Decide what the state does when a state writer fails](15-failed-state-writer.md), test-first: the `Err` path of `AgentRunner::after_narration` keeps the previous state instead of `QuantifierResult::default()`, its doc comment is corrected, and the banner reads "Quantifier failed — keeping the Room and the Characters present". Every new state writer follows the same rule.
- From [Decide whether this map designs memory and state, or only keeps room for them](16-memory-and-state-scope.md): the plan designs no memory or state. It adds no hook, no empty slot and no `SwipeState` field. It keeps the room: an open list of block types, the stepped history cut as one explicit step with its dropped range, and no part tied to exactly the Quantifier and Options. Memory and state that depend on the story are pinned to the Swipe.

## Answer

Resolved 2026-10-31. The owner changed the output: the engine changes are task tickets in this map, not a plan in `docs/plans/` (owner, 2026-10-31). A single plan was written first and reviewed. The owner found it too big, and its phases cut the work by code, not by purpose. So the plan was deleted, and its content moved into the tickets below. Each choice made here now lives in the ticket that applies it.

The tickets, in a useful order:

1. [Make the trial script](18-make-the-trial-script.md) — first, because the other tickets check their work with it. It also makes the failure text name the `finish_reason`.
2. [Fix the prompt text](19-fix-the-prompt-text.md) — card order, `Scenario:`, empty layers, `<LatestMessage>`, card text placeholders, the Options closing line, the parser formats.
3. [Cut the history in blocks](20-cut-the-history-in-blocks.md) — after the prompt text, because both change the assembler and the cache gain needs a stable card order.
4. [Add the connection slots and remove the global response length](21-connection-slots-and-response-length.md).
5. [Put the call parameters on the presets](22-call-parameters-on-presets.md) — after the slots, because the ceiling comes from the Connection of the slot. The largest ticket. It names a split point.
6. [Keep the state when a state writer fails](23-keep-state-when-a-writer-fails.md).
7. [Correct the docs that are wrong today](24-correct-the-docs.md) — at any time.

[Settle the open questions of the engine tickets](17-settle-engine-questions.md) holds six owner decisions (O1 to O6), and the answers. Each engine ticket states the answer it applies.

**Facts that contradicted the tickets (known).**

- The default Quantifier preset names `<LatestNarration>` twice, not four times.
- `agent_system.md` lists no parser format today. [Fix the prompt text](19-fix-the-prompt-text.md) adds the list.
- The `Err` path of `after_narration` writes no LLM message, so it cannot raise the banner. The banner reads only LLM messages.
- With no Quantifier registered, every state update clears the Characters present.
- Ticket 07 calls the Connection value both a "ceiling" and ME `maxTokensOverride`. These act in opposite ways when only the Connection sets a value.
- A changed seed file or `data/settings.json` does not reach an existing database.

**For the layout trials (map fog).** A layout change is a code branch. The setup file has no layout switch. Two probe servers, each on its own port and target dir, run the same setup file, and `compare` sets the packs side by side.

**Review.** A read-only reviewer checked the single plan against the decision tickets and the code. It found 1 blocker, 15 should-fix items and 12 nits, all fixed before the split. All 25 facts that it checked held. The blocker: `CallSettings` in the application layer inside the domain `AgentSettings` would fail `arch-lint`, so both parameter types live in `src/domain/model/`.
