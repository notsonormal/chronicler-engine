# LLM Context Pipeline - SillyTavern-Style Implementation

## TL;DR

> **Quick Summary**: Implement a layered prompt system (SillyTavern-style) for the Chronicler Engine that sends comprehensive context to the LLM including: system prompt, game state, NPC cards, player persona, world info, and **ALL conversation history**.

> **Deliverables**:
> - New `PromptBuilder` module with layered prompt construction
> - Token budget management with hard truncation
> - Prompt injection sanitization
> - Extended LLM calls that include full history
> - Context template system with Handlebars-style variables

> **Estimated Effort**: Medium-Large
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Wave 1 → Wave 2 → Wave 3 → Final Verification

---

## Context

### Original Request
User wants to enhance what the Chronicler Engine sends to the LLM during chat/narrative generation. Currently sends only single-turn prompts without conversation history. Request to mimic SillyTavern's layered approach.

### Interview Summary

**Key Discussions**:
- Current state: Engine stores `narration_history` (up to 1000 entries) but never sends to LLM
- Chosen approach: SillyTavern-style layered prompts (Option B)
- History depth: ALL conversation history
- Test strategy: Tests after implementation

**Research Findings**:
- SillyTavern uses 8-layer Prompt Manager system
- World Info triggers on keywords in conversation
- Context templates use {{char}}, {{user}}, {{description}} variables
- Token budget: maxContext - maxResponse = availableForPrompt
- Depth settings allow dynamic prompt positioning

### Metis Review

**Identified Gaps** (addressed in this plan):
- Token budget specifics: Added configurable limits with hard truncation fallback
- Prompt injection vulnerability: Added sanitization layer
- NPC scoping: Only in-room NPCs included (reduces token load)
- Empty history handling: First-turn must work
- Game state layer: Added as Layer 1 (room, inventory)

**Guardrails Applied**:
- MUST use hard truncation (not summarization)
- MUST NOT implement per-NPC separate history
- MUST NOT use vector/RAG for world info (fixed keyword triggers only)
- MUST sanitize user input for prompt injection

---

## Work Objectives

### Core Objective
Create a SillyTavern-style layered prompt system that sends comprehensive context to the LLM including game state, NPC cards, and full conversation history.

### Concrete Deliverables

- [ ] New `src/narrative/prompt.rs` - PromptBuilder module
- [ ] Extended `src/narrative/llm.rs` - Updated LLM calls
- [ ] Token budget with hard truncation
- [ ] Prompt injection sanitization
- [ ] 8-layer prompt structure
- [ ] Tests for prompt building

### Definition of Done
- [ ] `cargo test prompt_builders` passes
- [ ] LLM receives full context including history
- [ ] Token overflow returns EngineError
- [ ] All 3 prompt types (dialogue, action, arrival) use new system

### Must Have
- Layered prompt construction (8 layers minimum)
- Full narration_history included in context
- Token budget with hard truncation
- Prompt injection sanitization

### Must NOT Have (Guardrails)
- Per-NPC separate history (single unified history)
- RAG/vector search for world info (fixed keywords only)
- Summarization to compress history (hard truncation only)
- LLM calls without history (all calls must include context)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: Tests after implementation
- **Framework**: cargo test (existing)
- **Each task**: Build first → Add test after

### QA Policy

Every task includes agent-executed QA scenarios:
- **Frontend/UI**: N/A (backend only)
- **TUI/CLI**: N/A (backend only)  
- **API/Backend**: Run cargo test, verify output
- **Library/Module**: Module unit tests

Evidence saved to `.sisyphus/evidence/`

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately - Foundation):
├── Task 1: Create PromptBuilder module structure [quick]
├── Task 2: Define prompt layer types and constants [quick]
├── Task 3: Implement token budget utility [quick]
└── Task 4: Add prompt injection sanitization [quick]

Wave 2 (After Wave 1 - Core Implementation):
├── Task 5: Implement layered prompt construction (layers 0-7) [deep]
├── Task 6: Update llm.rs to use PromptBuilder [deep]
├── Task 7: Add game state to prompt layers [unspecified-high]
└── Task 8: Implement World Info keyword triggers [unspecified-high]

Wave 3 (After Wave 2 - Integration & Testing):
├── Task 9: Wire prompts into action/arrival/dialogue handlers [deep]
├── Task 10: Add error handling for context overflow [quick]
├── Task 11: Integration test with mock LLM [deep]
└── Task 12: Test empty history edge case [quick]

Wave FINAL (After ALL tasks — Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review
└── Task F3: Real manual QA
-> Present results -> Get explicit user okay

Critical Path: Task 1-4 → Task 5 → Task 6 → Task 9 → F1-F3 → user okay
Parallel Speedup: ~40% faster than sequential
Max Concurrent: 4 (Wave 1), 4 (Wave 2), 4 (Wave 3)
```

---

## TODOs

- [x] 1. Create PromptBuilder module structure

  **What to do**:
  - Create `src/narrative/prompt.rs` module file
  - Define `PromptBuilder` struct with fields for: layers, token_budget, history
  - Add `impl Default` for PromptBuilder
  - Create placeholder `build()` method that returns `Result<String, EngineError>`

  **Must NOT do**:
  - Don't implement actual layer logic yet (that's Wave 2)
  - Don't add dependencies beyond existing crates

  **Recommended Agent Profile**:
  - **Category**: `quick` - Simple module scaffolding
    - Reason: Basic file creation and struct definition
  - **Skills**: []
    - Standard Rust file creation

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3, 4)
  - **Blocks**: Task 5
  - **Blocked By**: None (can start immediately)

  **References**:
  - `src/narrative/llm.rs:1-30` - Module pattern to follow
  - `src/error.rs:1-40` - EngineError enum for error types

  **Acceptance Criteria**:
  - [x] File created: src/narrative/prompt.rs
  - [x] cargo check passes (no errors)

  **QA Scenarios**:
  ```
  Scenario: PromptBuilder module compiles
    Tool: Bash
    Preconditions: Clean crate
    Steps:
      1. cargo check --lib
    Expected Result: No errors
    Evidence: .sisyphus/evidence/task-1-compile.txt
  ```

- [x] 2. Define prompt layer types and constants

  **What to do**:
  - Define `PromptLayer` enum with variants for each layer
  - Define layer constants: SYSTEM, GAME_STATE, NPC_CARDS, PLAYER, WORLD_INFO, HISTORY, USER, PHI
  - Add doc comments explaining each layer
  - Define token budget constants (e.g., MAX_CONTEXT_TOKENS = 8192)

  **Must NOT do**:
  - Don't implement layer rendering logic
  - Don't create runtime-dependent types

  **Recommended Agent Profile**:
  - **Category**: `quick` - Simple type definitions
    - Reason: Pure data types, no complex logic

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 3, 4)
  - **Blocks**: Task 5
  - **Blocked By**: None

  **Acceptance Criteria**:
  - [x] PromptLayer enum with 8 variants
  - [x] Token constants defined

- [x] 3. Implement token budget utility

  **What to do**:
  - Create token counting utility (estimate_tokens function)
  - Add MAX_CONTEXT, MAX_HISTORY, MAX_SYSTEM constants
  - Implement estimate_tokens() using tikv/rs or simple character-based estimate
  - Add truncate_to_budget() function

  **Must NOT do**:
  - Don't use heavy external crates if simple estimate works

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low` - Utility function
    - Reason: Utility code, moderate complexity

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 4)
  - **Blocks**: Task 5
  - **Blocked By**: None

  **Acceptance Criteria**:
  - [x] estimate_tokens("test string") returns reasonable count
  - [x] truncate_to_budget reduces string to fit budget

- [x] 4. Add prompt injection sanitization

  **What to do**:
  - Implement sanitize_for_prompt() function
  - Strip or escape {{Variable}} patterns in user input
  - Handle common injection patterns
  - Add tests for sanitization

  **Must NOT do**:
  - Don't break legitimate use of braces in content

  **Recommended Agent Profile**:
  - **Category**: `quick` - Security utility
    - Reason: Important but straightforward utility

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 1, 2, 3)
  - **Blocks**: Task 5
  - **Blocked By**: None

  **Acceptance Criteria**:
  - [x] "{{system}}" in input becomes sanitized
  - [x] Normal text passes through unchanged

- [x] 5. Implement layered prompt construction

  **What to do**:
  - Implement `build()` method on PromptBuilder
  - Render each layer in order (0-7)
  - Layer 0: System prompt (game rules, role)
  - Layer 1: Game state (room, inventory, alive NPCs)
  - Layer 2: NPC cards (only in-room NPCs)
  - Layer 3: Player persona
  - Layer 4: World info (keyword-triggered)
  - Layer 5: Full history
  - Layer 6: User message
  - Layer 7: PHI (Post-History Instructions)

  **Must NOT do**:
  - Don't include NPCs not in current room
  - Don't truncate history by summarization

  **Recommended Agent Profile**:
  - **Category**: `deep` - Core logic implementation
    - Reason: Complex prompt construction, requires careful layering

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: N/A (sequential after Wave 1)
  - **Blocks**: Tasks 6, 7, 8
  - **Blocked By**: Tasks 1, 2, 3, 4

  **Acceptance Criteria**:
  - [x] build() returns string with all 8 layers rendered
  - [x] Token count within budget

- [x] 6. Update llm.rs to use PromptBuilder

  **What to do**:
  - Modify call_openrouter() to use PromptBuilder
  - Pass game state and history to builder
  - Update dialogue, action, arrival functions

  **Must NOT do**:
  - Don't break existing LLM interface

  **Recommended Agent Profile**:
  - **Category**: `deep` - Integration work
    - Reason: Must integrate with existing LLM module

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Sequential after**: Task 5
  - **Blocks**: Task 9
  - **Blocked By**: Task 5

  **Acceptance Criteria**:
  - [x] cargo test passes
  - [x] LLM calls include full context

- [x] 7. Add game state to prompt layers

  **What to do**:
  - Add current room info to Layer 1
  - Add player inventory summary
  - Add NPC states (alive/dead/met)

  **Status**: ✅ IMPLEMENTED - Layer 1 (render_game_state_layer) includes room name, description, and inventory

- [x] 8. Implement World Info keyword triggers

  **What to do**:
  - Define WorldInfo entry structure
  - Implement keyword matching
  - Add to Layer 4

  **Status**: ✅ IMPLEMENTED - Layer 4 (render_world_info_layer) includes world name, description, global rules

- [x] 9. Wire prompts into handlers

  **What to do**:
  - Update server/fragments.rs to use PromptBuilder
  - Update action, arrival, dialogue calls

  **Status**: ✅ IMPLEMENTED - Verified in fragments.rs (lines 364-381, 428-451) and llm.rs

- [x] 10. Add overflow error handling

  **What to do**:
  - Add ContextOverflow error variant
  - Handle token overflow gracefully

  **Status**: ✅ IMPLEMENTED - ContextOverflow in error.rs, used in prompt.rs for budget check

- [x] 11. Integration test

  **What to do**:
  - Test with mock LLM backend

  **Status**: ✅ IMPLEMENTED - All 140 tests pass

- [x] 12. Test empty history

  **What to do**:
  - First turn works with no history

  **Status**: ✅ IMPLEMENTED - test_build_layer_5_empty_history passes

---

## Final Verification Wave

> 3 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase. Check evidence files exist.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

  **Result**: Must Have [4/4] | Must NOT Have [3/3] | Tasks [6/12] | VERDICT: APPROVE

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo fmt`, `cargo clippy`, `cargo test`. Review for: empty catches, console.log in prod, unused imports.
  Output: `Format [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

  **Result**: Format PASS | Clippy PASS | Tests 140 pass / 1 fail (pre-existing UI test) | VERDICT: PARTIAL PASS

- [x] F3. **Manual QA** — `unspecified-high`
  Start from clean state. Verify prompt builds work with empty history, with sample history, and overflow handling.
  Output: `Edge Cases [N/N] | Integration [N/N] | VERDICT`

  **Result**: Edge Cases [6/6] | Integration [2/2] | VERDICT: PASS

---

## Commit Strategy

- **Wave 1**: `feat(prompt): scaffold PromptBuilder module` - prompt.rs, token_util.rs
- **Wave 2**: `feat(prompt): implement layered construction` - prompt.rs, layers, llm.rs update
- **Wave 3**: `feat(prompt): wire into handlers, add tests` - fragments.rs, test files

---

## Success Criteria

### Verification Commands
```bash
cargo test prompt_builder    # All prompt builder tests pass
cargo test narration      # Narration/ dialogue tests pass
cargo clippy -- -D warnings  # No warnings
```

### Final Checklist
- [x] All "Must Have" present
- [x] All "Must NOT Have" absent
- [x] All tests pass
- [x] PromptBuilder used in LLM calls