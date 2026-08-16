# 01 — Research the Storage seam constraints

Type: research
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

What constraints must a trait-based Storage redesign preserve? Synthesize the
facts that ticket 02 (deepen Storage into a `StorageBackend` trait) needs
before grilling can decide the trait's shape — especially the test
failure-injection mechanism and the prior Storage-shape decisions.

## Background

Candidate 1 of the architecture review proposes turning `Storage` from a
concrete struct with a `Backend`/`BackendKind` enum dispatch into a
`StorageBackend` trait with two real adapters (`SqliteBackend`,
`InMemoryBackend`) plus a test adapter. Before grilling that decision, the
constraints baked into the current design must be made explicit, because two
non-obvious mechanisms depend on the current shape:

1. **Failure injection.** `src/adapters/driven/storage/core.rs` exposes
   `with_backend_mut(method, f)` and a `BackendKind::Test { base, overrides }`
   variant (gated by `#[cfg(feature = "testing")]`) backed by
   `test_support.rs` (`TestFailureHandle`, `TestOverride`). The
   `method: &'static str` token is how test overrides hook a specific
   operation. A trait redesign must preserve this failure-injection surface
   or replace it — the grilling cannot decide the trait shape without knowing
   what it must carry.

2. **Prior decisions.** `.scratch/inherent-impl-locality/` tickets 03 and 11
   already decided Storage-shape questions: the `backend/` folder was
   flattened into `storage/` root; single-file consolidation of the 13
   `impl Storage` blocks was **rejected as undesirable**; the
   inherent-impl-locality rule stays **name-only** (folder cohesion is review
   policy). These are the closest thing this repo has to ADRs on the Storage
   seam and must not be re-litigated.

## What this ticket resolves

A markdown summary (linked as an asset from this ticket's resolution) covering:

- The full surface of the failure-injection mechanism: how
  `with_backend_mut` + `BackendKind::Test` + `TestOverride` work today, which
  tests rely on them, and what a trait-based design must provide to keep them
  working (or what replaces them).
- The prior Storage-shape decisions from `inherent-impl-locality` tickets 03
  and 11, restated as constraints on any new trait design.
- A concrete list of "invariants the trait must preserve" — the facts ticket
  02 will grill against.

This ticket does **not** decide the trait shape. It gathers the constraints.
Ticket 02 makes the decision.

## Constraints

- Read-only investigation. No code changes.
- Cite file:line for every claim about the current mechanism.
- Do not re-open the prior decisions — record them as constraints, not as
  questions.

## Notes

- Source files: `src/adapters/driven/storage/core.rs`,
  `src/adapters/driven/storage/test_support.rs`, and the
  `#[cfg(feature = "testing")]` paths in the storage module.
- Prior decisions: `.scratch/inherent-impl-locality/issues/03-*.md` and
  `11-*.md` (read their `## Answer` sections).
- Asset on resolution: save the summary as
  `.scratch/architecture-deepening/assets/storage-seam-constraints.md` and
  link it from the resolution comment.
