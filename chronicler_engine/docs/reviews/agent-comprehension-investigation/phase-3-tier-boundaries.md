# Phase 3: Tier Boundary Confusion Analysis

**Date:** 2026-05-30  
**Scope:** Verify tier boundaries are respected and identify semantic coupling issues  
**Method:** Cross-tier import analysis + arch-lint verification

---

## Executive Summary

The tier boundaries are **architecturally correct** — all cross-tier imports respect the `arch-lint.toml` rules. However, there is **semantic coupling** between `model/quantifier.rs` and `narrative/agents/quantifier/` that creates confusion about ownership.

**Critical Finding:** `NpcEvent` types live in `model/quantifier.rs` but are produced by the narrative tier and consumed by the engine tier. This creates a misleading association between the model types and the quantifier module.

---

## 1. Architecture Overview

From `arch-lint.toml`, the defined scopes and their dependencies:

```
model ───innermost─── cannot depend on any outer layer
engine ─────────────── cannot depend on server, application, narrative
application ────────── cannot depend on server
narrative ──────────── no restrictions defined
server ─────────────── cannot depend on storage
bootstrap ──────────── no restrictions defined
test-support ───────── no restrictions defined
storage ────────────── no restrictions defined
```

### Dependency Rules (from `arch-lint.toml`)

| From | To | Allowed? |
|------|----|----------|
| model | engine | ❌ DENIED |
| model | narrative | ❌ DENIED |
| model | application | ❌ DENIED |
| model | server | ❌ DENIED |
| engine | narrative | ❌ DENIED |
| engine | application | ❌ DENIED |
| engine | server | ❌ DENIED |
| application | server | ❌ DENIED |
| server | storage | ❌ DENIED |

---

## 2. Cross-Tier Import Analysis

### 2.1 Application → Engine

**File:** `src/application/action_pipeline/pipeline.rs:6-8`

```rust
use crate::engine::action_processing::{
    FreeActionContext, TriggerContinuationRequest, TriggerMatch, apply_npc_events,
    commit_trigger_narration, execute_freeaction_impl,
};
```

**Assessment:** ✅ Correct — `application` CAN depend on `engine` (no rule forbids it).

**What it means:** The application tier orchestrates the engine. This is the seam between "what happens" (engine) and "how it's coordinated" (application).

### 2.2 Application → Narrative

**Files:** Multiple

| File | Imports | Purpose |
|------|---------|---------|
| `game_service/service.rs:10-13` | `QuantifierAgent`, `AgentRegistry`, `LlmCallResult`, `LayeredPromptAssembler` | Service needs agents + prompt builder |
| `action_pipeline/pipeline.rs:21-22` | `LlmCallResult`, `PromptAssembler` | Pipeline needs LLM backend + prompt assembly |
| `game_service/service.rs:13` | `LlmCallResult` | HTTP response wrapping |

**Assessment:** ✅ Correct — `application` CAN depend on `narrative` (no rule forbids it).

**What it means:** The application tier coordinates narrative generation. The quantifier agent and prompt builder are consumed here.

### 2.3 Engine → Model::Quantifier

**File:** `src/engine/action_processing.rs:10`

```rust
use crate::model::quantifier::{
    NpcEvent, NpcEventType, QuantifierResult, compute_npc_events
};
```

**Assessment:** ✅ Correct — `engine` CAN depend on `model`.

**What it means:** The engine consumes `NpcEvent` types from the model. However, the module is named `quantifier`, which creates semantic coupling.

---

## 3. Semantic Coupling Analysis

### 3.1 The NpcEvent Naming Problem

**Location:** `src/model/quantifier.rs`

**Types defined:**
- `NpcEvent` — struct: `{ npc_id: String, event_type: NpcEventType }`
- `NpcEventType` — enum: `Entered | Left`
- `NpcEventList` — struct: collection of events with confidence

**Problem:** These types are in `model/quantifier.rs`, which is named after the narrative quantifier module. But:

1. **The quantifier doesn't own these types** — the quantifier *produces* `QuantifierResult`, which then gets diffed to produce `NpcEvent` list
2. **The engine consumes these types** — `apply_npc_events()` in `engine/action_processing.rs` processes `NpcEvent` list
3. **These are engine state-transition events**, not quantifier-specific

**Flow:**
```
Narrative Quantifier
└── produces QuantifierResult (who is present)
        ↓
Application/Engine diff
└── produces NpcEvent list (who entered/left)
        ↓
Engine processes
└── apply_npc_events() → updates npc_encounter_log
```

**The confusion:** An AI reading `model/quantifier.rs` might think "these are narrative types" when they're actually engine state-transition types that happen to be produced by diffing quantifier results.

### 3.2 QuantifierResult Location

**Location:** `src/model/quantifier.rs:50`

```rust
pub struct QuantifierResult {
    pub npcs: QuantifierParseResult,
    pub movement: MovementParseResult,
    pub confidence: QuantifierConfidence,
}
```

**Problem:** This is named "QuantifierResult" but it lives in `model/`, not in `narrative/`.

**Why it matters:** The quantifier module produces this result. Naming it after the module creates a tight coupling. If the quantifier were renamed or restructured, this type name would become misleading.

**Alternative:** Could be named `SceneQuantifierResult` or `NpcSceneResult` to decouple from the module name.

### 3.3 TriggerEffect Naming (Resolved)

**Previous issue:** Phase 1 found `TriggerAction` was misnamed.

**Current state:** `TriggerEffect` is the correct name in `model/trigger.rs`. The previous `TriggerAction` name has been fixed.

---

## 4. Tier Boundary Discoverability

### How Clear Are the Boundaries?

| Boundary | Clarity | Notes |
|----------|---------|-------|
| model ↔ engine | HIGH | arch-lint enforces, docs explain |
| engine ↔ narrative | HIGH | arch-lint enforces, no imports found |
| application ↔ engine | HIGH | arch-lint enforces, natural seam |
| application ↔ narrative | HIGH | arch-lint enforces, natural seam |
| server ↔ storage | HIGH | arch-lint enforces, all access via ApplicationService |
| model ↔ storage | HIGH | arch-lint enforces, DB models isolated |

### Violation Risk Assessment

| Scenario | Risk | Impact |
|----------|------|--------|
| AI adds `use crate::server` in engine | LOW | arch-lint catches it at test time |
| AI adds `use crate::narrative` in engine | LOW | arch-lint catches it at test time |
| AI adds `use crate::application` in server | LOW | arch-lint catches it at test time |
| AI adds semantic coupling (logic in wrong tier) | MEDIUM | arch-lint only checks imports, not logic |

---

## 5. Cross-Tier Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         SERVER TIER                             │
│  Axum HTTP/WebSocket handlers, HTMX fragments                  │
└─────────────────────────────────┬───────────────────────────────┘
                                  │ calls
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                       APPLICATION TIER                          │
│  ActionPipeline, GameService, ApplicationService               │
│  ┌─────────────────────────────────────────────────────────────┤
│  │ Imports:                                                     │
│  │ - engine/action_processing (FreeActionContext, TriggerMatch) │
│  │ - narrative/llm/backend (LlmCallResult, LlmBackend)          │
│  │ - narrative/agents/quantifier (QuantifierAgent)              │
│  │ - narrative/prompt (PromptAssembler, LayeredPromptAssembler)   │
│  └─────────────────────────────────────────────────────────────┘
└───────────────────────┬─────────────────────────────────────────┘
                        │ orchestrates
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                         ENGINE TIER                              │
│  action_processing, trigger_eval, logic, parser                  │
│  ┌─────────────────────────────────────────────────────────────┤
│  │ Imports:                                                     │
│  │ - model/quantifier (NpcEvent, NpcEventType, compute_npc_events)│
│  │ - model/state (GameState, LogType)                             │
│  │ - model/character (NpcCard)                                   │
│  └─────────────────────────────────────────────────────────────┘
└───────────────────────┬─────────────────────────────────────────┘
                        │ pure data
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                         MODEL TIER                               │
│  WorldCard, MapDef, NpcCard, GameState, Trigger, QuantifierResult │
│  ┌─────────────────────────────────────────────────────────────┤
│  │ Semantic issue: model/quantifier.rs contains NpcEvent types   │
│  │ which are engine state-transition events, not narrative types │
│  └─────────────────────────────────────────────────────────────┘
└─────────────────────────────────────────────────────────────────┘
                        ▲
                        │ produces
                        │
┌─────────────────────────────────────────────────────────────────┐
│                       NARRATIVE TIER                            │
│  LlmBackend, QuantifierAgent, PromptAssembler                   │
│  ┌─────────────────────────────────────────────────────────────┤
│  │ Produces: QuantifierResult                                    │
│  │ Consumed by: application (GameService)                       │
│  │ NOT imported by: engine (verified via search)                 │
│  └─────────────────────────────────────────────────────────────┘
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. Recommendations

### Immediate

1. **Rename `NpcEvent` module location** — Consider moving `NpcEvent`, `NpcEventType`, `NpcEventList` to `model/state.rs` or `model/event.rs` to decouple from the quantifier module name
   - Current: `model/quantifier.rs` → misleading association
   - Proposed: `model/event.rs` → clear ownership (engine state transitions)
   - OR: Keep in `model/quantifier.rs` but add doc comment explaining it's engine state, not narrative

2. **Add doc comments to `model/quantifier.rs`** explaining the dual ownership:
   ```rust
   // This module contains:
   // 1. QuantifierResult + parse types — produced by narrative quantifier
   // 2. NpcEvent + NpcEventType — consumed by engine for state transitions
   //    (These are engine types, not narrative types, despite living here)
   ```

3. **Update arch-lint to document why `model/quantifier.rs` has engine-relevant types**

### Medium-term

4. **Consider renaming `QuantifierResult` to `SceneQuantifierResult`** — decouples from module name
5. **Add architecture diagram to `docs/architecture/system.md`** showing the cross-tier flows (similar to above)
6. **Add a "Tier Responsibility" table** to AGENTS.md:
   | Tier | Responsible For | Produces | Consumes |
   |------|----------------|----------|----------|
   | model | Data structures | All domain types | None |
   | engine | Game logic | State mutations | model types |
   | narrative | LLM integration | QuantifierResult | model types |
   | application | Orchestration | Actions/Results | engine + narrative |
   | server | HTTP handling | Responses | application |

---

## 7. Summary

| Finding | Severity | Status |
|---------|----------|--------|
| Tier boundaries architecturally correct | HIGH | ✅ All imports respect arch-lint |
| arch-lint enforced at test time | HIGH | ✅ Scope + deny-scope-dep rules work |
| `NpcEvent` semantic coupling | MEDIUM | ⚠️ Type lives in `model/quantifier.rs` but is engine state |
| `QuantifierResult` naming | LOW | ⚠️ Named after module, could be decoupled |
| Server ↔ storage boundary | HIGH | ✅ All server storage access via ApplicationService |
| Cross-tier flow discoverability | MEDIUM | ⚠️ No architecture diagram showing flows |

**Overall Risk:** LOW for violations, MEDIUM for semantic confusion.

---

*Phase 3 complete. Proceeding to Phase 4: Documentation-Code Consistency Check.*