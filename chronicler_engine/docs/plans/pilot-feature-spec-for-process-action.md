# Pilot: Feature Spec for `process_action`

## Summary

Pilot a feature-spec layer on the `process_action` feature (HTTP `POST /action`) to test whether specs make integration test coverage more visible and reduce documentation drift. The pilot produces one spec file, scenario annotations on existing tests, and a no-CLI Python validator. After running it manually, we record an expand-or-stop decision in a footer on the spec itself, and decide whether to wire the validator into `build.py` and add more feature specs.

The action pipeline is referenced under "Implementation" but is not the spec's subject — it is implementation detail. The pilot deliberately keeps the spec at the user-visible feature boundary.

## Key Decisions (locked from this conversation)

- Spec location: `chronicler_engine/docs/specs/process_action.md`
- Test annotations: `// SCENARIO: X.Y` comments immediately above `#[test]` (or `#[tokio::test]`)
- Coverage source of truth: comment annotations
- Spec scope: `process_action` feature only — retry, retrigger, message editing, HTTP-layer concerns are separate features and out of scope
- Scenario IDs: bullet-list items with bold inline IDs (`- **1.1** description`), grouped under behavior H3 sections
- Test scope for annotations: `tests/integration/application/action_pipeline/{actions.rs, pipeline.rs}`. Flow tests and `retry.rs` are out of scope for this pilot
- Validator: no CLI params, simple Python (lighter-style scripts, not heavy `validate_docs.py` pattern)
- Pilot decision recorded as `<!-- pilot-decision: pending|expand|stop -->` footer on the spec
- Wiring into `build.py`: deferred until pilot succeeds (Issue 2)
- **All work performed by the primary agent** (no subagent dispatch for this pilot)

## Key Changes

| Change | Type | Where |
|---|---|---|
| New feature spec | New file | `chronicler_engine/docs/specs/process_action.md` |
| New spec directory | New directory | `chronicler_engine/docs/specs/` |
| Scenario annotations | Edit (23 comments) | `tests/integration/application/action_pipeline/actions.rs`, `pipeline.rs` |
| New validator | New file | `chronicler_engine/scripts/validate_feature_spec.py` |

## Implementation

### Phase 1: Spec authoring (manual)

- [ ] #### Task 1.1: Create spec directory and write `process_action.md` (1 SP)
  - [ ] ##### SubTask 1.1.1: Create `chronicler_engine/docs/specs/` directory
  - [ ] ##### SubTask 1.1.2: Write `process_action.md` with 22 scenarios across 5 behavior groups (Normal flow, Error recovery, State hygiene, Async/cancellation, Snapshots), 5 invariants, Implementation reference, Out-of-scope list. Footer: `<!-- pilot-decision: pending -->`. Use bold-inline ID format (`- **1.1** description`)

### Phase 2: Test annotations (mechanical)

- [ ] #### Task 2.1: Annotate existing integration tests with `// SCENARIO: X.Y` comments (3 SP)
  - [ ] ##### SubTask 2.1.1: Annotate `tests/integration/application/action_pipeline/actions.rs` (8 tests, scenarios 1.1, 1.2, 1.5, 2.1, 2.2, 3.1, 3.4)
  - [ ] ##### SubTask 2.1.2: Annotate `tests/integration/application/action_pipeline/pipeline.rs` (15 tests, scenarios 1.3, 1.4, 2.3, 2.4, 3.2, 3.3, 4.1, 4.2, 4.3, 5.1, 5.2)

### Phase 3: Validator script (mechanical)

- [ ] #### Task 3.1: Write `validate_feature_spec.py` (5 SP)
  - [ ] ##### SubTask 3.1.1: Discover spec files in `chronicler_engine/docs/specs/*.md`
  - [ ] ##### SubTask 3.1.2: Parse each spec for scenario IDs via regex `^\s*-\s+\*\*(\d+\.\d+)\*\*\s+`
  - [ ] ##### SubTask 3.1.3: Walk `chronicler_engine/tests/integration/**/*.rs` for `// SCENARIO: X.Y` comments above `#[test]` attributes
  - [ ] ##### SubTask 3.1.4: Cross-reference: report scenarios with no covering test (gaps) and annotations pointing to undeclared scenarios (orphans), with file:line locations
  - [ ] ##### SubTask 3.1.5: Exit codes: 0 if no gaps/orphans, 1 if any, 2 on parse errors. Output: summary to stdout ("N scenarios declared, M covered, K gaps, J orphans"), violations to stderr

### Phase 4: Pilot verification

- [ ] #### Task 4.1: Run validator and verify behavior (1 SP)
  - [ ] ##### SubTask 4.1.1: Run validator on the complete pilot — must report 22 declared, 22 covered, 0 gaps, 0 orphans, exit 0
  - [ ] ##### SubTask 4.1.2: Introduce a deliberate gap (comment out one annotation in a test file) — must report 1 gap with the right scenario ID, exit 1
  - [ ] ##### SubTask 4.1.3: Restore annotation, introduce an orphan (add `// SCENARIO: 99.99` to a test) — must report 1 orphan, exit 1
  - [ ] ##### SubTask 4.1.4: Restore state — confirm validator returns to 0 gaps, 0 orphans
  - [ ] ##### SubTask 4.1.5: Run `cargo nextest run --package chronicler_engine --test integration` — all green, no behavior change
  - [ ] ##### SubTask 4.1.6: Update spec footer from `pending` to `expand` or `stop` based on the experience (record decision rationale in commit message)

## Test Plan

| What | How | Pass criterion |
|---|---|---|
| Spec file is well-formed markdown | Read it back | 22 scenarios numbered, 5 invariants, format `**X.Y**` |
| Test annotations are correct | `git diff` of annotated files | Each annotation refers to a real scenario ID |
| Validator parses the spec correctly | Run on complete pilot | Reports 22 declared scenarios |
| Validator discovers annotations | Run on complete pilot | Reports 22 covered scenarios |
| Validator detects gaps | Run after commenting one annotation | Reports 1 gap with correct ID, exits 1 |
| Validator detects orphans | Run after adding `// SCENARIO: 99.99` | Reports 1 orphan, exits 1 |
| Existing tests still pass | `cargo nextest run --package chronicler_engine --test integration` | All green, no behavior change |

## Per Task Validation Steps

| Task | Validation |
|---|---|
| 1.1.2 | `grep -c "^\- \*\*[0-9]" chronicler_engine/docs/specs/process_action.md` returns 22. `grep -c "^\- \*\*I\." chronicler_engine/docs/specs/process_action.md` returns 5. Footer line present |
| 2.1.1, 2.1.2 | `grep -rn "SCENARIO:" chronicler_engine/tests/integration/application/action_pipeline/` returns ≥23 matches |
| 3.1.5 | `python chronicler_engine/scripts/validate_feature_spec.py` runs to completion, exits 0 |
| 4.1.1 | Validator exits 0, prints "22 declared, 22 covered" |
| 4.1.2 | Validator exits 1, names missing scenario ID |
| 4.1.3 | Validator exits 1, names orphan annotation with file:line |
| 4.1.5 | `cargo nextest run --package chronicler_engine --test integration` passes |
| 4.1.6 | Spec footer updated, commit message captures decision rationale |

## Assumptions

- The 22 scenarios derived from existing 23 tests (some tests overlap with 1.1 — `test_pipeline_with_quantifier`, `test_pipeline_continues_when_quantifier_save_warns`). Multiple tests per scenario is fine; validator checks ≥1 coverage per scenario
- `flow/*.rs` tests and `retry.rs` are intentionally not annotated in this pilot. Future work
- The validator is a no-CLI script. Spec discovery is hardcoded to `docs/specs/*.md`, test discovery to `tests/integration/**/*.rs`. Both can be made configurable later
- `docs/specs/` is classified as EXCLUDED by `validate_docs.py` (correct for feature specs — they aren't standards docs)
- Invariants are documented in the spec but not enforced by the validator. Future work could add invariant annotations or proptest
- The spec is feature-level (`process_action`), not component-level (`action_pipeline`). `docs/system/action_pipeline.md` remains as component documentation; the new spec is additive
- Pilot scope is one feature. We are NOT committing to spec'ing every feature in the codebase — that's a decision made after evaluation
- Wiring into `build.py` is deferred per Issue 2. If the pilot's decision is "expand", the next plan will add the `timed_step` invocation
- All four phases run sequentially in this session by the primary agent; no subagent dispatch
