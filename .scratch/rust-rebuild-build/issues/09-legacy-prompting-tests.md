# Move or delete the 15 legacy prompting tests

Type: grilling (HITL)
Status: resolved
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

## Comments

### Audit of the 15 tests (lead, 2026-10-19)

Production path: `Narrator` → `assembler.rs` `render_and_fit` → `fit_messages_to_context` → `trim_history_to_budget`; `render_history_layer` → `truncate_to_budget(.., MAX_HISTORY_TOKENS)`. `render_history_layer` always writes both `<ConversationHistory>` tags, and an empty history writes a placeholder line.

| Test | Pins | Narrator case that covers it | Note |
|---|---|---|---|
| `test_token_budgets` | four constant values | — | restates the source |
| `test_estimate_tokens_{empty,single_char,exact_four,five_chars,many_chars}` (5) | four characters per token, rounded up | indirect: `narrator-history-small-budget`, `narrator-context-overflow` | pure function; the exact rate is not pinned by a case |
| `test_truncate_to_budget_{no_truncate_needed,exact_fit}` (2) | text that fits is unchanged | `narrator-history-large-budget` (no cut below the cap) | pure function |
| `test_truncate_to_budget_{truncate,preserves_recent,zero_tokens}` (3) | the cut keeps the tail | none: no case passes `MAX_HISTORY_TOKENS` (16000 tokens) | the history cap has no seam case |
| `test_trim_history_to_budget_no_history_tag` | no tag → unchanged | — | production always writes the tag (unreachable) |
| `test_trim_history_to_budget_no_close_tag` | no close tag → unchanged | — | production always writes the tag (unreachable) |
| `test_trim_history_to_budget_empty_content` | empty history content | — | production writes a placeholder line (unreachable); the assertion accepts either result |
| `test_make_prompt_context` | `PromptContext::new` copies its fields | every Narrator case builds a `PromptContext` | restates the source |

## Answer

Owner, 2026-10-19: **option D**. `application/prompting/` is inside the Narrator box, so the Narrator's seam cases are its coverage (`tests/STRATEGY.md`, the one rule). Built the same day:

- Deleted all 15 tests: `token_budget_tests.rs` and `utils/context_tests.rs`, their `mod` lines and their two legacy-list entries.
- Added the case `narrator-history-cap` to `application/narration/narrator_tests.rs`: a history of 2000 entries stays within `MAX_HISTORY_TOKENS`, keeps its newest entry and drops its oldest. The case fails when the cap is off (checked).
- Accepted gap: the exact rate of four characters per token has no direct pin; the budget cases see a change to it only indirectly.
- Regenerated `guardrails.md` (a line number moved). `python build.py --no-browser` is green: 1331 passed, 2 skipped (14 fewer tests).
- Not committed: the standing approval names the slices, not the follow-up tickets.
