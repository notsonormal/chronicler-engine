---
diataxis: explanation
title: Storage and Bootstrap Design
---

## The storage struct and its test decorator

`Storage` is one concrete struct, and every public method dispatches through one wrapper that resolves the backend. The `Test` decorator wraps the real backend and short-circuits only the methods that a test names. Every other call runs the production code path. Tests get deterministic failure injection with no per-trait mock struct.

A `Storage` holds no Game, so each caller names the Game it reads or writes. A Job therefore cannot write to a Game that a switch made active while the Job ran.

## Change counters in the storage layer

Every save of data passes the one wrapper that dispatches a `Storage` method, or the commit of a transaction. The change counters rise at those two points, so rename, the LLM message recorder, Settings, prompt presets and Worlds raise their numbers with no code of their own. Two exhaustive maps name the kinds of each call and each write, so a new save path that forgets its kinds fails the build instead of leaving a part of the screen stale. The numbers are counters, not dates, so they do not depend on the system clock. The application layer holds no counter port, because the storage adapter may not depend on the application layer.

## The database as the sole source of truth at runtime

The engine reads seed JSON files once, at boot, and reads only the database after that. A runtime card carries no file pointer, so runtime code has no path back to the files. Code that reads a JSON file at runtime has crossed the bootstrap boundary.

A malformed `data/settings.json` stops the boot. The engine does not start with a half-read configuration.

## The seeding contract

Each property of seeding supports a development workflow:

- **Idempotent.** A re-run over a seeded database writes nothing new, so the engine can restart without a rebuild of the world catalogue.
- **Startup-blocking.** A partial seed has broken foreign-key edges, so the server binds only after seeding completes.
- **Non-fatal on a corrupt file.** A typo in one Character's file stops only that Character. The rest of the world stays playable, and the warning is in the log.

Worlds and maps seed before Characters, because a Character row references the numeric id that exists only after its world row is written.

## Transactions

Sequential statements allow a crash between two statements, which leaves a text with no state, or a state with no text. The transaction closes that gap. The Swipe holds its own Swipe state, so the common write is one Swipe row. A write that needs more than one row (create, a text save, a swipe switch, a delete) writes them in one transaction.

The dock record is a table of its own, because the config routes write the Game record with no busy check.

`GameWriter` decides what a transaction holds, because it knows the request and checks the job id in the same hold. `Storage` supplies the mechanism and knows nothing of Jobs. A reader of a `Tx` method sees one record write, and a reader of `GameWriter` sees which records change together.

Every test that uses `Storage` runs on SQLite. A rollback and a failure in the middle of a transaction are properties of the real backend, and a fake would only mimic them.

The seed flow sits outside this rule. It runs at startup, not on a user request, and no concurrent reader exists then.

## The paired read helpers

`get_*` and `require_*` let the storage interface tell the caller which meaning of "missing" applies. A listing or an existence check treats absence as a valid state. A request that cannot proceed without the row gets a typed not-found error in place of an empty result that each caller would interpret.

Characters have no required-read helper, because the domain reads Characters as the roster of a world.

## Messages and Swipes

LLM narration is non-deterministic. The same input can give a strong paragraph on one run and a flat one on the next. Each Retry therefore adds a new `Swipe` to the same `Message` and keeps the previous one. A Retry costs a fresh LLM call, and the prior generation is not lost.

### Per-swipe state binding

Narration changes state: the Quantifier detects Characters and movement, and the game rules update the scene and the encounters. Two narrations of one input therefore leave two different game states. Each Swipe keeps the Swipe state of its own generation, so the text on screen and the world state always come from the same generation. A text-only swipe would show the text of one generation over the state of another: a "ghost state".

The current state is the Swipe state of the active Swipe of the last Message. A swipe switch and a delete change the current state by changing which Swipe is active. Nothing is copied or restored.

### Two writes to one Swipe state

The story log shows a new text as soon as the narrator call returns, and the analysis follows a few seconds later. The Job therefore saves the text with its pre-analysis state first, and writes the analysed state after. The pre-analysis state says that the player has not moved and no Trigger fired, which is true until the analysis lands.

### Last-message-only swiping

Each Message depends on the state of the Message before it: the Characters that one narration's Quantifier detected, and the Trigger that one event header recorded. A swipe on an earlier Message would rewind the state that every later Message was built on, so those Messages could not stand.

## Document References

- [Storage](../reference/storage.md) — the storage contract and the records of a Game.
- [Startup and Bootstrap](../reference/startup.md) — the bootstrap boundary and the seeding order.
