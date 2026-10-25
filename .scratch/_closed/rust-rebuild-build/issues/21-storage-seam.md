# Decide the storage seam: the in-memory adapter, the story projections and the read failures

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

The `InMemory` backend can no longer create a Game, and 14 arms only refuse. What is the storage seam? Decide:

1. **Delete the in-memory adapter** (SQLite `:memory:` is the one stand-in) **or split the seam** along the catalogue and story line (candidate M2, finding 7). Either way, the unit-tier row of `tests/STRATEGY.md` follows.
2. **The read-failure injection**: string-keyed today, against the typed `TxWrite` for writes; and the storage seam's missing read-failure case.
3. **One story entry instead of three projections** (`StoryRow`, `MessageEntry`, `StoryEntry`; candidate M5), where serde and `agent-edge` allow it.

## Context

- Inputs: [synthesis.md](../assets/architecture-review/synthesis.md) rows M2, M5; [FINDINGS.md](../assets/whole-rebuild-review/FINDINGS.md) finding 7 and the storage-seam low finding; the three runs differ here (Opus deletes, GLM and Qwen split).
- Skills: `/grilling`, `/domain-modeling`, `/codebase-design` (its design-it-twice pattern for the interfaces). Terms: `CONTEXT.md`. Design: `.scratch/_closed/turn-state-model/assets/target-design.md`.

## Answer

(The rounds, 2026-10-20.) Round 1 owner answers: Q1 A (delete the in-memory adapter; `Storage::new_in_memory()` keeps its name and opens SQLite `:memory:`); Q2 A (a typed `StorageCall` key for read failures; `db-error-context` of ticket 20 is the storage read-failure case); Q3 A (`StoryRow` is the one story entry; `MessageEntry` and `StoryEntry` go). 

### The answer (confirmed by the owner, 2026-10-20)

| Q | Decision |
|---|---|
| Q1 | Delete the in-memory adapter. `Backend` has one variant, SQLite. `Storage::new_in_memory()` keeps its name and opens SQLite `:memory:`, so the 159 calls in 32 files do not change. Deleted: `in_memory_data.rs`, the 40 `Backend::InMemory` arms (9 only refuse), `no_in_memory`, the no-op branch of `touch_game_updated_at`, the placeholder backend in `add_failure`, the in-memory branch of `seed_game_row`. |
| Q2 | A read failure takes a typed key `StorageCall`, one variant per storage method (38 today), the same way as `TxWrite`; a typo does not compile. The `"get_gam"` test goes. The `db-error-context` case of ticket 20 is the storage read-failure case of finding 28. Ticket 22 decides if the test layer ships in production. |
| Q3 | `StoryRow` is the one story entry. It gets `Serialize` and serves the story log view models, the debug DTO, the prompt assembler history and the Agents. Deleted: `MessageEntry`, `StoryEntry` and the 3 `From` impls. |

**Doc changes with the build:** `tests/STRATEGY.md` unit-tier row ("SQLite `:memory:` `Storage`" in place of "in-memory `Storage`"); `docs/diataxis/reference/storage.md` (remove "the in-memory maps").

**Tests that pin it.**

| Tier | Asserts |
|---|---|
| storage, `db-error-context` (ticket 20) | a failed read gives `Database { context, .. }` |
| unit, `core_tests.rs` | a `StorageCall` key fails only the call it names; `clear_all` removes it |
| existing | every storage, unit and tier-1 test stays green on SQLite `:memory:` |

**Risk (inferred):** unit tests that depend on in-memory behaviour (the `touch_game_updated_at` no-op, `settings_seeded`) can fail on SQLite; the build fixes each one. The build plan of the response counts them.
