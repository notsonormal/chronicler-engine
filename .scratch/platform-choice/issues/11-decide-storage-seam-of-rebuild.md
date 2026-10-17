# Decide the storage seam of the rebuild

Type: grilling
Status: resolved
Blocked by: 10

## Question

What shape does the storage code of the rebuild take? Four questions, in this order:

1. **Backend dispatch.** The build writes `stories.rs`, `docks.rs` and `transaction.rs`, about 10 record writes, for two backends. Do they extend today's `Backend` enum dispatch (one more pair of `match` arms per write), or does the build introduce a `StorageBackend` trait with a SQLite and an in-memory adapter? If a trait: do today's storage files move onto it in the build, or later?
2. **Repositories.** Are the reads (`games`, `stories`, `docks`) and the `Tx` writes port traits or methods on `Storage`? If port traits: where do they live, and which arch-lint rule changes?
3. **The CRUD services.** Do `PersonaCatalogue`, `SettingsService`, `PromptPresetService` and `WorldCatalogue` take the same shape in the build, or stay as they are?
4. **The lookup by World.** Where do "the newest Game of a World" and "the Game names of a World" live? Today they are raw SQL in `bootstrap/run.rs:168-215`, on SQLite only.

## Context

- Merges [architecture-deepening ticket 02](../../architecture-deepening/issues/02-deepen-storage-into-backend-trait.md), [ticket 03](../../architecture-deepening/issues/03-replace-crud-services-with-repository-port.md) and the lookup half of [ticket 16](../../architecture-deepening/issues/16-bootstrap-games-through-game-catalogue.md). Reasons: [Research: sort the open architecture-deepening tickets against the new design](04-research-sort-architecture-deepening.md), [asset](../assets/research-04-architecture-deepening-sort.md) §2.2, §2.3, §2.9.
- The constraints: [Research: the storage constraints that the rebuild must keep](10-research-storage-constraints-for-rebuild.md).
- Fixed by the design, not open here: repositories per record, every read takes a `GameId`, record writes only on a `Tx`, only `GameWriter` opens a transaction (sketch 25 §3.1, §6.2 rules 1, 3, 9, 14).
- The tests: ticket 26 R5 (the storage tier, one suite on both backends, about 8 cases).
- [Prototype: decide the crate split and the dev-build loop](07-prototype-crate-split-and-dev-build-loop.md) needs the answer to question 2: it sets which way `application/` and `storage/` depend on each other.
- Skills: `/grilling`, `/codebase-design` (one adapter is a hypothetical seam, two is a real one).

## Done when

The owner has answered the four questions, and the answer states the arch-lint changes, if any.

## Comments

**2026-10-16, from [Decide the test rule outside the designed scope](08-decide-test-rule-outside-designed-scope.md) (resolved).** The storage tests outside the designed records (about 3,300 unit-test lines in `src/adapters/driven/storage/`) are on the legacy list. When the storage seam that this ticket chooses breaks them, the build rewrites them into the driven-adapter seam contract in `tests/storage/`, on both backends (ticket 08 R5, R6). The answer should say which storage tests the chosen seam breaks.

## Answer

**2026-10-16, owner (grilling, two rounds).** The storage code of the rebuild keeps today's shape and adds the new records to it. Facts from [research 10](../assets/research-10-storage-constraints.md) and a check of the code at the time of the grilling. Labels: **known**, **inferred**, **guessed**.

### Rulings

| # | Question | Ruling | Reason |
|---|---|---|---|
| R1 | Backend dispatch (Q1) | **Keep the `Backend` enum.** Each new record write on `Tx` is one `match` with a SQLite arm and an in-memory arm. The `Tx` may itself be an enum over the two backends. No `StorageBackend` trait, in the build or for today's files. | The in-memory backend is the second adapter, and only tests use it, so the enum already is a seam with two adapters. A trait with `transaction<T>` is not dyn-compatible, so `Storage` becomes generic and the parameter spreads to `GameWriter` and every `Arc<Storage>` holder. A rename or merge of the `impl Storage` blocks is not free (constraints C14, C16). The full trait move ([architecture-deepening ticket 02](../../architecture-deepening/issues/02-deepen-storage-into-backend-trait.md)) is out of scope. |
| R2 | Repositories (Q2) | **Methods on `Storage`** (shape C of research 10 §3.3), as sketch 25 §3.1 draws them: `games()`, `stories()`, `docks()`, and the record writes on `Tx`. No port trait. | No arch-lint change, no dyn question, and the in-memory backend stays the fake. A port would be a seam with one adapter. |
| R3 | The unit on the in-memory backend (C5) | **M1: copy and swap.** The in-memory arm of a `Tx` clones the whole `InMemoryData`, writes on the clone, and swaps it in on `Ok`. On `Err` or a panic the clone is dropped. `#[derive(Clone)]` goes on `InMemoryData` and its helper types. | Simplest of M1 to M5. It covers `Err`, an injected failure and a panic, and it restores the id counters. The cost falls on tests only (research 10 §2.1); tier-1 Games are small (guessed). |
| R4 | Failure injection (C6) | **F2: a typed key.** An enum of record writes (for example `TxWrite::InsertSwipe`). Each record write on `Tx` checks the key above its backend `match`, so one check covers both backends. The `Tx` carries the override map from `BackendKind::Test`; in `Direct` the check does nothing (C8). Today's two error kinds (`Config`, `Internal`). No count key ("fail the Nth write") and no separate failing-closure case. Today's name-keyed whole-call override (`with_failure`, `add_failure`, `with_test_failures`, `TestOverride`) stays for every kept method (C7). | The compiler refuses a wrong key, which today's string key does not. |
| R5 | The CRUD services (Q3) | **They stay as they are.** `PersonaCatalogue`, `SettingsService`, `PromptPresetService` and `WorldCatalogue` already hold `Arc<Storage>` and call `Storage` methods, which is the shape of R2. [architecture-deepening ticket 03](../../architecture-deepening/issues/03-replace-crud-services-with-repository-port.md) is settled by R2: no Repository port. | The build does not touch them. |
| R6 | The lookup by World (Q4) | **One new `Storage` read**, `newest_game_of_world(world_key) -> Option<GameId>` (name is a placeholder), on both backends, through `with_backend_mut`, with today's order: the newest `messages.timestamp`, else `games.updated_at`. `init_game` calls it through `GameCatalogue`. `find_latest_game_for_world`, `list_game_names_for_world` and their raw SQL in `bootstrap/run.rs` go. | No behaviour change; startup can run on the in-memory backend; the failure override reaches it. `list_game_names_for_world` goes by itself, because `init_game` calls `GameCatalogue::create_game` ([research 04](../assets/research-04-architecture-deepening-sort.md) §2.9), which computes the names from `list_games()` (known, `application/games/catalogue.rs:38`). |
| R7 | The names of the transaction entries | **Unique names:** `Storage::transaction` and `Storage::transaction_new_game` (in place of sketch 25's `create`). Rule 3 of sketch 25 §6.2 stays a name-only call-site walker. | `create` is a common method name, so a name-only walker gets false hits. |

**A precision on rule C1** ("every read takes a `GameId`"), from R6. The rule applies to the reads of one Game's records (`games().get`, `stories()`, `docks()`). The reads that **find** Games take no `GameId`: `list_games()` (known, exists today) and the lookup of R6.

### arch-lint

**No change.** R2 keeps the storage code free of the application layer, so `storage → application` (`arch-lint.toml:164-169`) holds. `application → storage` is allowed today, and `server → storage` (`:138-142`) and the HTTP text match (`tests/infrastructure/guardrails/layers.rs:228-253`) stay. The read handles (`GameReads`, `StoryReads`, `DockReads`) are storage types, so the HTTP views read through `GameViewQuery` (research 10 §3.3, inferred).

**For [Prototype: decide the crate split and the dev-build loop](07-prototype-crate-split-and-dev-build-loop.md):** the dependency direction is `application → storage`. A crate split puts storage below the application.

### Constraints that the build keeps (research 10 §5)

- `Tx`, each read handle and any backend struct with an inherent impl hold that impl in their own file (C15). New storage files stay flat in `storage/`; one `impl Storage` per file (C14).
- No `Storage` call inside the closure of a `Tx` (C9). Lock order: the `GameWriter` guard, the backend mutex, the connection mutex (C10).
- On SQLite the `Tx` wraps one rusqlite transaction that rolls back on drop (C4). Every table already uses `AUTOINCREMENT` (known, `storage/utils/plumbing.rs:47` and others), so a rolled-back insert also rolls back its `sqlite_sequence` update (inferred).

### The storage tests that the chosen seam breaks

**The seam itself breaks no kept test** (inferred). R1 and R2 keep today's shape. R3 adds a derive. R4 adds a check inside `Tx` and keeps the whole-call override. The breaks come from the design's record changes (sketch 25 §3.1, §7.1). Line counts are known (`wc -l`):

| File | Lines | Fate | Why |
|---|---|---|---|
| `snapshots_tests.rs`, `messages_tests.rs`, `swipes_tests.rs`, `mappers/state_snapshot_tests.rs` | 1,387 | go with their module | the modules are replaced by `stories.rs`, `docks.rs`, `transaction.rs` |
| `mappers/message_tests.rs` | 163 | breaks (inferred) | `Message` gets `kind`, `Swipe` gets `maker` and `state` |
| `core_tests.rs` | 191 | breaks | 14 override keys name `save_snapshot` and `load_latest_snapshot`, which go. The mechanism tests are rekeyed to a kept method |
| `games_tests.rs` | 275 | partly | `create_game` and `delete_game` move onto `Tx`; `set_game_id` goes |
| `db_tests.rs` | 269 | partly | the schema cases of `game_state_snapshots` and `message_swipes` |
| `bootstrap/run_tests.rs`, the lookup cases | — | move | R6; they become cases of the storage suite on both backends |
| characters, llm_messages, personas, presets, settings, worlds, `mappers/llm_message` | ~1,750 | stay | outside the designed records; ticket 08 R5 |

Each broken test is rewritten into the storage seam contract in `tests/storage/`, on both backends (ticket 08 R5, R6; ticket 26 R5). The new R4 failure injection serves the "one failure in the middle of a save unit" case of that suite.

### A risk for [Decide the build path](09-decide-build-path.md) (outside this ticket)

The design removes `Storage::game_id` (sketch 25 §3.1). That is a compile break across 240 calls to `set_game_id`, `current_game_id` and `storage.game_id()` in 50 files, and 34 `Storage::new_sqlite(pool, game_id)` calls in 9 files (known, `grep` over `src/` and `tests/`). The build path must plan for it.

**2026-10-17, amended by [Review: the design and the build decisions against standard engineering practice](15-review-design-against-standard-practice.md).** The owner's rulings there change parts of this answer (see its Comments: Q2 for ticket 08, Q4 for ticket 11, Q3 for ticket 13). Where the two conflict, ticket 15 wins.

**2026-10-17, amended by [Decide the four gaps that the build plan found](16-decide-the-build-plan-gaps.md) Q1.** Today's Game writes do not move onto `Tx` in P5. They move with create and reset onto `GameWriter` in the core step. In P5, `Tx` has no production caller.
