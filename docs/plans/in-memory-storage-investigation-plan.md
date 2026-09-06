# Plan: In-memory storage backend investigation

**Date:** 2026-08-16
**Status:** Draft — investigation only, no code changes
**Goal:** Decide whether the `Backend::InMemory` storage backend should be kept, reduced, or removed. Record the decision with evidence.

## Origin

A code review of the replay-blob work surfaced that the **Pattern 2 storage-backend pair** (`unit_test_standards.md` Pattern 2) tests the in-memory backend against sqlite `:memory:` for parity, while the in-memory backend's stated purpose is unit-test speed. The two goals are in tension: a paired test runs the body twice — once in-memory, once sqlite — so the pair delivers no speed benefit over sqlite-only. Speed only materializes where in-memory is used *instead of* sqlite, which happens in the app/pipeline/bootstrap unit tests (Role A), not in the storage pairs (Role B).

The question: does `Backend::InMemory` earn its keep, given that sqlite `:memory:` is already a fast in-process backend?

## Current state (verified 2026-08-16)

- `Backend` is a two-arm enum: `Backend::Sqlite { pool }` and `Backend::InMemory(Box<InMemoryData>)` (`src/adapters/driven/storage/core.rs:22`).
- **Production wires sqlite only.** `run.rs:60` constructs `Storage::new_sqlite`. `Backend::InMemory` is never constructed outside `*_tests.rs`, `test_support`, and `bootstrap/*_tests.rs`. Confirmed by search.
- `Backend::InMemory` arms exist in 11 storage files (≈43 `match` arms total) plus `InMemoryData` (112 lines in `in_memory_data.rs`) — a full second hand-maintained implementation of every storage operation.
- `Storage::new_in_memory()` is used in 36 `src/` files; `sqlite_storage()` in 11 (all `src/adapters/driven/storage/*_tests.rs` plus `settings_tests.rs` and `fixtures.rs`).
- **Pattern 2 pairs:** ~28 `_sqlite`-suffixed test functions across `src/adapters/driven/storage/*_tests.rs` (17) and `tests/storage/*.rs` (11). Each pair is two near-identical bodies differing only in storage construction.
- **Unit suite runtime:** ~17s warm for 982 tests (measured 2026-08-16). Cold ~57s (mostly compile).
- **Failure injection is backend-agnostic.** `TestOverride` lives in `BackendKind::Test { overrides, base }` (`core.rs:29`), which wraps *any* base backend. `with_backend_mut` (`core.rs:124`) intercepts the override *before* dispatching to the inner backend, so `with_test_failures()` works over sqlite `:memory:` too. **`TestOverride` does not depend on `Backend::InMemory`.** This is the load-bearing finding: removing the in-memory backend does not remove failure injection.

## Investigation questions

### Q1. Speed delta — in-memory vs sqlite `:memory:`
Benchmark the two backends to quantify the marginal speed the in-memory backend actually buys.

- **Microbench:** a representative storage operation (e.g. `insert_message` + `load_messages_with_swipes`) repeated N times against `Storage::new_in_memory()` vs `sqlite_storage()`. Use `std::time::Instant`, not a benchmarking crate (keep the investigation zero-dep). Write results to `tmp/in-memory-bench.md`.
- **Macrobench:** the full unit suite with each backend forced. Run `cargo test --lib` three times warm for each of: (a) current (mixed), (b) a hypothetical all-sqlite build. For (b), a temporary patch that makes `new_in_memory()` return `sqlite_storage()` is acceptable *inside the investigation scratch* — never committed. Record mean + range. The recommendation in Q5 judges whether the measured delta is material; no threshold is fixed here.

### Q2. Parity value — does the second implementation pay for itself?
The Pattern 2 pairs exist to catch drift between `Backend::Sqlite` and `Backend::InMemory` arms. Assess whether that drift is a real risk worth ~28 duplicated test bodies.

- Count the historical drift catches: search the git log / blame for commits where a storage method's two arms were fixed to agree. If the count is near zero, the parity test has never caught a real bug.
- Enumerate storage methods where the in-memory arm is *non-trivially* different from sqlite (hand-rolled iteration, HashMap mutation, ordering logic). These are the arms where drift could hide a bug. Methods where the in-memory arm is a thin `Vec::clone` are low drift-risk.
- **Decision input:** if the in-memory arms are mostly thin clones (low drift risk) and there are no historical drift catches, the parity pairs are insuring against a risk that isn't materializing.

### Q3. Maintenance tax
Quantify the cost of keeping `Backend::InMemory`.

- Line count: `InMemoryData` struct + impls, plus every `Backend::InMemory =>` arm across `src/adapters/driven/storage/`. Report total lines.
- Touch-cost: count storage methods that require editing *two* arms for any behavior change (e.g. the replay-blob migration touched both arms). This is the "shotgun surgery" cost the review flagged — every new storage column or method adds a third edit site (in-memory arm) on top of sqlite + mapper.
- Coupling: does any non-storage code depend on `Backend::InMemory`-specific behavior (e.g. assuming a fresh empty store, no persistence)? If production code paths ever branched on backend kind, that would block removal.

### Q4. Removal sketch and blast radius
If the investigation leans toward removal, sketch the actual change so the cost is concrete, not abstract.

- **Option A — keep current.** Document the trade-off in `docs/diataxis/reference/coding_standards/unit_test_standards.md` Pattern 2 and in `docs/diataxis/explanation/storage_design.md`. No code change.
- **Option B — remove `Backend::InMemory`, keep `TestOverride`.** Delete the `Backend::InMemory` variant, `InMemoryData`, all in-memory arms. `new_in_memory()` becomes an alias returning `sqlite_storage()` (or is deleted in favor of `sqlite_storage()` directly). `TestOverride` stays — it already wraps sqlite. App/pipeline/bootstrap tests switch to `sqlite_storage()`. Pattern 2 pairs collapse to single sqlite tests. Report: files deleted, files edited, test-count delta, suite-runtime delta (from Q1 macrobench).
- **Option C — remove in-memory AND collapse the test-override seam.** Out of scope here; the test-override seam is owned by `docs/plans/storage-test-seam-investigation-plan-revised.md`. This plan treats it as a dependency, not a target. Note any interaction.
- **Option D — keep in-memory for app tests, drop the storage pairs.** Stop running Pattern 2 pairs (delete the `_sqlite` siblings, or delete the in-memory twins). In-memory backend stays for Role A speed; its parity goes untested. Middle ground: keeps the speed benefit where it's real, drops the maintenance where it's circular. Report the drift risk from Q2 for this option specifically — if parity is untested, how quickly would in-memory drift from sqlite in app-test usage?

### Q5. Recommendation
- **GO** (proceed to a removal/follow-up implementation plan), **NO-GO** (keep, document the trade-off), or **PARTIAL** (e.g. Option D — keep in-memory for app tests, drop the pairs).
- Record the recommendation in `docs/diataxis/explanation/storage_design.md` as a short note (the ADR directory no longer exists per `storage-test-seam-investigation-plan-revised.md`).
- If GO/PARTIAL, open a follow-up implementation plan in `docs/plans/`.

## Out of scope

- Implementing any refactor or removal. This plan produces a decision and a sketch, not code.
- The `TestOverride` / `BackendKind::Test` seam — owned by `storage-test-seam-investigation-plan-revised.md`. This plan depends on its finding that `TestOverride` is backend-agnostic (already verified, `core.rs:124`) but does not redesign that seam.
- `GameStateSnapshot`, the prompt assembler, or anything outside the storage driven adapter.
- The steering-and-guided-generation effort — independent of this investigation.

## Deliverables

1. `tmp/in-memory-storage-findings.md` with Q1–Q4 numbers and write-ups, and a Q5 recommendation with threshold reasoning.
2. If GO or PARTIAL: a follow-up implementation plan in `docs/plans/`.
3. If NO-GO: a short note added to `docs/diataxis/explanation/storage_design.md` recording why the in-memory backend is retained despite the parity-test tension, and an update to `unit_test_standards.md` Pattern 2 explaining when the pair is and isn't worth the duplication.

## Verification

- `python build.py` remains green (investigation produces no committed diff; any bench scratch lives in `tmp/`).
- `git status` shows only new `tmp/` docs and this plan.

## Risks of the investigation itself

- **Microbench noise.** A single `Instant`-based bench can be dominated by HashMap allocation vs sqlite's in-memory pager. Run the macrobench (full suite) as the primary signal; treat the microbench as a sanity check, not the decision driver.
- **Confirmation bias.** The review that spawned this plan was skeptical of the pattern. The investigation must genuinely test the "keep" case — any outcome (GO, NO-GO, PARTIAL) is valid if the evidence supports it.
- **Scope creep into the test-override seam.** Resist it. The two seams are separable; keep this plan to the in-memory backend.
