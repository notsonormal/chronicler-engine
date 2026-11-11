# A ticket's doc step dictates sentences that break the doc rules

Type: grilling
Status: open
Blocked by: —

## Question

An engine ticket says which sentences a doc must get. The implementer writes them as given, and the doc review then rejects most of them. Where must the doc rules reach the work: in the ticket's doc step, in the implementer brief, or only in the review?

## Context

Source: the wayfinder session of 2026-10-31 that resolved "Fix the prompt text" (`work/wayfinder-maps/prompt-flexibility/issues/19-fix-the-prompt-text.md`). Implementer session `63d80157`, doc-review session `9820602b`.

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

New evidence, 2026-11-01 (layout-trials ticket 09, `work/wayfinder-maps/layout-trials/issues/09-move-changing-blocks-after-history.md`; implementer session `f6c26d0f`, doc-review session `f640621a`, Standards session `b2d72e25`):

- The coordinator's plan review wrote the doc step. It said "Give the measured gain (Narrator-role cached share 55.2 % to 78.8 %, ticket 03)". The implementer copied the figures into `prompt_system_design.md`. `chronicler-docs-hygiene` flags "PR-style validation metrics". The doc review did not flag the figures. It only corrected their method. The Standards review caught them.
- The implementer also wrote a false claim: a Room change "can move the history start by one block". `HistoryCut::of` makes the whole cut again from the space, so the start can move by any number of Messages. The doc review caught it.
- The doc review found 14 problems in two docs: 3 errors and 6 warnings. Here the fault came from the plan review, not from a Decision of the ticket. So option A must also reach the agent that writes the plan review (`/improve-ai-plan`).

New evidence, 2026-11-03 (long-term-memory ticket 09, `work/wayfinder-maps/long-term-memory-and-state/issues/09-history-size-field.md`; implementer session `876bb59e`, doc-review session `5b0c85b9`, Standards session `dda2215d`, Spec session `4587d0e0`):

- The ticket's glossary step said "the tokens of history that one Narrator-role call may hold". "Narrator-role" is not a glossary term, and the scope is false: an event Narration is a Trigger Role call with the same history space. The implementer copied the term into `CONTEXT.md`, the field doc comment, `settings.schema.json` and the `trial.py` docstrings. The doc review caught the glossary entry, and the Spec review caught the other places.
- The ticket's doc step named two docs that do not exist: "the Settings reference of the Connection form" and "the setup-file fields of the trial in its docs". The implementer and the doc review each spent a search to confirm it.
- The ticket's Decisions put the default constant in `src/application/prompting/token_budget.rs`. The domain resolve method needs it, and `arch-lint.toml` forbids that import. The implementer moved it to `src/domain/model/settings.rs`. So the fault reaches code placement too, not only doc sentences: a ticket that the coordinator writes from a grilling names places that no one checked.

## Options to decide

- **A.** The build-plan ticket names the facts that become true or false, not the sentences. The implementer writes the sentences under `docs/AGENTS.md`.
- **B.** The implementer brief of an engine ticket lists `docs/AGENTS.md` in its files, so the rules reach the writer.
- **C.** No change. The doc review is the gate, and its cost is accepted.
