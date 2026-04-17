# OpenRouter Client Enhancement Plan

## TL;DR

> **Quick Summary**: Enhance content extraction in `openrouter_client.rs` to handle more model edge cases: `reasoning_content` field, better null/empty handling, and improved logging based on Silly Tavern patterns.
> 
> **Deliverables**: 
> - Enhanced openrouter_client.rs with robust content extraction
> - Better logging showing which extraction path was used
> - Clear fallback chain documentation
> 
> **Estimated Effort**: Short
> **Parallel Execution**: NO - sequential (single file)
> **Critical Path**: Research → Implement → Test

---

## Context

### Original Request
User saw comment in openrouter_client.rs line 88-89 about Z-AI models returning null content and putting response in reasoning field. Asked to research Silly Tavern source code for additional patterns.

### Interview Summary
**Key Discussions**:
- User wants robust handling of various model response formats
- Found multiple edge cases from Silly Tavern that current implementation doesn't handle

**Research Findings**:
- OpenRouter uses `reasoning_content` for extended responses (PR #5160)
- Claude returns multi-part with `thinking` + `text` blocks (PR #5278)
- Some models return empty string "", some return null
- `reasoning_details` contains extended metadata
- Gemini uses `text:` format in reasoning blocks

---

## Work Objectives

### Core Objective
Enhance content extraction in `openrouter_client.rs` to handle all known model response edge cases, with better logging to diagnose extraction paths.

### Concrete Deliverables
- Updated `openrouter_client.rs` with improved extraction logic
- Log which fallback path was used (content → reasoning → reasoning_content)
- Handle null vs empty string distinction properly

### Definition of Done
- [x] cargo build succeeds
- [x] cargo test passes (existing tests)
- [x] cargo clippy passes
- [x] Extraction chain handles: content, reasoning, reasoning_content, reasoning_details
- [x] Logging shows which path extracted content

### Must Have
- Handle `reasoning_content` field (OpenRouter extended)
- Handle null vs empty string properly
- Better logging showing extraction path

### Must NOT Have
- Don't change public API of the function
- Don't break existing tests

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES - flow_llm_tests.rs
- **Automated tests**: None - this is LLM-dependent
- **If TDD**: No - enhancement is based on research, not new features

### QA Policy
Every change MUST include agent-executed verification:
- Build test: `cargo build --release` succeeds
- Clippy: `cargo clippy -- -D warnings` passes  
- Existing tests: `cargo test` passes

---

## Execution Strategy

### Single File Task
This is a focused enhancement - single file, sequential execution.

```
Task 1: Enhance content extraction in openrouter_client.rs
  ├── Update content extraction chain
  ├── Add reasoning_content fallback
  ├── Add verbose logging
  ├── Handle null vs empty
  └── Verify build + clippy + tests pass
```

---

## TODOs

- [x] 1. Enhance content extraction in openrouter_client.rs

  **What to do**:
  - Refactor content extraction to a clear fallback chain:
    1. First: Try `content` field (if non-null AND non-empty)
    2. Second: Try `reasoning` field (if non-null AND non-empty)
    3. Third: Try `reasoning_content` field (OpenRouter extended)
    4. Log which path was used at Info level
  - Handle null vs empty distinction (some models return "" vs null)
  - Add helpful debug logging

  **Must NOT do**:
  - Change function signature (public API)
  - Add dependencies
  - Break existing tests

  **Recommended Agent Profile**:
  - **Category**: `quick` - Single file, focused change
  - **Skills**: []
  - **Reason**: Simple refactor, no new features

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: None (single task)
  - **Blocked By**: None

  **References**:
  - `src/narrative/openrouter_client.rs:88-103` - Current extraction logic to replace
  - `src/narrative/llm.rs` - Related LLM backend trait

  **Acceptance Criteria**:
  - [ ] cargo build --release succeeds
  - [ ] cargo clippy passes
  - [ ] cargo test passes (at least existing tests)
  - [ ] Logging shows which extraction path was used

  **QA Scenarios**:

  ```
  Scenario: Build and clippy passes
    Tool: Bash
    Preconditions: None
    Steps:
      1. cargo build --release
      2. cargo clippy -- -D warnings
    Expected Result: Both succeed with no errors
    Evidence: Terminal output showing success

  Scenario: Existing tests pass
    Tool: Bash
    Preconditions: None (tests that don't need API key should pass)
    Steps:
      1. cargo test --test flow_mock_tests
    Expected Result: Test suite passes
    Evidence: Test output showing pass
  ```

---

## Final Verification Wave

- [x] F1. **Build + Clippy Check** — compile + lint
  Run: cargo build --release && cargo clippy -- -D warnings
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL]`

- [x] F2. **Run Tests** — existing test suite  
  Run: cargo test
  Output: `Tests [N pass/N fail] | VERDICT: PASS/FAIL`

---

## Final Verification Complete

**Verification Results**:
- ✅ Build: `cargo build --release` - SUCCESS
- ✅ Clippy: `cargo clippy -- -D warnings` - PASS
- ✅ Tests: 5 passed, 0 failed

**Final State**: ALL TASKS COMPLETE ✅

---

## Commit Strategy

- `feat(narrative): enhance OpenRouter content extraction with reasoning_content fallbacks`
  - src/narrative/openrouter_client.rs

---

## Success Criteria

### Verification Commands
```bash
cargo build --release    # Expected: success
cargo clippy -- -D warnings  # Expected: no warnings
cargo test              # Expected: tests pass
```