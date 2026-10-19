# Build a ticket's list of edits from the tree, not from memory

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Two ticket claims in the core step were false when the next agent read them. The C5 implementer's table "Spec text fixed beyond the c4-docrev list" named edits that were not in the tree (the doc review found it). Ticket 07 said four services produced `ConcurrentGeneration`; C6 found that none did. Each cost a review finding or a check by the next agent.

Which guard fits?

- **The brief asks for it.** An implementer builds each list of edits in a ticket from `git diff --stat` or `rg` output, and names the command.
- **The doc review checks it.** The doc review reads the ticket section of the step and checks each claim against the tree (see [42](42-doc-review-ticket-claims.md)).
- **Leave it.** The next agent re-checks claims anyway (this bucket's Notes: "Evidence is re-checked, not trusted").

## Context

- Source: `/retro` 2026-10-19 on the core step of the Rust rebuild (ticket 07, C5 doc review finding; C6 section, "Deviations from the plan" 1).
