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

## Options to decide

- **A.** The build-plan ticket names the facts that become true or false, not the sentences. The implementer writes the sentences under `docs/AGENTS.md`.
- **B.** The implementer brief of an engine ticket lists `docs/AGENTS.md` in its files, so the rules reach the writer.
- **C.** No change. The doc review is the gate, and its cost is accepted.
