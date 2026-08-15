# 11 — Enforce folder cohesion in the inherent-impl-locality rule

Type: grilling
Status: ready-for-agent
Blocked by: 01

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
