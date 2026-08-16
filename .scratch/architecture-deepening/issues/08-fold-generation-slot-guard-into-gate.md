# 08 — Fold generation slot and guard into GenerationGate

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to folding `GenerationSlot` (`slot.rs`) and `GenerationGuard`
(`guard.rs`) into `GenerationGate` (`gate.rs`) as internal details of one deep
module — and if so, what is the shape of the deepened module?

## Background

This is **candidate 7** of the architecture review — marked **Speculative**.
See `architecture-review.html` for the before/after diagram and evidence.

The friction: one generation-lock concept is split into three tiny files.
`GenerationSlot` (`slot.rs:11-49`) is a two-variant enum plus `is_generating`
and a free `release_owned_slot` function. `GenerationGuard` (`guard.rs:10-38`)
is an RAII wrapper whose `Drop` calls `release_owned_slot`. `GenerationGate`
(`gate.rs:29-40`) exposes `new()` and `heal_stale()`; the real depth is in
`try_claim`. The registry (`Arc<RwLock<HashMap<…>>>`) is passed between files
as if it were an internal detail that leaked across the file split.

The deletion test *vanishes*: moving `slot.rs` and `guard.rs` into `gate.rs`
reappears no complexity; the registry stays encapsulated in one file.

## What this ticket resolves

- **Commit or reject.** Given this is speculative and low-friction, is the
  fold worth the churn, or does the three-file split earn its locality?
- **Interface shape.** If committed: `GenerationGate`'s unchanged external
  interface (`new` / `heal_stale` / `try_claim`), with slot and guard as
  private internals.
- **What survives.** Tests should cross the gate interface unchanged.

## Constraints

- This is the weakest candidate. The grilling should seriously consider
  **reject** with a load-bearing reason (the split aids navigation; the churn
  isn't worth it) — and if rejected, offer to record an ADR so future reviews
  don't re-suggest it.
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling`.
- Domain term: Generation (CONTEXT.md — the per-game slot/gate concept, not the
  `GenerationStatus` pipeline phase).
