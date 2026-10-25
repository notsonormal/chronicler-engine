# Research: the storage constraints that the rebuild must keep

Type: research
Status: resolved
Blocked by: —

## Question

What must the storage of the rebuild keep, and what must change, so that the shape of [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) §3.1 works? The shape: repositories per record, every read takes a `GameId`, record writes only on a `Tx`, one `Tx` inside one `with_backend_mut` call, both backends.

Answer four points:

1. **Failure injection.** Today the test override fires by method name at the entry of `with_backend_mut`, before the closure runs (`src/adapters/driven/storage/core.rs:126-152`, `test_support.rs`). A `Tx` inside one call can only fail whole. [Turn-state-model ticket 26](../../turn-state-model/issues/26-decide-test-placement.md) R5 needs "one failure injected in the middle of a save unit", on both backends. What must failure injection carry for that, which tests use today's mechanism, and what are the candidate mechanisms?
2. **Atomic units on the in-memory backend.** How does one `Tx` land whole or not at all on `InMemory`, also when a write in the middle fails?
3. **arch-lint.** `arch-lint.toml:164-169` forbids storage to import the application layer, ports included. `arch-lint.toml:138-142` forbids the HTTP layer to import storage. State what each storage shape (port trait in `application/`, trait in `domain/`, `Storage` methods) needs from these rules.
4. **Prior decisions.** The `## Answer` sections of `.scratch/inherent-impl-locality/` tickets 03 and 11 (the `backend/` folder flattened; one `impl Storage` per file kept; the locality rule name-only), restated as constraints.

This ticket does not choose the shape. [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md) chooses it.

## Context

- Replaces [architecture-deepening ticket 01](../../architecture-deepening/issues/01-research-storage-seam-constraints.md), re-scoped to the new design by [Research: sort the open architecture-deepening tickets against the new design](04-research-sort-architecture-deepening.md) ([asset §2.1](../assets/research-04-architecture-deepening-sort.md)).
- Read-only. Cite `file:line` for every claim about the code.

## Done when

An asset `../assets/research-10-storage-constraints.md` with the four points and a list of "constraints the storage shape must keep" for ticket 11 to grill against.

## Answer

Resolved 2026-10-16. Asset: [research 10](../assets/research-10-storage-constraints.md). Facts read at commit `d067420b`.

1. **Failure injection** (asset §1). Today's override fires by name at the entry of `with_backend_mut`, before the closure, and on every call until cleared (`core.rs:141-149`). So it cannot fail half a `Tx`. 87 call sites in 28 files use it; about half name methods that the build deletes. The rest stay while the build does not break them (ticket 08 R5), so the name-keyed whole-call API must stay for those methods. Injection must move to each `Tx` record write, above the backend match. Candidates: a name key per write (F1), a typed key (F2), a count key (F3), a test closure that fails by itself (F4, no production code), or a failing adapter (F5, needs a backend trait).
2. **In-memory atomic units** (asset §2). The in-memory backend has no unit today, and a panic is not contained (poison recovery at `core.rs:134-137`). Its record writes never fail, so a mid-unit stop comes only from injection, the closure's `Err`, or a panic. SQLite rolls back on drop (rusqlite 0.34 default). Candidates: copy the whole data and swap (M1, simplest, covers a panic, test-only cost), copy one Game (M2), an undo log (M3), staged writes (M4, does not fit SQLite ids), persistent structures (M5, new dependency).
3. **arch-lint** (asset §3). arch-lint 0.4.3 scopes have no exclude and the scope-dep rule has no inline allow. A port file is in both `ports` and `application`. So a port trait in `application/ports/` needs `arch-lint.toml:164-169` changed (list the other application scopes, move the ports out, or drop the rule). A trait in `domain/model/` and methods on `Storage` need no change. A port held as `dyn` cannot carry a `transaction<T>` method that is generic over its result.
4. **Prior decisions** (asset §4). Files flat in `storage/`; one `impl Storage` per file; `mod.rs` declarations only; the locality rule name-only, so each new type with an inherent impl (`Tx`, the read handles, a backend struct) holds it in its own file or a folder with its own name; renaming the `Storage` struct is not free.

Asset §5 lists 17 constraints (C1 to C17) for [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md) to grill against. §6 gives facts for its Q1 and Q4. §7 lists three points not checked.

No new tickets. Ticket 11 is now unblocked.
