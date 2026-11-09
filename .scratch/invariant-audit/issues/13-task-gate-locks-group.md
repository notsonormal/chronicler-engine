# Decide and close the gate locks group (tier 4)

Type: task (AFK, decide and do)
Status: open
Blocked by: —

## Question

Give the one verdict for the gate locks group under the enforcement order of the map Notes, and put it in place. A part of the group can get its own verdict when one mechanism does not fit it.

| Finding | Invariant that nothing enforces |
|---|---|
| gate-01 | The active Game pointer names an existing Game. At boot, a missing row only logs a warning, and `touch_game_updated_at` ignores 0 affected rows, so `PointerHold::set` cannot fail on a missing Game. |
| gate-03 | The lock order: no `Storage` call inside a `Tx` closure, no hold inside a hold, no `ActiveGame` read under the guard. Only comments state it. A violation hangs the whole server, the Stop button too. |
| gate-04 | One server process for each database file. The busy registry is in memory, and no file lock and no `busy_timeout` exist. Two processes can both accept a Job on one Game. |

The candidate mechanisms, from the report: boot fails on a missing Game row and `set` checks the row count (gate-01); a thread-local marker that panics on a second entry into a unit, a hold or the pointer (gate-03); an exclusive lock on the database file in `DbPool::new` (gate-04).

## Context

- Findings in full: [the gate report](../assets/gate.md). The storage report says that a second process "fails loudly", and the gate report says that it can run two Jobs on one Game. Both are partly right: a competing write fails at once, but the busy state is per process ([the report](../assets/REPORT.md), Disagreements). Nobody confirmed when the migrations run compared with the port bind.
- [Fix the active pointer that can name a deleted Game](../../statechart-models/issues/10-task-fix-the-active-pointer-that-can-name-a-deleted-game.md) (statechart-models) added the `PointerHold` and the TLA+ rule `ActiveExists`. gate-01 is the boot path, which that ticket did not change.
- These functions are covered by `verify-gate`, so the task runs `/verify-gate` (map Notes). A model check is not runtime enforcement: the answer says which of these invariants only the TLA+ model holds today.
- A marker that panics in a release build changes what the player sees (a crash in place of a hang). Under the decide-and-do rule, ask the owner before that choice.
- The six end conditions of the map Notes apply.

## Answer
