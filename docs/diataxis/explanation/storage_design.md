---
diataxis: explanation
title: Storage and Bootstrap Design
---

## Overview

The persistence + bootstrap subsystem has five moving parts that fit together as one design: a single concrete `Storage` struct whose backend sits behind a mutex and is selected from a `Backend` enum; a `BackendKind` decorator that wraps a real backend for failure injection in tests; a two-phase bootstrap that seeds the database once at boot and then reads only from the database at runtime; a save-unit rule that every write to a Game's story records runs inside one `Tx`, opened by `GameWriter`, so the records of one request land whole; and a paired `get_*` / `require_*` read-helper contract that lets the storage interface distinguish absence-as-OK from absence-as-error.

## The storage struct and its backend decorator

The persistence layer is a single concrete struct. `Storage` holds the backend behind a mutex, and every public method dispatches through one wrapper method that resolves the backend and runs the body. Callers always see one `Storage` type. The engine uses one concrete impl: the `Backend` enum names exactly two real backends (`Sqlite` for production, `InMemory` for development and unit tests), and that closed set is what the storage layer dispatches over.

The backend shape is two enums layered by decorator. The inner enum names the real backends. The outer enum is a decorator: one variant for plain use and another that wraps a real backend with failure injection. The decorator is a peer of the plain-use variant — it is always a wrapper around one of the real backends, never names a third backend of its own. The wrapper holds a single boxed real backend, so at most one decorator layer can wrap a backend at a time. A misconfigured test that tries to wrap a wrapper is a compile error; the replace-not-nest invariant is structural.

The dispatch seam sits on every method. The wrapper method acquires the mutex, unwraps any decorator wrapper, hands the underlying real backend to the method body, and routes through the real backend's implementation. Whether a failure-injection override is active or not, the production code path is the one that runs; the override short-circuits only the specific methods named by the test handle. Tests get deterministic failure injection without per-trait mock structs.

A `Storage` holds no Game, so each caller names the Game it reads or writes, and a Job cannot write to a Game that a switch made active. Tier-1 tests construct a per-test `Storage` against a SQLite `:memory:` database; production constructs a single `Storage` per session against the SQLite file.

## The database as the sole source of truth at runtime

The engine reads from SQLite at runtime and never from JSON files. The runtime card types (`WorldCard`, `PersonaCard`, `NpcCard`) carry no filesystem pointer — the database row is the only authority. If a piece of runtime code reaches for a JSON file instead of the database, it has crossed a boundary that the design holds strict.

The boundary is enforced by a two-phase bootstrap. Phase 1 runs once at boot: JSON files under `data/` are read, parsed into seed manifest types, and inserted into the database. Phase 2 covers everything after seeding: every read comes from the database, and the JSON files are no longer consulted. The HTTP server binds only after Phase 1 has completed; the bootstrap boundary is a hard precondition for serving requests.

The seed manifests are an intermediate type that exists only to parse JSON files. They are converted to runtime card types via the `WorldManifest` → `WorldCard` conversion, which strips the file-pointer fields (`map_file`, `characters_dir`) on the way in. The runtime card has no notion of where the seed JSON lived — the database row is its only home, and the manifest-to-card conversion is the seam where the filesystem coupling is dropped.

Settings live the same way: a singleton row in the `settings` table, seeded once from `data/settings.json`, then read from the database on every use. The bootstrap fact that belongs here is that the settings row is a database artifact from the moment seeding completes, and the runtime reads the row, never the file. Seeding the settings row is the one fail-loud exception to the corrupt-file tolerance below: a malformed `data/settings.json` stops the boot instead of starting the engine with a half-read configuration.

## The seeding contract

Seeding is the contract that makes the bootstrap boundary safe to re-run during development. The contract has four properties, and each one supports a different class of dev/iteration workflow.

Idempotent. Existing rows are not duplicated. The seeder checks for the row by primary key before writing — insert-without-conflict semantics for the rows the seeder creates. A re-run over an already-seeded database is a no-op on the populated rows; the engine can be stopped, restarted, and rebooted without rebuilding the world catalogue.

Startup-blocking. The HTTP server binds only after seeding has completed. A partial seed is an unsafe state — some rows present, others missing, foreign-key edges broken — so the contract is hard: the engine refuses to serve requests until Phase 1 has finished. The seeder running to completion is the precondition for runtime.

Non-fatal on corrupt files. A malformed JSON file is skipped with a tracing warning, and the seeder continues with the remaining files. A typo in one NPC's seed file stops only that one character from loading; the engine still boots and the rest of the world remains playable. The error surfaces in the logs, where the developer can find it.

DB is the sole source of truth afterwards. Once a row exists, the JSON file that produced it is no longer consulted. The seed files are templates the seeder reads once; the database is authoritative for every read after that.

The seeding order satisfies the foreign-key edges. Worlds and maps are seeded first because characters reference the world's numeric id, and that id exists only after the world row is written. Personas are world-independent and seeded in their own pass; prompt presets are independent of world, character, and persona. Game-scoped rows that reference any seeded row see a fully-populated parent.

## Save units

Every write to the records of a Game's story runs inside one `Tx`. A `Tx` is one SQLite transaction. It holds the backend mutex for its whole run and rolls back when it is dropped without a commit. A save unit therefore lands whole or not at all.

The unit is the answer to a failure that sequential statements allow: a crash between two statements leaves a text with no state, or a state with no text. The Swipe holds its own Swipe state, so the common write is one Swipe row, and a write that needs more than one row (create, a text save, a swipe switch, a delete) writes them in one unit.

The dock record is a table of its own, because the config routes write the Game record with no busy check.

`GameWriter` decides what a unit holds, because it knows the request and checks the job id in the same hold. `Storage` supplies the mechanism and knows nothing of Jobs. The split keeps the layers narrow: a reader of a `Tx` method sees one record write, and a reader of `GameWriter` sees which records change together.

A unit holds the backend mutex for its whole run, so a `Storage` call inside the closure of a unit would wait on the mutex that the unit itself holds.

The new records run on SQLite only. Tier-1 tests run on a SQLite `:memory:` database for this reason: a rollback and a failure in the middle of a unit are properties of the real backend, and an in-memory fake would only mimic them. Legacy unit tests in `src/` keep the in-memory backend for the tables it still holds.

The seed flow sits outside this rule. It runs at startup, not on user request, and its non-atomicity is bounded by the absence of concurrent readers.

## The paired read-helper contract

Some entity rows are absent-as-OK; some are absent-as-error. `Storage` exposes both shapes as paired helpers for the entities where this distinction matters, and the choice between them is the storage interface's way of telling the caller which interpretation of "missing" applies.

`get_*` returns the row wrapped in an optional — absent-as-OK. It is the right choice when the caller treats absence as a legitimate runtime state: catalogue listings, fallback paths, existence guards, validation checks that surface as a validation error rather than a not-found error. A listing of worlds can return zero rows; a check for whether a world exists returns yes or no, never raises.

`require_*` returns the row directly — absent-as-error. It is the right choice when the caller cannot make progress without the row. The helper maps an absent result to a typed not-found variant; a backend-side read failure propagates unchanged. A request to load the active game cannot proceed without a game row, so the helper surfaces `GameNotFound(game_id)` rather than letting the caller interpret an empty result.

The not-found variants carry the kind of id the storage interface uses for that entity. `GameNotFound` and `MessageNotFound` carry numeric ids because game and message ids are numeric throughout the storage interface. World and persona lookups use string keys, and their typed not-found variants follow the same shape with the matching key type.

The shape of the entity determines whether it has a required-read helper at all. Characters come back through a single roster helper, `list_characters(world_id)`, which returns the full character set for a world — character reads in the domain are world-scoped rosters. `Persona` and `Character` are distinct domain entities with their own storage interfaces; the persona interface keeps its `get_persona` + `require_persona` pair, while character reads stay roster-shaped.

## Messages-and-swipes

LLM narration is non-deterministic. The same player input can produce a strong paragraph on one run and a flat one on the next. The message aggregate carries this non-determinism directly: each retry of an AI message produces a new `Swipe` on the same `Message`, and the previous swipe is preserved. The narrative cost of retry is a fresh LLM call. The information cost — losing the prior generation — is zero.

### Per-swipe state binding

Narration mutates state. The Quantifier runs after the narration LLM. It detects NPCs and movement, and the game rules update the scene state and the encounter counters. Two narrations of one input therefore leave two different game states. Each Swipe keeps the Swipe state that its own generation produced, so the text on screen and the world state always come from the same generation. A text-only swipe would show the text of one generation over the state of another: a "ghost state".

A Swipe state on the Swipe also removes the need for a live state. The current state is the Swipe state of the active Swipe of the last Message, so a swipe switch and a delete change the current state by changing which Swipe is active. Nothing is copied or restored.

### Two writes to one Swipe state

The story log shows a new text as soon as the narrator call returns. The analysis follows a few seconds later. The Job therefore writes the Swipe state twice: the pre-analysis state at the text save, and the analysed state at the state update. A stop or a restart between the two keeps the text with its pre-analysis state, which says that the player has not moved and no Trigger fired. Only the Job that made the Swipe writes its Swipe state, so no other writer can overwrite what that Job saved.

### Last-message-only swiping

Swiping is bounded to the last message. Each message depends on the state produced by the message before it: a narration's quantifier detected NPCs the next narration assumes are present; an event header recorded a trigger firing the next message's state reflects.

A swipe on a non-last message would discard every message after it — the swipe rewinds state the subsequent messages were built on, so they cannot stand.

## Document References

- [Storage](../reference/storage.md) — current contract for the storage layer and entity persistence.
- [Startup and Bootstrap](../reference/startup.md) — current bootstrap boundary, seeding order, and schema files.
