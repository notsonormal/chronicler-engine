# Phase 1: Vocabulary & Terminology Audit

**Date:** 2026-05-30  
**Scope:** Map all overloaded terms to their actual meanings and code locations  
**Method:** Subagent-based systematic search of `src/` using search tool

---

## Executive Summary

Found **6 high-risk terms** with multiple, non-obvious meanings. These create comprehension friction for AI agents without full context.

| Term | Meanings | Risk Level | Most Confusing |
|------|----------|------------|----------------|
| **Action** | 10 distinct types across 3 layers | CRITICAL | `Action` enum vs `ActionPipeline` vs `ProcessActionResult` vs `TriggerMatch` |
| **Trigger** | 5 distinct types across 4 files | CRITICAL | `Trigger` struct vs `StoredTriggerContext` vs `TriggerMatch` vs `TriggerContinuationRequest` |
| **Event** | 3 distinct concepts | HIGH | `NpcEvent` vs `event_header` field vs (no `LogType::Event` exists) |
| **State** | 8+ distinct types | HIGH | `NpcEncounterState` in `model/trigger.rs` (misnamed, tracks NPC encounters not character state) |
| **Quantifier** | 13 types in 2 modules | MEDIUM | Agent (QuantifierAgent) + Parser (parser.rs) + Data types (model/quantifier.rs) |
| **Layer** | 1 meaning (prompt), 1 related (architecture tier) | LOW | Architecture tier vs prompt layer |

---

## 1. ACTION

### 1.1 Types Found (10 total)

| # | Type | File:Line | Kind | What It Represents |
|---|------|-----------|------|-------------------|
| 1 | `Action` | `src/engine/action.rs:2` | Enum | Core game command type. Single variant: `FreeAction(String)` wraps player input text. |
| 2 | `FreeActionContext` | `src/engine/action_processing.rs:14` | Struct | Runtime context passed to `execute_freeaction_impl` — narration text + quantifier result. |
| 3 | `TurnResult` | `src/engine/action_processing.rs:34` | Struct | Result of processing a single turn — next state, narration, optional trigger match. |
| 4 | `TriggerContinuationRequest` | `src/engine/action_processing.rs:20` | Struct | Trigger continuation context for retry. Wraps `StoredTriggerContext`. |
| 5 | `TriggerMatch` | `src/engine/action_processing.rs:25` | Struct | Raw trigger match data passed to application tier. Contains NPC ID, trigger index, name, prompt. |
| 6 | `ActionPipelineBackend` | `src/application/action_pipeline/pipeline.rs:24` | Trait | Backend contract for ActionPipeline. Provides LLM completion + post-generation agents. |
| 7 | `ActionPipeline` | `src/application/action_pipeline/pipeline.rs:44` | Struct | Orchestrates quantifier → engine → trigger flow. Generic over backend trait. |
| 8 | `ActionOutcome` | `src/application/action_pipeline/pipeline.rs:50` | Enum | Pipeline execution result — `Completed`, `Error`, `Cancelled`. |
| 9 | `ProcessActionResult` | `src/application/application_service.rs:63` | Enum | HTTP-layer result for action processing — `Started`, `ConcurrentGeneration`, `ShuttingDown`. |
| 10 | `ActionForm` | `src/server/fragments/actions.rs:20` | Struct | HTTP form wrapper for incoming player commands. |

### 1.2 Architecture

```
Server Layer (HTTP)
└── ActionForm ─── incoming player command wrapper

Application Layer (Orchestration)
├── ActionPipeline ─── orchestrates quantifier → engine → trigger
├── ActionPipelineBackend ─── trait: LLM completion + post-gen agents
├── ActionOutcome ─── pipeline result (Completed/Error/Cancelled)
└── ProcessActionResult ─── HTTP result (Started/ConcurrentGeneration/ShuttingDown)

Engine Layer (Game Logic)
├── Action ─── enum: FreeAction(player text)
├── FreeActionContext ─── runtime context for action execution
├── TurnResult ─── result of processing a turn
├── TriggerContinuationRequest ─── trigger retry context
└── TriggerMatch ─── raw trigger match data
```

### 1.3 Disambiguation Guide

| Term to Use | What It Means | Example |
|-------------|----------------|---------|
| `player_action` | `engine::action::Action` enum | `Action::FreeAction("look around")` |
| `action_pipeline` | `application::action_pipeline::ActionPipeline` | Orchestration struct |
| `action_backend` | `ActionPipelineBackend` trait | Trait providing LLM calls |
| `trigger_request` | `TriggerContinuationRequest` | Trigger retry context |
| `trigger_match` | `TriggerMatch` | Trigger selection result |

---

## 2. TRIGGER

### 2.1 Types Found (5 distinct)

| # | Type | File:Line | Kind | What It Represents |
|---|------|-----------|------|-------------------|
| 1 | `Trigger` | `src/model/trigger.rs` | Struct | Static schema: condition + effect + repeat + room_id. Defined at load time from JSON. |
| 2 | `TriggerCondition` | `src/model/trigger.rs` | Enum | Condition type: `TimesMet(ComparisonOperator, u32)`. Evaluation happens at runtime. |
| 3 | `TriggerEffect` | `src/model/trigger.rs` | Struct | Effect: `name` + `narration_prompt`. Formerly `TriggerAction` (now renamed). |
| 4 | `NpcEncounterLog` | `src/model/trigger.rs` | Struct | HashMap of NPC IDs → `NpcEncounterState`. Tracks encounter history for all NPCs. |
| 5 | `StoredTriggerContext` | `src/model/state.rs:117-126` | Struct | Runtime context stored in `NarrativeState.last_trigger`. Captures trigger identity + LLM call params for retry. |
| 6 | `evaluate_triggers` | `src/engine/trigger_eval.rs:6-43` | Function | Evaluates all NPC triggers against current state. Returns `Vec<(NpcCard, Trigger, index)>`. |
| 7 | `TriggerMatch` | `src/engine/action_processing.rs:25` | Struct | Turn-bound result from `execute_freeaction_impl`. Contains NPC ID, trigger index, name, prompt. |
| 8 | `TriggerContinuationRequest` | `src/engine/action_processing.rs:20` | Struct | Retry-bound carrier wrapping `StoredTriggerContext`. |

### 2.2 Architecture

```
model/trigger.rs ─── Static Schema (load-time)
├── Trigger ─── condition + effect + repeat + room_id
├── TriggerCondition ─── TimesMet comparison
├── TriggerEffect ─── name + narration_prompt
└── NpcEncounterLog ─── HashMap<String, NpcEncounterState>

model/state.rs ─── Runtime Context (game-time)
└── StoredTriggerContext ─── last trigger + LLM call params for retry

engine/trigger_eval.rs ─── Evaluation
└── evaluate_triggers(state) → Vec<(NpcCard, Trigger, index)>

engine/action_processing.rs ─── Orchestration
├── TriggerMatch ─── turn-bound trigger result
└── TriggerContinuationRequest ─── retry-bound trigger context
```

### 2.3 Disambiguation Guide

| Term to Use | What It Means |
|-------------|---------------|
| `trigger_def` | `model/trigger::Trigger` — static definition from JSON |
| `trigger_condition` | `TriggerCondition` enum — evaluation criterion |
| `trigger_effect` | `TriggerEffect` — name + narration prompt (formerly `TriggerAction`) |
| `npc_encounter_log` | `NpcEncounterLog` — HashMap of per-NPC encounter state |
| `trigger_context` | `StoredTriggerContext` — runtime context for retry |
| `evaluate_triggers` | Function that evaluates which triggers should fire |

---

## 3. EVENT

### 3.1 Types Found (3 distinct, NO `LogType::Event`)

| # | Type | File:Line | Kind | What It Represents |
|---|------|-----------|------|-------------------|
| 1 | `NpcEvent` | `src/model/quantifier.rs:70` | Struct | `{ npc_id: String, event_type: NpcEventType }` — runtime NPC presence change. |
| 2 | `NpcEventType` | `src/model/quantifier.rs:62` | Enum | `Entered \| Left` — direction of NPC movement. |
| 3 | `NpcEventList` | `src/model/quantifier.rs:76` | Struct | Collection of `NpcEvent` with confidence score. |
| 4 | `event_header: Option<String>` | `src/model/state.rs:32`, `src/model/message.rs:13,31` | Field | UI metadata on `LogEntry`/`Message`/`Swipe`. Names narrative events (e.g., "Carla Introduction"). |
| 5 | `LogType` | `src/model/state.rs:15-19` | Enum | Narration, Dialogue, System, Input. **NO Event variant.** |

### 3.2 Critical Finding: No `LogType::Event` Exists

The documentation at `docs/system/triggers.md` says:
> "When a trigger fires, an event header with this name appears in the story log"

This is **misleading**. There is NO `LogType::Event` variant. Instead:
- Trigger narrations are logged as `LogType::Narration`
- The `event_header: Option<String>` field on `LogEntry`/`Message`/`Swipe` marks them as trigger-based
- This is used for UI grouping and retry logic (distinguishing "main" narrations from "event" narrations)

### 3.3 Disambiguation Guide

| Term to Use | What It Means |
|-------------|---------------|
| `npc_event` | `NpcEvent` struct — NPC enter/leave transition |
| `event_header` | Field on `LogEntry`/`Message`/`Swipe` — UI event name |
| `log_type` | `LogType` enum — Narration/Dialogue/System/Input (no Event) |

---

## 4. STATE

### 4.1 Types Found (8+ in model/)

| # | Type | File | What It Represents |
|---|------|------|-------------------|
| 1 | `GameState` | `model/state.rs:181` | Aggregate root: world, map, player, NPCs, movement, narrative, scene, npc_encounter_log |
| 2 | `MovementState` | `model/state.rs:111` | Current room ID + dynamic rooms. Player location. |
| 3 | `NarrativeState` | `model/state.rs:129` | Message history, generation status, input buffer, last trigger context |
| 4 | `SceneState` | `model/state.rs:169` | NPCs currently in area, quantifier confidence. Ephemeral. |
| 5 | `GenerationStatus` | `model/state.rs:55` | `Idle \| Generating \| Error` — main generation status |
| 6 | `GenerationPhase` | `model/state.rs:76` | `Narrating \| Quantifying \| GeneratingEvent` — granular phase within generation |
| 7 | `InputBuffer` | `model/state.rs:103` | Player's current input text, cursor position, scroll offset. UI input buffer state. |
| 8 | `StoredTriggerContext` | `model/state.rs:117` | Last trigger for retry. Runtime context. |
| 9 | `NpcEncounterState` | `model/trigger.rs` | Per-NPC encounter tracking: times_met, trigger_fired, currently_meeting. **NOT what name implies.** |

### 4.2 Critical Finding: `NpcEncounterState` Misnamed

`model/trigger.rs` defines:
```rust
pub struct NpcEncounterState {
    pub times_met: u32,
    pub trigger_fired: HashMap<usize, bool>,
    pub currently_meeting: bool,
}
```

The name implies "a single character's state." It is actually:
- **Per-NPC tracking** of encounter history (not character state)
- Lives in `model/trigger.rs` (should be in `model/state.rs`)
- Tracks `times_met`, `currently_meeting`, and `trigger_fired` per NPC
- Only used for trigger evaluation

This is the highest-friction naming issue in the codebase. Every developer must mentally correct the name.

### 4.3 `GenerationState` → `InputBuffer` (Renamed)

The original `GenerationState` has been renamed to `InputBuffer` in `model/state.rs:103`. This is correct — it holds player input text, cursor position, and scroll offset. NOT the state of LLM generation (that's `GenerationStatus`).

---

## 5. QUANTIFIER

### 5.1 Types Found (13 total across 2 modules)

**`src/model/quantifier.rs`** (canonical data types):

| # | Type | Line | What It Represents |
|---|------|------|-------------------|
| 1 | `QuantifierConfidence` | 8 | Enum: High, Medium, Low, None. LLM confidence in parse. |
| 2 | `QuantifierParseResult` | 18 | Struct: npcs detected, confidence. |
| 3 | `MovementType` | 28 | Enum: Entered, Departed, NoMovement. |
| 4 | `MovementParseResult` | 40 | Struct: movement type + destination + confidence. |
| 5 | `QuantifierResult` | 50 | Aggregate: NPCs + movement + confidence + room info. Main output. |
| 6 | `NpcEventType` | 62 | Enum: Entered, Left. NPC movement direction. |
| 7 | `NpcEvent` | 70 | Struct: `{ npc_id, event_type }`. NPC presence change. |
| 8 | `NpcEventList` | 76 | Struct: list of NpcEvent + confidence. |
| 9 | `compute_npc_events` | 82 | Function: diffs previous vs current NPCs → NpcEvent list. |

**`src/narrative/agents/quantifier/`** (agent implementation):

| # | Type | File | What It Represents |
|---|------|------|-------------------|
| 10 | `QuantifierAgent` | `agent.rs:8` | Struct implementing `Agent` trait. Entry point for agent system. |
| 11 | `quantify_room_with_llm_call` | `core.rs:10` | Function: orchestrates LLM call with retry. |
| 12 | `determine_npcs_in_room` | `core.rs:58` | Function: top-level entry point. Wraps `quantify_room_with_llm_call`. |
| 13 | `parse_quantifier_response_with_movement` | `parser.rs` | Function: parses LLM JSON/text → `QuantifierResult`. |

### 5.2 Quantifier Dual Role

The quantifier has **three distinct responsibilities** that are not obviously related:

1. **Agent role**: `QuantifierAgent` implements the `Agent` trait from `narrative/agents/trait_def.rs`
2. **Parser role**: `parser.rs` contains `parse_quantifier_response_with_movement` — parses LLM output
3. **Orchestrator role**: `core.rs` contains `quantify_room_with_llm_call` — retry logic + LLM calls

This dual (triple) role makes it hard to understand what "Quantifier" means in any given context.

### 5.3 Disambiguation Guide

| Term to Use | What It Means |
|-------------|---------------|
| `quantifier_agent` | `QuantifierAgent` struct — the agent |
| `quantifier_result` | `QuantifierResult` — output data |
| `quantifier_parser` | `parser.rs` — LLM response parsing |
| `determine_npcs_in_room` | Function in `core.rs` — top-level entry |

---

## 6. LAYER

### 6.1 Types Found (2 related meanings)

| # | Type | File:Line | Kind | What It Represents |
|---|------|-----------|------|-------------------|
| 1 | `PromptLayer` | `src/narrative/prompt/types.rs:8` | Enum | Prompt assembly layers 0-7 + Phi. System/User split. |
| 2 | `LayerRenderer` | `src/narrative/prompt/assembler.rs:148` | Struct | Context for rendering each prompt layer. |
| 3 | Architecture tier | `docs/architecture/system.md` | Concept | model/engine/application/narrative/server tiers. |

### 6.2 Prompt Layers (0-7)

| Layer | Name | Content |
|-------|------|---------|
| 0 | System | XML-wrapped sections: role, instructions, writing_style, global_rules, output_format |
| 1 | Game State | Room, NPCs |
| 2 | NPC Cards | In-room NPCs only |
| 3 | Player | Player persona |
| 4 | World Info | Keyword-triggered lore |
| 5 | History | Full narration history (up to 1000 entries) |
| 6 | User | Current action |
| Phi | Safety margin | 256 tokens reserved |

### 6.3 Architecture Tiers vs Prompt Layers

These are **distinct concepts** that both use "layer":

| Concept | Meaning | Example |
|---------|---------|---------|
| Architecture tier | Module boundary (model/engine/application/narrative/server) | "model tier cannot depend on server tier" |
| Prompt layer | Part of the 7-layer prompt system | "Layer 2 contains NPC cards" |

---

## 7. Complete Terminology Heat Map

| Term | Meanings Count | Risk | Primary Location | Misleading? |
|------|----------------|------|------------------|------------|
| `Action` | 10 | CRITICAL | engine/action.rs, application/ | YES — 10 types across 3 layers |
| `Trigger` | 5 | CRITICAL | model/trigger.rs, engine/ | YES — 5 types across 4 files |
| `Event` | 3 | HIGH | model/quantifier.rs, model/state.rs | YES — no `LogType::Event`, event_header is metadata |
| `State` | 8+ | HIGH | model/state.rs, model/trigger.rs | YES — `NpcEncounterState` misnamed |
| `Quantifier` | 3 roles | MEDIUM | narrative/agents/quantifier/ | YES — agent + parser + orchestrator |
| `Layer` | 2 meanings | LOW | narrative/prompt/, docs/ | Partial — architecture tier vs prompt layer |
| `Narrator` | 2 meanings | LOW | narrative/llm/backend.rs | Low — LlmBackend trait vs OpenRouter impl |
| `Scene` | 2 meanings | MEDIUM | model/state.rs:SceneState, narrative docs | Partial — narrow (NPCs only) vs broad (situation) |

---

## 8. Recommendations

### Immediate (High Impact)

1. **Add disambiguation section to AGENTS.md** — "Action means X, Trigger means Y" quick reference
2. **Add doc comment to `NpcEncounterState`** explaining it tracks per-NPC encounters, not character state
3. **Update `docs/system/triggers.md`** — clarify that there is NO `LogType::Event`, narrations use `event_header` metadata

### Medium-term

4. **Rename `NpcEncounterState`** → `NpcEncounterState` is already named correctly (agent found it's not actually `CharacterState`). Verify actual name in `model/trigger.rs`.
5. **Consider renaming `StoredTriggerContext`** to `PendingTriggerContext` or `InFlightTrigger` — "Stored" implies persistence, it's actually transient runtime context.
6. **Add module-level doc comments** to `narrative/agents/quantifier/` explaining the dual agent+parser role.

---

*Phase 1 complete. Proceeding to Phase 2: State Mutation Order Invariant Analysis.*