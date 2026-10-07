# Give ticket resolution an owner after the commit

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

[Split Settings into Connections and Text Check sub-tabs](../dashboard-ui-review/issues/68-split-settings-sub-tabs.md) and [Remove the story-log ✓ and `POST /check-text`](../dashboard-ui-review/issues/70-remove-story-log-check.md) were implemented, reviewed, and committed as `6916556a`, but both stayed `Status: claimed` and neither reached the map's Decisions-so-far. A later session rebuilt both answers from session logs.

The resolve step is defined in `docs/agents/issue-tracker.md`, and the map names the coordinator as its owner. It still fell through, because the coordinator session ended at the hand-off and `/commit-and-push` ran in a separate session with no resolve instruction.

Where should the resolve step live so it survives a session boundary?

- **`/commit-and-push` resolves referenced tickets.** The commit session is the first session that knows the work landed.
- **A map Notes line owns it.** "After `/commit-and-push`, resolve the ticket(s) and update Decisions-so-far."
- **A check flags the debt.** A script lists `Status: claimed` tickets whose work is committed.

## Context

- Commit `6916556a feat(settings): split Settings into Connections and Text Check sub-tabs` (2026-10-07); the push failed with a GitHub `Internal Server Error`.
- Coordinator session `01a11349-3753-7324-8b0b-739bc366fec5` ended at: "After that I can resolve both tickets on the map and remove the two worktrees."
- Commit session `01a11748-fdeb-7324-8b0b-739dcd0c874c` ran `git add -A`, which committed the tickets as-is.
- `docs/agents/issue-tracker.md` → "Wayfinding operations" → "Resolve".
