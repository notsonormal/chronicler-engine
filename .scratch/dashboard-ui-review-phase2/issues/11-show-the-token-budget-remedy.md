# Show the whole token-budget failure, remedy included

Type: task (AFK)
Status: open
Blocked by: —

## Question

The `TokenBudgetSpent` status sentence is cut at its second line, and its remedy is visible nowhere. How does the player read the whole sentence?

## Context

- Finding **S2 (P2)** of the [2026-10-11 re-review](../re-review-2026-10-11.md), shot `tmp/p2-07/16-token-budget.png`.
- The sentence is 130 characters. [Make every failure display tell the truth](02-make-failure-display-truthful.md) clamps `#status-display .error-disclosure-message` to two lines in the 240px column; this sentence needs four (`scrollHeight` 67 > `clientHeight` 34). The cut is silent (no ellipsis). Details shows only the raw error.
- Ticket 02's follow-up predicted this; `tests/browser/stub/failure_display.rs` drives only the `PromptTooLong` sentence. Extend it to the longest sentence of every `GenerationFailureKind`.
- Options: a shorter sentence, the remedy inside Details, or a taller clamp. The action area's 64px height budget is ticket 02's constraint.
- The remedy names "Max Tokens", but no dashboard control sets it ([Fail a reply that has no content](08-fail-reply-without-content.md), open question). Make the wording name only what the player can do, or raise the fog item on the map.
- This failure is common with the default Narrator: 2 of 11 Narrator calls in the re-review.

## Done when

- Every `GenerationFailureKind` sentence reads in full (or its remedy reads in Details) at 1280 and 1024 widths.
- The stub test covers the longest sentence. Name the tier.
- `python build.py` is green. Commit after user approval.
