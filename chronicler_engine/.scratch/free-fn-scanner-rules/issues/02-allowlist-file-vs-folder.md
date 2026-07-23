# 02: Allowlist granularity — file-level vs folder-level category subfolders

Status: open
Type: grilling
Assignee: (unassigned)
Blocked by: (none)

## Question

Should the module-allowlist rule be **file-level** (C1) — name specific files where honest free fns may live — or **folder-level via category subfolders** (C2) — refactor the honest free fns into new dedicated subfolders whose names declare the category?

## Context

The user's insight: `storage/mappers/` works as an allowlist category because the subfolder *names the category* — "free fn in mappers/" reads honestly. The question is whether `domain/engine/*` and `application/action_pipeline/*` should be narrowed the same way.

### C1 — File-level allowlist (zero refactor)

Allowlist names specific files, not whole folders:

```
adapters/driven/storage/mappers/*.rs          (folder — already a category)
domain/engine/action_processing.rs
domain/engine/logic.rs
domain/engine/state_diagnostics.rs
domain/engine/trigger_eval.rs
application/action_pipeline/retry.rs          (after 01 converts the *_impl fns to methods, only honest helpers remain — verify none left)
application/action_pipeline/actions.rs        (after 01 — verify empty or honest)
application/narrative_prompt/*.rs             (folder)
application/generation_gate/slot.rs
bootstrap/load.rs
bootstrap/validate.rs
test_support/context.rs
settings.rs
adapters/driving/http/locks.rs
```

~14 file patterns instead of 21 function patterns. Zero refactor.

- *Cost*: coarser than per-function. A new method-shaped free fn added *inside* an allowlisted file passes silently. But files tend to be cohesive — a file named `retry.rs` with spawn-blocking helpers reads as honestly as a folder named `orchestrators/`.

### C2 — Folder-level via category subfolders (refactor churn)

Move honest free fns into new dedicated subfolders so the folder name declares the category:

- `domain/engine/stateless_ops/` ← `action_processing.rs`, `logic.rs`, `state_diagnostics.rs`, `trigger_eval.rs`
- `application/action_pipeline/orchestrators/` ← `retry.rs`, `actions.rs` (only if honest free fns remain after 01; if 01 empties these files, no move needed)
- `application/narrative_prompt/` — already a folder, no move.
- `adapters/driven/storage/mappers/` — already a folder, no move.

Then allowlist names only category folders. Tighter signalling.

- *Cost*: 2-4 SP of refactor churn — move files, update `mod.rs`, update imports, move `_tests.rs` siblings. Marginal gain over C1 is a folder name vs a file name.

### Tradeoff

C1 keeps the honest free fns where they are; cohesion argument cuts the same way for files as for folders. C2 makes the category name visible in the path at the cost of churn. The user leaned toward subfolders in grilling but did not confirm after the C1/C2 split was stated.

## Recommendation

**C1 (file-level).** The marginal gain of C2 (folder name vs file name) does not justify the churn. Files like `retry.rs` and `logic.rs` are already cohesive. If a file later grows a mix of honest free fns and method candidates, that's a signal to split it — not a reason to pre-emptively folder everything now.
