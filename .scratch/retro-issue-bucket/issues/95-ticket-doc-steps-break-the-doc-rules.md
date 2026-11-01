# A ticket's doc step dictates sentences that break the doc rules

Type: grilling
Status: open
Blocked by: —

## Question

An engine ticket says which sentences a doc must get. The implementer writes them as given, and the doc review then rejects most of them. Where must the doc rules reach the work: in the ticket's doc step, in the implementer brief, or only in the review?

## Context

Source: the wayfinder session of 2026-10-31 that resolved "Fix the prompt text" (`.scratch/prompt-flexibility/issues/19-fix-the-prompt-text.md`). Implementer session `63d80157`, doc-review session `9820602b`.

- Step 9 of the ticket listed the doc sentences. It asked for the reason of one label value (`Scenario:`) in `prompt_system_design.md`. `docs/AGENTS.md` ("Where a fact lives", "Values stay in the code") puts that reason in a code comment beside the value.
- The implementer added a paragraph to `prompt_system.md` that told the history of the change ("Layer 6 is the blank case today", "stay"). A Diátaxis doc states no plan and no history.
- The ticket Decision "one card reads the same in all three calls" became the doc sentence "One placeholder gives one value in every call that renders it". The sentence is false: the agent calls fix `{{narrative_perspective}}` and `{{narrative_tense}}` to `third` and `past`.
- The doc review found 2 errors and 8 warnings in three changed docs. The coordinator made 14 fixes by hand. The Standards and Spec reviews found none of the doc faults.
- The doc review of `/simple-after-plan` caught every fault, so no fault reached the gate. The cost is one review pass and one fix pass for each ticket that changes docs.

New evidence, 2026-11-01 (tickets 20 and 21, doc-review session `adb40d4d`):

- The doc review found 3 errors and 5 warnings in the new doc text. Two errors came from ticket text: the ticket 20 Decision "the space changes only when a part before the history changes" became "the parts before the history" and "it holds still between two calls of one kind". Both are false: the space also subtracts the post-history splice, and a room change moves it. The ticket 21 rewrite of **Connection** in `CONTEXT.md` was false too.
- The ticket 20 doc step asked for a reason about a later memory. The implementer saw that the doc rules forbid it and wrote a substitute reason, which the review then removed.
- The Standards and Spec reviews again found none of these faults.


New evidence, 2026-11-01 (tickets 22 and 23, doc-review session `7d165829`):

- The doc review found 3 errors and 6 warnings. Two faults came from ticket text again. The ticket 22 Decision "A value that is not set is not sent" became a doc sentence, and it is false for `max_tokens`: an unset value sends `DEFAULT_MAX_TOKENS`. The ticket 23 doc step "Every state writer follows this rule" was copied as given, and "state writer" is not a glossary term.
- The implementer of ticket 22 also wrote that the preset service checks a seed file. The seed calls `PromptPreset::validate` directly.
- The Standards and Spec reviews ran after the doc fixes, so they did not show whether they would catch these faults.

## Options to decide

- **A.** The build-plan ticket names the facts that become true or false, not the sentences. The implementer writes the sentences under `docs/AGENTS.md`.
- **B.** The implementer brief of an engine ticket lists `docs/AGENTS.md` in its files, so the rules reach the writer.
- **C.** No change. The doc review is the gate, and its cost is accepted.
