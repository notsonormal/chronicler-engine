# Show the whole token-budget failure, remedy included

Type: task (AFK)
Status: resolved
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

## Answer

The sentence is shorter and keeps its remedy. Every failure kind now reads in full at both widths, and the wording names no control the dashboard lacks.

### The sentence

`generation_failure_summary` (`src/adapters/driving/http/utils/error.rs`) renders `The model used its budget on reasoning. Try another model.` for `TokenBudgetSpent` — 58 characters against the old 130. Its doc comment now states the two constraints a sentence lives under (it has to read in full in the clamped column, and a remedy it names has to be one the dashboard can carry out), and the variant carries the reason the wording dropped Max Tokens.

The remedy is real: a connection's `Model` field is editable in Settings → Connections, and a role can be pointed at another connection. **No Max Tokens mention remains in any wording.** The raw text (`LLM returned no answer after spending its whole token budget (2048 completion tokens, 2041 of them reasoning)`) is untouched — it is the provider's diagnostic, not our wording, and it stays behind Details.

### Decisions

1. **A shorter sentence, not the remedy inside Details.** The status line is where the player is already looking when the turn fails; a remedy behind a diagnostics popover needs a second action to reach. It also keeps the disclosure's one job (the raw error) and needs no new element in `error_disclosure`, so the blast radius is one string.
2. **Not a taller clamp.** Ticket 02's 64px action area is the reason the column is clamped; three lines of 12px/1.4 text need 50.4px inside a 42px content box, which would overflow the fixed-height action area.
3. **Max Tokens dropped rather than kept as a fog item.** The ticket offers both; dropping it needs no follow-up and the sentence is truer without it. The map's fog item "A Max Tokens control, or a larger default" still stands on its own.
4. **"Try another model" over "choose a model that does less reasoning".** The precise phrasing does not fit beside the cause within two lines (74 characters, measured). The cause half names the reasoning, so the player has both halves of the old message.

### Fit, measured

The 240px column clamps `.error-disclosure-message` to `2.8em` at 12px/1.4, and the Details toggle leaves the message 183px per line. Measured in headless Chrome against the real stylesheet: the new line wraps to two lines of 158.1px and 159.4px — 24px of slack on the wider line. The old line needed 67px in a 34px box, exactly the re-review's S2 numbers. `PromptTooLong` (50 characters) is now the second longest and has always fit.

### Tests

- **Tier 2 (stub browser)**, `tests/browser/stub/failure_display.rs`: `check_every_failure_sentence_reads_in_full` (new scenario 16.43 in `docs/specs/browser_dashboard.md`), driven by `run_failure_display_slot_checks`. It walks all eight `GenerationFailureKind` values at 1280×800 and 1024×700, and for each one asserts the drawn text is the endpoint's sentence, `scrollHeight <= clientHeight`, `scrollWidth <= clientWidth`, that the box stays inside the column, and that the command input and action area keep their sizes. The `TokenBudgetSpent` line is the longest of the eight, so the loop covers the worst case. **Verified to fail:** with the old 130-character sentence restored, the check panics `the TokenBudgetSpent sentence is clipped at 1280px: it needs 67px in 34px` while the twelve other stub checks stay green.
- **Tier 1 (http)**, `tests/http/failure_display.rs` (scenario 38.7, extended): `test_spent_token_budget_clamps_with_its_own_line` pins the new sentence and adds `!body.contains("Max Tokens")`, so a line that names a control the dashboard lacks fails the gate. Scenario 38.7's Gherkin gained the two matching lines.
- `docs/diataxis/reference/frontend/dashboard.md`: the "Failures and Health" section now states the invariant — a generation failure's sentence is drawn in full, and names only an action the dashboard itself offers.

A new `GenerationFailureKind` variant breaks the exhaustive match in `assert_kind_is_listed` at compile time, so it cannot skip the clamp check. The trade-off is that the variant list and that match carry the same eight names; the tripwire is worth it because the ticket's point is total coverage of the kinds.

### Gate

Full gate green: `python build.py`, log `logs/build_20261011_152704.log`

```
nextest: 1 passed, 0 failed  (Running architecture tests...)
nextest: 162 passed, 0 failed  (Running guardrail tests...)
nextest: 1610 passed, 0 failed, 2 skipped  (Running integration tests...)
nextest: 27 passed, 0 failed  (Running browser tests...)
```

### Follow-ups (ticket 13)

- The clamp still fails silently for a sentence that grows past two lines: the new check catches it loudly (with the clipped text named in the panic), but the sentence itself is still cut for the player. A future long kind wants a real answer — a shorter sentence, or the remedy in Details.
- `FAILURE_KINDS` in `tests/browser/stub/failure_display.rs` is the one place the kinds are listed outside the enum. The compile-time tripwire forces the author to notice it.

Not in scope, left alone: the `#command-form button` scope (ticket 13) and the pipeline (ticket 09). No CSS, JS or template changed.
