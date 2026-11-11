# The first TLC run of `/statechart` has no bound

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A config with millions of TLC states runs for longer than one bash call allows, and the result is lost. Must the first run stop at a bound?

## Context

Source: `/retro` 2026-11-08, the coordinator session of statechart-models tickets 08, 10, 11, 12, 13 and 16 (`01a1bd22`), and its subagent sessions (t13 `41648274`).

- **known** t13 ran two configs in one call. The report said "TLC reached 6,916,020 states … 681s", then the call ended with "Command timed out after 1200 seconds". The second result was lost. Then t13 waited about 10 more minutes in a `sleep 10` poll loop.
- **known** Ticket 13's Answer: "The first TLC run still has no bound, as before." The dump run stops at `DUMP_STATES` and the later runs at `RERUN_STATES` (`scripts/ir.ts`).
- **known** Interim fix, applied 2026-11-08: `statechart/SKILL.md`, "Show the chart", says to run one config per call and a config over about 1,000,000 states in the background.
- **guessed** TLC 1.7.4 can stop at a state count through a CONSTRAINT on `TLCGet("distinct")`.

## Options to decide

- **A.** Stop the first run at a bound, with a flag to lift it, and say so in `report.md`.
- **B.** Keep the interim `SKILL.md` line only.
