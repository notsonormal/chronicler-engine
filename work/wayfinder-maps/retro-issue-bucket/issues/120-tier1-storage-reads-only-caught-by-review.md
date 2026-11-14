# A tier-1 test that reads state through `Storage` is caught only by review

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Can a guardrail flag `Storage` reads in the assert part of `tests/http/**` tests?

## Context

Source: `/retro` 2026-11-13, the long-term-memory-and-state session `01a1cc56` (base story memory, ticket 14, and its `/simple-after-plan` pass), with the build session `48a609ae`.

- **known** The Standards review of round 1 reported that the tier-1 tests read the stored story memory through `Storage`; fix item 4 rewrote six tests.
- **known** `tests/STRATEGY.md:101-119` marks the rule "Check: review-only".
- **guessed** The rule allows `Storage` in the arrange part, so a check needs an allowlist or a marker for arrange code.

## Options to decide

- **A.** A guardrail on `tests/http/**` outside `support/` that flags `Storage` read calls after the first request, with an allowlist.
- **B.** Nothing: the rule stays review-only.
