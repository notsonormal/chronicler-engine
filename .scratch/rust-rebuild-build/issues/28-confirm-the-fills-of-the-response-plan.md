# Confirm three fills of the build plan of the response

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

The build plan of the response ([§12](../../../docs/plans/rust-rebuild-response-build-plan.md#12-gaps-the-plan-states)) fills three gaps that change an owner decision or a line that a player sees. Confirm or change each one:

1. **G1, the Game rules of an event.** Ticket 20 Q13 deleted `PreAnalysisState`, so `game_rules::apply(state, &QuantifierResult, &GameWorld)` cannot tell an event (no movement, no Trigger check) from a Narration. Fill: `apply` takes the `Kind` of the order as a fourth parameter. This amends the signature of ticket 20 Q13.
2. **G3, a missing hook preset.** Ticket 20's read-line rule (`Record::Preset`) makes a missing Quantifier or Options preset the failure line "The data for this request is missing: the preset.", in place of a run with no preset (`job.rs:798`). Fill: yes, as the rule says.
3. **G6, a missing narrator preset.** Today it shows its own line, `PresetMissing`: "The active prompt preset is missing." (scenario at `docs/specs/failure_display.md` line 88). With G3, one fault has two lines, against ticket 20 Q2. Fill: `Record::Preset` for every preset; `PresetMissing` goes; the scenario changes its line.

R4 (G1) and R7 (G3, G6) wait for this ticket. R1 to R3 do not.

## Context

- The plan's reviewer (Opus, 2026-10-20) found G1 sound and small, and G6 a decision (its H3).
- Skills: `/grilling`. Terms: `CONTEXT.md`.

## Answer

Resolved with the owner on 2026-10-20. The lead explained each gap with its options; the owner chose option A for all three. Plan §12 records them as decisions.

| Gap | Decision |
|---|---|
| G1 | `game_rules::apply(state: SwipeState, result: &QuantifierResult, world: &GameWorld<'_>, kind: Kind)`. The Game-rules step passes `order.kind()`. This amends the three-parameter signature of [ticket 20](20-shape-of-the-job.md) Q13. Not taken: two entry points (`apply`, `apply_event`); event rules in the Job (finding 4); `&Generation` (texts that the rules do not read) |
| G3 | A missing Quantifier or Options preset is the failure line "The data for this request is missing: the preset." (`Record::Preset`), not a run with no preset. On an analysis, the Narration stays and the state update does not run; the Options run still follows (ticket 20 Q1) |
| G6 | One line for every missing preset: `Record::Preset`. `GenerationFailureKind::PresetMissing` ("The active prompt preset is missing.") goes, and the scenario of `docs/specs/failure_display.md` (line 88 at `5d3cfe5f`) changes its line in R7. Not taken: `PresetMissing` for every preset (an exception to the read rule); two lines for one fault |

R4 and R7 are no longer blocked by this ticket.
