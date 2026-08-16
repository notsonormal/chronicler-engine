# 02 — Deepen Storage into a StorageBackend trait

Type: grilling
Status: open
Blocked by: 01
Assignee: (unclaimed)

## Question

Do we commit to deepening `Storage` from a concrete struct with
`Backend`/`BackendKind` enum dispatch into a `StorageBackend` trait with two
real adapters (`SqliteBackend`, `InMemoryBackend`) plus a test adapter — and
if so, what is the shape of the deepened module: its interface, its seam, what
sits behind it, and what tests survive?

## Background

This is **candidate 1** of the architecture review — the top recommendation.
See `architecture-review.html` (this directory) for the before/after diagram
and full evidence.

The friction: `Storage` exposes ~30 inherent `pub fn` methods, and every
method repeats the same `match backend { Sqlite=>.., InMemory=>.. }` dispatch
(`core.rs:117-141` `with_backend_mut`; `games.rs`, `worlds.rs`, `messages.rs`,
`snapshots.rs` each duplicate the match shape). The seam is the helper plus
the `Backend` enum, not a trait. Every new operation adds two identical arms.

The deepening: a `StorageBackend` trait turns the dispatch into two real
adapters. The codebase-design principle applies — "one adapter means a
hypothetical seam, two means a real one"; here there are two real backends
plus a test double, so the seam earns its place.

## What this ticket resolves

- **Commit or reject.** Is the trait deepening worth the migration, or does
  the current enum dispatch already earn its keep?
- **Interface shape.** If committed: the trait's method set, how it groups
  (one trait vs per-concern sub-traits), and where the seam lives.
- **Test-double story.** How failure injection (ticket 01's constraints) is
  preserved behind the new interface.
- **What survives.** Which existing tests cross the same seam unchanged; which
  must be rewritten.

## Constraints

- Must respect the prior Storage-shape decisions synthesized in ticket 01 —
  do not re-litigate them.
- Must keep `arch-lint.toml` layer rules satisfied (or propose a rule change
  as part of the decision).
- This is a **decision** ticket. No implementation. The output is the module
  shape, handed off to a later planning effort.

## Notes

- Resolution uses `/grilling` (one question at a time) and `/domain-modeling`
  if a new term enters the vocabulary.
- If the trait is rejected with a load-bearing reason, offer to record it as
  an ADR (the skill's callout) so future reviews don't re-suggest it.
- This ticket blocks ticket 03 (the Repository port) — its outcome shapes 03's
  interface.
