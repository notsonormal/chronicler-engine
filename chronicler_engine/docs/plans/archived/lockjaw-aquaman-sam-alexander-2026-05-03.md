# Implementation Plan: Isolate Slow LLM Tests

## Overview

Prevent `tests/flow_llm_tests.rs` from slowing down daily `cargo test` / `python build.py` runs, while ensuring the tests remain visible and runnable so they do not rot. The approach uses Rust's built-in `#[ignore]` attribute combined with enhanced `build.py` flags and updated agent-facing documentation.

## Architecture Decisions

- **`#[ignore]` over feature flags**: Standard Rust idiom. Works with both `cargo test` and `cargo nextest run`. Compilation overhead of one 389-line file is negligible compared to overall build time.
- **Two flags in `build.py`**: `--include-llm` for full-suite validation, `--llm-only` for focused LLM iteration. Gives developers precise control.
- **Notice on every default build**: Guarantees the tests cannot be forgotten. The message prints after the test step so it is seen without being noisy during compilation.
- **Agent skill updates**: AI agents working on `src/narrative/` must know to run `--llm-only`. Skills are the canonical instruction channel for agents.

## Task List

### Phase 1: Code Changes

#### Task 1: Tag LLM tests with `#[ignore]` and add module doc

**Description:** Add `#[ignore = "slow: requires OPENROUTER_API_KEY"]` to each of the 3 tests in `flow_llm_tests.rs`. Keep the existing `has_llm_api_key()` early-return logic. Add a module-level doc comment explaining why the tests are ignored and how to run them.

**Acceptance criteria:**
- [ ] `test_llm_generates_narration_for_free_action` has `#[ignore]`
- [ ] `test_llm_narration_appears_via_polling` has `#[ignore]`
- [ ] `test_llm_handles_arrival_narration` has `#[ignore]`
- [ ] Module doc comment at top of file explains the ignore rationale and run commands
- [ ] Existing `has_llm_api_key()` skip logic is preserved

**Verification:**
- [ ] `cargo test --test flow_llm_tests` reports "3 tests, 0 passed, 3 ignored"
- [ ] `cargo test --test flow_llm_tests -- --ignored` attempts to run them (will skip if no key)

**Dependencies:** None

**Files likely touched:**
- `chronicler_engine/tests/flow_llm_tests.rs`

**Estimated scope:** Small (1 file, additive changes)

---

#### Task 2: Add `--include-llm` and `--llm-only` flags to `build.py`

**Description:** Extend the `build.py` argument parser with two mutually exclusive flags. Wire them into the test execution step. When neither flag is passed, print a one-line notice after tests complete reminding the user that LLM tests were skipped.

**Acceptance criteria:**
- [ ] `python build.py` runs fast suite and prints a visible notice about skipped LLM tests
- [ ] `python build.py --include-llm` runs `cargo nextest run --run-ignored all`
- [ ] `python build.py --llm-only` runs `cargo nextest run --run-ignored all --test flow_llm_tests`
- [ ] `--include-llm` and `--llm-only` are documented in `--help` output
- [ ] Existing `--coverage` and `--release` flags continue to work

**Verification:**
- [ ] `python build.py --help` shows the new flags
- [ ] `python build.py` completes and notice is visible in output
- [ ] `python build.py --llm-only` executes only flow_llm_tests binary

**Dependencies:** None

**Files likely touched:**
- `chronicler_engine/build.py`

**Estimated scope:** Small (1 file, additive CLI changes)

---

### Phase 2: Documentation Updates

#### Task 3: Update testing reference docs

**Description:** Update `docs/reference/testing.md` and `docs/system/testing.md` to document the new commands. Replace outdated "All tests" runtime estimates with the new fast-suite target.

**Acceptance criteria:**
- [ ] Both docs list the 3 ways to run tests: default fast, `--include-llm`, `--llm-only`
- [ ] Command examples are accurate and match `build.py` implementation
- [ ] Runtime expectations table reflects that LLM tests are excluded by default

**Verification:**
- [ ] Read both docs and confirm commands match `build.py --help`

**Dependencies:** Task 2

**Files likely touched:**
- `chronicler_engine/docs/reference/testing.md`
- `chronicler_engine/docs/system/testing.md`

**Estimated scope:** Small (2 files, doc updates)

---

#### Task 4: Update README and AGENTS.md

**Description:** Add the LLM test policy to `AGENTS.md` so AI agents know when to run `--llm-only`. Update `README.md` Quick Start section with the new commands.

**Acceptance criteria:**
- [ ] `AGENTS.md` has an "LLM Test Policy" section under conventions
- [ ] Policy states: when modifying `src/narrative/` or LLM behavior, run `python build.py --llm-only`
- [ ] `README.md` Quick Start shows `python build.py --llm-only`

**Verification:**
- [ ] Read both files and confirm instructions are consistent with docs

**Dependencies:** Task 2

**Files likely touched:**
- `chronicler_engine/AGENTS.md`
- `chronicler_engine/README.md`

**Estimated scope:** Small (2 files, doc updates)

---

#### Task 5: Update agent skill files

**Description:** Update `.kimi/skills/chronicler-dev-workflow/SKILL.md` and `.kimi/skills/test-police/SKILL.md` to reference the new `--llm-only` flag in their validation and review sections.

**Acceptance criteria:**
- [ ] `chronicler-dev-workflow/SKILL.md` Validation Commands section includes `python build.py --llm-only` for narrative changes
- [ ] `test-police/SKILL.md` review checklist includes verifying LLM tests were run for narrative changes

**Verification:**
- [ ] Read both skill files and confirm additions are present

**Dependencies:** Task 2

**Files likely touched:**
- `.kimi/skills/chronicler-dev-workflow/SKILL.md`
- `.kimi/skills/test-police/SKILL.md`

**Estimated scope:** Small (2 files, doc updates)

---

## Checkpoints

### Checkpoint: After Tasks 1–2 (Foundation)
- [ ] `cargo test --test flow_llm_tests` shows 3 ignored
- [ ] `python build.py` runs fast and prints notice
- [ ] `python build.py --llm-only` runs only flow_llm_tests
- [ ] `python build.py --include-llm` runs full suite with LLM tests

### Checkpoint: After All Tasks (Complete)
- [ ] All 8 files updated consistently
- [ ] Documentation commands match `build.py --help`
- [ ] Agent skills reference the new flag
- [ ] No references to old "all tests ~3 min" claim if it is no longer accurate

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Notice in `build.py` becomes noise / ignored | Low | Keep it to one line, printed only after test step, not during compilation. |
| Agent ignores skill update and skips LLM tests | Medium | Update both `chronicler-dev-workflow` (author) and `test-police` (reviewer) skills. Two checkpoints. |
| `--llm-only` flag name conflicts with future flag | Low | Names are descriptive and scoped to test behaviour. |
| `cargo nextest` command syntax is wrong | Medium | Verify with actual execution in Checkpoint 1. |

## Open Questions

- [ ] Should `--llm-only` also support `--coverage`? (e.g., `python build.py --llm-only --coverage`) — not required for initial implementation.
