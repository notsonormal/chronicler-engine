# ADR-030: is_generating Dual-Source Invariant — AtomicBool Is Cached View of Persisted Status

**Date:** 2026-07-06
**Status:** Accepted

## Context

The application state holds two representations of the same fact — "is a generation in progress?":

1. `src/application/application_service.rs` (`DefaultApplicationService`) holds `is_generating: Arc<AtomicBool>` and exposes it via `pub fn is_generating(&self) -> &Arc<AtomicBool>` (per-instance, process-scoped). The atomic boolean is a hot-path-readable boolean used by the HTTP poll endpoint — accessed without the application-tier API churn that op-context lookups previously required.
2. `state.narrative.input_buffer.status: GenerationStatus` is the persisted, durable record of the same state, written through `Storage` and read back on demand.

The atomic boolean exists because the poll endpoint is the hottest read path in the application; hitting the storage layer on every poll would dominate latency and consume connection-pool budget for no semantic benefit. The persisted enum exists because the generation status must survive process restarts and be readable by recovery / debugging tooling.

The dual-source design is a latent race / state-divergence risk: two writers could change the atomic boolean and the persisted status independently and let them drift.

Three mitigations were considered:

1. **Collapse to single source.** Make the persisted `GenerationStatus` the only source of truth. Rejected: re-introduces a storage read on every poll, breaking the latency budget that motivated the AtomicBool in the first place.
2. **Strict single-writer enforcement via the type system.** Make the AtomicBool unreachable outside `ApplicationService` by hiding it behind a newtype whose only mutating methods are `pub(crate)` and take `&ApplicationService`. Rejected: requires wrapping every poll read site to go through `ApplicationService` rather than the per-instance accessor, churning the public surface of `src/application/application_service.rs`.
3. **Document the invariant + enforce with a property test.** Lock the rule that only `ApplicationService` mutates both representations, in the same critical section; require a property test that asserts no drift post-mutation. Accepted.

The third option is the lightest-weight mitigation that closes the divergence risk: it makes the rule machine-checkable without restructuring the poll path or rewriting every read site.

## Decision

`is_generating: Arc<AtomicBool>` is a cached view of the persisted `GenerationStatus` enum, not an independent source of truth. The dual source is intentional, not accidental, and the invariant governing it is part of the architectural contract.

### Roles

- `GenerationStatus` (persisted in `state.narrative.input_buffer.status`) is the **source of truth**.
- `is_generating: Arc<AtomicBool>` is a **hot-path cache** to avoid a storage read on every poll.

### Single-Writer Rule

- `ApplicationService` is the single writer for `true` transitions (CAS `false → true` at generation start) and for the persisted `GenerationStatus` field on both branches.
- `GenerationGuard::Drop` is a second writer for the AtomicBool `false` transition only — the RAII fallback that clears the cache when the generation task completes or panics. It does **not** touch the persisted status.
- Two writers never disagree on value: both write `false` on completion / failure paths; only `ApplicationService` writes `true`. The cache is therefore monotone-falling outside `ApplicationService`.
- The mutation is performed **atomically** with respect to observers: the AtomicBool store and the persistence write happen in the same critical section, with no observable intermediate state in which the two representations disagree.
- "Atomic with respect to observers" means: any caller reading the AtomicBool after observing a persisted write must see a value consistent with that write. In practice, the order is `store AtomicBool` → `persist GenerationStatus`, so a reader polling during the gap sees the new AtomicBool value paired with the not-yet-flushed status, which the next poll (after persistence completes) converges on.

### Read-Only Elsewhere

- All code paths outside `ApplicationService` and `GenerationGuard::Drop` treat the AtomicBool as **read-only**.
- Mutation sites outside `ApplicationService` (true transitions + persisted status) and `GenerationGuard::Drop` (false RAII transition only) are forbidden. Any **third** writer is a bug.
- The read path (HTTP poll endpoint) continues to use `is_generating.load()` directly. This is the whole point of the cache.

### Verification Strategy

- A property test asserts that after any `ApplicationService` mutation, `AtomicBool.load() == (persisted_status == GenerationStatus::Generating)`.
- The property test must be able to detect injected divergence (i.e. if the test artificially mutates one representation but not the other, the test fails).
- Concurrent execution (4 threads driving `DefaultApplicationService::process_action`) must produce no observed divergence.

### Why Dual Source Over Collapse

- The poll path is hot. A storage read per poll defeats the design intent of the AtomicBool cache and would dominate request latency under load.
- The AtomicBool gives O(1) read with a single, documented writer. The cost of dual sources is paid once, at mutation time, by holding the critical section that touches both representations.
- A property test turns the invariant from "documented convention" into "machine-checked contract". Drift between the two representations fails the test suite rather than shipping.
- Collapsing the sources does not actually reduce total complexity — it just shifts the cost from the mutation path to the read path, where it is observed by every poll request.

## Consequences

### Positive

- The poll path stays O(1). No storage read on every poll request. Latency budget preserved.
- The invariant is enforceable via a property test, giving machine-checked confidence in the single-writer rule.
- The dual-source design is documented as intentional, removing the "is this drift a bug or by design?" question that future maintainers would otherwise face.
- The cost of dual sources is concentrated at the mutation site, not spread across the read path.

### Negative

- Two representations of the same fact means the invariant **must** be documented and audited. A future contributor who adds a new mutation path without reading this ADR risks introducing drift.
- The AtomicBool and the persisted status are coupled in time, not in type. A test that catches drift must exercise both representations explicitly.
- The "atomic with respect to observers" guarantee is weaker than a single-source-of-truth design. A reader polling during the store-then-persist window sees a transiently consistent view. Acceptable because the persisted write follows within milliseconds and the next poll converges.

### Trade-offs

- Chose dual source with documented invariant + property test over collapse to single source — the latency cost of collapse is paid by every poll, which is unacceptable for the hot path.
- Chose single-writer rule enforced by documentation + audit over type-system-level enforcement (newtype + `pub(crate)` mutators) — the type-system approach would force a wider refactor of `src/application/application_service.rs` and the poll endpoint, with marginal safety benefit once the property test exists.
- Chose store-then-persist ordering over persist-then-store — the read path observes the new AtomicBool value first, which matches the "is generating right now?" semantics a poll reader expects (better to briefly report "generating" than to briefly report "idle" while a generation is mid-flight).
- Accepted that the property test is the binding safety mechanism. If the test is ever deleted or weakened, the invariant degrades from "machine-checked" to "convention only".

## Access Pattern

The `pub(crate)` widening on `DefaultApplicationService` storage/generation fields was deliberate and expected to be re-tightened by the T2 god-class split (completed 2026-07-09, tickets 00–04). After T2, generation-related access now flows through `application/generation_gate/` and storage access through `application/persistence_gate/`.

Ticket 07 refines, but does not discard, the single-writer invariant on `is_generating`. `GenerationGate` owns the per-game `Arc<RwLock<HashMap<GameId, GenerationSlot>>>` registry and the `Arc<AtomicBool>` projection. The registry claim/release path is the single writer. `GameCatalogue` lifecycle operations do not write the projection and no longer guard on it.

## Related ADRs

- ADR-010: Concurrency and Generation Gate Model — established the original `AtomicBool` generation gate. ADR-030 extends that decision by adding the persisted-source-of-truth requirement and the single-writer rule.
- ADR-027: Hexagonal Architecture Migration — parent decision on the storage direct-access exemption (`Storage` is accessed directly by the application persistence boundary). Relevant because the persisted `GenerationStatus` write goes through `Storage`, which is a documented exemption per ADR-027.

## History

- **2026-07-06**: Initial decision. Locks the dual-source roles, the single-writer rule, the read-only-everywhere-else rule, and the property-test verification strategy.
- **2026-07-06**: Acknowledged `GenerationGuard::Drop` as second writer for the AtomicBool (RAII panic-safety, writes `false` only). Tightened verification strategy to fail-fast on injected `(cached=false, persisted=Generating)` divergence.
- **2026-07-10**: Amended for Ticket 07 per-game generation tracking. The registry is now the write-side truth and `is_generating` is a projection.

## Amendment: Ticket 07 (2026-07-10)

Ticket 07 preserves ADR-030's invariant while refining the live generation gate from a global `AtomicBool` claim to per-game tracking. Earlier wording that describes `ApplicationService` as the writer of the global CAS is historical for the pre-07 model. Current live concurrency tracking is owned by `GenerationGate`.

### Per-game tracking and projection

`GenerationGate` now owns an `Arc<RwLock<HashMap<GameId, GenerationSlot>>>` registry. That registry is the write-side truth for live generation claims and releases. `is_generating: Arc<AtomicBool>` remains, but it is a read-only projection for callers: `true` means at least one registry slot is `GenerationSlot::Generating`.

The single-writer rule is still preserved. The writer is now the registry claim/release path, not a standalone global CAS on the `AtomicBool`. The same path that mutates the per-game registry also updates the `AtomicBool` projection: claim stores `true`; release stores `false` only after the registry has no remaining generating slots. No other code path may mutate the projection.

The persisted `GenerationStatus` still records durable phase/status for recovery and debugging. It is no longer the live concurrency gate; the per-game registry is the live gate, and the atomic is only its hot-path projection.

### Game lifecycle guard removal

`GameCatalogue::create_game`, `GameCatalogue::switch_game`, and `GameCatalogue::delete_game` no longer guard against `is_generating`. Ticket 05 explicitly authorized `reset()` to proceed while generation is in flight, and Ticket 07 extends the same mechanical safety to create/switch/delete.

Guards are removed because the α-check at phase boundaries handles in-flight generations mechanically: an old generation's `game_id` no longer matches the current game, so it aborts at the next boundary without further persistence after the mismatch is observed. This keeps lifecycle operations consistent with reset and avoids reintroducing a global generation lock.

### α-check race boundary

The α-check samples `current_game_id()` at three phase boundaries in `phases.rs`. The pipeline records the `started_for` game id when the run begins, then compares it with storage's current game id at those boundaries.

`save_snapshot` still keys by storage's current `game_id` (the storage atomic), not by the pipeline's `started_for` value. A race between an α-check pass and a later save can therefore persist up to one phase of stale work under the new current game. This is bounded by design per ticket 05: the next α-check sees the game mismatch and aborts. It is not a defect.

### GenerationGuard drop ownership check

`GenerationGuard` now carries both `game_id` and `generation_id`. On `Drop`, it verifies that it still owns the active generation slot for that game before mutating the registry or the `AtomicBool` projection.

If old generation A is superseded by generation B — for example, reset creates a new game and B claims the slot — A's `Drop` is a no-op. It must not clear B's registry entry or lower the projection while B is still active. This closes the P4-serialize race where stale cleanup from an older generation could clobber a newer generation.
