# Investigation Plan: AI Agent Comprehension Challenges in Chronicler Engine

**Date:** 2026-05-30  
**Objective:** Systematically identify what is difficult for an AI agent to understand when reading or modifying this codebase  
**Scope:** All source code, documentation, and tests under `chronicler_engine/src/`

---

## Executive Summary

The codebase has **high documentation quality** but **significant comprehension friction** in five critical areas. These are not bugs — they're architectural complexity that requires domain knowledge to navigate. An AI agent without full context will misunderstand, make incorrect changes, or miss load-bearing invariants.

---

## Investigation Method

### Phase 1: Vocabulary & Terminology Audit
**Goal:** Map overloaded terms → their actual meanings and locations

| Term | Overloaded Meanings | Code Locations | Risk Level |
|------|---------------------|----------------|------------|
| `Action` | `Action` enum (player commands), `TriggerAction` (narration effect), FreeAction (player input) | engine/action.rs, model/trigger.rs | HIGH |
| `Trigger` | Trigger struct, trigger evaluation, trigger firing, trigger continuation | engine/trigger_eval.rs, model/trigger.rs, application/ | HIGH |
| `Event` | `LogType::Event`, `NpcEvent`, generic occurrence | model/state.rs, narrative/quantifier, engine/action_processing | MEDIUM |
| `State` | `GameState`, `CharacterState` (NPC tracker), `MovementState`, `NarrativeState`, `SceneState`, `GenerationState` | model/state.rs, model/trigger.rs | MEDIUM |
| `Quantifier` | Module, trait, LLM call, result struct | narrative/agents/quantifier/ | MEDIUM |
| `Narrator` | `LlmBackend` trait, OpenRouter implementation, Game Master role | narrative/llm/backend.rs | LOW |

**Method:**
1. Search for each term in source + docs
2. Document all struct/module uses
3. Identify where context disambiguates (or doesn't)

**Deliverable:** Terminology heat map with disambiguation guide

---

### Phase 2: State Mutation Order Invariant Analysis
**Goal:** Verify the trigger system's critical ordering constraint is discoverable

From `docs/system/triggers.md`:
> Step 1: `handle_movement()` → update `current_room_id`  
> Step 2: Resolve NPCs from quantifier result  
> Step 3: `state.add_log(narration_text)`  
> Step 4a: `evaluate_triggers()` + build prompt  
> Step 4b: Trigger LLM call (outside lock)  
> Step 4c: `commit_trigger_narration()`  
> Step 5: `apply_npc_events()` — `times_met` increment AFTER trigger evaluation

**Critical Invariant:** Triggers are evaluated BEFORE `times_met` is incremented. If order is swapped, `TimesMet Eq 0` never fires.

**Method:**
1. Trace `execute_freeaction_impl` line-by-line
2. Verify each step in the mutation order
3. Check if code comments reference the invariant
4. Check if tests cover the invariant
5. Find any code that violates or could violate this order

**Deliverable:** Annotated mutation order diagram with violation points

---

### Phase 3: Tier Boundary Confusion Analysis
**Goal:** Identify where concepts cross tier boundaries in non-obvious ways

From `docs/architecture/system.md`, the boundaries are:
- `model/` → pure data, no dependencies on other tiers
- `engine/` → game logic, cannot depend on `server/` or `narrative/`
- `application/` → orchestration, cannot depend on `server/`
- `narrative/` → LLM integration
- `server/` → HTTP layer

**Cross-boundary flows to verify:**
1. `narrative::quantifier::NpcEvent` → `engine::action_processing` (engine imports from narrative)
2. `model::trigger::CharacterState` lives in `model/trigger.rs` but tracks NPC encounters, not trigger definitions
3. `StoredTriggerContext` lives in `model/state.rs` but is produced by narrative and consumed by application

**Method:**
1. List all cross-tier imports
2. Verify each is documented in `arch-lint.toml` scope rules
3. Check if cross-tier coupling is intentional or accidental

**Deliverable:** Cross-tier dependency map with rationale

---

### Phase 4: Documentation-Code Consistency Check
**Goal:** Find places where docs and code disagree

**Key documents to verify:**
- `docs/system/triggers.md` vs `src/engine/action_processing.rs`
- `docs/system/game_flow.md` vs `src/application/action_pipeline/pipeline.rs`
- `docs/system/navigation.md` vs `src/engine/logic.rs`
- `docs/architecture/system.md` vs actual module structure

**Method:**
1. Read each system doc
2. Trace the described flow in code
3. Note any discrepancies
4. Check if discrepancies are intentional (code evolved, doc stale)

**Deliverable:** Discrepancy report with severity

---

### Phase 5: The "What Is This?" Test
**Goal:** For each major module, identify what an AI needs to know but isn't obvious from reading the code alone

**For each module, identify:**
1. What problem does this solve?
2. What invariants must be preserved?
3. What are the side effects?
4. What naming is misleading?
5. What is load-bearing vs incidental?

**Modules to analyze:**
- `engine/action_processing.rs` — the mutation order
- `narrative/agents/quantifier/core.rs` — dual role (agent + parser)
- `model/trigger.rs` — misnamed `CharacterState`
- `application/action_pipeline/pipeline.rs` — orchestration complexity
- `storage/` — snapshot + message coordination

**Deliverable:** Module mental model documentation

---

### Phase 6: Test Coverage for Invariants
**Goal:** Verify critical behaviors are tested

From `docs/system/game_flow.md`, these behaviors must be tested:
- Main narration retry re-runs quantifier + triggers
- Event continuation retry preserves quantifier result
- Swipe navigation restores snapshot
- Trigger fires on `TimesMet Eq 0` before increment

**Method:**
1. List critical behaviors from docs
2. Find tests covering each behavior
3. Identify untested critical paths

**Deliverable:** Behavior-to-test mapping with gaps

---

## Output Format

1. **Terminology Heat Map** — Excel-style table with disambiguation guide
2. **Mutation Order Diagram** — Annotated flowchart with violation points
3. **Cross-Tier Dependency Map** — Visual + rationale for each cross-boundary import
4. **Documentation Discrepancy Report** — List of doc-code disagreements
5. **Module Mental Model Sheets** — Per-module "what you need to know" cards
6. **Critical Path Test Coverage** — Which invariants are untested

---

## Priority Ranking

| Finding | Severity | Agent Impact |
|---------|----------|--------------|
| State mutation order not enforced | CRITICAL | Will break trigger system |
| `CharacterState` misnamed + misplaced | HIGH | Confusion in trigger code |
| `NpcEvent` lives in narrative, consumed by engine | HIGH | Cross-tier coupling |
| Quantifier dual role (agent + parser) | MEDIUM | Hard to understand responsibility |
| `GenerationState` name mismatch | LOW | Minor confusion |
| `TriggerAction` name collision | MEDIUM | Code navigation harder |

---

## Timeline

| Phase | Estimated Effort |
|-------|------------------|
| Phase 1: Vocabulary | 2-3 hours |
| Phase 2: Mutation Order | 3-4 hours |
| Phase 3: Tier Boundaries | 2 hours |
| Phase 4: Doc-Code Consistency | 2-3 hours |
| Phase 5: Mental Models | 4-5 hours |
| Phase 6: Test Coverage | 2-3 hours |

**Total:** ~15-20 hours for full investigation

---

## Recommendations (Preliminary)

Based on initial review, these changes would improve AI comprehension:

1. **Rename `CharacterState` → `NpcRelationsState`** in `model/trigger.rs` and move to `model/state.rs`
2. **Move `NpcEvent` types** from `narrative/` to `model/` so engine can use without cross-tier import
3. **Add load-bearing comments** in `execute_freeaction_impl` citing the mutation order invariant
4. **Add doc anchors** referencing `docs/system/triggers.md` for the critical ordering section
5. **Create a "Key Invariants" doc** that lists behaviors an AI must preserve
6. **Add a "Mental Model" section** to AGENTS.md explaining the trigger + quantifier + state relationship

---

*This plan will be updated as investigation proceeds.*