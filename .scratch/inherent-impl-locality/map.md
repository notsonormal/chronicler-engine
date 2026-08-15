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
- Existing guardrail harness lives at `tests/infrastructure/guardrails/` (rule files: `enums.rs`, `free_fn.rs`, `layers.rs`, `location.rs`, `mod.rs`, `nesting.rs`, `structure.rs`, `style.rs`; each ships a `*_tests.rs` companion). Pattern: `check_src_files(rule_name, check_fn)` walks `src/**/*.rs` and feeds one file at a time to `check: impl FnMut(&str, &str) -> Vec<Violation>`. `Violation` type already exists. `syn` is a dev-dependency. NOTE: the one-file-at-a-time shape does not fit a cross-file def-index rule — ticket 01's blueprint walks `src/` directly via `discover_rs_files` in two passes rather than reusing `check_src_files`.
- Existing related rules already on the books: `guardrails_mod_purity` (mod.rs contains declarations/re-exports only), `guardrails_file_length_src` / `guardrails_file_length_tests` (max 2000 non-blank lines), `guardrails_free_fn_location` (top-level free fns must live in an allowlisted folder — already shipped, see `free_fn.rs`). Layer-direction rules (application may not import Storage directly, etc.) live in `arch-lint.toml`, not the guardrails suite — the old `guardrails_application_storage_direct` / `ADR-027` citation is stale (ADR-027 was removed in `e0f22e0`; that named guardrail no longer exists).
- Architectural layering (per `CONTEXT.md`): `domain` → `application` → `adapters`. Cross-layer type placement is a violation in its own right and is surfaced by this rule.
- Rule excludes: `_tests.rs` files, `#[cfg(test)] mod` inner blocks, trait impls (`item.trait_.is_some()`), and `main.rs` (vacuously clean — no inherent impls).
- User standing constraint: no LLM-based decision rules at enforcement time. Guardrails must be deterministic (AST + path matching). Semantic judgments ("is this cohesive?") are review policy, not hard rules.
- Out-of-scope for this effort: trait-impl locality policy, test-target-location rule. (Free-function locality is already shipped as `guardrails_free_fn_location`; `find_free_fn_smells.py` is already retired — neither is this effort's concern.)

## Decisions so far

<!-- one-line gist per closed ticket; detail lives in the ticket -->

- [02 — Run audit and confirm violation set](issues/02-run-audit-and-confirm-violation-set.md) — audit run via 01's trial. 27 violations captured, 3 discrepancies against the expected table itemized as findings for refactor tickets: `ActionPipeline` NOT flagged (folder-exemption formula too loose — map-or-04 decision), `QuantifierResult` additionally flagged (08 must handle both siblings), `AppState` and `PromptContext` newly surfaced (no refactor ticket yet).
- [03 — Refactor Storage and InMemoryData to module-per-type](issues/03-refactor-storage-and-in-memory-data.md) — resolved by events, not by this ticket's planned refactor. The `backend/` folder was flattened into `storage/` root (commit `6cb2049`, hexagonal-direction fix), which makes the folder exemption apply to every `impl Storage` (parent dir now ends `/storage` = `snake(Storage)`); `InMemoryData` consolidated into one file; the `bootstrap/load.rs` `impl Storage` was removed. No code was written against this ticket.
- [08 — Resolve QuantifierParseResult cross-layer placement](issues/08-resolve-quantifier-parse-result-cross-layer-placement.md) — resolved by events. The cross-layer `impl QuantifierParseResult` / `impl QuantifierResult` in `application/agents/quantifier/parser.rs` were removed; the sole remaining `impl QuantifierParseResult` is co-located with its def in `domain/model/quantifier.rs`, and `QuantifierResult` now has no inherent impls. No grilling was held — the violation disappeared before the ticket was worked.
- [04 — Refactor ActionPipeline to module-per-type](issues/04-refactor-action-pipeline-and-pipeline-run.md) — resolved via a pure `action_pipeline/` type-split subfolder (shape A): `ActionPipeline` struct + all its impls (incl. `phase_engine_commit` moved back from what was `phases.rs`) live in `application/pipeline/action_pipeline/`, while `PipelineRun`/`PhaseError`/`spawn` stayed in the `pipeline/` parent (the `PipelineRun` file was later renamed `phases.rs`→`pipeline_run.rs` for clarity) so the exemption folder holds only `ActionPipeline`'s split. Single-file consolidation was rejected — it collided with `test_file_location`/`file_length`/`mod_purity`. External imports stayed stable via re-export; 7 fields + 1 method widened `pub(super)`→`pub(crate)` because `PipelineRun` is no longer under `action_pipeline`'s `super`. Verified green (fmt/clippy/lib/guardrails/arch/http-actions/pipeline unit tests); the `inherent_impl_locality` guardrail itself doesn't exist yet (01 deferred) so zero-violation is verified structurally. Surfaced the folder-cohesion gap → ticket 11. Two browser test failures were flaky test-side status-polling races (not a `b319579` regression, not this refactor) — fixed this session in `tests/test_utils/` (`wait_for_status_generating` + `wait_for_status_ready` timeout bump); `build.py` fully green.

<!-- ticket 01 was tried, then removed at user direction — deferred, not resolved. Re-add as the last step before 09 once the remaining refactor tickets land. See ticket 01's body for the implementation blueprint + 27-violation trial-run findings + 3 discrepancies. NOTE: 01's blocker list and trial-run paths are stale post-refactor (see 01's body). -->

## Not yet specified

<!-- fog: suspected decisions that can't be pinned until the frontier advances -->

- `AppState` (def `adapters/driving/http/app_state.rs`, impl in `layout/renderers/fragment_renderers.rs`) — surfaced by 01's trial, still a live violation, still has no refactor ticket. It lives in the `adapters/driving/http` layer, separate from the `application/prompting` cluster ticket 07 covers, so folding into 07 does not fit. Resolves to its own task ticket when the frontier reaches the http layer.
- **Folder-cohesion question graduated to [11 — Enforce folder cohesion in the inherent-impl-locality rule](issues/11-enforce-folder-cohesion-in-inherent-impl-locality-rule.md).** During ticket 04, the choice between a pure `action_pipeline/` subfolder (shape A) and a flat `pipeline/`→`action_pipeline/` rename (shape D) exposed that the rule checks folder *name* but not folder *contents* — shape D's impurity (unrelated types in an `ActionPipeline`-named folder) is invisible to the rule. The question is now sharp enough to ticket; 11 grills whether to tighten the rule to enforce folder cohesion. Blocked by 01 (the rule must exist before it can be tightened).

## Out of scope

<!-- work ruled beyond the destination; never graduates -->

- [05 — Refactor DefaultApplicationService to module-per-type](issues/05-refactor-default-application-service.md) — the `DefaultApplicationService` type was deleted outright in the facade-removal refactor series (`5fd2c54` wired collaborators into `AppState` and deleted the facade; `3adc5e5`/`fbd7120` continued it). The type no longer exists in `src/`, so there is nothing to refactor. Closed as out of scope: the violation evaporated when the type did, not by satisfying the rule.
