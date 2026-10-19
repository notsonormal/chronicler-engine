# Move or delete the 15 legacy prompting tests

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Fifteen unit tests in the application layer stay on the legacy list: 11 in `src/application/prompting/token_budget_tests.rs` and 4 in `src/application/prompting/utils/context_tests.rs`. Under the one rule of `tests/STRATEGY.md`, a unit test lives in a seam contract, a `domain/model/` file, or a listed legacy file, and the legacy list only shrinks. What happens to these 15 tests?

- **Option A: move the helpers.** Move the pure functions they test (the token budget and the context fit) into `domain/model/`, so the tests become domain tests there and leave the legacy list.
- **Option B: delete the tests** whose behaviour a Narrator seam case already covers (`application/narration/narrator_tests.rs`, the `narrator-history-*` cases), and move the rest as in A.
- **Option C: leave them** on the legacy list until a change touches the prompting code.

Recommendation (inferred): B. P4 already deleted 6 duplicates of the Narrator seam cases from `context_tests.rs`, so more overlap is likely. Each remaining test needs a check against the seam cases first.

## Context

- Source: P4 ([P4: the Agent edge on values](05-p4-agent-edge-on-values.md), open owner items), repeated in the core step's retro ([The core step](07-core-step.md)).
- Plan: `docs/plans/rust-rebuild-build-plan.md` §10, "Legacy unit tests move to the one rule when the owner decides, one module at a time".
- Counts re-checked on 2026-10-19: `rg -c '#\[test\]'` gives 11 and 4.
