# A failed Job whose failure line does not save leaves the Game Idle with no line

Source: ticket 02 of the state-machine map, the second scan (scan 2, 2026-11-09): `scan2-application.md`, A2 *Job life* ([catalogue](../../wayfinder-maps/state-machines-next/assets/catalogue.md), row A2). Checked by the parent session in the working tree.

## Owner of the area

The Job is in the `verify-gate` TLA+ model (row A2), which the statechart-models map owns. The agent that fixes this brings the TLA+ model along before it ends its task (statechart-models ticket 09, Q3). Run `/verify-gate` after the fix.

## The defect

`Job::end` (`src/application/pipeline/job.rs:385-416`) saves the failure line through `JobWriter::save_failure` (`src/application/generation/gate.rs:828-832`). When that save fails with an error other than `EntryMoved`, the `Err(refused)` arm (`job.rs:398-404`) only logs a warning and returns `JobEnd::Failed`.

So the Job failed, but the Game stores no failure line. After the `JobWriter` drops, `status` reads the Game as Idle (`src/application/games/view_query.rs`, `status`). The player sees no report of the failure. An Autoplay run ends with `JobFailed`, but its failure line is also missing.

## Impact (inferred)

Low. The path needs a storage write to fail at the end of a Job, which most often means that the database itself fails. Then other reads fail too. But the result contradicts `docs/diataxis/reference/game_flow.md:99`: a failed Job leaves a failure line that "stays until the player acts".

## Done when

- A decision: what the player sees when the failure line itself cannot be saved. Options: a failure line that only memory holds (for example in the busy registry, until the next accept), or a log line only, documented as the known limit.
- A test with an injected failure of `set_game_failure` at the end of a failed Job checks that decision.
- `VerifyGate.tla` models the branch, or `coverage.toml` of `verify-gate` names it as a branch that the TLA+ model leaves out.
