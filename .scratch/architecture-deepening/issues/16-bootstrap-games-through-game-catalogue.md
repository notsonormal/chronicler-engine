# 16 — Bootstrap creates and finds Games only through GameCatalogue

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to making bootstrap create and look up Games only through
`GameCatalogue` (and Storage), removing its raw SQL and its second
Game-creation path — and if so, what does `GameCatalogue` need to expose for
startup?

## Background

This is **candidate H** of the 2026-10-04 review, rated **Worth exploring**.
See `assets/architecture-review-2026-10-04.html`, card H.

The friction:

- **Raw SQL in the composition root.** `src/bootstrap/run.rs` (around lines
  164-212) calls `db_pool.conn()`, prepares statements and runs
  `query_row` / `query_map` against the games table by `world_key`. The
  in-memory backend never sees these queries, so this startup path cannot run
  on `BackendKind::InMemory`.
- **Second creation path.** `bootstrap/init_game.rs::resolve_game_id` builds
  its own `NewGame` plus `mode_preset_registry.bundle_for(...)`, mirroring
  `GameCatalogue::create_game` (`application/games/catalogue.rs`). A change to
  how a Game is created (Narrator Mode preset, display name) must be made in
  both.

Both errors are reported as `EngineError::Config`.

## What this ticket resolves

- **Commit or reject.**
- **Interface.** Which lookups `GameCatalogue` (or Storage) gains, such as
  "games for World", and whether `resolve_game_id` becomes a catalogue call.
- **Test impact.** Whether startup becomes testable on the in-memory backend,
  and which tests would cover it.

## Constraints

- Touches the Storage seam. Read ticket 01's asset (once it exists) as a
  constraint. Do not re-open the rejected single-file `impl Storage`
  consolidation (`.scratch/inherent-impl-locality/`).
- `bootstrap/` is allowed to import everything (composition root). The
  question is duplication and testability, not a layer-rule breach.
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling` and `/domain-modeling` (Game, World, Narrator
  Mode).
- Not hard-blocked by 01–03, but if ticket 02 adopts a backend trait, these
  queries would need a home in it anyway.
