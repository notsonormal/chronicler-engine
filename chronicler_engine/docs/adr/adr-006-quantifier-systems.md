# ADR-006: Quantifier-Driven Game Systems

**Date:** 2026-04-17

---

## Context

> [!NOTE]
> This ADR covers three related features plus the dual-LLM architecture that enables them.

**Four components** work together:
1. **Dual-LLM Architecture** - Separate quantifier vs storyteller models
2. **Quantifier-Driven Movement** - LLM detects movement from natural language
3. **Quantified NPCs in Sidebar** - Dynamic NPC presence (not static map config)
4. **Reactive Movement/Triggers** - Auto-trigger continuation narrations on room entry

*Original Problem*: A single model handled both narration and scene understanding, which was expensive and slow.

---

## Decision

**The quantifier (extended LLM call) drives key game systems using dual-LLM architecture.**

### Dual-LLM Architecture

Separate models optimize for different tasks:

| Model | Purpose | Configuration |
|-------|---------|-------------|
| **Storyteller** | Main narrative generation | `LLM_MODEL` env var |
| **Quantifier** | Scene analysis (NPCs, movement) | `QUANTIFIER_MODEL` env var (defaults to free model) |

- **Quantifier uses reduced prompt**: 3-4 history entries instead of full history (~7000 tokens vs ~3000)
- **Separate free model**: Often uses `z-ai/glm-4.5-air:free` or similar for fast/cheap inference
- **JSON + text fallback**: Quantifier returns structured JSON, with text fallback

### Feature 1: Quantifier-Driven Movement

Instead of explicit commands (`go north`), the LLM analyzes narration to detect movement:

```
User: "I walk through the front gate"
LLM generates narration
Quantifier analyzes: "movement to Entrance Hall"
Room updated without explicit WalkTo command
```

- Movement intent: entering/in/leaving detection
- Invalid destinations: Create pseudo-room (dynamic room, session-only)
- Semantic exits: map.json with triggers, keywords

### Feature 2: Quantified NPCs

NPCs in the visual sidebar use quantifier's dynamic detection (not static map.json):

```rust
// GameState persists quantifier result
pub npcs_in_area: Vec<NpcCard>  // Dynamic from quantifier

// Re-quantification after EVERY main narration
// Note: This is now a SINGLE PASS post-narration that extracts both movement and NPCs
```

- **Storage**: Quantifier result persisted in `GameState`
- **Fallback**: Static `room.npcs` when quantifier unavailable
- **Quantification**: Runs once post-narration to grab both movement and NPC presence

### Feature 3: Reactive Auto-Trigger Movement

When player moves to room with NPC triggers, second LLM call continues the scene:

```
Player enters room with Gabriella (first encounter)
LLM narration: "You enter the hall."
Quantifier detects: movement to Entrance Hall
TRIGGER: Gabriella.times_met == 0
SECOND LLM call: continuation narration (unified 8-layer PromptBuilder)
COMBINED response delivered to player
```

- **Trigger conditions**: `TimesMet` comparison (Eq, Lt, Gte)
- **Character state**: `times_met` counter per NPC (in-memory)
- **Continuation prompt**: Includes first narration for continuity

---

## Consequences

### Positive
- **Natural language**: No explicit commands needed
- **Dynamic NPCs**: Sidebar reflects narrative reality, not config
- **Immersion**: Auto-triggers create seamless scene transitions
- **Graceful fallbacks**: Static data when LLM unavailable

### Negative
- **LLM dependency**: All features require LLM calls
- **Token cost**: Additional quantifier calls add latency and cost
- **No persistence**: Character state in-memory only
- **Pseudo-rooms**: Dynamic rooms don't persist

### Trade-offs
- Chose quantifier over explicit commands for natural feel
- Chose in-memory state (V1) over persistence
- Chose re-quantification after EVERY narration (not just keywords)

---

## Related ADRs

- [ADR-004: XML-Structured LLM Prompts](./adr-004-xml-prompt-format.md) - Quantifier uses XML format

---

## History

- **2025-04-13**: Quantifier movement implemented
- **2026-04-17**: Quantified NPCs sidebar
- **2026-04-18**: Reactive auto-trigger movement

---

## Historical Note

These features evolved incrementally:
- Quantifier movement: detect navigation from narration
- Quantified NPCs: sidebar shows dynamic NPC presence
- Reactive triggers: auto-fire scene continuations (unified 8-layer PromptBuilder with PhiMode::Continuation)
- Dual-LLM: separate quantifier model for scene analysis

The dual-LLM architecture (scene_quantification_v2) was the latest enhancement, allowing separate cheap/fast model for scene understanding.

---

## Implementation Patterns

### Character State (in-memory)

```rust
pub struct NpcEncounterState {
    pub times_met: u32,
    pub trigger_fired: HashMap<String, bool>,
}

pub struct CharacterState {
    pub npcs: HashMap<String, NpcEncounterState>,
}
```

### Trigger Definition

```rust
pub struct Trigger {
    pub condition: TriggerCondition,  // TimesMet(Eq, 0)
    pub action: TriggerAction,        // narration_prompt
    pub repeat: bool,               // fire once or repeatable
}
```