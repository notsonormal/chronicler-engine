# A refused accept on a removed Game makes its change counters again

Source: ticket 03 of the state-machine map, the apply ticket of the server Game state ([dispositions](../../wayfinder-maps/state-machines-next/assets/03-dispositions.md), row B11; scan 2 "to check" item of B11)

## The defect

`docs/diataxis/reference/storage.md:68` says: "A Game that the server deletes or replaces loses its per-Game numbers." `GameWriter::replace` and `GameWriter::delete` call `ChangeCounters::forget_game` after their hold (`src/application/generation/gate.rs:576-593`).

A later request can still raise the numbers of the removed Game:

1. Request A reads the active Game id once, in the `CurrentGame` extractor (`src/adapters/driving/http/utils/current_game.rs`, `from_request_parts`) (known).
2. Request B resets the Game. `replace` removes the old Game and calls `forget_game(old)` (known).
3. Request A calls `GameWriter::accept` with the old id, for example for an Action. The removed Game has no tail, so `JobPlan::narration_like` answers `PlanError::Missing(Record::Story)` (`src/domain/model/generation.rs:307-314`). `plan_and_start` turns it into `GameWriter::missing` (`gate.rs:415-424`, `gate.rs:630`) (known).
4. `not_accepted_with_failure` (`gate.rs:648-658`) opens a transaction that calls `Tx::set_game_failure`. That `UPDATE` (`src/adapters/driven/storage/transaction.rs:234-249`) changes 0 rows and does not fail. The transaction commits, and `raise_unit` raises busy state for the old id (`transaction.rs:406-407`) (known).

So the counters hold a per-Game entry for a Game that no longer exists. `Tx::set_game_config` checks the number of changed rows, but `set_game_failure` does not.

## Impact (inferred)

Low. Game ids come from `AUTOINCREMENT` (`src/adapters/driven/storage/utils/plumbing.rs:66`), so no later Game gets the old id, and `GET /changes` answers only for the active Game. A client sees no difference. The entry is a small memory leak, and a write that "succeeds" on a missing Game hides the refusal reason in the log.

## Done when

- A write of the failure line on a missing Game fails, as `set_game_config` does, so the transaction rolls back and raises nothing.
- A test removes a Game, then sends a refused accept for its id, and finds no per-Game entry for it in `ChangeCounters::versions`.
