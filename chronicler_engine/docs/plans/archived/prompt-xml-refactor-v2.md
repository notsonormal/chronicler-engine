# Prompt Refactor v2: Silly Tavern Integration + Quantifier XML

## TL;DR

> **Quick Summary**: Add Silly Tavern-inspired behavioral instructions to the normal narrative prompt (input validation, state tracking, world dynamics, causality) and convert the quantifier prompt from flat text to XML-structured format.
> 
> **Deliverables**: 
> - Updated `render_system_layer()` in `prompt.rs` with Silly Tavern behavioral rules
> - Updated `render_phi_layer()` in `prompt.rs` with looser AuxiliaryInstructions
> - Updated `build_system_prompt()` and `build_user_prompt()` in `quantifier.rs` with XML structure
> - All existing tests pass (update assertions where needed)
> 
> **Estimated Effort**: Small
> **Parallel Execution**: YES - 2 parallel tasks (prompt.rs + quantifier.rs are independent)
> **Critical Path**: Task 1 → Tests → Task 2 → Tests (sequential within each file)

---

## Context

### Original Request
User reviewed a Silly Tavern prompt and asked which parts could be reused for the Chronicler Engine's normal or quantifier prompts.

### Interview Summary
**Key Discussions**:
- Silly Tavern prompt has strong input validation, state tracking, world dynamics, and causality rules — none of which exist in Chronicler's current prompts
- Normal prompt already uses XML tags (from a previous refactor); quantifier prompt uses flat text
- User approved adding: Input Validation, State Tracking (knowledge boundaries), World Dynamics, Causality, Dialogue Grounding
- User flagged AuxiliaryInstructions was too rigid (2-4 paragraph constraint + 4-part structure) — loosened to allow natural pacing
- User flagged quantifier issues: (1) parenthetical inside XML block, (2) closing question only covered NPCs not movement — both fixed

**Research Findings**:
- Normal prompt: 8 XML-tagged layers in `prompt.rs` (SystemPrompt, GameState, Npcs, NpcsInRoom, PlayerCharacter, WorldLore, ConversationHistory, PlayerInput, AuxiliaryInstructions)
- Quantifier prompt: zero XML tags, flat text with hyphens/colons in `quantifier.rs`
- Quantifier tests check for content like "scene quantifier", "npcs_in_room", NPC names, room names
- Token budget: MAX_CONTEXT_TOKENS=8192, system layer currently ~400 chars — additions will add ~600 chars, still well within budget

### Metis Review
**Identified Gaps** (addressed):
- Token overhead from XML tags in quantifier: ~200 chars added, negligible vs 4000 char budget
- Test assertions checking for old flat-text strings in quantifier: will need updates
- No game state stores prompts — prompts are built at runtime, so no migration needed
- Scope locked to prompt.rs + quantifier.rs only, no engine logic changes

---

## Work Objectives

### Core Objective
Improve narrative quality and world-state consistency by adding Silly Tavern behavioral rules to the normal prompt, and improve quantifier prompt clarity with XML structure.

### Concrete Deliverables
- `src/narrative/prompt.rs`: Updated system layer + PHI layer
- `src/narrative/quantifier.rs`: XML-structured system + user prompts

### Definition of Done
- [ ] `cargo test --test flow_mock_tests` passes (0 failures)
- [ ] `cargo test` passes (0 failures)
- [ ] `cargo clippy` passes (0 warnings)

### Must Have
- Input validation rules in system prompt
- Knowledge boundary rules (NPCs only know what they've witnessed)
- World dynamics rules (time moves, routines continue)
- Causality rule (physical prerequisites)
- XML structure in quantifier prompts
- Looser AuxiliaryInstructions (no paragraph count or rigid structure)

### Must NOT Have (Guardrails)
- No changes to engine logic, parsing, or state mutation
- No changes to token budget constants
- No changes to the quantifier JSON response format or parsing logic
- No changes to prompt sanitization logic
- No changes to the 8-layer architecture or layer ordering

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: Tests-after
- **Framework**: Rust built-in (`cargo test`)
- **Agent-Executed QA**: ALWAYS (mandatory for all tasks)

### QA Policy
Every task includes agent-executed QA scenarios. Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately - 2 parallel tasks):
├── Task 1: Update normal prompt (prompt.rs) [quick]
└── Task 2: Update quantifier prompt (quantifier.rs) [quick]

Wave 2 (After Wave 1 - fix tests):
└── Task 3: Update test assertions [quick]

Wave FINAL (After ALL tasks):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay
```

### Dependency Matrix
- **1-2**: - (parallel, independent files)
- **3**: 1, 2 (tests depend on both prompts being updated)
- **FINAL**: 3 (verification depends on all changes)

### Agent Dispatch Summary
- **1**: **2** - T1 → `quick`, T2 → `quick`
- **2**: **1** - T3 → `quick`
- **FINAL**: **4** - F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 1. Update normal prompt — system layer + PHI layer (`prompt.rs`)

  **What to do**:
  - Replace `render_system_layer()` content with the approved Silly Tavern-enhanced version:
    - Add Input Validation section (treat input as attempted, not absolute)
    - Add State Tracking section (physical, knowledge, relationship state)
    - Add Knowledge Boundaries (NPCs only know what witnessed/told)
    - Add World Dynamics (time moves, routines continue, environmental shifts)
    - Add Dialogue Grounding rules
    - Add Causality rule (physical prerequisites)
    - Keep existing: Core Role, Writing Style, Never rules, Game Rules injection
  - Replace `render_phi_layer()` content with the loosened version:
    - Remove "2-4 paragraphs" constraint
    - Remove rigid 1-2-3-4 structure
    - Keep "don't ask player what to do" rule
    - Add "match pacing to what's happening" guidance

  **Must NOT do**:
  - Don't change layer ordering or add/remove layers
  - Don't change token budget constants
  - Don't modify GameState, NpcCards, Player, WorldLore, History, User layers
  - Don't change sanitization logic

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2)
  - **Parallel Group**: Wave 1 (with Task 2)
  - **Blocks**: Task 3
  - **Blocked By**: None

  **References**:
  - `src/narrative/prompt.rs:257-286` - Current `render_system_layer()` to replace
  - `src/narrative/prompt.rs:447-464` - Current `render_phi_layer()` to replace
  - Final approved system prompt text — see conversation above
  - Final approved PHI text — see conversation above

  **Acceptance Criteria**:
  - [ ] `render_system_layer()` contains "Input Validation" section
  - [ ] `render_system_layer()` contains "State Tracking" section
  - [ ] `render_system_layer()` contains "World Dynamics" section
  - [ ] `render_system_layer()` contains "Causality" rule
  - [ ] `render_phi_layer()` does NOT contain "2-4 paragraphs"
  - [ ] `render_phi_layer()` does NOT contain numbered structure (1., 2., 3., 4.)
  - [ ] `cargo build` succeeds

  **QA Scenarios**:
  ```
  Scenario: System layer contains all new sections
    Tool: Bash
    Preconditions: prompt.rs modified
    Steps:
      1. cd D:/John/DevContainer/mrn-general/chronicler_engine
      2. grep -c "Input Validation" src/narrative/prompt.rs
      3. grep -c "State Tracking" src/narrative/prompt.rs
      4. grep -c "World Dynamics" src/narrative/prompt.rs
      5. grep -c "Causality" src/narrative/prompt.rs
    Expected Result: Each grep returns count >= 1
    Evidence: .sisyphus/evidence/task-1-system-sections.txt

  Scenario: PHI layer is loosened
    Tool: Bash
    Preconditions: prompt.rs modified
    Steps:
      1. cd D:/John/DevContainer/mrn-general/chronicler_engine
      2. grep -c "2-4 paragraphs" src/narrative/prompt.rs
      3. grep "Immediate consequence" src/narrative/prompt.rs
    Expected Result: grep "2-4 paragraphs" returns 0 matches; "Immediate consequence" returns 0 matches (old structure removed)
    Evidence: .sisyphus/evidence/task-1-phi-loosened.txt
  ```

  **Commit**: YES (groups with 2)
  - Message: `refactor(prompt): add Silly Tavern behavioral rules + loosen PHI`
  - Files: `src/narrative/prompt.rs`

- [x] 2. Update quantifier prompt — XML structure (`quantifier.rs`)

  **What to do**:
  - Replace `build_system_prompt()` with XML-structured version:
    - Wrap task description in `<QuantifierTask>` tags
    - Move rules inside `<QuantifierTask>`
    - Wrap NPC list in `<AvailableNpcIds>` with `<Npc id="" name="" />` format
    - Wrap room list in `<AvailableRooms>` with `<Room id="" name="" />` format
  - Replace `build_user_prompt()` with XML-structured version:
    - Wrap room info in `<CurrentRoom>` with `<Name>`, `<Description>`, optional `<Navigation>`
    - Wrap previous NPCs in `<PreviousRoomNpcs>` (no parenthetical)
    - Wrap configured NPCs in `<RoomConfiguredNpcs>`
    - Wrap history in `<RecentHistory>` with `<Entry sender="">` tags
    - Wrap player action in `<PlayerAction>`
    - Replace closing question with `<Query>` block covering both NPC presence AND movement

  **Must NOT do**:
  - Don't change the JSON response format expected from the LLM
  - Don't change `parse_quantifier_response` or `parse_quantifier_response_with_movement`
  - Don't change `QuantifierConfidence` enum or `QuantifierResult` struct
  - Don't change `QuantifierBackend::quantify_room` logic

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 1)
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: Task 3
  - **Blocked By**: None

  **References**:
  - `src/narrative/quantifier.rs:91-119` - Current `build_system_prompt()` to replace
  - `src/narrative/quantifier.rs:122-168` - Current `build_user_prompt()` to replace
  - Final approved quantifier system prompt — see conversation above
  - Final approved quantifier user prompt — see conversation above

  **Acceptance Criteria**:
  - [ ] `build_system_prompt()` output contains `<QuantifierTask>`, `<AvailableNpcIds>`, `<AvailableRooms>`
  - [ ] `build_user_prompt()` output contains `<CurrentRoom>`, `<PreviousRoomNpcs>`, `<RoomConfiguredNpcs>`, `<RecentHistory>`, `<PlayerAction>`, `<Query>`
  - [ ] No flat-text hyphen lists in system prompt (all structured with XML)
  - [ ] No parenthetical "(These NPCs may have followed the player.)" in user prompt
  - [ ] `<Query>` block mentions both NPC presence AND movement
  - [ ] `cargo build` succeeds

  **QA Scenarios**:
  ```
  Scenario: System prompt has XML structure
    Tool: Bash
    Preconditions: quantifier.rs modified
    Steps:
      1. cd D:/John/DevContainer/mrn-general/chronicler_engine
      2. grep -c "<QuantifierTask>" src/narrative/quantifier.rs
      3. grep -c "<AvailableNpcIds>" src/narrative/quantifier.rs
      4. grep -c "<AvailableRooms>" src/narrative/quantifier.rs
    Expected Result: Each grep returns count >= 1
    Evidence: .sisyphus/evidence/task-2-system-xml.txt

  Scenario: User prompt has XML structure
    Tool: Bash
    Preconditions: quantifier.rs modified
    Steps:
      1. cd D:/John/DevContainer/mrn-general/chronicler_engine
      2. grep -c "<CurrentRoom>" src/narrative/quantifier.rs
      3. grep -c "<PreviousRoomNpcs>" src/narrative/quantifier.rs
      4. grep -c "<Query>" src/narrative/quantifier.rs
      5. grep -c "may have followed" src/narrative/quantifier.rs
    Expected Result: XML tags present (count >= 1); "may have followed" returns 0 (removed from user prompt)
    Evidence: .sisyphus/evidence/task-2-user-xml.txt

  Scenario: Quantifier tests still pass
    Tool: Bash
    Preconditions: quantifier.rs modified
    Steps:
      1. cd D:/John/DevContainer/mrn-general/chronicler_engine
      2. cargo test quantifier
    Expected Result: All quantifier tests pass
    Evidence: .sisyphus/evidence/task-2-quantifier-tests.txt
  ```

  **Commit**: YES (groups with 1)
  - Message: `refactor(prompt): add Silly Tavern behavioral rules + loosen PHI`
  - Files: `src/narrative/quantifier.rs`

- [x] 3. Update test assertions for new prompt content

  **What to do**:
  - Run `cargo test` to identify failing tests
  - Update any assertions in `prompt.rs` tests that check for old system prompt text
  - Update any assertions in `quantifier.rs` tests that check for old flat-text format
  - Tests should verify new XML tags and new behavioral rules exist
  - Do NOT weaken test coverage — only update expected strings

  **Must NOT do**:
  - Don't remove any tests
  - Don't change test logic, only expected string values
  - Don't add new tests beyond what's needed for the changes

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2 (sequential after Tasks 1+2)
  - **Blocks**: Final verification
  - **Blocked By**: Task 1, Task 2

  **References**:
  - `src/narrative/prompt.rs:467-995` - Test module with assertions to update
  - `src/narrative/quantifier.rs:498-907` - Test module with assertions to update

  **Acceptance Criteria**:
  - [ ] `cargo test` passes (0 failures)
  - [ ] `cargo test --test flow_mock_tests` passes (0 failures)

  **QA Scenarios**:
  ```
  Scenario: All tests pass
    Tool: Bash
    Preconditions: Tests updated
    Steps:
      1. cd D:/John/DevContainer/mrn-general/chronicler_engine
      2. cargo test
    Expected Result: All tests pass, 0 failures
    Evidence: .sisyphus/evidence/task-3-all-tests.txt

  Scenario: Mock flow tests pass
    Tool: Bash
    Preconditions: Tests updated
    Steps:
      1. cd D:/John/DevContainer/mrn-general/chronicler_engine
      2. cargo test --test flow_mock_tests
    Expected Result: All mock flow tests pass
    Evidence: .sisyphus/evidence/task-3-mock-tests.txt
  ```

  **Commit**: YES
  - Message: `test(prompt): update assertions for new prompt content`
  - Files: `src/narrative/prompt.rs`, `src/narrative/quantifier.rs`
  - Pre-commit: `cargo test`

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
>
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, grep for content). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy` + `cargo test`. Review all changed files for: code quality, no regressions, XML well-formedness in prompt strings. Check AI slop: excessive comments, over-abstraction.
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [x] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

- **1+2**: `refactor(prompt): add Silly Tavern behavioral rules + XML quantifier structure` - src/narrative/prompt.rs, src/narrative/quantifier.rs
- **3**: `test(prompt): update assertions for new prompt content` - src/narrative/prompt.rs, src/narrative/quantifier.rs, cargo test

---

## Success Criteria

### Verification Commands
```bash
cargo build          # Expected: success
cargo test           # Expected: 0 failures
cargo clippy         # Expected: 0 warnings
```

### Final Checklist
- [ ] Input validation rules present in system prompt
- [ ] State tracking rules present in system prompt
- [ ] World dynamics rules present in system prompt
- [ ] Causality rule present in system prompt
- [ ] Knowledge boundary rules present in system prompt
- [ ] PHI layer loosened (no paragraph count, no rigid structure)
- [ ] Quantifier system prompt uses XML tags
- [ ] Quantifier user prompt uses XML tags
- [ ] Quantifier Query covers both NPC presence AND movement
- [ ] All tests pass
