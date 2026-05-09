# Plan: Fast-Fail Build & Test Localization

**Date:** 2026-05-09
**Status:** Planned
**Goal:** Reduce time-to-first-signal and make failure localization precise.

---

## Overview

Currently, diagnosing a problem requires:
1. Running `python build.py` — 8 sequential steps (fmt, clippy, architecture tests, guardrails, test structure check, build, tests, report)
2. If tests fail, parsing output from `component_tests.rs` (1,504 lines) or `e2e_tests.rs` (781 lines)
3. Manually mapping the failure line to a subsystem

This plan restructures the build pipeline and test suite so failures are caught earlier, named more precisely, and linked to relevant documentation automatically.

---

## Background

**Build pipeline today:**
```
fmt → clippy → architecture tests → guardrails → test structure → build → tests → report
```
A logic error in `trigger_eval.rs` is only caught at the `tests` step, after ~2-5 minutes of prior steps.

**Test structure today:**
- `tests/component_tests.rs` — 1,504 lines. Tests server routing, template rendering, engine logic, and state mutation in one file.
- `tests/e2e_tests.rs` — 781 lines. End-to-end flows through the HTTP API.
- `tests/game_service_tests.rs` — 33,952 bytes. Game service integration tests.
- `tests/diagnostic_benchmark.rs` — 40,488 bytes. Diagnostic signal quality tests.

When `cargo nextest` reports a failure in `component_tests.rs:423`, the failure could be in templates, routing, or engine logic. The agent must read the test body to determine the subsystem.

**Documentation is rich but unlinked from failures:** `docs/system/triggers.md`, `docs/system/navigation.md`, etc. exist, but a failing test does not tell you which doc to read.

---

## Architecture Decisions

1. **Tests are organized by subsystem, not by granularity.** Unit tests stay in `src/*_tests.rs`. Integration tests move from monolithic files to `tests/<subsystem>_tests.rs`.
2. **Build steps are ordered by speed-to-signal.** `cargo check` (fast syntax/type check) runs before `cargo build` (slow codegen). A subset of tests runs before the full suite.
3. **Failure messages link to docs.** Custom `nextest` config and test macros print `See: docs/system/<relevant>.md` on assertion failure.
4. **Invariant violations get their own fast test target.** The load-bearing invariants (state mutation order, trigger timing) are extracted into `tests/invariant_contract_tests.rs` that runs in <1 second.

---

## Phase 1: Investigation — Map Current Failure Modes

### Task 1.1: Catalog Current Test Coverage by Subsystem
- Analyze `tests/component_tests.rs`, `tests/e2e_tests.rs`, `tests/game_service_tests.rs`, and `tests/diagnostic_benchmark.rs` to classify every test by subsystem.
- Subsystem taxonomy:
  - `server` — routing, templates, fragments, HTTP handlers
  - `engine` — action processing, parser, logic
  - `trigger` — trigger evaluation, firing conditions
  - `narrative` — prompt building, LLM client, quantifier
  - `state` — state mutation, serialization, invariants
  - `bootstrap` — CLI parsing, settings loading, world loading
- **Deliverable:** A table mapping each test function to its subsystem and current file location.

### Task 1.2: Measure Build Step Durations
- Run `python build.py` 3 times and record per-step timing.
- Identify which steps dominate wall-clock time.
- **Deliverable:** Table of mean duration per step.

### Task 1.3: Identify Invariant Tests
- Find all tests that verify the documented load-bearing invariants:
  - State mutation order in `action_processing.rs`
  - Trigger timing (evaluate BEFORE increment `times_met`)
  - `LogType::System` ordering constraints
- **Deliverable:** List of test functions that are actually invariant guards.

---

## Phase 2: Implementation — Test Reorganization

### Task 2.1: Extract Subsystem-Specific Test Files
Split the monolithic integration test files into focused files. Do not change test logic — only move functions and fix imports.

**Target structure:**
```
tests/
  server/
    routing_tests.rs      (from component_tests.rs)
    template_tests.rs     (from component_tests.rs)
    fragment_tests.rs     (from component_tests.rs)
  engine/
    action_tests.rs       (from component_tests.rs + game_service_tests.rs)
    logic_tests.rs        (from component_tests.rs)
    parser_tests.rs       (from component_tests.rs)
  trigger/
    eval_tests.rs         (from trigger_tests.rs + component_tests.rs)
    timing_tests.rs       (new — extracted invariant tests)
  narrative/
    prompt_tests.rs       (from narrative tests)
    quantifier_tests.rs   (from narrative tests)
    llm_client_tests.rs   (from narrative tests)
  e2e/
    flow_mock_tests.rs    (existing, keep)
    flow_llm_tests.rs     (existing, keep)
  invariant_contract_tests.rs  (new — fast invariant checks)
```

- **Files:** All `tests/*.rs` files
- **Acceptance criteria:**
  - [ ] Every test function lives in a file whose name indicates the subsystem
  - [ ] `cargo test` passes with same count as before
  - [ ] No test logic changed (only imports and module-level helpers)

### Task 2.2: Add Invariant Contract Tests
Create `tests/invariant_contract_tests.rs` with fast, focused tests that verify documented invariants:

1. **State Mutation Order**
   - Mock an action that causes movement + NPC resolution + narration + trigger eval.
   - Verify via `GameState` inspection that history ordering matches the invariant.

2. **Trigger Timing**
   - Set `times_met = 0` and create a trigger with condition `TimesMet Eq 0`.
   - Verify trigger fires before `times_met` is incremented.

3. **Log Ordering**
   - Verify `AI response must be after input` invariant is enforced by `state.rs`.

- **File:** `tests/invariant_contract_tests.rs` (new)
- **Acceptance criteria:**
  - [ ] Each invariant test runs in <500ms
  - [ ] Tests use only mock backends (no network)
  - [ ] Failure message names the specific invariant violated

### Task 2.3: Enrich Test Failure Output
- Add a test helper macro `assert_with_doc!(condition, "docs/system/triggers.md")` that on failure prints:
  ```
  assertion failed: condition
  See: docs/system/triggers.md
  Relevant invariant: Trigger evaluates before times_met increments
  ```
- Apply to all invariant and subsystem tests.
- **File:** `tests/test_utils.rs` (extend)
- **Acceptance criteria:**
  - [ ] A failing test prints a relevant doc link
  - [ ] Macro is used in ≥50% of integration tests

---

## Phase 3: Implementation — Build Pipeline Restructuring

### Task 3.1: Add `--quick` Mode to `build.py`
Add a `--quick <pattern>` flag that:
1. Skips `fmt`, clippy, architecture tests, and guardrails
2. Runs `cargo check --tests`
3. Runs only tests matching the pattern: `cargo nextest run <pattern>`

- **File:** `build.py`
- **Acceptance criteria:**
  - [ ] `python build.py --quick trigger` runs in <30 seconds
  - [ ] Pattern supports subsystem names (e.g., `trigger`, `server`)
  - [ ] Returns non-zero on test failure

### Task 3.2: Reorder Build Steps for Faster Signal
Restructure the default build path:
```
cargo check --tests       (fast type/syntax check — catches most logic errors)
fmt                       (only if --no-fmt not set)
clippy                    (linting)
quick smoke tests         (invariant_contract_tests + guardrails — <10s)
full build
test structure check
architecture tests
full test suite
report
```

- **File:** `build.py`
- **Acceptance criteria:**
  - [ ] A type error in `trigger_eval.rs` is caught at step 1, not step 6
  - [ ] Total build time for a clean compile is unchanged
  - [ ] A failing invariant test stops the build before full tests run

### Task 3.3: Add Subsystem Filter to Nextest Config
Update `tests/nextest.toml` with profile `subsystem`:
```toml
[profile.subsystem]
filter = 'test(/^server::/) | test(/^engine::/) | test(/^trigger::/)'
```

- **File:** `tests/nextest.toml`
- **Acceptance criteria:**
  - [ ] `cargo nextest run --profile subsystem` runs only subsystem-organized tests

---

## Phase 4: Verification

### Task 4.1: Measure Post-Reorganization MTTD
- Introduce the same controlled failures from the observability plan baseline.
- Time how long it takes to identify the correct subsystem and root cause.
- **Acceptance criteria:**
  - [ ] Time to identify correct subsystem ≤30 seconds
  - [ ] Agent reads ≤2 source files before diagnosis

### Task 4.2: Validate Build Speed Improvements
- Run `python build.py` with a deliberate type error in `src/engine/trigger_eval.rs`.
- Measure time to first failure signal.
- **Acceptance criteria:**
  - [ ] Type errors caught in `cargo check` step (<30s from build start)
  - [ ] Invariant violations caught in smoke test step (<60s from build start)

### Task 4.3: Ensure No Coverage Regression
- Run `python build.py --coverage` before and after reorganization.
- **Acceptance criteria:**
  - [ ] Total line coverage unchanged (±2%)
  - [ ] No files dropped below their previous coverage threshold

---

## Dependencies

| Task | Depends on | Blocks |
|------|-----------|--------|
| 1.1 Catalog tests | None | 2.1 |
| 1.2 Measure build | None | 3.2 |
| 1.3 Identify invariants | None | 2.2 |
| 2.1 Extract tests | 1.1 | 2.3, 4.1, 4.3 |
| 2.2 Invariant tests | 1.3 | 3.2, 4.2 |
| 2.3 Enrich failures | 2.1 | 4.1 |
| 3.1 `--quick` mode | None | 4.1 |
| 3.2 Reorder steps | 1.2, 2.2 | 4.2 |
| 3.3 Nextest config | 2.1 | 4.1 |
| 4.1 MTTD measure | 2.1, 2.3, 3.1, 3.3 | — |
| 4.2 Build speed | 3.2 | — |
| 4.3 Coverage | 2.1 | — |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| Moving tests breaks imports or shared helpers | Extract shared helpers to `tests/test_utils.rs` first; move tests in batches |
| `cargo nextest` filter syntax is wrong | Test each profile manually before committing |
| Build reorder breaks agent expectations | Update `AGENTS.md` and all skill files with new build flow |
| Coverage drops because tests moved | Run coverage diff before/after; add tests if coverage drops |

---

## Success Criteria

1. A test failure names the subsystem in the test file path (e.g., `tests/trigger/eval_tests.rs` not `tests/component_tests.rs`).
2. A type or logic error is caught by `cargo check --tests` or smoke tests within 60 seconds of build start.
3. Every invariant test runs in <500ms and fails with a message naming the specific invariant.
4. No regression in total test count or coverage.
