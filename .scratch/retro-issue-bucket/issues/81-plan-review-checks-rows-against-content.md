# The plan review checks each test row and Done-when against its Content item

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

In four of nine slices of the response build, the plan contradicted itself or the glossary, and the slice found it late: an implementer report, a review finding, or an amendment. Where should a check catch it before the build?

- **A checklist item in `/improve-ai-plan`** (or in the ticket that writes a build plan): each test row, each Done-when line and each named line text agrees with its Content item, with the tickets it cites, and with the terms in `CONTEXT.md`.
- **A checklist item in the slice brief**: the implementer reports each disagreement before it builds.
- **Leave it**: the slice review catches it.

## Context

- Source: lead session `01a15b99` (2026-10-21).
- R1: the Done-when kept `parse_datetime` (`plumbing.rs`) as a surviving `Config` site; Content item 3 says a datetime wrap takes `Parse` (implementer `79d77dec`).
- R7: the §9 test row and ticket 20 say a failed state update shows "the save line"; the Content item (finding 24) says the contradiction line. The fix amended the target design §4.3, §4.7 and plan §9 (reviewer `77cb5e82` finding 1).
- R9: ticket 26 Q3 says a failed Game select answers "the failure line"; `CONTEXT.md` defines a failure line as the line stored on a Game. Now a fill for the owner to confirm (ticket 37 of the rebuild map).
- R8: the plan allows "a free helper fn"; the guardrail refuses it ([77](77-plan-text-versus-free-fn-guardrail.md)).
