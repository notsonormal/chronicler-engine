# Agent Comprehension Investigation: Final Synthesis

**Date:** 2026-05-30  
**Scope:** All phases complete — 6-phase systematic investigation  
**Deliverables:** 6 phase documents + this synthesis

---

## Executive Summary

The Chronicler Engine has **excellent documentation** (75+ .md files) but **significant comprehension friction** in 5 critical areas. An AI agent without full context will:

1. **Misunderstand critical invariants** — mutation order breaks triggers silently
2. **Get confused by overloaded terms** — 10+ types named "Action", 5+ named "Trigger"
3. **Make incorrect changes** — engine imports from narrative (NpcEvent), model has engine types (quantifier.rs)
4. **Miss load-bearing code** — snapshots, cancel checks, phase tracking
5. **Implement wrong behavior** — no LogType::Event exists, dynamic room creation undocumented

**Risk Level:** MEDIUM-HIGH without fixes, LOW with recommended changes.

---

## Critical Findings (Must Fix)

### Finding 1: Mutation Order Invariant Not Enforced

**Severity:** CRITICAL  
**Location:** `src/engine/action_processing.rs:128-180`

The `execute_freeaction_impl` function has a load-bearing mutation order:
1. `handle_movement()` → update room
2. `add_log(narration)` → narration in history
3. `evaluate_triggers()` → BEFORE times_met increment ← CRITICAL
4. `apply_npc_events()` → times_met increment AFTER trigger eval

**Problem:** Violating this order breaks the trigger system. `TimesMet Eq 0` triggers never fire if step 4 happens before step 3.

**Current state:**
- Documented in `docs/system/triggers.md` ✅
- Commented in `action_processing.rs:151-158` ✅
- Happy path tested ✅
- **Violation tests missing** ❌
- **Runtime enforcement none** ❌

**Fix:**
1. Add explicit violation test
2. Add doc anchor in `execute_freeaction_impl` citing the invariant
3. Consider runtime assertion or test that proves violation fails

---

### Finding 2: Terminology Overload Creates Navigation Confusion

**Severity:** CRITICAL  
**Locations:** Throughout codebase (see Phase 1 report)

| Term | Meanings | Risk |
|------|----------|------|
| `Action` | 10 distinct types across 3 layers | Navigation impossible without full context |
| `Trigger` | 5 distinct types across 4 files | Same issue |
| `Event` | 3 distinct concepts (no LogType::Event exists) | Doc-code mismatch |

**Problem:** Searching for "Action" returns 10 unrelated types. Searching for "Trigger" returns 5. An AI must read context to understand which type is meant.

**Fix:**
1. Add disambiguation section to AGENTS.md
2. Add module-level doc comments explaining responsibilities
3. Add doc comment to `NpcEncounterState` explaining it tracks per-NPC encounters

---

### Finding 3: NpcEvent Types Misplaced (Semantic Coupling)

**Severity:** HIGH  
**Location:** `src/model/quantifier.rs`

`NpcEvent`, `NpcEventType`, `NpcEventList` live in `model/quantifier.rs` but are:
- Produced by diffing `QuantifierResult` (not by the quantifier directly)
- Consumed by `engine/action_processing.rs` (not by narrative)
- Engine state-transition events, not narrative types

**Problem:** Module name creates misleading association. An AI might think "these are narrative types" when they're actually engine state types.

**Fix:**
1. Move to `model/state.rs` or `model/event.rs`, OR
2. Add doc comment explaining dual ownership
3. Add to arch-lint rationale

---

### Finding 4: Documentation Gaps

**Severity:** HIGH  
**Locations:**
- `docs/system/game_flow.md` — Phase 4.5 order misleading
- `docs/system/navigation.md` — dynamic room creation not documented
- `docs/architecture/system.md` — server_helpers module doesn't exist

**Problems:**
1. Phase 4.5 says movement is processed after quantifier — code does it during
2. `create_dynamic_room` not mentioned anywhere
3. `server_helpers` documented but never implemented

**Fix:**
1. Update Phase 4.5 description
2. Add dynamic room creation section
3. Remove server_helpers from docs

---

## Medium Findings (Should Fix)

### Finding 5: Quantifier Dual Role Confusing

**Severity:** MEDIUM  
**Location:** `src/narrative/agents/quantifier/`

The quantifier module has three responsibilities:
1. **Agent role** — `QuantifierAgent` implements Agent trait
2. **Parser role** — `parser.rs` parses LLM output
3. **Orchestrator role** — `core.rs` manages retry logic

**Problem:** "Quantifier" could mean agent, parser, result, or LLM call. Context disambiguates but adds cognitive load.

**Fix:**
1. Add module-level doc comment explaining three responsibilities
2. Consider renaming (quantifier_agent, quantifier_parser, quantifier_result)

---

### Finding 6: Test Coverage Gaps

**Severity:** MEDIUM  
**Locations:** See Phase 6 report

| Gap | Severity | Impact |
|-----|----------|--------|
| INV-003: Swipe state restoration not verified | MEDIUM | Swipe navigation could silently fail |
| INV-005: Mutation order handle_movement not tested | MEDIUM | Refactoring could break order |
| INV-007: Dynamic room only in benchmark | LOW | Could be skipped in CI |

**Fix:**
1. Add INV-003 integration test
2. Add INV-005 explicit order test
3. Promote INV-007 benchmark to unit test

---

## Low Findings (Nice to Have)

| Finding | Location | Fix |
|---------|----------|-----|
| `GenerationState` renamed to `InputBuffer` | Already done ✅ | - |
| `TriggerAction` renamed to `TriggerEffect` | Already done ✅ | - |
| `test_app_builder` duplicate in docs | `docs/architecture/system.md` | Remove duplicate |
| 7-layer vs 8-layer (Phi) | `docs/system/game_flow.md` | Clarify |
| Layer confusion (prompt layer vs architecture tier) | Throughout | Add disambiguation |

---

## Recommendations Priority Matrix

| Priority | Finding | Action | Impact |
|----------|---------|--------|--------|
| 1 | Mutation order | Add violation test + doc anchor | Prevents silent trigger breakage |
| 2 | Terminology | Add disambiguation to AGENTS.md | Improves navigation |
| 3 | Documentation | Fix 3 doc gaps | Prevents misleading info |
| 4 | NpcEvent placement | Move or add doc comment | Reduces semantic coupling |
| 5 | Quantifier role | Add module doc comment | Clarifies responsibilities |
| 6 | Test coverage | Add 3 tests | Closes gaps |

---

## Files to Modify

| File | Change |
|------|--------|
| `AGENTS.md` | Add "Terminology Disambiguation" section |
| `src/engine/action_processing.rs` | Add doc anchor for mutation order |
| `docs/system/triggers.md` | Fix LogType::Event ambiguity, add violation section |
| `docs/system/game_flow.md` | Fix Phase 4.5, clarify 8 layers |
| `docs/system/navigation.md` | Document dynamic room creation |
| `docs/architecture/system.md` | Remove server_helpers, fix duplicate |
| `src/model/quantifier.rs` | Add doc comment explaining types |
| `src/narrative/agents/quantifier/mod.rs` | Add module doc comment |
| `tests/` | Add 3 gap-closing tests |

---

## Investigation Artifacts

All findings are documented in phase files:

```
docs/plans/agent-comprehension-investigation/
├── phase-1-vocabulary-audit.md      ─── 6 terms mapped, 10+ meanings documented
├── phase-2-mutation-order.md       ─── Invariant traced, documented, gaps found
├── phase-3-tier-boundaries.md       ─── 16 cross-tier imports verified, 1 semantic coupling
├── phase-4-doc-code-consistency.md  ─── 16 discrepancies found (3 HIGH, 8 MEDIUM, 5 LOW)
├── phase-5-module-mental-models.md  ─── 4 modules documented, AI modification checklists
├── phase-6-test-coverage.md         ─── 7 invariants covered, 3 gaps identified
└── INVESTIGATION_SYNTHESIS.md      ─── This file
```

---

## Conclusion

The Chronicler Engine is **well-documented but architecturally complex**. The main comprehension challenges are:

1. **Overloaded terminology** — 10 types named "Action", etc.
2. **Load-bearing invariants** — mutation order, snapshoting, cancellation
3. **Semantic coupling** — NpcEvent in quantifier module, engine imports narrative types
4. **Documentation gaps** — missing dynamic room docs, wrong Phase 4.5 description

**The good news:** All critical issues have clear fixes. The codebase is well-structured; the problems are documentation and naming, not architecture.

**Next steps:** Implement the Priority 1-3 fixes (mutation order violation test, terminology disambiguation, documentation fixes). This will significantly improve AI comprehension without changing architecture.