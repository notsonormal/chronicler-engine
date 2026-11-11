# An accept clears the failure line but keeps the Options failure

Source: ticket 02 of the state-machine map, the second scan (scan 2, 2026-11-09): `scan2-application.md`, finding 1 ([catalogue](../../wayfinder-maps/state-machines-next/assets/catalogue.md), row A7). Checked by the parent session in the working tree.
Waits for: the answer to question 13 of [ticket 03](../../wayfinder-maps/state-machines-next/issues/03-research-apply-the-rule-to-the-server-game-state.md), which the owner gives in ticket 06.

## Owner of the area

The failure line (`failLine`) and the Options failure (`optFail`) are in the `verify-gate` TLA+ model (row A7), which the statechart-models map owns. A change brings the TLA+ model along (statechart-models ticket 09, Q3). Run `/verify-gate` after the change.

## The behaviour

`GameWriter::accept_in_tx` (`src/application/generation/gate.rs:700-704`) clears `games.failure` when a Job starts. It does not touch the dock, so an Options failure from an earlier Job stays.

- If the new Job saves a Narration, the text save empties the dock (`gate.rs`, `save_text`), and the Options failure goes. Scenario 24.27 of `docs/specs/options.md` covers this.
- If the new Job fails before its Narration is saved, the old Options failure stays next to the new failure line. No scenario covers this.

This is not a plain defect: the specs do not say which result the player must see.

## Done when

- The owner answers question 13 of ticket 03.
- If the answer changes the behaviour: a scenario in `docs/specs/options.md`, a test, the code change, and the matching change in `VerifyGate.tla`.
- If the answer keeps the behaviour: a scenario in `docs/specs/options.md` that pins it, and a test. Then close this ticket.
