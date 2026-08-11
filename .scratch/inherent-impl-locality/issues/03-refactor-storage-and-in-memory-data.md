# 03 — Refactor Storage and InMemoryData to module-per-type

Type: task
Status: ready-for-agent
Blocked by: (none)

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
