# 08 — Fold generation slot and guard into GenerationGate

Type: grilling
Status: open
Blocked by: 09
Assignee: (unclaimed)

## Question

Do we commit to folding `GenerationSlot` (`slot.rs`) and `GenerationGuard`
(`guard.rs`) into `GenerationGate` (`gate.rs`) as internal details of one deep
module — and if so, what is the shape of the deepened module?

## Background

This is **candidate 7** of the architecture review — marked **Speculative**.
See `assets/architecture-review.html` for the before/after diagram and evidence.

The friction: one generation-lock concept is split into three tiny files.
`GenerationSlot` (`slot.rs`, 53 lines) is a two-variant enum plus
`is_generating` and a free `release_owned_slot` function. `GenerationGuard`
(`guard.rs`, 37 lines) is an RAII wrapper whose `Drop` calls
`release_owned_slot`. `GenerationGate` (`gate.rs`, 147 lines) exposed `new()`
and `heal_stale()` on 2026-08-16, with the real depth in `try_claim`. It now
exposes more — see Current state. The registry (`Arc<RwLock<HashMap<…>>>`) is passed between files
as if it were an internal detail that leaked across the file split.

The deletion test *vanishes*: moving `slot.rs` and `guard.rs` into `gate.rs`
reappears no complexity; the registry stays encapsulated in one file.

## What this ticket resolves

- **Commit or reject.** Given this is speculative and low-friction, is the
  fold worth the churn, or does the three-file split earn its locality?
- **Interface shape.** If committed: `GenerationGate`'s external interface,
  with slot and guard as private internals. It is no longer just
  `new` / `heal_stale` / `try_claim` (see Current state). The grilling must
  also decide whether the gate's interface should shrink back.
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
  `GenerationStatus` pipeline phase). Note: CONTEXT.md has no "Generation"
  entry today. If the term is needed, add it via `/domain-modeling`.
- Blocked by ticket 09 (input-buffer transitions): `gate.rs` writes
  `input_buffer.status` / `.phase` by hand (in `heal_stale` and `try_claim`).
  09 decides whether those writes become named transitions, which changes
  what the gate's interface looks like.

## Current state (2026-10-04)

Worse. `GenerationGate`'s interface grew from 3 to 7 items: `new`,
`heal_stale`, `try_claim`, `guard` (`pub(crate)`), `release_generation_slot`,
`release_generation_slot_for_game`, `is_busy`. `release_owned_slot` is still a
free fn in `slot.rs`, called from `gate.rs` and `guard.rs`, and the
`Arc<RwLock<HashMap<…>>>` registry still crosses files. Still the weakest
candidate on its own. The wider interface makes the fold, or an interface
cut, more worth discussing than on 2026-08-16.
