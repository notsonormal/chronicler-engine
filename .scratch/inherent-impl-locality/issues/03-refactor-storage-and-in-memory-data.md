# 03 — Refactor Storage and InMemoryData to module-per-type

Type: task
Status: resolved
Blocked by: (none)

> **Resolved by events, not by this ticket's planned refactor.** The violation
> disappeared as a side effect of an unrelated hexagonal-direction fix (commit
> `6cb2049`, "flatten backend module into storage root"). No code was written
> against this ticket. See `## Answer`.

## Question

Refactor `Storage` and `InMemoryData` so their inherent impls satisfy `guardrails_inherent_impl_locality`.

Current state:
- `Storage` defined in `adapters/driven/storage/backend/core.rs`
- `impl Storage` blocks spread across `backend/{characters,games,messages,personas,presets,settings,snapshots,swipes,worlds,llm_messages}.rs` + `bootstrap/load.rs`
- `InMemoryData` defined in `backend/core.rs`
- `impl InMemoryData` spread across `backend/{messages,swipes}.rs`

Target shape (pick one per type; if a different shape fits better, surface it before proceeding):

Option A (folder per type):
```text
adapters/driven/storage/
  storage/
    mod.rs      # struct Storage
    characters.rs
    games.rs
    ...
  in_memory_data/
    mod.rs      # struct InMemoryData
    messages.rs
    swipes.rs
```

Option B (flat with type-named files):
```text
adapters/driven/storage/
  storage.rs         # struct Storage + all impl Storage (large file — check file_length guardrail, 2000 line cap)
  in_memory_data.rs  # struct InMemoryData + impl
```

`bootstrap/load.rs` `impl Storage` is a special case: lives outside `storage/` entirely. This is the rule's strict-mode violation. Resolution: either move the bootstrap impl into `storage/` (preferred if it's behaviorally storage setup) or move the bootstrap-specific method onto a different abstraction. Surface this as a finding if it's not a clean move.

Constraints:
- `build.py` must remain green at every landed step.
- Preserve all existing call sites (adjust imports as needed).
- Preserve `guardrails_mod_purity` (mod.rs contains declarations/re-exports only).
- Preserve `guardrails_application_storage_direct` (application/ may not import Storage directly; 6 grandfathered files — don't add a 7th).
- Do NOT touch `Db*` types — those are ticket 06.
- Do NOT touch trait impls (`impl LlmProvider for ...` etc.) — out of scope.

Acceptance:
- `cargo test --test guardrails guardrails_inherent_impl_locality` reports zero `Storage` and `InMemoryData` violations.
- Full `build.py` green.
- No new `guardrails_*` failures.

## Answer

**Resolved by events.** Commit `6cb2049` ("refactor(storage): flatten backend
module into storage root") moved every file under
`adapters/driven/storage/backend/` up one level into `adapters/driven/storage/`
root, and extracted `InMemoryData` into its own `in_memory_data.rs`. That
flattening is what satisfies the rule — not the folder-per-type shape this
ticket proposed:

- **`Storage`**: def still in `storage/core.rs`; every `impl Storage` block now
  lives in a file whose parent dir is `adapters/driven/storage`, which ends with
  `/storage` = `snake(Storage)`. The folder exemption applies, so all 11
  split-file `impl Storage` blocks are clean. The `bootstrap/load.rs`
  `impl Storage` (the strict-mode outlier this ticket flagged as a special case)
  was removed outright — `grep -rn "impl Storage" src/` now returns only
  `storage/*.rs`.
- **`InMemoryData`**: def + impl consolidated into the single file
  `storage/in_memory_data.rs` (`impl_path == def_path` → clean).

The ticket's planned refactor (Options A/B, plus the bootstrap special-case
handling) was never executed. The destination condition — zero `Storage` and
`InMemoryData` violations under the rule — is nevertheless met on `main`.

### Caveat for ticket 01's re-run

The trial-run violation list in ticket 01 still quotes `backend/` paths and the
`bootstrap/load.rs` `impl Storage`. Those entries are stale and will not
reproduce. 01's re-creation pass must re-scan `main` from scratch rather than
re-run the preserved 27-violation set.
