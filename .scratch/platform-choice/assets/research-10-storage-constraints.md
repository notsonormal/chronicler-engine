# Research 10: the storage constraints that the rebuild must keep

For [Research: the storage constraints that the rebuild must keep](../issues/10-research-storage-constraints-for-rebuild.md). Read-only. Facts read at commit `d067420b`. Labels: **known** (read in the code or a document, with a citation), **inferred** (follows from known facts), **guessed**.

The shape under test is [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) §3.1: repositories per record, every read takes a `GameId`, record writes only on a `Tx`, one `Tx` inside one `with_backend_mut` call, both backends. This asset does not choose the storage shape. [Decide the storage seam of the rebuild](../issues/11-decide-storage-seam-of-rebuild.md) chooses it.

## 0. Summary

1. **Failure injection** must move from the entry of `with_backend_mut` to each record write of a `Tx`, or a test must be able to make the save unit fail by itself. Today's mechanism can only refuse a whole call before it starts. 87 call sites in 28 files use it. About half of them name methods that the build deletes. The rest name methods outside the designed scope, and ticket 08 R5 keeps those tests while the build does not break them. So today's name-keyed API must stay for the kept methods.
2. **The in-memory backend has no unit today.** Each write changes the data in place. A `Tx` needs a copy-then-swap, an undo log, or a staged list. A copy of the whole `InMemoryData` is the simplest and also covers a panic. The cost falls only on tests, because only tests use the in-memory backend.
3. **arch-lint 0.4.3 cannot make an exception for `ports`.** A scope has only a name and paths, with no exclude. A port file is in the `ports` scope and in the `application` scope. So a port trait in `application/ports/` that storage implements needs a changed rule, not an added rule. A trait in `domain/model/` and methods on `Storage` need no arch-lint change.
4. **The prior decisions** keep the files flat in `storage/`, keep one `impl Storage` per file, and keep the locality rule name-only. Each new type with an inherent impl (`Tx`, the read handles, any backend struct) must hold its impls in its own file or in a folder with its own name.

§5 lists 17 constraints for ticket 11.

## 1. Failure injection

### 1.1 How it works today (known)

| Part | Where | What it does |
|---|---|---|
| The key | `core.rs:126-133` | Every storage method passes its name as `method: &'static str` to `with_backend_mut`. 47 calls in 11 files (`grep with_backend_mut`, non-test). Nothing checks that the string equals the method name. |
| The test layer | `core.rs:29-36` | `BackendKind::Test { base, overrides }` wraps the real backend. `overrides` is a `HashMap<&'static str, TestOverride>` behind `Arc<Mutex<..>>`. |
| The check | `core.rs:141-149` | On entry, before `f` runs: if `overrides` holds the method name, return `override_.to_error()`. `f` does not run, so the backend does not change. |
| The override | `test_support.rs:9-19, 67-87` | An error kind (`Config` or `Internal`) and a message. `to_error` gives `EngineError::Config` or `EngineError::Internal`. |
| Persistence | `core.rs:142-147` | The check uses `get`, not `remove`. An override fires on every call until the test clears it. There is no "fail once" and no "fail on the Nth call". |
| The handle | `test_support.rs:21-55` | `set`, `clear`, `clear_all`, `assert_no_unconsumed`. `assert_no_unconsumed` panics when the map is not empty. Because nothing removes a fired override, it means "not cleared", not "not fired" (inferred from `:48-54` and `core.rs:142-147`). |
| The entry points | `core.rs:55-97, 100-119` | `with_failure(method, o)` (builder), `add_failure(&self, method, o)` (on a shared `Storage`), `with_test_failures()` (returns the handle). They replace the backend; they never nest (`core_tests.rs:126-160`). |
| The feature gate | `core.rs:7-14`, `Cargo.toml:55-58` | `#[cfg(feature = "testing")]`. `testing` is a default feature, so the `Test` variant is in every default build (known). `build.py:1790` builds the release with `cargo build --release` and no feature flags, so the release has it too (inferred). |
| Paths that skip it | `bootstrap/run.rs:168-212`; `db.rs:37-67` | The lookup by World and `insert_game_from_request` call `DbPool::conn()` directly. They skip `with_backend_mut`, so no override reaches them (known). |

### 1.2 Who uses it (known, counted with `perl` over `with_failure`, `add_failure` and `.set` with a string key)

| Group | Files | Call sites | Method names |
|---|---|---|---|
| Storage unit tests, `src/adapters/driven/storage/*_tests.rs` | 11 | 49 | each file names its own methods; `core_tests.rs` uses `save_snapshot` and `load_latest_snapshot` (13) |
| Application unit tests | 8 | 19 | `get_world` (5), `save_snapshot` (4), `load_snapshot_by_id` (3), `get_persona` (2), `insert_swipe`, `load_message_rows`, `load_latest_snapshot`, `get_game`, `update_swipe_location_header` |
| `src/utils/settings_tests.rs` | 1 | 1 | `get_settings` |
| HTTP handler unit tests | 2 | 8 | `save_preset` (3), `get_preset` (2), `delete_preset`, `update_settings`, `list_presets` |
| Tier 1, `tests/http/` | 7 | 10 | `update_game_config` (2), `save_preset` (2), `get_settings`, `update_settings`, `create_world`, `update_world`, `get_game`, `list_games` |
| `tests/storage/` | 0 | 0 | — |
| **Total** | **28** | **87** | |

Only SQLite users: `settings_tests.rs` and `presets_tests.rs` in storage. Every other user wraps `Storage::new_in_memory()` (known: the only files that call both an injection entry point and `new_sqlite` or `DbPool::new` are `core.rs`, `core_tests.rs`, `settings_tests.rs` and `presets_tests.rs`).

**Which users the build breaks** (inferred from sketch 25 §7.1 and ticket 26 R10):

- **They go with their module:** the storage tests of `snapshots`, `messages` and `swipes`; `core_tests.rs` (it uses `save_snapshot`); the tests of `arrival_service`, `message_service`, `narration_generation` and the `action_pipeline` files. Their keys name methods that the build deletes: `save_snapshot`, `load_latest_snapshot`, `load_snapshot_by_id`, `insert_message`, `insert_swipe`, `load_message_rows`, `update_swipe_*`, `update_active_swipe`, `shift_swipe_indices`, `soft_delete_message`, `restore_soft_deleted`, `purge_soft_deleted`, `delete_message`, `load_swipes_for_messages`.
- **They may break:** `catalogue_tests.rs` (`save_snapshot`; create and reset change); `games_tests.rs` in storage (`create_game` and `delete_game` move onto `Tx`, sketch 25 §3.1); `tests/http/requires_migration/core.rs` (`get_game`, "simulated reset failure"; reset changes).
- **They stay** while the build does not break them (ticket 08 R5): the storage tests of `characters`, `llm_messages`, `personas`, `presets`, `settings` and `worlds`; `settings_tests.rs` in `utils`; the two HTTP handler test files; the tier-1 tests in `failure_display.rs`, `games_config.rs`, `prompt_presets.rs`, `requires_migration/fragment.rs`, `settings.rs` and `worlds.rs`. Their keys name methods outside the designed scope, except `update_game_config`, `get_game` and `list_games` on the Game record.

### 1.3 What ticket 26 R5 needs that today's mechanism cannot give

R5: "One suite runs on both backends, with one failure injected in the middle of a save unit." Rule 14 of sketch 25 §6.2: "a transaction that fails half-way leaves no row".

- A `Tx` runs inside one `with_backend_mut` call (sketch 25 §3.1). The check fires before the closure (`core.rs:141-149`). So an override on the `transaction` call fails the unit **before** its first write, and the test sees nothing half-way (known, ticket 04 asset §2.1 fact 1).
- An override cannot fire **between** two writes, because the record writes of a `Tx` do not go through `with_backend_mut`. They take `&mut Backend` from the one closure, as `worlds.rs:148-191` does today (inferred). A record write that called `with_backend_mut` again would deadlock on the `std::sync::Mutex` at `core.rs:134` (known: the std mutex is not re-entrant).

### 1.4 What failure injection must carry

1. **A position inside the unit.** The test must name which write fails: the record write (a name or a typed kind), or its order in the unit (the Nth write).
2. **A check at the write, above the backend match.** The same check must fire for SQLite and for the in-memory backend, so one suite covers both (R5).
3. **The error kinds of today** (`Config`, `Internal`), so the error path above storage does not change.
4. **Today's whole-call override for the kept methods** (§1.2 "they stay"), unless ticket 11 moves their files onto a new shape. Then those tests break and ticket 08 R5 rewrites them into the storage seam contract.
5. **No cost in a normal build.** Today the check runs only in `BackendKind::Test` (`core.rs:139` is a plain call in `Direct`). A new check must also do nothing outside the test layer.

### 1.5 Candidate mechanisms

| # | Mechanism | Mid-unit | Both backends | Production code | Risk |
|---|---|---|---|---|---|
| F1 | **Name key at each `Tx` write.** The `Tx` carries the override map. Each record write calls `self.check("insert_swipe")?` before its `match` on the backend. | Yes | Yes | One line per record write (about 10, sketch 25 §3.1), plus the map in `Tx` | The key is a string. A typo never fires; today's only guard is the panic in `assert_no_unconsumed` (`core_tests.rs:115-121`) |
| F2 | **Typed key.** As F1, with an enum of record writes (`TxWrite::InsertSwipe`) in place of the string. | Yes | Yes | As F1, plus the enum | The compiler refuses a wrong key. The enum must change with each new write (inferred) |
| F3 | **Count key.** "Fail the Nth record write of the next `Tx`." | Yes | Yes | A counter in `Tx` | The test does not name the write. A suite can walk every position of a unit, as the G3 stop matrix walks every step (ticket 05). A change of write order changes which write fails, silently (inferred) |
| F4 | **A failing closure.** The test calls `Storage::transaction` with its own closure: write, write, then return `Err`. | Yes, at the closure level | Yes | None | It tests "the closure fails" only, not "a record write fails". On SQLite a real write failure can be made with a foreign key (`db.rs:21` turns `foreign_keys` on); the in-memory backend has no such failure (§2.2) |
| F5 | **A failing backend adapter.** A third adapter that wraps a real one and fails a chosen write. | Yes | Yes | The wrapper | Needs a backend trait: ticket 11 Q1, option "trait" |

F1 to F3 can combine: a typed key with an optional count. F4 needs no mechanism and fits rule 14's wording ("a transaction that fails half-way"). Today the call-site walker of rule 3 checks `src/` files only (`check_src_files`, `tests/infrastructure/guardrails/mod.rs`), so a test in `tests/storage/` may call `Storage::transaction` (inferred).

## 2. Atomic units on the in-memory backend

### 2.1 Today (known)

- **One transaction in all of storage:** `write_world` (`worlds.rs:155-166`, SQLite arm). The world row and the map row land together. The in-memory arm (`worlds.rs:167-190`) changes the data in place. It can return `Err` (`WorldAlreadyExists`, `:174`) only before it writes.
- **One multi-write with no unit:** `persist_swipes` (`swipes.rs:200-207`) calls `insert_swipe` once per Swipe, logs each error and goes on. A failure leaves part of the Swipes.
- **The migrations** run in one transaction (`utils/plumbing.rs:35`, `:724`).
- **In-memory writes do not fail.** The in-memory arms of `games.rs`, `messages.rs`, `swipes.rs` and `snapshots.rs` hold no `Err(` (an `awk` scan of each arm; the one hit, `swipes.rs:203`, is in `persist_swipes`, outside any arm). So on the in-memory backend a unit can stop half-way only for three reasons (inferred):
  1. an injected failure (§1);
  2. the closure of the save unit returns `Err` between two writes (application code);
  3. a panic between two writes.
- **A panic is not contained.** `with_backend_mut` recovers a poisoned mutex with `into_inner` (`core.rs:134-137`). After a panic in the middle of an in-memory unit, the next caller sees the half-written data (inferred).
- **SQLite rolls back by itself.** rusqlite 0.34 `Transaction` has `DropBehavior::Rollback` by default (`rusqlite-0.34.0/src/transaction.rs:127`), and its `Drop` finishes by that behaviour (`:242-246`, `:225`). An `Err` with `?` or a panic drops the transaction, so the unit rolls back (inferred). `worlds.rs:161` uses `unchecked_transaction` because `pool.conn()` gives a `MutexGuard<Connection>` (`db.rs:30-35`), not a `&mut Connection` (inferred).
- `InMemoryData` has no `#[derive(Clone)]` (`in_memory_data.rs:16`). Its record types derive `Clone` (`game.rs:11`, `message.rs:8, 22`, `llm_message.rs:7`; known for these).
- Only tests use the in-memory backend. `Storage::new_in_memory()` has no caller in `src/` outside `core.rs` and the test files (known: `grep`, non-test files of `src/` without `src/test_support/`). `src/test_support/` uses it for test apps (`test_app_builder.rs`, `context.rs`).

### 2.2 Candidates

| # | Mechanism | Closure `Err` | Injected failure | Panic | Cost |
|---|---|---|---|---|---|
| M1 | **Copy the whole data, write on the copy, swap on `Ok`.** | Yes | Yes | Yes: the copy is dropped | `#[derive(Clone)]` on `InMemoryData` and its three helper types. One full copy per unit, about 5 lines. Time grows with all the stored data; each Swipe will hold a full `SwipeState` (sketch 25 §3.1, design §7). Tests only (§2.1) |
| M2 | **Copy only the records of the unit's Game.** The `Tx` holds one `GameId` (rule 1), so a unit touches one Game. | Yes | Yes | Yes | Less copy time. More code: the copy must know every table that a unit touches. `create` has no Game yet, and `games().remove` touches the list of Games (inferred) |
| M3 | **Undo log.** Each record write pushes its inverse; on `Err`, replay the log backwards. | Yes | Yes | Only with a drop guard | One inverse per record write (about 10). A wrong inverse is a silent defect in the fake that every tier-1 test uses (inferred) |
| M4 | **Staged writes.** The `Tx` keeps the writes as values and applies them at the end. | Yes | Yes | Yes | A later write of a unit often needs the id of an earlier one (a Swipe needs its Message id). SQLite gives the id only after the insert (`last_insert_rowid`, `db.rs:66`). So staging fits the in-memory backend only, and the two backends then differ in shape (inferred) |
| M5 | **Persistent data structures** (for example the `im` crate): a copy costs O(1), then as M1. | Yes | Yes | Yes | A new dependency. Every field type of `InMemoryData` changes (guessed) |

The id counters (`next_game_id`, `next_message_id`, `in_memory_data.rs:18-22`) return to their old values with M1, M2 and M5. With M3 they need their own inverse. SQLite reuses a rolled-back rowid when the table has no `AUTOINCREMENT` (inferred; not checked against the schema).

### 2.3 Locks around a unit (known, unless marked)

- A unit takes the backend mutex (`core.rs:134`), then, on SQLite, the connection mutex (`db.rs:30-35`).
- `GameWriter` takes its guard first (sketch 25 §3.2). Research 06 §2.1: the guard, then the storage mutex; a read takes the storage mutex only. Its proposed rule 18 makes `hold(game, |entries, tx| ..)` the one place that locks (research 06 §5). Ticket 12 has not ruled on it yet.
- The lookup by World takes the connection mutex with no backend mutex (`bootstrap/run.rs:172, 203`). This keeps the order, because it takes no other lock (inferred).
- A `Storage` call inside the closure of a `Tx` deadlocks (§1.3). So every read that a save unit needs happens before the `Tx`, under the guard, or the `Tx` offers it (inferred).

## 3. arch-lint

### 3.1 What arch-lint 0.4.3 can express (known)

- A scope is a name and a list of path globs (`arch-lint-core-0.4.3/src/declarative/config_dto.rs:33-38`). There is no exclude.
- `deny-scope-dep` maps the target of each `use crate::…` item to every scope that holds its file (`rules.rs:377-404`). A file in `src/application/ports/` is in scope `ports` and in scope `application` (`arch-lint.toml:50-52, 69-71`). So a rule that denies `application` also denies `ports`. `arch-lint.toml:145-147` notes the same overlap from the other side.
- The rule reads only `use` items with a `crate::` prefix (`rules.rs:303-306`, `:407-447`). A full path inside an expression, or a `super::` path, is not checked. `bootstrap/run.rs:168-215` uses full paths, not `use` items.
- No inline allow exists for this rule. The visitor at `rules.rs:407-447` reads no attribute; only `required_crate.rs` calls the allow helpers (`utils/allowance.rs`).

### 3.2 The rules that touch storage today (known)

| Rule | Where | Says |
|---|---|---|
| storage → application | `arch-lint.toml:164-169` | Storage may depend on domain models only. Ports included. |
| server → storage | `arch-lint.toml:138-142` | HTTP goes through the application layer. |
| ports → storage, storage-models, … | `arch-lint.toml:148-151` | A port may not name an adapter. |
| X → storage-models | `arch-lint.toml:108-136` | Only storage sees its row types and mappers. |
| HTTP storage leak (syn) | `tests/infrastructure/guardrails/layers.rs:228-253` | A text match on `storage::Storage` in HTTP files. |

Also known: 16 non-test files in `src/application/` import `adapters::driven::storage`, and no rule denies it. The LLM and text-check adapters import `application::ports` (`ollama.rs:8`, `deepseek.rs:7`, `openrouter.rs:8`, `mock.rs:9`, `harper_text_checker.rs:15`), and no rule denies it. Storage is the only driven adapter that may not see the ports. `guardrails_application_storage_direct`, which inherent-impl-locality ticket 03 named, no longer exists (no match in `tests/infrastructure/`).

### 3.3 What each storage shape needs

| Shape | arch-lint | Other facts |
|---|---|---|
| **A. Port traits in `application/ports/`**, storage implements them | `:164-169` must change. An added rule cannot help (§3.1). Options: (a) replace `to = ["application"]` with scopes that list every part of `application/` except `ports/`; a new folder in `application/` is then not covered until someone adds it (inferred); (b) move the ports out of `application/` (for example `src/ports/`); the five adapter files above change their imports (known count); (c) drop storage → application. The ports rule (`:148-151`) already holds: a port names domain types and `crate::error` only (`src/error.rs` is in no scope). | Follows the precedent of `LlmProvider` and `TextChecker`. A trait method that is generic over its result, as `transaction<T>(f: impl FnOnce(&mut Tx) -> …)` is, cannot be called through `dyn` (Rust's dyn-compatibility rule, known). `LlmProvider` is held as `Arc<dyn LlmProvider>` (`llm_recorder.rs:26`). So a port either makes its users generic (`GameWriter<S>`), or uses a non-generic signature, or keeps `Tx` a concrete type (inferred). The `layers.rs:228-253` text match does not see trait names (inferred) |
| **B. Traits in `domain/model/`**, storage implements them | No change. Storage → `model` has no deny rule. `model` must not import `application`, `server`, `narrative` or `storage-models` (`:78-81`, `:108-111`); a trait that names domain types and `crate::error` passes (inferred). Sketch 25 rule 12 adds `model` → `storage`, which this shape keeps | Puts an outbound port in the domain layer. The repo puts driven ports in `application/ports/` (`AGENTS.md`: "Application ports: outbound interfaces (driven port traits)"). The ports rule (`:148-151`) would not cover it (inferred). The dyn-compatibility fact of A applies |
| **C. Methods on `Storage`** (the sketch's draft) | No change. Application → storage is allowed today, and server → storage stays denied | The in-memory backend stays the fake (`tests/STRATEGY.md`, the unit row). The read handles (`GameReads`, `StoryReads`, `DockReads`) are storage types, so an HTTP file may not name them (`:138-142`); views read through `GameViewQuery` (inferred). No dyn question |

For all three shapes: sketch 25 rule 3 is a call-site walker on the names `transaction` and `create`. Today's walkers match names, not resolved paths (sketch 25 §6.3). `create` is a common method name, so the row needs the receiver type or the path to avoid false hits (inferred).

## 4. Prior decisions, as constraints

From the `## Answer` sections of `.scratch/_closed/inherent-impl-locality/issues/03-…` and `11-…` (the issue's path `.scratch/inherent-impl-locality/` moved to `_closed/`).

| # | Decision | Source | Constraint on the rebuild |
|---|---|---|---|
| P1 | The `backend/` folder is flattened. Every storage file sits in `adapters/driven/storage/` | 03, `:62-77` | New storage files go in the storage root, as sketch 25 §3.1 already writes (`stories.rs`, `docks.rs`, `transaction.rs`) |
| P2 | One `impl Storage` per file stays. A single-file consolidation was rejected as undesirable | 11, `:138-140` | Do not merge the `impl Storage` blocks to make a trait or a dispatch change easier |
| P3 | The locality rule stays name-only. Folder cohesion is review policy | 11, `:98-104` | The rule (`inherent_impl.rs:111`) passes an `impl Foo` only in the file that defines `Foo` or in a folder named `snake(Foo)`. It skips trait impls (`inherent_impl.rs:101`). So `impl Tx` sits in the file that defines `Tx`; `impl StoryReads` in the file that defines `StoryReads`; a backend struct (`SqliteBackend`) holds its inherent impls in its own file or in `sqlite_backend/`. `impl StorageBackend for SqliteBackend` may sit anywhere (inferred from the two lines) |
| P4 | Renaming `Storage` (for example to `StorageBackend`) re-flags all split impls, unless the impls move into a matching folder | 11, `:141-144` | A trait may take the name `StorageBackend`. Renaming the struct is not free |
| P5 | `mod.rs` holds declarations and re-exports only | 03, `:50`; `guardrails_mod_purity`, `tests/infrastructure/guardrails/mod.rs:189` | `storage/mod.rs` stays declarations only |

## 5. Constraints the storage shape must keep (for ticket 11)

Source: **D** = the design or a ruling (fixed); **F** = a fact of the code or the tools; **I** = inferred here.

| # | Constraint | Source |
|---|---|---|
| C1 | Every read takes a `GameId`; every record write goes through a `Tx` that holds one `GameId`; only `GameWriter` opens a `Tx` | D: sketch 25 §3.1, §6.2 rules 1, 3, 9, 14 |
| C2 | One `Tx` is one hold of the backend mutex: it runs inside one `with_backend_mut` call | D: sketch 25 §3.1; research 06 §2.1 |
| C3 | A unit lands whole on both backends, also when the closure returns `Err` or panics in the middle | D: rule 14, ticket 26 R5. The panic part: F, §2.1 (poison recovery) |
| C4 | On SQLite the `Tx` wraps one rusqlite transaction; drop rolls back | F: §2.1 |
| C5 | On the in-memory backend the `Tx` needs one of M1 to M5 (§2.2). Today it has none | F: §2.1 |
| C6 | Failure injection can fail one record write in the middle of a `Tx`, on both backends, with today's two error kinds | D: ticket 26 R5. F: §1.3 |
| C7 | The whole-call override (`with_failure`, `add_failure`, `with_test_failures`, `TestOverride`) keeps working for every method whose tests the build does not break | D: ticket 08 R5. F: §1.2 |
| C8 | The test layer costs nothing outside `BackendKind::Test` | F: `core.rs:139` |
| C9 | No `Storage` call inside the closure of a `Tx`: the std mutex is not re-entrant | F: `core.rs:134` |
| C10 | Lock order: the `GameWriter` guard, then the backend mutex, then the connection mutex. No path takes them in another order | F: `core.rs:134`, `db.rs:30-35`. D: research 06 §2.1 (rule 18, open in ticket 12) |
| C11 | Storage does not import the application layer, ports included, unless ticket 11 changes `arch-lint.toml:164-169`. arch-lint 0.4.3 cannot add an exception; the rule itself must change | F: §3.1, §3.2 |
| C12 | HTTP does not import storage | F: `arch-lint.toml:138-142`, `layers.rs:228-253` |
| C13 | A port trait that `GameWriter` holds as `dyn` cannot have a method that is generic over its result | F: Rust dyn compatibility; I: §3.3 |
| C14 | Storage files stay flat in `storage/`; one `impl Storage` per file; `mod.rs` declarations only | D: P1, P2, P5 |
| C15 | Each new type with an inherent impl holds it in its own file or a folder with its own name; trait impls are free | F: P3 (`inherent_impl.rs:101, 111`) |
| C16 | Renaming the `Storage` struct is not free | D: P4 |
| C17 | `tests/storage/` is one suite on both backends; about 50 `_sqlite`/`_in_memory` twins in `src/` fold into it | D: ticket 26 R5. F: 50 twin functions in `src/adapters/driven/storage/*_tests.rs`; `tests/STRATEGY.md:21` still names SQLite only, and `tests/storage/message_storage.rs:193` already uses the in-memory backend |

## 6. Facts for the other questions of ticket 11

- **Q1, the dispatch.** 47 `with_backend_mut` calls in 11 files (§1.1). Each record write today is one `match` with a SQLite arm and an in-memory arm (for example `insert_swipe`, `swipes.rs:11-48`). The build adds about 10 record writes and 5 reads (sketch 25 §3.1).
- **Q4, the lookup by World.** `find_latest_game_for_world` and `list_game_names_for_world` (`bootstrap/run.rs:168-212`) run raw SQL on `DbPool`. They exist on SQLite only, skip `with_backend_mut` (so no override and no backend mutex), and order by the newest Message, then `games.updated_at` (`:175-184`). `insert_game_from_request` (`db.rs:37-67`) is a third write on `DbPool` outside `Storage`. Sketch 25 §3.1 removes `set_game_id`, which today touches `games.updated_at` (`core.rs:155-181`); `ActiveGame::set` keeps that touch (sketch 25 §3.2).

## 7. Not checked

- Whether the schema uses `AUTOINCREMENT` on `games`, `messages` and `message_swipes` (matters for M3 and for rolled-back ids on SQLite).
- The run time of a full `InMemoryData` copy in a large tier-1 test (M1). Guessed small: tier-1 Games hold few Messages.
- Whether `tests/storage/` already runs any case on both backends in one function.
