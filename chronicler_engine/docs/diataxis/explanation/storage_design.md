---
diataxis: explanation
title: Storage and Bootstrap Design
---

## Overview

The persistence + bootstrap subsystem has five moving parts that fit together as one design: a single concrete `Storage` struct whose backend sits behind a mutex and is selected from a `Backend` enum; a `BackendKind` decorator that wraps a real backend for failure injection in tests; a two-phase bootstrap that seeds the database once at boot and then reads only from the database at runtime; an application-tier rule that each `Storage` method touches exactly one table and that multi-table operations compose in the application orchestrator (`ActionPipeline`, `PersistenceGate`, `GameCatalogue`, `WorldPersonaCatalogue`); and a paired `get_*` / `require_*` read-helper contract that lets the storage surface distinguish absence-as-OK from absence-as-error.

## The storage struct and its backend decorator

The persistence layer is a single concrete struct. `Storage` holds the backend behind a mutex, and every public method dispatches through one wrapper method that resolves the backend and runs the body. Callers always see one `Storage` type. The engine uses one concrete impl: the `Backend` enum names exactly two real backends (`Sqlite` for production, `InMemory` for development and unit tests), and that closed set is what the storage layer dispatches over.

The backend shape is two enums layered by decorator. The inner enum names the real backends. The outer enum is a decorator: one variant for plain use and another that wraps a real backend with failure injection. The decorator is a peer of the plain-use variant — it is always a wrapper around one of the real backends, never names a third backend of its own. The wrapper holds a single boxed real backend, so at most one decorator layer can wrap a backend at a time. A misconfigured test that tries to wrap a wrapper is a compile error; the replace-not-nest invariant is structural.

The dispatch seam sits on every method. The wrapper method acquires the mutex, unwraps any decorator wrapper, hands the underlying real backend to the method body, and routes through the real backend's implementation. Whether a failure-injection override is active or not, the production code path is the one that runs; the override short-circuits only the specific methods named by the test handle. Tests get deterministic failure injection without per-trait mock structs.

A `Storage` instance carries its game id at construction time rather than per call. Game-scoped operations read or write exactly one row of one game-scoped table (`games`, `game_state_snapshots`, `messages`, `message_swipes`); the game id is part of every key. Tests construct a per-test `Storage` against an in-memory database; production constructs a single `Storage` per session against the SQLite file.

## The database as the sole source of truth at runtime

The engine reads from SQLite at runtime and never from JSON files. The runtime card types (`WorldCard`, `PersonaCard`, `NpcCard`) carry no filesystem pointer — the database row is the only authority. If a piece of runtime code reaches for a JSON file instead of the database, it has crossed a boundary that the design holds strict.

The boundary is enforced by a two-phase bootstrap. Phase 1 runs once at boot: JSON files under `data/` are read, parsed into seed manifest types, and inserted into the database. Phase 2 covers everything after seeding: every read comes from the database, and the JSON files are no longer consulted. The HTTP server binds only after Phase 1 has completed; the bootstrap boundary is a hard precondition for serving requests.

The seed manifests are an intermediate type that exists only to parse JSON files. They are converted to runtime card types via the `WorldManifest` → `WorldCard` conversion, which strips the file-pointer fields (`map_file`, `characters_dir`) on the way in. The runtime card has no notion of where the seed JSON lived — the database row is its only home, and the manifest-to-card conversion is the seam where the filesystem coupling is dropped.

Settings live the same way: a singleton row in the `settings` table, seeded once from `data/settings.json`, then read from the database for the lifetime of the process. The bootstrap fact that belongs here is that the settings row is a database artifact from the moment seeding completes, and the runtime reads the row, never the file.

## The seeding contract

Seeding is the contract that makes the bootstrap boundary safe to re-run during development. The contract has four properties, and each one is load-bearing for a different class of dev/iteration workflow.

Idempotent. Existing rows are not duplicated. The seeder checks for the row by primary key before writing — insert-without-conflict semantics for the rows the seeder creates. A re-run over an already-seeded database is a no-op on the populated rows; the engine can be stopped, restarted, and rebooted without rebuilding the world catalogue.

Startup-blocking. The HTTP server binds only after seeding has completed. A partial seed is an unsafe state — some rows present, others missing, foreign-key edges broken — so the contract is hard: the engine refuses to serve requests until Phase 1 has finished. The seeder running to completion is the precondition for runtime.

Non-fatal on corrupt files. A malformed JSON file is skipped with a tracing warning, and the seeder continues with the remaining files. A typo in one NPC's seed file stops only that one character from loading; the engine still boots and the rest of the world remains playable. The error surfaces in the logs, where the developer can find it.

DB is the sole source of truth afterwards. Once a row exists, the JSON file that produced it is no longer consulted. The seed files are templates the seeder reads once; the database is authoritative for every read after that.

The seeding order satisfies the foreign-key edges. Worlds and maps are seeded first because characters reference the world's numeric id, and that id exists only after the world row is written. Personas are world-independent and seeded in their own pass; prompt presets are independent of world, character, and persona. Game-scoped rows that reference any seeded row see a fully-populated parent.

## Cross-table coordination in the orchestrator

Each `Storage` method touches exactly one table. The rule is structural — every public method on `Storage` opens one row or one query against one table and returns. The rule keeps the storage layer narrow: a reader of any method sees what it reads, and a writer sees what it writes.

Multi-table operations compose one tier up, in the application orchestrator (`ActionPipeline`, `PersistenceGate`, and catalogue collaborators). Save-message-and-snapshot writes the snapshot row first, then appends the message row that references the snapshot's id. Hydrate-messages-with-swipes reads the message rows and attaches the swipe rows. The orchestrator owns the order and the references; the storage layer owns the rows.

Atomicity within a single multi-table operation comes from sequential SQLite statements on one connection. The window in which a crash can leave the tables partially updated is small — a process exit between the two statements is the only way to land in a half-finished state — and the application treats the operation as atomic for non-critical data. The seed flow itself sits outside this trade-off: it runs at startup, not on user request, and its non-atomicity is bounded by the absence of concurrent readers.

## The paired read-helper contract

Some entity rows are absent-as-OK; some are absent-as-error. `Storage` exposes both shapes as paired helpers for the entities where this distinction matters, and the choice between them is the storage surface's way of telling the caller which interpretation of "missing" applies.

`get_*` returns the row wrapped in an optional — absent-as-OK. It is the right choice when the caller treats absence as a legitimate runtime state: catalogue listings, fallback paths, existence guards, validation surfaces that surface as a validation error rather than a not-found error. A listing of worlds can return zero rows; a check for whether a world exists returns yes or no, never raises.

`require_*` returns the row directly — absent-as-error. It is the right choice when the caller cannot make progress without the row. The helper maps an absent result to a typed not-found variant; a backend-side read failure propagates unchanged. A request to load the active game cannot proceed without a game row, so the helper surfaces `GameNotFound(game_id)` rather than letting the caller interpret an empty result.

The not-found variants carry the kind of id the storage interface uses for that entity. `GameNotFound` and `MessageNotFound` carry numeric ids because game and message ids are numeric throughout the storage interface. World and persona lookups use string keys, and their typed not-found variants follow the same shape with the matching key type.

The shape of the entity determines whether it has a required-read helper at all. Characters come back through a single roster helper, `list_characters(world_id)`, which returns the full character set for a world — character reads in the domain are world-scoped rosters. `Persona` and `Character` are distinct domain entities with their own storage surfaces; the persona surface keeps its `get_persona` + `require_persona` pair, while character reads stay roster-shaped.

## Messages-and-swipes

LLM narration is non-deterministic. The same player input can produce a strong paragraph on one run and a flat one on the next. The message aggregate carries this non-determinism directly: each retry of an AI message produces a new `Swipe` on the same `Message`, and the previous swipe is preserved. The player navigates between swipes; the engine holds all alternatives. The narrative cost of retry — a fresh LLM call, a snapshot to restore — is paid in full. The information cost — losing the prior generation — is zero.

### Per-swipe state binding

A swipe is not alternate text alone. Each `Swipe` carries its own `snapshot_id` pointing at the `GameStateSnapshot` that produced it. When the player navigates to a different swipe, the engine restores the entire world state that produced that swipe's text, not just the text itself.

Narration mutates state. The quantifier runs after the narration LLM and detects NPCs and movement; it updates scene state and increments encounter counters. Two different narrations produce two different post-narration states. A model that swapped only the text would leave the world state tied to whichever swipe was generated last — a "ghost state" where the displayed text no longer matches the underlying world.

The per-swipe `snapshot_id` binds each swipe to the state that produced it. Switching swipes rewinds the world to the moment that swipe was committed. Text and state stay coherent because they were captured together. The snapshot reference is deliberately not a SQL FK: declaring it as a FK would cascade snapshot deletion to swipes, which the retry semantics don't want.

### Last-message-only swiping

Swiping is bounded to the last message. Each message depends on the state produced by the message before it: a narration's quantifier detected NPCs the next narration assumes are present; an event header recorded a trigger firing the next message's state reflects.

A swipe on a non-last message would discard every message after it — the swipe rewinds state the subsequent messages were built on, so they cannot stand. The engine's retry operation handles that case as a single, well-named flow: roll the world back to a snapshot, soft-delete the messages that depended on it, regenerate. Carrying the same flow under two names (retry plus non-last swiping) would not gain capability.

The player's A/B comparison lives at the last message. Comparing earlier messages means rolling back everything after them via retry.

### Independent swipe sets across narration and event

A message can be a narration (the LLM's response to the player's action) or an event continuation (a trigger firing after the narration). Each message has its own swipe set: retrying a narration does not disturb the event that followed it; retriggering an event does not disturb the narration that preceded it.

The distinction lives in the last message's `event_header`. The retry path reads the header to decide which kind of retry to run — narration retry or event retrigger. Each generation keeps its grip on the previous independent of the other.

## Document References

- [Storage](../reference/storage.md) — current contract for the storage layer and entity persistence.
- [Startup and Bootstrap](../reference/startup.md) — current bootstrap boundary, seeding order, and schema files.
- [ADR-008: SQLite Snapshot Persistence](../../docs/adr/adr-008-sqlite-snapshot-persistence.md) — supplies the `GameStateSnapshot` that each swipe references for state-consistent switching.
- [ADR-017: Message Swipes](../../docs/adr/adr-017-message-swipes.md) — historical decision record for the swipe model.
- [ADR-020: Unified Storage Struct](../../docs/adr/adr-020-storage-consolidation.md) — historical decision record for collapsing the six-trait repository pattern into a single `Storage` struct with the `Backend` enum and `BackendKind` decorator.
- [ADR-024: Migrate Game Data to SQLite with Seed Pattern](../../docs/adr/adr-024-game-data-migration-to-sqlite.md) — historical decision record for the JSON-to-SQLite migration and the idempotent seed pattern.
- [ADR-026: Relocate Persona Binding from World to Game](../../docs/adr/adr-026-persona-relocation-to-game.md) — historical decision record for moving the persona binding off the world and onto the game, which made personas a top-level world-independent directory in the seed flow.
