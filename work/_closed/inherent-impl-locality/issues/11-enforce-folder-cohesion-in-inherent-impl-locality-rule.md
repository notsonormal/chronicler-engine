# 11 — Enforce folder cohesion in the inherent-impl-locality rule

Type: grilling
Status: resolved
Blocked by: (none)
Assignee: agent

## Question

Should `guardrails_inherent_impl_locality` (ticket 01's rule) enforce not just
that an `impl Foo` lives in a folder named `snake(Foo)`, but that the folder
holds **only** `Foo`'s own files — and if so, how is "only" defined and checked?

## Background

The rule as specified in the map checks the folder *name* only:

```text
Violation if:
  impl_path != def_path, AND
  NOT (impl_path's parent dir ends with /snake)
```

This grants a folder exemption: any `impl ActionPipeline` inside a folder whose
parent dir ends `/action_pipeline` is clean, regardless of what else lives in
that folder. The exemption is sound — it lets a large type split its impls
across files. But the rule is blind to folder *contents*.

### The gap, surfaced during ticket 04

While refactoring `ActionPipeline` to module-per-type, three resolution shapes
were weighed for where its split impls should live:

- **A (chosen).** A pure `action_pipeline/` subfolder holding only
  `ActionPipeline`'s struct + impls + private helpers, with the subsystem's
  other types (`PipelineRun`, `PhaseError`, `spawn`) staying in the `pipeline/`
  parent.
- **D (rejected).** A flat rename `pipeline/` → `action_pipeline/`, leaving
  `PipelineRun`, `PhaseError`, and `spawn` sitting inside an
  `ActionPipeline`-named folder.

Shape D satisfies the rule as written — every `impl ActionPipeline`'s parent
dir ends `/action_pipeline`. But D is impure: the exemption folder would hold
unrelated types, defeating the "module-per-type" intent the rule exists to
enforce. The user caught this: a folder named after a type is a *type-split*
folder and should hold only that type's split. Shape A was chosen precisely
because the mechanical rule could not enforce this — the decision had to be
made by a human reviewing the layout.

That is the gap: a cohesion judgment the rule delegates to review policy but
could potentially encode.

## What this ticket resolves

Whether to tighten the rule to enforce folder cohesion, and if so, the
precise mechanical definition. Candidate definitions to grill through:

1. **Strict single-type.** A folder ending `/snake(Foo)` may contain only
   files that define or impl `Foo` (plus `Foo`'s private helpers and `Foo`'s
   test files). Any other type def or inherent impl in that folder is a
   violation.
2. **Single-type + colocated helpers.** As (1), but explicitly permits
   private helper structs/enums used only by `Foo`'s impls (e.g. `RetryTarget`
   inside `action_pipeline/retry.rs`). Requires a reachability definition of
   "used only by."
3. **Leave as review policy.** The rule stays name-only; cohesion stays a
   human judgment, documented as a convention but not enforced. This is the
   status quo.

Each option has trade-offs in mechanical-check cost, false-positive risk
(private helpers, re-exports in `mod.rs`), and how much it forecloses future
legitimate layouts. The grilling should surface which the codebase actually
needs.

## Constraints

- This is a **rule-design** decision, not a refactor. Do not implement the
  tightened rule here — that belongs in ticket 01's blueprint (or a follow-up
  to it) once the definition is settled.
- The standing constraint from the map applies: no LLM-based decision rules at
  enforcement time. Any tightening must be deterministic (AST + path
  matching).
- Do not regress the existing folder exemption's legitimate use (a type
  splitting its impls across files inside its named folder is the rule's
  intended escape hatch, not a dodge).

## Notes

- Motivating example: ticket 04's resolution chose shape A over shape D
  because shape D's impurity was invisible to the rule. See ticket 04's
  resolution comment for the full option comparison.
- Related: `guardrails_mod_purity` already enforces that `mod.rs` holds
  declarations/re-exports only — a partial cohesion guard, but only for
  `mod.rs`, not folder contents.

## Answer

**Decision: leave cohesion as review policy. Do not tighten the rule.**

`guardrails_inherent_impl_locality` stays **name-only**. Folder cohesion is
review policy, not a mechanically enforced rule. The shape-D dodge (a
folder named `snake(Foo)` that holds unrelated types) stays a human-judgment
responsibility, caught at review — not encoded in the guardrail.

### Why tightening was rejected

A syntactic cohesion check cannot distinguish a genuine type-split folder
from a subsystem folder that shares its name with a struct. That is the
semantic judgment the map's standing constraint forbids at enforcement time.

`src/adapters/driven/storage/` is the canonical collision: `storage/` is the
subsystem, `Storage` (struct in `core.rs:16`) is one struct among many pub
types defined in the folder root (`DbPool`, `InMemoryData`, `Backend`,
`BackendKind`, etc.). `snake(Storage) == "storage"`, so the name-only rule
classifies the folder as `Storage`'s type-split folder — but it isn't one. A
cohesion rule would flag it indefinitely, forcing a consolidation or rename to
satisfy a rule that misread the layout.

This is not a one-off. The pattern is normal Rust: a subsystem folder
(`storage/`, `rendering/`, `networking/`) sharing its name with a struct whose
impls split across the subsystem. Tightening would false-positive on every
future occurrence, not just `storage/` once.

### Why the benefit doesn't justify the cost

The benefit of tightening is mechanically blocking the shape-D dodge. That
dodge has occurred once, during ticket 04's refactor, and was caught
immediately by human review — review policy working as intended. The cost is
perpetual false-positives on the legitimate subsystem-name-collision pattern,
repeatedly, forever. The frequency and the review catch make the cost
asymmetric.

### `storage/` consequence

None. Under name-only, `storage/` stays clean as-is. No consolidation, no
rename, no re-split, no new task ticket.

- Single-file consolidation of all 13 `impl Storage` blocks into `core.rs`
  was weighed (combined ~1650 lines, under the 2000-line `file_length_src`
  cap) and **rejected as undesirable** by the user — the split is intentional.
- Renaming `Storage` → `StorageBackend` (to break the name collision) was
  mooted and found to re-flag all 13 split impls (the folder `storage/` would
  no longer match the exemption), so it only works paired with consolidation
  or a genuine `storage_backend/` subfolder move — not a free fix.
- Renaming a port to `database` had no target: there is no storage/database
  port trait in `src/application/ports/` or the storage layer; `Storage` is a
  concrete struct, not a trait.

### Where the decision is recorded

This resolution comment and the map's Decisions-so-far pointer only. Ticket
10 (document the rule) is untouched — the cohesion boundary is not folded into
its scope (Q2 → Option A). The map stays low-res.

### Effect on the frontier

- `11` removed from `01`'s and `07`'s `Blocked by` lists.
- `01` now `Blocked by: 07` only.
- `07` now `Blocked by: (none)` — it is the new frontier ticket.
