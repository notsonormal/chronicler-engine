# Reconcile a ticket's Done-when with its Answer at resolution

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Ticket 09's `## Done when` says "A failed event Retry (and any failed retry) leaves the current game state as it was before the click." Its `## Answer` narrows that: the gate "keeps the attempt's world when a narration *is* on screen, because the engine commit or the trigger commit already advanced it". Both are in the resolved ticket, so the file states two different acceptance rules, and the stricter one is the false one. The next reader of the Done-when has no way to know it was narrowed — in this run the code review's spec axis spent a finding on it ("**09 — partial (a recorded deviation; owner needs to decide)**"), comparing the shipped behaviour against the stale line.

`docs/agents/issue-tracker.md` ("Wayfinding operations" → "Resolve") says append the answer, set `Status: resolved`, add the map pointer. Nothing says the Done-when follows the answer.

Where should the two be reconciled?

- **The resolve step amends the Done-when.** An Answer that narrows or drops a Done-when line edits that line in the same resolution — the ticket ends with one rule, and a later review cannot read a target that was never shipped.
- **The Answer opens with the deviation.** "Narrowed: …" above the answer; the Done-when keeps the original target as the record of what was asked. The reader has to read both, which is what failed here.
- **Nothing — reviews are the check.** A spec-axis finding is the right place for it, and the effort that owns the code pays for it. Cheapest, and it already happened once.

## Context

- Source: `/retro` 2026-10-11 on the phase2 tickets 09–13.
- `.scratch/dashboard-ui-review-phase2/issues/09-fix-event-retry-snapshot.md:22` (Done-when) against its own Answer ("How the failure path stops rolling the game back"); the shipped rule is also in `docs/specs/retrigger.md` I.9 and scenario 11.10.
- Spec-axis session `01a12c70-90fa-72fb-844d-ee138e30611f`, finding 09 — `partial`, with the code path and the gate quoted (`src/application/pipeline/action_pipeline/retry.rs`).
- Ticket 11 in the same run shows the other face of the same drift: its Done-when was met, but its *Answer* named `dashboard.md` as carrying an invariant that a later doc pass then deleted (see [Check the tickets a doc review is about to make false](42-doc-review-ticket-claims.md)).
- Related: [Give ticket resolution an owner after the commit](01-ticket-resolution-owner-after-commit.md) owns *whether* the resolve step runs; this one owns what it writes.
