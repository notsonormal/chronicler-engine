# A rule quote in a TLA+ model drifts from its spec scenario, and no check finds it

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A rule of `VerifyGate.tla` quotes the spec scenario that it covers. When the scenario text changes, the quote stays as it was, and `check.py` still passes. Must a check compare each quote with the spec?

## Context

Source: `/retro` 2026-11-08, the coordinator session of statechart-models tickets 08, 10, 11, 12, 13 and 16 (`01a1bd22`), and its subagent sessions (Spec review `a3bf53a2`).

- **known** The document review changed the Then of `docs/specs/autoplay.md` 41.27 to "Then the status display answers the label of the Job of the Action". The quote above `RunHoldsItsGame` in `.agents/skills/verify-gate/model/VerifyGate.tla` still said "the status display answers that the Job of the Action still holds the Game". Only the Spec review found it. The coordinator fixed the quote.
- **known** Ticket 12, Failure modes: "check.py never compares quotes with the spec, so a stale quote fails no check."
- **known** The quotes have one form: `\* <file>.md <number>: "<text>"`. `verify-gate/SKILL.md` repeats them in its Rule table.

## Options to decide

- **A.** `check.py` (or `scripts/verify_coverage.py check`) reads each quote and warns when its text is not in the named scenario. A warning, as the staleness warning is.
- **B.** The same check fails the gate.
- **C.** Review only, as now.
