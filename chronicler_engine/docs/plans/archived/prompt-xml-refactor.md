# Prompt XML Refactor Plan

## TL;DR

> **Quick Summary**: Replace `=== SECTION ===` delimiters with XML tags `<Section></Section>` in the LLM prompt builder.
> 
> **Deliverables**: 
> - Updated prompt.rs with 8 XML section headers
> - Updated tests checking for old format
> 
> **Estimated Effort**: Small
> **Parallel Execution**: NO - sequential (2 tasks)
> **Critical Path**: Update delimiters → Fix tests

---

## Context

### Original Request
User wants to change how the system prompt is presented from plain text sections:
```
=== PLAYER CHARACTER ===

=== WORLD LORE ===

=== CONVERSATION HISTORY ===
```

To XML blocks:
```
<PlayerCharacter></PlayerCharacter>

<WorldLore></WorldLore>

<ConversationHistory></ConversationHistory>
```

### Metis Review
**Identified Gaps** (addressed):
- Found 8 section headers total (not 5) - including SYSTEM PROMPT, GAME STATE, NPC PRESENCE at top of file
- Tests at lines 628+ assert old format and will fail

---

## Work Objectives

### Core Objective
Convert all 8 section headers in `src/narrative/prompt.rs` from `=== HEADER ===` to `<Header></Header>` format, then fix failing tests.

### Must Have
- [ ] All 8 section delimiters updated
- [ ] All tests pass (`cargo test`)

### Must NOT Have
- [ ] No changes to content structure (only delimiters)
- [ ] No behavioral changes to prompt generation

---

## Verification Strategy

**Test Decision**:
- **Infrastructure exists**: YES
- **Automated tests**: Tests-after
- **Framework**: Rust built-in

---

## TODOs

- [x] 1. Update 8 section delimiters in prompt.rs

  **What to do**:
  - Change line 243: `=== SYSTEM PROMPT ===` → `<SystemPrompt></SystemPrompt>`
  - Change line 258: `=== GAME STATE ===` → `<GameState></GameState>`
  - Change line 282: `=== NPC PRESENCE ===` → `<NpcPresence></NpcPresence>`
  - Change line 309: `=== PLAYER CHARACTER ===` → `<PlayerCharacter></PlayerCharacter>`
  - Change line 328: `=== WORLD LORE ===` → `<WorldLore></WorldLore>`
  - Change line 349: `=== CONVERSATION HISTORY ===` → `<ConversationHistory></ConversationHistory>`
  - Change line 372: `=== PLAYER INPUT ===` → `<PlayerInput></PlayerInput>`
  - Change line 381: `=== AUXILIARY INSTRUCTIONS ===` → `<AuxiliaryInstructions></AuxiliaryInstructions>`

  **Must NOT do**:
  - Don't change content, only delimiters

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **References**: 
  - `src/narrative/prompt.rs:243` - First header to change
  - `src/narrative/prompt.rs:258` - Second header  
  - `src/narrative/prompt.rs:282` - Third header
  - `src/narrative/prompt.rs:309` - Fourth header
  - `src/narrative/prompt.rs:328` - Fifth header
  - `src/narrative/prompt.rs:349` - Sixth header
  - `src/narrative/prompt.rs:372` - Seventh header
  - `src/narrative/prompt.rs:381` - Eighth header

  **Acceptance Criteria**:
  - [ ] `grep -n "===" src/narrative/prompt.rs` returns 0 matches

  **QA Scenarios**:
  ```
  Scenario: Verify all delimiters converted
    Tool: Bash
    Preconditions: File modified
    Steps:
      1. grep -n "===" src/narrative/prompt.rs
    Expected Result: No matches (0 results)
    Evidence: Terminal output showing 0 matches

  Scenario: Verify XML tags exist
    Tool: Bash
    Preconditions: File modified
    Steps:
      1. grep -n "<.*>" src/narrative/prompt.rs | head -20
    Expected Result: 8+ XML tag matches
    Evidence: Terminal output showing XML tags
  ```

  **Commit**: YES (1)
  - Message: `refactor(prompt): convert === to <> XML delimiters`
  - Files: `src/narrative/prompt.rs`
  - Pre-commit: `cargo test`

- [x] 2. Fix failing tests (assert old format)

  **What to do**:
  - Run `cargo test` to find failing test assertions
  - Update tests at lines 628+ that check for `"=== PLAYER CHARACTER ==="` etc.
  - Change to check for `<PlayerCharacter></PlayerCharacter>` format

  **Must NOT do**:
  - Don't weaken test coverage

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **References**:
  - `src/narrative/prompt.rs:628-900` - Test assertions to update

  **Acceptance Criteria**:
  - [ ] `cargo test` passes (0 failures)

  **QA Scenarios**:
  ```
  Scenario: Tests pass
    Tool: Bash
    Preconditions: Tests updated
    Steps:
      1. cd D:/John/DevContainer/mrn-general/chronicler_engine
      2. cargo test
    Expected Result: All tests pass
    Evidence: Test output showing 0 failures
  ```

  **Commit**: YES (2)
  - Message: `test(prompt): update assertions for XML format`
  - Files: `src/narrative/prompt.rs`
  - Pre-commit: `cargo test`

---

## Final Verification Wave

- [x] F1. **Build verification** — `quick`
  Run `cargo build` to ensure code compiles
  Output: `Build [PASS/FAIL] | VERDICT`

- [x] F2. **Test verification** — `quick`
  Run `cargo test` to ensure all tests pass
  Output: `Tests [N/N pass] | VERDICT`

---

## Commit Strategy

- **1**: `refactor(prompt): convert === to <> XML delimiters` - src/narrative/prompt.rs, cargo test
- **2**: `test(prompt): update assertions for XML format` - src/narrative/prompt.rs, cargo test

---

## Success Criteria

### Verification Commands
```bash
cargo build  # Expected: success
cargo test   # Expected: 0 failures
grep -n "===" src/narrative/prompt.rs  # Expected: 0 matches
```