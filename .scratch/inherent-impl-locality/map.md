# Map: Inherent Impl Locality Guardrail

## Destination

A `guardrails_inherent_impl_locality` test in `tests/infrastructure/guardrails/` that enforces module-per-type for inherent impls, wired into `cargo test --test guardrails` and `build.py`, and passes green on `main` after all current violations are refactored.

Rule statement:

```text
For every inherent impl `impl Foo` in production src/ (trait impls excluded,
test files excluded):

  Let snake = snake_case(Foo simple name).
  Let impl_path = relative path of impl file.
  Let def_path = relative path of file where Foo is defined.

  Violation if:
    impl_path != def_path, AND
    NOT (impl_path's parent dir ends with /snake)
```

i.e. impls may split across files only inside a folder named after the type. Cross-folder or cross-layer splits are violations; their fix is per-type (consolidate, rename folder, or relocate the type to the layer that owns its behavior).

## Notes

- This effort carries execution into the map itself (destination requires code, not just decisions).
- Tracker: local markdown (`.scratch/inherent-impl-locality/`). See `docs/agents/issue-tracker.md`, `docs/agents/triage-labels.md`.
- Skills every session should consult: `/grilling`, `/domain-modeling`, `/planning-and-task-breakdown`, `/subagent-driven-development` (for refactors ≥5 SP), `chronicler-after-plan-workflow` (after landed code changes).
- Existing guardrail harness lives at `tests/infrastructure/guardrails/` (rule files: `enums.rs`, `layers.rs`, `location.rs`, `mod.rs`, `nesting.rs`, `structure.rs`, `style.rs`). Pattern: `check_src_files(rule_name, check_fn)` walks `src/**/*.rs`, feeds `(relative_path, content)` to each rule. `Violation` type already exists. `syn` is a dev-dependency.
- Existing related rule already on the books: `guardrails_mod_purity` (mod.rs contains declarations/re-exports only), `guardrails_file_length` (max 2000 non-blank lines), `guardrails_application_storage_direct` (application/ may not import Storage directly, ADR-027).
- Architectural layering (per `CONTEXT.md` if present, else `docs/diataxis/reference/architecture_system.md`): `domain` → `application` → `adapters`. Cross-layer type placement is a violation in its own right and is surfaced by this rule.
- Rule excludes: `_tests.rs` files, `#[cfg(test)] mod` inner blocks, trait impls (`item.trait_.is_some()`), and `main.rs` (vacuously clean — no inherent impls).
- User standing constraint: no LLM-based decision rules at enforcement time. Guardrails must be deterministic (AST + path matching). Semantic judgments ("is this cohesive?") are review policy, not hard rules.
- Out-of-scope for this effort: trait-impl locality policy, free-function location (issue 10 / issue 11), test-target-location rule, retiring `find_free_fn_smells.py`.

## Decisions so far

<!-- one-line gist per closed ticket; detail lives in the ticket -->

- [02 — Run audit and confirm violation set](issues/02-run-audit-and-confirm-violation-set.md) — audit run via 01's trial. 27 violations captured, 3 discrepancies against the expected table itemized as findings for refactor tickets: `ActionPipeline` NOT flagged (folder-exemption formula too loose — map-or-04 decision), `QuantifierResult` additionally flagged (08 must handle both siblings), `AppState` and `PromptContext` newly surfaced (no refactor ticket yet).

<!-- ticket 01 was tried, then removed at user direction — deferred, not resolved. Blocked by 03–08. The rule file is gone from main; re-add as the last step before 09 once refactor tickets land. See ticket 01's body for the implementation blueprint + 27-violation trial-run findings + 3 discrepancies. -->

## Not yet specified

<!-- fog: suspected decisions that can't be pinned until the frontier advances -->

- Does flattening `backend/` into `storage/` require a separate module-rename ticket, or is it bundled into the `Storage` refactor ticket? Resolves when `Storage` ticket (03) is picked up.
- `AppState` (def `adapters/driving/http/app_state.rs`, impl in `fragments/renderers/fragment_renderers.rs`) — surfaced by 01's trial but has no refactor ticket. Fold into a new ticket, or into 07 (same `assembler.rs` cluster as `PromptContext`)? Resolves when the frontier reaches the `application/narrative_prompt` cluster.
- Is the `ActionPipeline` folder exemption a rule-formula bug (tighten to "folder holds only this type") or a feature (accept that folder exemption = type name match)? Currently the rule does NOT flag `ActionPipeline`. 04 inherits this question when worked.

## Out of scope

<!-- work ruled beyond the destination; never graduates -->
