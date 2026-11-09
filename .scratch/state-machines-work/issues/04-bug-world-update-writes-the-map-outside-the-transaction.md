# A World update writes the World row and the map row with no transaction

Status: needs-triage
Source: ticket 02 of the state-machine map, the second scan (scan 2, 2026-11-09): `scan2-driven.md`, *Storage calls that write in two statements with no transaction* (no catalogue row; a defect, not a machine)

## The defect

`Storage::update_world` (`src/adapters/driven/storage/worlds.rs:297-335`) runs `UPDATE worlds` and then `write_map_row` (`:332`) on the plain connection, with no transaction. If the map write fails, the World row keeps its new values with the old map. The call returns `Err`, so the caller reports a failure for a write that is half done.

The code elsewhere treats this state as invalid. `write_world` (`worlds.rs:163-185`) wraps the same two writes in one transaction, with the comment "World and map are one write: a failed map write must not leave a re-seeded world behind a stale map" (`:174`).

Both World routes call `update_world`: the detail save (`src/adapters/driving/http/worlds/handlers/worlds.rs:213`) and the posture save (`:292`), through `src/application/world_catalogue.rs:59`.

## Impact (inferred)

Low. The map write must fail after the World write succeeded, for example on a full disk or a lock error. The `worlds` change number is not raised on `Err` (`src/adapters/driven/storage/core.rs:249-252`), so an open dashboard does not reload the half-written World.

## Done when

- `update_world` writes the World row and the map row in one transaction, as `write_world` does.
- A test makes the map write fail and finds the World row unchanged.
