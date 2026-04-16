# Location Display Fix - Work Plan

## TL;DR

> **Quick Summary**: Fix two issues: (1) Auto-set location on game start, (2) Format location display as "Location - Time" inline
> 
> **Deliverables**:
> - Location entry created on first game entry using world starting_room_id
> - Location displays inline with timestamp as "Entrance Hall - 18:57" with distinct styling
> 
> **Estimated Effort**: Short
> **Parallel Execution**: NO - sequential (template + CSS are in same wave)
> **Critical Path**: Template edit → CSS edit → Verify

---

## Context

### Original Request
1. First entry when starting game doesn't have a location set - needs to auto-set to world spawn
2. When moving to new location, format "Entrance Hall" should display as "Entrance Hall - 18:57" (inline, right-aligned), visually distinct from LLM text

### Interview Summary
**Key Discussions**:
- Use starting_room_id from world.json for initial location
- Location display format: "Location - Time" inline with right-alignment, visually distinct

**Research Findings**:
- world.json has `starting_room_id: "front_gates"` at line 19
- main.rs initializes GameState with `manifest.starting_room_id.clone()` at line 229
- Template at `src/server/templates.rs:119` renders location AFTER timestamp as separate block element
- CSS in `assets/index.html` lines 80-109 styles .location-header as block with green color

---

## Work Objectives

### Core Objective
1. First game entry shows location using the starting room from world.json
2. Location entries display inline with timestamp as "Room Name - HH:MM" format

### Concrete Deliverables
- Location entry appears in narration history on first game entry (using starting_room_id)
- Location entry renders as "Entrance Hall - 18:57" inline (not stacked)

### Definition of Done
- [x] Start game → narration entry shows "front_gates - HH:MM" format
- [x] Move to new room → location displays as "Room Name - HH:MM" inline

### Must Have
- Location entry created with first scenario/narration
- Inline format "Location - Time" with visual distinction

### Must NOT Have
- NO stacked location + timestamp (user explicitly wants inline)
- NO template changes that break other entry types (dialogue, system, input)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (cargo test)
- **Automated tests**: Tests after (add verification tests)
- **Framework**: Rust built-in

### QA Policy
- Agent-Executed QA via curl/Playwright for UI verification
- Manual verification: start game and verify location display

---

## Execution Strategy

### Tasks

**Wave 0 (Docs + Tests - BEFORE implementation!)**:
1. Update docs/architecture/system.md with location tracking changes
2. Create/update unit tests for LogEntryView location detection
3. Create integration test for initial location on game start
4. Create integration test for WalkTo location entry format

**Wave 1 (Template + CSS changes)**:
5. Modify location template to render inline "Location - Time" format
6. Update CSS for inline styling with visual distinction
7. Verify location entry created on game start

---

## TODOs

- [x] 0a. **Update architecture docs**

  **What to do**:
  - Edit `docs/architecture/system.md`
  - Document: location tracking now includes initial spawn location
  - Document: location display format is "Room Name - HH:MM" inline
  
  **References**:
  - `docs/architecture/system.md` - Current architecture document

  **Acceptance Criteria**:
  - [x] Location tracking documented
  - [x] Display format documented

- [x] 0b. **Add unit tests for location detection**

  **What to do**:
  - Add tests to `src/server/templates.rs` inline tests (near line 369)
  - Test: location entry detection (sender + empty text)
  - Test: inline "Location - Time" format

  **References**:
  - `src/server/templates.rs:369-385` - Existing tests

  **Acceptance Criteria**:
  - [x] Location detection test passes
  - [x] Inline format test passes

- [x] 0c. **Add integration test for initial location**

  **What to do**:
  - Add test to `tests/flow_mock_tests.rs` or create new test file
  - Test: first game entry shows location from starting_room_id

  **References**:
  - `tests/flow_mock_tests.rs` - Existing flow tests

  **Acceptance Criteria**:
  - [x] Initial location test passes

- [x] 0d. **Add integration test for WalkTo location format**

  **What to do**:
  - Add test to verify WalkTo creates location entry
  - Test: location displays as "Room - Time" format

  **References**:
  - `tests/flow_mock_tests.rs` - Existing WalkTo tests

  **Acceptance Criteria**:
  - [x] WalkTo location format test passes

- [x] 1. **Modify location template for inline display**

  **What to do**:
  - Edit `src/server/templates.rs` line 119
  - Change order: location BEFORE timestamp
  - Format: "Location - Time" inline (location header + " - " + timestamp)
  
  **Must NOT do**:
  - Don't break non-location entry rendering

  **References**:
  - `src/server/templates.rs:85-107` - LogEntryView struct and conversion
  - `src/server/templates.rs:119` - Current template HTML
  - Line 101 shows timestamp format: `timestamp: entry.timestamp.format("%H:%M").to_string()`

  **Acceptance Criteria**:
  - [x] Template modified for inline format
  - [x] Non-location entries still render correctly

  **QA Scenarios**:
  ```
  Scenario: Location entry renders inline
    Tool: Playwright (dev-browser skill)
    Steps:
      1. Start game with world redmist_estate
      2. Navigate to dashboard
      3. Inspect location entry HTML
    Expected Result: Contains "front_gates - 18:57" inline format
  ```

- [x] 2. **Update CSS for visual distinction**

  **What to do**:
  - Edit `assets/index.html` lines 80-109
  - Location header: inline-block instead of block
  - Add margin-right for spacing between location and "-"
  - Right-align location container
  - Visual distinction: green color (#4ade80), bold, slightly larger

  **References**:
  - `assets/index.html:80-100` - Current .location-header styles

  **Acceptance Criteria**:
  - [x] Location inline with timestamp
  - [x] Right-aligned container
  - [x] Distinct color (#4ade80 or similar)

  **QA Scenarios**:
  ```
  Scenario: Location visually distinct
    Tool: Playwright (dev-browser skill)
    Steps:
      1. Start game
      2. Take screenshot
      3. Verify location styling
    Expected Result: Location text is green, bold, right-aligned
  ```

- [x] 3. **Verify initial location entry created**

  **What to do**:
  - Check main.rs to ensure location entry created on game start
  - If not, add code to create location log entry with starting_room_id

  **References**:
  - `src/main.rs:224-244` - GameState initialization and scenario handling

  **Acceptance Criteria**:
  - [x] First narration entry shows location

---

## Final Verification Wave

- [x] Location format verification - F1 (unspecified-high)
- [x] Initial location verification - F2 (quick)

---

## Commit Strategy

- type(fix): location display and initial spawn

---

## Success Criteria

### Verification Commands
```bash
cargo build  # Builds without errors
```

### Final Checklist
- [x] First entry shows location "front_gates - HH:MM"
- [x] Location displays inline with timestamp
- [x] Distinct visual styling applied