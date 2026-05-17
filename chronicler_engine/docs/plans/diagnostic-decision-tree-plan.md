# Plan: Diagnostic Decision Tree as Agent Infrastructure

**Date:** 2026-05-09
**Status:** Planned
**Goal:** Make agent diagnostic protocols systematic instead of ad-hoc.

---

## Overview

Currently, when an agent encounters a test failure or runtime error, it follows an ad-hoc process:
1. Read the error message
2. Grep the codebase for relevant strings
3. Read whatever files seem related
4. Cross-reference `DEBUGGING.md` or `error_catalog.md` if the agent remembers they exist

This plan encodes the diagnostic expertise from `DEBUGGING.md`, `error_catalog.md`, and `docs/system/*.md` into a structured, traversable decision tree. The agent (human or AI) follows the tree systematically, eliminating hypotheses in a defined order.

---

## Background

**Existing diagnostic knowledge is high-quality but passive:**
- `DEBUGGING.md` (127 lines) — symptom-based playbook (Trigger Not Firing, Narration Empty, Wrong Room, Test Failure)
- `error_catalog.md` (185 lines) — structured reference for every `EngineError` variant with "First Check" and "Common Causes"
- `docs/system/*.md` (12 files, 2,000-10,000 bytes each) — domain docs for triggers, navigation, LLM processing, etc.

**The gap:** These documents require the agent to *know which symptom to look up*. If the agent misidentifies the symptom (e.g., thinks it's a trigger bug when it's actually a quantifier fallback), the investigation goes down the wrong path.

**Evidence of ad-hoc diagnosis:**
- `AGENTS.md` test failure handling section: "If you're unsure why a test failed, say so and investigate — don't invent explanations." This exists because agents *do* invent explanations.
- `DEBUGGING.md` has a "Mandatory protocol — do not skip" for test failures, implying agents skip steps.

---

## Architecture Decisions

1. **Decision tree is markdown-based and human-readable.** It lives in `docs/diagnostics/decision_tree.md` and is structured as a nested list. This makes it editable without code changes and reviewable by humans.
2. **Tree nodes link to existing docs.** We do not duplicate content from `error_catalog.md` or `docs/system/*.md`. Nodes contain links and brief routing logic.
3. **Agent skill provides the traversal algorithm.** The skill (`chronicler-diagnose`) reads the tree and follows it. The tree itself is data.
4. **Machine-readable metadata augments errors.** Where practical, we add `#[diagnostic(...)]` attributes to error variants so the tree can route automatically from an error string.

---

## Phase 1: Investigation — Codify Implicit Diagnostic Knowledge

### Task 1.1: Interview Existing Docs for Decision Points
- Read every doc in `docs/system/` and `docs/diagnostics/` and extract "if X, check Y" statements.
- Also grep source code for `// TODO:`, `// FIXME:`, and `// Workaround` comments that indicate known fragile areas.
- **Deliverable:** A raw list of 50+ diagnostic decision points.

### Task 1.2: Classify Error-to-Cause Mappings
- For each `EngineError` variant, document:
  - Primary suspect file(s)
  - Most likely cause (ranked)
  - Single best log line or debug endpoint to check
  - Test file that would catch this if it were covered
- **Deliverable:** Extended error catalog table (append to `error_catalog.md` or create `error_catalog_routing.md`).

### Task 1.3: Map Test Failure Patterns
- Review the last 20 test-related commits (`git log --oneline --since="2026-04-01" -- tests/`).
- Classify each fix by: root cause category, file changed, time to fix (from commit message tone), and whether a decision tree would have caught it faster.
- **Deliverable:** Table of historical test failures with "would decision tree have helped?" column.

---

## Phase 2: Implementation — Decision Tree

### Task 2.1: Write Master Decision Tree
Create `docs/diagnostics/decision_tree.md` with this structure:

```markdown
# Diagnostic Decision Tree

## Root: Something is wrong

### Branch: Test failure
1. Quote the verbatim failure message. STOP if you cannot.
2. Identify the test file. If it is `components.rs`, determine subsystem from assertion context.
3. Read the test code. Do not form a hypothesis before this step.
4. Route to subsystem:
   - `trigger/` tests → [Trigger branch](#branch-trigger)
   - `narrative/` tests → [Narrative branch](#branch-narrative)
   - `server/` tests → [Server branch](#branch-server)
   - `engine/` tests → [Engine branch](#branch-engine)

### Branch: Runtime error (user-visible)
1. Get exact `EngineError` variant from logs or UI.
2. Route to variant in `error_catalog.md`.
3. Follow "First Check" instruction.
4. If "First Check" is inconclusive, return to this tree for secondary branches.

### Branch: Trigger
1. Check `room_id` match → `docs/system/triggers.md section: Scope`
2. Check `times_met` counter → `GET /debug/state` or test state builder
3. Check `trigger_fired` flag → non-repeatable triggers
4. Check NPC presence in `state.npcs` → quantifier result or static NPCs
5. Check quantifier confidence → logs for `[Quantifier] Low confidence`

... (and so on for each subsystem)
```

- **File:** `docs/diagnostics/decision_tree.md` (new)
- **Acceptance criteria:**
  - [ ] Tree covers every `EngineError` variant
  - [ ] Tree covers every subsystem documented in `docs/system/`
  - [ ] Every leaf node either resolves the issue or links to a specific file + line to inspect
  - [ ] Tree is ≤500 lines (if it grows larger, split into `decision_tree_*.md`)

### Task 2.2: Add Diagnostic Metadata to Errors
Add a `#[diagnostic(...)]` attribute macro (or doc comment convention) to `EngineError` variants:

```rust
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Room not found: {0}")]
    #[diagnostic(
        check_first = "state.movement.current_room_id",
        likely_causes = ["dynamic_room_creation", "room_id_typo"],
        doc = "docs/system/navigation.md",
        test_file = "tests/engine/logic_tests.rs"
    )]
    RoomNotFound(String),
    // ...
}
```

If a proc macro is too heavy, use a doc-comment convention:
```rust
/// [DIAGNOSTIC: check_first="state.movement.current_room_id", doc="docs/system/navigation.md"]
```

- **Files:**
  - `src/error.rs`
  - `src/test_support/diagnostic_macros.rs` (new, if proc macro)
- **Acceptance criteria:**
  - [ ] Every `EngineError` variant has diagnostic metadata
  - [ ] Metadata is parseable by a simple script (e.g., `scripts/parse_diagnostics.py`)
  - [ ] No runtime overhead (metadata is compile-time only)

### Task 2.3: Build Diagnostic Parser Script
Create `scripts/parse_diagnostics.py` that:
- Reads `src/error.rs`
- Extracts `#[diagnostic(...)]` or doc-comment metadata
- Generates a JSON or markdown lookup table: `error_variant → {check_first, likely_causes, doc, test_file}`
- Validates that every `doc` link points to an existing file

- **File:** `scripts/parse_diagnostics.py` (new)
- **Acceptance criteria:**
  - [ ] Script runs in <1 second
  - [ ] Output is consumed by the agent skill
  - [ ] CI fails if a diagnostic link points to a missing file

---

## Phase 3: Implementation — Agent Skill

### Task 3.1: Create `chronicler-diagnose` Skill
Create `.kimi/skills/chronicler-diagnose/SKILL.md` and `.opencode/skills/chronicler-diagnose/SKILL.md`:

```markdown
# Chronicler Diagnose

## When to use
When a test fails, a runtime error occurs, or behavior does not match expectations.

## Protocol
1. **Collect the signal.** Get the exact error message, test failure output, or log line. Do not paraphrase.
2. **Load the decision tree.** Read `docs/diagnostics/decision_tree.md`.
3. **Route.** Match the signal to the closest branch in the tree.
4. **Follow.** Execute each step in the branch in order. Do not skip.
5. **Record.** If a step is inconclusive, note which step and why before branching deeper.
6. **Escalate.** If you reach a leaf node without resolution, create an issue in `.scratch/issues/` with the full path through the tree.

## Rules
- NEVER guess the root cause before step 3.
- NEVER read source code before reading the test code (for test failures).
- ALWAYS quote the verbatim failure message in your first response.
```

- **Files:**
  - `.kimi/skills/chronicler-diagnose/SKILL.md`
  - `.opencode/skills/chronicler-diagnose/SKILL.md`
- **Acceptance criteria:**
  - [ ] Skill is listed in both agent config directories
  - [ ] Skill references `decision_tree.md` and `error_catalog.md`
  - [ ] Skill includes explicit "do not skip" language matching `DEBUGGING.md`

### Task 3.2: Update Existing Skills to Reference Diagnosis
Update `chronicler-dev-workflow`, `test-police`, and `chronicler-ui-investigator` skills to invoke `chronicler-diagnose` on failure instead of ad-hoc investigation.

- **Files:**
  - `.kimi/skills/chronicler-dev-workflow/SKILL.md`
  - `.kimi/skills/test-police/SKILL.md`
  - `.kimi/skills/chronicler-ui-investigator/SKILL.md`
- **Acceptance criteria:**
  - [ ] Each skill has a "On failure, diagnose first" section
  - [ ] No skill advises grepping for strings before reading the decision tree

---

## Phase 4: Verification

### Task 4.1: Blind Test with Historical Failures
Take 3-5 historical test failures (from Task 1.3) and have an agent diagnose them using only the decision tree and error metadata.
- **Acceptance criteria:**
  - [ ] Agent reaches correct diagnosis for ≥80% of historical cases
  - [ ] Agent follows steps in order without skipping

### Task 4.2: Update AGENTS.md and DEBUGGING.md
- Add a section to `AGENTS.md` referencing the decision tree as the primary diagnostic tool.
- Update `DEBUGGING.md` to link to `decision_tree.md` at the top.
- **Acceptance criteria:**
  - [ ] `AGENTS.md` mentions `docs/diagnostics/decision_tree.md`
  - [ ] `DEBUGGING.md` redirects to the tree for systematic investigation

### Task 4.3: Guardrail — Decision Tree Freshness
Add a guardrail test or script that:
- Verifies every `EngineError` variant appears in the decision tree
- Verifies every `docs/system/*.md` file is linked from at least one tree node
- **File:** `tests/guardrails.rs` or `scripts/check_decision_tree.py`
- **Acceptance criteria:**
  - [ ] Build fails if a new error variant is added without tree coverage
  - [ ] Build fails if a system doc is orphaned (not linked from tree)

---

## Dependencies

| Task | Depends on | Blocks |
|------|-----------|--------|
| 1.1 Interview docs | None | 2.1 |
| 1.2 Error mappings | None | 2.2, 2.1 |
| 1.3 Historical review | None | 4.1 |
| 2.1 Decision tree | 1.1, 1.2 | 3.1, 4.1, 4.2 |
| 2.2 Error metadata | 1.2 | 2.3, 4.3 |
| 2.3 Parser script | 2.2 | 3.1, 4.3 |
| 3.1 Diagnose skill | 2.1, 2.3 | 3.2, 4.1 |
| 3.2 Update skills | 3.1 | 4.2 |
| 4.1 Blind test | 1.3, 2.1, 3.1 | — |
| 4.2 Update docs | 2.1 | — |
| 4.3 Guardrail | 2.2, 2.3 | — |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| Decision tree becomes stale | Guardrail (Task 4.3) + mandatory tree update in PR template |
| Agents ignore the skill | Update all existing skills to reference it; add to `AGENTS.md` |
| Tree is too large to be useful | Split by subsystem; keep each branch ≤20 steps |
| Metadata maintenance burden | Use doc comments (zero-compile-cost) instead of proc macros |

---

## Success Criteria

1. An agent encountering a new `EngineError` variant follows the decision tree and reaches the correct diagnostic file within 3 steps.
2. Every error variant and system doc is linked from the decision tree.
3. A blind test on historical failures achieves ≥80% correct diagnosis using only the tree.
4. No new error variant or system doc can be added without updating the tree (enforced by guardrail).
