# Make the World edit and the reset all or nothing

Type: task (AFK, decide and do)
Status: open
Blocked by: —

## Question

Make two writes all or nothing. Choose each fix under the enforcement order of the map Notes, and put it in place.

- **The World edit (storage-04).** `write_world` uses a transaction. `update_world` writes the `worlds` row and then the `maps` row on the bare connection (`src/adapters/driven/storage/worlds.rs:305-332`). A failed map write leaves the card and the map different. The fix is one transaction, as `write_world` does.
- **The reset (gate-07).** `replace` commits the create unit, removes the old holder, then runs the delete unit (`src/application/generation/gate.rs:606-616`). When the delete fails, the running Job is stopped by a reset that reported failure, and the new Game stays as an orphan. Its doc comment says that a failure leaves the old Game, its entry and the pointer as they were.
  A `Tx` is bound to one Game, so the create and the delete cannot share one transaction today. The candidate fixes: a transaction for two Games; or undo the create when the delete fails, and remove the old holder only after the delete commits.

## Context

- Findings in full: storage-04 in [the storage report](../assets/storage.md), gate-07 in [the gate report](../assets/gate.md). The coordinator confirmed both in the code.
- A transaction for two Games changes the shape of `Tx`. If that is not a local change, stop and say so (map Notes).
- `replace` is covered by `verify-gate`, so the task runs `/verify-gate` (map Notes).
- The test of each fix uses the injected failures of `src/adapters/driven/storage/write_guard.rs`.
- The six end conditions of the map Notes apply.

## Answer
