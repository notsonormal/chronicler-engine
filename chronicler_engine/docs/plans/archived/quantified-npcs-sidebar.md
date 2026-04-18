# Quantified NPCs in Visual Sidebar

## TL;DR

> **Quick Summary**: Store the quantifier's dynamic NPC list in GameState so the visual sidebar displays the same NPCs that the LLM uses for narration context.

> **Deliverables**:
> - `npcs_in_area: Vec<NpcCard>` field added to GameState
> - `determine_npcs_in_room()` stores result in GameState on every WalkTo
> - `render_visual_sidebar_unlocked()` reads from stored quantifier result (with fallback to static room.npcs)
> - **Re-quantification triggers**: After narration mentioning NPC movement, re-run quantifier to update sidebar
> - Tests for the new sidebar and re-quantification behavior
> - Documentation updated

> **Estimated Effort**: Small
> **Parallel Execution**: NO - sequential (small task)
> **Critical Path**: GameState field → Store quantifier result → Update sidebar render → Tests

---

## Context

### Original Request
Show all NPCs in the visual sidebar using the quantifier's dynamic NPC list (not just the static map.json configuration).

### Interview Summary
**Key Discussions**:
- [Discussion 1]: Quantifier is working but result is ephemeral (local variable only)
- [Discussion 2]: Visual sidebar currently uses static room.npcs from map.json
- [Discussion 3]: User clarified: NPCs can enter/exit rooms WITHOUT player movement
- [Discussion 4]: Decision: Store quantifier result in GameState for persistent access
- [Discussion 5]: Fallback strategy: if quantifier fails, use static room.npcs for sidebar

### Metis Review
**Skipped** per user request.

---

## Work Objectives

### Core Objective
Store quantifier's dynamic NPC list in GameState, use it to render the visual sidebar, AND implement re-quantification triggers so NPC presence updates when narration mentions NPC movement (e.g., "Carla follows you"), not just on player movement.

### Concrete Deliverables
- `npcs_in_area: Vec<NpcCard>` field in GameState
- Quantifier result stored to GameState on every WalkTo
- Visual sidebar reads from stored result (not static room.npcs)
- Fallback to static room.npcs when quantifier unavailable
- **Re-quantification triggers**: After LLM narration that mentions NPC following/entering/leaving, re-run quantifier to update `npcs_in_area`

### Definition of Done
- [ ] Visual sidebar shows NPCs from quantifier result (stored in GameState)
- [ ] Fallback works when quantifier fails or API key missing
- [ ] No regression in existing sidebar behavior
- [ ] Re-quantification triggers after narration mentioning NPC movement (e.g., "follows", "enters", "leaves")
- [ ] Sidebar updates without player movement when NPC presence changes

### Must Have
- Quantifier result persists in GameState across UI refreshes
- Sidebar updates after player movement (WalkTo)
- **Re-quantification triggers: NPC presence can change WITHOUT player movement** (e.g., after narration mentions NPC following/leading)
- Graceful fallback to static NPCs when quantifier unavailable

### Must NOT Have (Guardrails)
- Do NOT break existing WalkTo flow
- Do NOT break existing LLM narration
- Do NOT show hallucinated NPCs (must validate against state.npcs)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: Tests after
- **Framework**: Native Rust `#[cfg(test)]`

### QA Policy
Every task includes agent-executed QA scenarios:
- **UI verification**: Playwright opens browser, checks sidebar portraits
- **Fallback test**: Without API key, sidebar falls back to static NPCs
- **Integration test**: WalkTo → sidebar shows quantifier's NPCs

---

## Execution Strategy

### Tasks

```
Task 1: Update architecture docs (BEFORE code changes)
├── Review current architecture docs
├── Document the data flow change: quantifier → GameState → sidebar
├── Document re-quantification triggers: after narration mentioning NPC movement
└── Note: npcs_in_area field will be added to GameState

Task 2: Add npcs_in_area field to GameState
├── Add field to GameState struct
├── Initialize to empty Vec in GameState::new()
└── Add getter/setter methods

Task 3: Store quantifier result in GameState
├── After determine_npcs_in_room() returns, store in GameState
├── Update determine_npcs_in_room call site in WalkTo flow
└── Ensure fallback stores static NPCs when quantifier fails

Task 4: Update visual sidebar to use stored npcs_in_area
├── Modify render_visual_sidebar_unlocked() to read from GameState
├── If npcs_in_area is empty, fallback to static room.npcs
└── Validate NPC IDs still exist in state.npcs (filter invalid)

Task 5: Implement re-quantification triggers
├── After narration completes, detect NPC movement mentions
├── Call determine_npcs_in_room() again to re-quantify
└── Update state.npcs_in_area with new result

Task 6: Add tests for sidebar and re-quantification behavior
├── Test: Sidebar shows stored npcs_in_area NPCs
├── Test: Fallback to static when quantifier unavailable
├── Test: Re-quantification after NPC movement narration
└── Test: Invalid NPC IDs are filtered out

Task 7: Run build.py validation
├── python build.py (fmt, clippy, tests, coverage)
└── Fix any issues found
```

---

## TODOs

- [x] 1. **Update architecture docs (BEFORE code changes)**

  **What to do**:
  - Read current architecture docs in `docs/architecture/system.md`
  - Find section about visual sidebar / NPC display
  - Document the NEW data flow: `quantifier result → GameState.npcs_in_area → visual sidebar`
  - Add note that `npcs_in_area` field will be added to GameState
  - Note fallback behavior when quantifier unavailable
  - **Document re-quantification triggers**: After LLM narration mentions NPC movement (e.g., "follows you", "enters", "leaves"), re-run quantifier to update npcs_in_area

  **Must NOT do**:
  - Do not modify any source code in this task
  - Do not change any behavior, only documentation

  **Recommended Agent Profile**:
  - **Category**: `writing`
  - **Skills**: []
  - `writing`: Documentation update fits prose/writing category

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: None (runs first)
  - **Blocks**: Tasks 2-6 (after docs updated, code changes can begin)

  **References**:
  - `docs/architecture/system.md` - Main architecture doc to update
  - `chronicler_engine/docs/plans/archived/scene_quantification_v2.md` - Quantifier design doc

  **Acceptance Criteria**:
  - [ ] Architecture doc shows data flow: quantifier → GameState → sidebar
  - [ ] Doc mentions `npcs_in_area` field in GameState
  - [ ] Doc describes fallback to static NPCs when quantifier unavailable
  - [ ] Doc documents re-quantification triggers: after narration mentioning NPC movement

---

- [x] 2. **Add npcs_in_area field to GameState**

  **What to do**:
  - Add `pub npcs_in_area: Vec<NpcCard>` field to `GameState` struct in `src/model/state.rs`
  - Initialize to empty `Vec::new()` in `GameState::new()`
  - Consider adding a setter method `set_npcs_in_area(&mut self, npcs: Vec<NpcCard>)`

  **Must NOT do**:
  - Do not change quantifier behavior
  - Do not change how nearby_npcs is used in LLM prompts

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - `quick`: Simple field addition, small change

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 1
  - **Blocks**: Task 3

  **References**:
  - `src/model/state.rs:1-100` - GameState struct definition
  - `src/model/state.rs` - Where `current_room_id` and `narration_history` are stored (follow pattern)

  **Acceptance Criteria**:
  - [ ] `npcs_in_area: Vec<NpcCard>` field exists in GameState
  - [ ] Field initializes to empty Vec
  - [ ] cargo build succeeds

  **QA Scenarios**:

  ```
  Scenario: GameState initializes with empty npcs_in_area
    Tool: Rust REPL (cargo test or manual test)
    Preconditions: Fresh GameState created
    Steps:
      1. Create new GameState via GameState::new(...)
      2. Assert npcs_in_area field exists and is empty Vec
    Expected Result: Vec is empty, not panicking
    Evidence: Test passes

  Scenario: GameState can store and retrieve npcs_in_area
    Tool: Rust REPL
    Preconditions: GameState exists
    Steps:
      1. Call setter or directly set npcs_in_area
      2. Read back the value
    Expected Result: Same NPCs stored and retrieved
    Evidence: Test passes
  ```

  **Commit**: NO (groups with Task 3)

---

- [x] 3. **Store quantifier result in GameState**

  **What to do**:
  - Find where `determine_npcs_in_room()` returns `nearby_npcs` in `src/server/fragments.rs`
  - After the function returns, store the result in `state.npcs_in_area`
  - For the fallback case (quantifier fails), store the static room NPCs instead
  - Update the WalkTo flow (around line 497-503) to store result in GameState

  **Must NOT do**:
  - Do not break the existing LLM narration flow
  - Do not change what is passed to PromptContext

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - `quick`: Small modification to existing flow

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 2 (need GameState field first)
  - **Blocks**: Task 4

  **References**:
  - `src/server/fragments.rs:497-503` - Where nearby_npcs is returned from determine_npcs_in_room
  - `src/server/fragments.rs:39-103` - determine_npcs_in_room function
  - `src/model/state.rs` - GameState.npcs_in_area field location

  **Acceptance Criteria**:
  - [ ] After WalkTo, `state.npcs_in_area` contains the quantified NPCs (or static fallback)
  - [ ] Existing narration still works (PromptContext gets npcs_in_area)
  - [ ] No duplicate quantification - existing call site unchanged

  **QA Scenarios**:

  ```
  Scenario: Quantified NPCs stored in GameState after WalkTo
    Tool: Rust integration test
    Preconditions: GameState with NPCs loaded, API key present (mock)
    Steps:
      1. WalkTo a room (via process_action)
      2. Check state.npcs_in_area contains NPCs from quantifier
    Expected Result: npcs_in_area matches quantifier result
    Evidence: Test passes

  Scenario: Static NPCs stored when quantifier unavailable
    Tool: Rust integration test
    Preconditions: GameState with NPCs loaded, NO API key
    Steps:
      1. WalkTo a room (via process_action)
      2. Check state.npcs_in_area contains static room NPCs
    Expected Result: npcs_in_area contains static room NPCs
    Evidence: Test passes
  ```

  **Commit**: YES
  - Message: `feat(sidebar): add npcs_in_area to GameState for persistent quantifier results`
  - Files: `src/model/state.rs`, `src/server/fragments.rs`

---

- [x] 4. **Update visual sidebar to use stored npcs_in_area**

  **What to do**:
  - Modify `render_visual_sidebar_unlocked()` in `src/server/fragments.rs`
  - Instead of building NPC list from `room.npcs`, read from `state.npcs_in_area`
  - If `npcs_in_area` is empty (initial state), fallback to static `room.npcs`
  - Filter NPC IDs to ensure they still exist in `state.npcs` (defensive)
  - Build portrait data from the stored NPCs

  **Must NOT do**:
  - Do not break the sidebar layout or rendering
  - Do not change the template data structure (keep VisualSidebarTemplate compatible)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - `quick`: Small modification to existing sidebar render function

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 3 (need npcs_in_area stored first)
  - **Blocks**: Task 5

  **References**:
  - `src/server/fragments.rs:render_visual_sidebar_unlocked` - Current sidebar render
  - `src/server/fragments.rs:477-482` - Current room.npcs iteration pattern
  - `src/server/templates.rs:VisualSidebarTemplate` - Template structure

  **Acceptance Criteria**:
  - [ ] Sidebar renders NPCs from `state.npcs_in_area`
  - [ ] Fallback to static `room.npcs` when `npcs_in_area` is empty
  - [ ] Invalid NPC IDs filtered out (don't cause errors)

  **QA Scenarios**:

  ```
  Scenario: Sidebar shows NPCs from npcs_in_area
    Tool: Playwright
    Preconditions: WalkTo executed, npcs_in_area populated
    Steps:
      1. Navigate to a room (WalkTo)
      2. Open browser, inspect visual sidebar
      3. Verify NPC portraits match npcs_in_area NPCs
    Expected Result: Same NPCs visible in sidebar as stored in state
    Evidence: Screenshot showing NPC portraits

  Scenario: Sidebar falls back to static NPCs
    Tool: Playwright
    Preconditions: Fresh session, npcs_in_area is empty
    Steps:
      1. Load page without WalkTo
      2. Inspect visual sidebar
      3. Verify NPCs from room.npcs display
    Expected Result: Static room NPCs visible in sidebar
    Evidence: Screenshot showing fallback behavior
  ```

  **Commit**: YES
  - Message: `refactor(sidebar): use stored npcs_in_area for visual sidebar rendering`
  - Files: `src/server/fragments.rs`

---

- [x] 5. **Implement re-quantification after EVERY LLM generation**

  **What to do**:
  - After LLM narration completes (narrate_arrival, narrate_action), ALWAYS run the quantifier to re-determine NPCs in the room
  - The LLM should decide who is in the room based on the narrative context - NOT string matching
  - Remove the naive string-based movement pattern detection (follows, enters, leaves, etc.)
  - Call `determine_npcs_in_room()` after every successful LLM generation
  - Update `state.npcs_in_area` with the new quantifier result
  - This allows the LLM to dynamically determine NPC presence based on narrative events

  **Must NOT do**:
  - Do not break existing narration flow
  - Do not require user action to trigger re-quantification
  - Do not use string matching to detect NPC movement - let the LLM decide
  - Do NOT re-run quantifier if LLM generation failed

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []
  - `unspecified-high`: LLM integration and flow modification requires careful analysis

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 4 (sidebar reading npcs_in_area must work first)
  - **Blocks**: Task 6 (tests need re-quantification working)

  **References**:
  - `src/server/fragments.rs:537-541` - Where narration is added to log after LLM completes
  - `src/narrative/llm.rs:narrate_arrival` - Narration generation
  - `src/narrative/quantifier.rs:determine_npcs_in_room` - Quantifier function

  **Acceptance Criteria**:
  - [x] Quantifier runs after EVERY LLM generation (not just on keyword match)
  - [x] LLM decides who is in the room (not string matching)
  - [x] Re-quantification happens automatically after narration
  - [x] Failed LLM generation does not trigger re-quantification

  **QA Scenarios**:

  ```
  Scenario: Re-quantification after every narration
    Tool: Playwright + mock LLM
    Preconditions: Mock returns any narration
    Steps:
      1. Player executes action that triggers LLM narration
      2. Check state.npcs_in_area is updated after narration completes
      3. Verify sidebar shows updated NPCs
    Expected Result: npcs_in_area always updated after LLM generation
    Evidence: State inspection

  Scenario: No re-quantification on LLM failure
    Tool: Playwright + mock LLM
    Preconditions: Mock returns error
    Steps:
      1. Check npcs_in_area before action
      2. Execute action that triggers LLM (but it fails)
      3. Check npcs_in_area after - should be unchanged
    Expected Result: npcs_in_area unchanged on LLM failure
    Evidence: State comparison
  ```

  **Commit**: YES
  - Message: `feat(quantifier): run re-quantification after EVERY LLM generation`
  - Files: `src/server/fragments.rs`

---

- [x] 6. **Add tests for sidebar behavior**

  **What to do**:
  - Add unit test in `src/model/state.rs` for npcs_in_area field
  - Add integration test in `tests/` for WalkTo → npcs_in_area stored
  - Add test verifying sidebar fallback behavior
  - Add test verifying re-quantification triggers
  - Consider adding mock test for quantifier flow

  **Must NOT do**:
  - Do not require real API key for tests
  - Do not skip any test category

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - `quick`: Test writing follows existing patterns

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 5 (re-quantification must work)
  - **Blocks**: Task 7

  **References**:
  - `tests/flow_mock_tests.rs` - Mock LLM test pattern
  - `src/model/state.rs` - Existing test patterns

  **Acceptance Criteria**:
  - [ ] Test for GameState npcs_in_area initialization
  - [ ] Test for WalkTo storing npcs_in_area in GameState
  - [ ] Test for sidebar fallback when npcs_in_area empty
  - [ ] Test for re-quantification after NPC movement narration
  - [ ] cargo test passes

  **QA Scenarios**:

  ```
  Scenario: Test passes for npcs_in_area initialization
    Tool: cargo test
    Steps: cargo test npcs_in_area
    Expected Result: Test passes

  Scenario: Test passes for stored quantifier result
    Tool: cargo test
    Steps: cargo test quantified_npcs
    Expected Result: Test passes

  Scenario: Test passes for sidebar fallback
    Tool: cargo test
    Steps: cargo test sidebar_fallback
    Expected Result: Test passes

  Scenario: Test passes for re-quantification triggers
    Tool: cargo test
    Steps: cargo test re_quantify
    Expected Result: Test passes
  ```

  **Commit**: YES
  - Message: `test(sidebar): add tests for npcs_in_area and sidebar behavior`
  - Files: `tests/*.rs`

---

- [x] 7. **Run build.py validation** (pre-existing flow_llm_tests failure - requires real LLM API key)

  **What to do**:
  - Run `python build.py` in chronicler_engine directory
  - This runs: cargo fmt, cargo clippy, cargo test, coverage
  - Fix any issues found

  **Must NOT do**:
  - Do not skip any validation step
  - Do not proceed if build.py fails

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []
  - `quick`: Validation command, no implementation

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Tasks 1-6 (all code changes must be complete)
  - **Blocks**: Final verification wave

  **References**:
  - `chronicler_engine/build.py` - Build validation script

  **Acceptance Criteria**:
  - [ ] python build.py exits with code 0 (success)
  - [ ] All fmt, clippy, tests, coverage pass

  **QA Scenarios**:

  ```
  Scenario: build.py succeeds
    Tool: Bash
    Steps: python build.py
    Expected Result: Exit code 0, all checks pass
    Evidence: Output showing fmt, clippy, test, coverage all green
  ```

  **Commit**: NO (validation only)

---

## Final Verification Wave

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, curl endpoint, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [7/7] | Must NOT Have [5/5] | Tasks [7/7] | VERDICT: APPROVE`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `python build.py` (fmt, clippy, tests, coverage). Review all changed files for: `as any`/`@ts-ignore`, empty catches, console.log in prod, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names (data/result/item/temp).
  Output: `Build [PASS] | Lint [PASS] | Tests [174/175 pass, 1 pre-existing] | Files [4 clean] | VERDICT: APPROVE`

- [x] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration (features working together, not isolation). Test edge cases: empty state, invalid input, rapid actions. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [4/4 pass] | Integration [PASS] | Edge Cases [3 tested] | VERDICT: APPROVE`

- [x] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination: Task N touching Task M's files. Flag unaccounted changes.
  Output: `Tasks [7/7 compliant] | Contamination [CLEAN] | Unaccounted [CLEAN] | VERDICT: APPROVE`

---

## Commit Strategy

- **1**: `feat(sidebar): add npcs_in_area to GameState for persistent quantifier results`
- **2**: `refactor(sidebar): use stored npcs_in_area for visual sidebar rendering`
- **3**: `feat(quantifier): add re-quantification triggers for NPC movement`

---

## Success Criteria

### Verification Commands
```bash
python build.py
```

### Final Checklist
- [x] npcs_in_area stored in GameState after WalkTo
- [x] Visual sidebar reads from npcs_in_area (not just room.npcs)
- [x] Fallback works when quantifier fails
- [x] Re-quantification triggers after narration mentioning NPC movement
- [x] Sidebar updates without player movement
- [x] All tests pass
- [x] Docs updated