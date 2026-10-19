# Say how an after-plan workflow closes out a wayfinder execution map

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Step 3 of the after-plan workflow says: follow the workflow in the wayfinder skill rather than archive the plan. The wayfinder skill has no close-out operation. It knows how to resolve a ticket, but not how to mark a map done, or what to do when every ticket is resolved but the Destination ("merged to `main`") is not reached. The lead improvised a sentence in "Not yet specified". The `turn-state-model` map uses a different form ("None. The map has reached its Destination on ..."). What is the close-out?

- **Add a "Reach the Destination" operation to the wayfinder skill**: the state line, where it goes in the map, and the case where every ticket is resolved but the Destination waits on an outside event.
- **Put it in `docs/agents/issue-tracker.md`** under "Wayfinding operations", and point step 3 of the workflow at it.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- .agents/skills/wayfinder/SKILL.md (no close-out step); `docs/agents/issue-tracker.md` "Wayfinding operations".
- `.scratch/turn-state-model/map.md`: "None. The map has reached its Destination on 2026-10-14".
