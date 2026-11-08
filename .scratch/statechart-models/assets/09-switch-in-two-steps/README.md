# The switch in two steps (ticket 09)

A scratch variant of the gate model, made on 2026-11-06 in `tmp/t09/`, to check the code gap of run 1.

- `switch-in-two-steps.diff`: against `VerifyGate.tla` at sync commit `e6715c60`. `YouSwitchTheGame` becomes the Game read, and `TheSwitchSetsThePointer` sets the pointer. The new rule `ActiveExists == pending.reset = 0 => active \in exists` comes from games.md 18.2 and 19.2.
- `switch-race.cfg`: the config that broke `ActiveExists` on the variant.
- `fix-mutant-on-top.diff`, `fix-both.cfg` and `noreset.cfg`: the fix mutant `delete-check-inside-the-hold` on top of the variant, with and without the reset. TLC still breaks `ActiveExists` on two of the three orders.

The three orders are in the Comments of [ticket 09](../../issues/09-grilling-decide-the-gaps-and-the-verify-skill.md).
