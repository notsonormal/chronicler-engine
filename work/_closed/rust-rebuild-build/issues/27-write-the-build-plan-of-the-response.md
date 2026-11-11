# Write the build plan of the response to the reviews

Type: task (AFK)
Status: resolved
Blocked by: 26

## Question

Write one build plan for the response to the reviews, in the form of [the build plan](../../../../docs/plans/rust-rebuild-build-plan.md): its slices in order, the tests and the `CASE` ids of each slice, the doc changes of each slice, and the merge. The plan takes only decided work:

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

## Answer

Written on 2026-10-20: [the build plan of the response](../../../../docs/plans/rust-rebuild-response-build-plan.md) (`docs/plans/rust-rebuild-response-build-plan.md`).

- **Nine slices, then the merge**, all on `wf/rebuild-core` (ticket 26 Q6): R1 the storage seam; R2 the story window; R3 the room is one fact; R4 the Quantifier result and the Game rules; R5 the kind carries its data, and the plan rules; R6 the preset module; R7 the failure lines and the loop; R8 the step rule and the narrator; R9 the small fixes; §13 the merge. Plan §1 gives the reasons for each change to the order of synthesis §6. Plan §14 places every item of the review in one slice or names the ticket that closed it.
- **The SQLite count** (ticket 21, its risk): a probe opened `new_in_memory()` on SQLite `:memory:` at `5d3cfe5f`. Of 1340 tests, 9 failed: 2 go with the in-memory refusal; 2 assert the backend name; 3 hit a foreign key that the in-memory maps did not enforce; 1 expects a no-op update of a missing World; 1 expects the insertion order of the preset list. Plan §3 gives the fate of each.
- **Points from the map sweep**, stated in the plan: two migrations, not one (G4: v29 drops `room_report` in R3; v30 deletes the saved Games in R4); a missing hook preset becomes a failure line (G3).
- **Gaps** (plan §12). G2 (ticket 20's `job_tests.rs` cases break the one rule; they become domain unit tests) and G5 (the room marker needs the Swipe state of each entry) are inferred fills. G1 (the Game rules need the `Kind` of an event, against the three-parameter signature of ticket 20 Q13), G3 and G6 (one line for a missing preset, so `PresetMissing` goes) change an owner decision or a line that a player sees: they go to the owner as [Confirm three fills of the build plan of the response](28-confirm-the-fills-of-the-response-plan.md), which blocks R4 and R7.
- **Outside check:** a reviewer subagent (Opus) read the plan against tickets 19 to 26 and the code. Verdict "ready with fixes": 3 high, 6 medium, 7 low. All are applied. Among them: `SwipeState::pre_analysis` stays until R5 (else R4 is red); G6 found; the R1 tests whose subject goes are listed; the `player_text` search is narrowed to `Job::player_text`.

Graduated: tickets 28 (grilling) and 29 to 38 (one task per slice and the merge), wired in plan order.
