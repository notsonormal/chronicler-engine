# Write the build plan of the response to the reviews

Type: task (AFK)
Status: open
Blocked by: 26

## Question

Write one build plan for the response to the reviews, in the form of [the build plan](../../../docs/plans/rust-rebuild-build-plan.md): its slices in order, the tests and the `CASE` ids of each slice, the doc changes of each slice, and the merge. The plan takes only decided work:

- [Decide the shape of a generation](19-shape-of-a-generation.md), with the amendment of ticket 20.
- [Decide the shape of the Job](20-shape-of-the-job.md).
- [Decide the storage seam](21-storage-seam.md).
- [Decide what the story log shows past 50 Messages](23-story-log-past-50-messages.md).
- [Rethink Messages and rooms](25-messages-rooms-and-mapless-worlds.md).
- The items that [Triage the small items of the review, and choose the branch to merge](26-triage-the-small-items-and-the-merge-branch.md) keeps, and the branch it chooses.

A gap that needs a decision does not go into the plan: it goes to the owner as a new ticket (map Notes).

## Context

- Order input: [synthesis.md](../assets/architecture-review/synthesis.md) §6 (recommended order); ticket 19 and ticket 20 change parts of it (no input bundle; `PreAnalysisState` deleted).
- Rules for every slice: build plan §2 (the commit gate). The standing approval of the map's Notes covers the commits of the slices.
- The resolution graduates one task ticket per slice, numbered after this ticket, with blocking edges in plan order.
- The target design is edited in place (owner, 2026-10-20): the slices make the doc changes that tickets 19, 20 and 25 list, and the label stays "version 2". No `target-design-v2.md` copy.
- Points from the map sweep (inferred) that the plan states, not decides silently: ticket 20's read-line rule (`Record::Preset`) makes a missing Quantifier or Options preset a failure line, not `None` (`job.rs:798`); the migration of ticket 19 (delete saved Games) and the drop of `message_swipes.room_report` (ticket 25) can be one migration.
- Skills: `/planning-and-task-breakdown`. Check the plan with a reviewer subagent before the resolution (the outside check).
