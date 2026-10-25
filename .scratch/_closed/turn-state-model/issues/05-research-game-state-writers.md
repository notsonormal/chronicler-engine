# Research: list the game-state writers and the slot each holds

Type: research
Status: resolved
Blocked by: —

## Question

Which code writes game state (Snapshots, the Game row's generation status, Message and Swipe rows), and does each writer hold the `GenerationGate` slot of that Game when it writes? Which writes can run at the same time for one Game, and which of those can lose an update?

## Scope

1. Every call site of `persist_snapshot_or_err`, `save_message_and_snapshot`, `Storage::update_swipe_location_header`, and every storage call that persists `GenerationStatus` or rewrites Messages or Swipes. The seed counts 17 game-state save sites; a grep on 2026-10-11 found 15 non-definition call lines of the first two. Reconcile the count.
2. For each writer: the entry point (HTTP route, spawned task, bootstrap), sync or `spawn_blocking`, and whether it holds the slot (`GenerationGate`, `GenerationSlot`, `try_claim`, `claim_and_spawn`).
3. `heal_stale_if_changed` (`action_pipeline/core.rs`): what it assumes about "`Generating` with no slot", and which writers break that assumption. Arrival narration is the known case (seed §3). Look for others: debug routes, swipe switch, reset, settings or World edits that touch a Game, startup.
4. How each writer loads the state it saves (fresh load, or a copy held across an LLM call). A copy held across an LLM call is a lost-update risk.
5. The arrival race in seed §3: confirm or refute each step from the code. Note what a test would need to reproduce it (the harness in `src/test_support/` never runs arrival).

## Context

- Seed: [three pipeline defects with one root](../assets/seed-evidence.md) §3 and "The shared root".
- `docs/diataxis/explanation/two-state-channels.md` (registry versus persisted status).
- [Trace the Snapshots each generation path saves](04-research-trace-generation-snapshots.md) owns the order and the pointers of each save. This ticket owns ownership and concurrency.

## Output

An asset `../assets/research-05-game-state-writers.md`: one table of writers (site, entry point, thread model, slot held, state source), a list of the concurrent pairs that can lose an update, and the assumptions `heal_stale_if_changed` makes. Label each fact known, inferred, or tested. Cite symbols, not line numbers.

## Feeds

[Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md), and the map's fog patch "The writer rule".

## Answer

Asset: [research-05-game-state-writers.md](../assets/research-05-game-state-writers.md).

- `GenerationStatus` lives only in the Snapshot, so every Snapshot write is a status write. Every write goes to the process-global `Storage::game_id()`, read per storage call. No writer passes a game id (known).
- Count: 15 call lines of the two symbols (14 distinct sites; one line is the wrapper body). The seed's 17 adds the 2 definition lines (inferred). The full writer set is larger: `save_state`, direct `save_snapshot`, and the Message/Swipe row writers.
- Slot-holding writers: everything in the spawned task of `claim_and_spawn`, plus `try_claim`. Writers with no slot: arrival, `switch_swipe` (reads `is_busy`, claims nothing), `edit_history`, `delete_last`, `/status/reset-generating`, the heal paths, and the Game catalogue (`reset`, `create_game`, `switch_game` retarget every later write).
- Nine concurrent pairs can lose an update. The largest: arrival vs a turn; a pipeline vs a game switch/reset/create (its later writes land in the new game, because `check_game_unchanged` never runs after the quantifier or options call); `handle_cancellation` writing `Idle` into the new game (tested); `/status/reset-generating` freeing a live slot; the pre-claim copy that `claim_and_spawn` and `heal_stale_if_changed` save.
- `heal_stale_if_changed` assumes that the slot owner of a game is the only writer of `Generating` into it, and that the copy it saves is current. Both break (pairs 2, 5, 7).
- Seed §3: arrival does **not** persist `Generating` (refuted), and heal has nothing to reset; the turn starts because no slot is held. The lost update by arrival's late save is confirmed as mechanism, inferred as race. New: arrival runs on every boot of a World with no scenario text. No test harness runs arrival; a crate-internal unit test can.
