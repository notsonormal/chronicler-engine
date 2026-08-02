---
diataxis: reference
title: Agent System
---

## Overview

The engine has an agent architecture. An **agent** is any type implementing the `Agent` trait. Agents are loaded from `AppSettings` at startup and registered in `AgentRegistry`. The pipeline iterates registered agents at each `ExecutionPhase` and dispatches them in registration order. The engine works with **zero agents** — all agent execution is optional.

The single agent in production today is `QuantifierAgent` (post-generation, scene analysis). It detects NPCs that appeared in the narration and any movement destination, returning a `StatePatch` that the pipeline translates back into a `QuantifierResult`. The body of this document covers the quantifier prompt shape.

## Trait Contract

Every agent implements four capabilities:

- **Name** — a human-readable identifier used in logs and forensics.
- **Phase** — declares when the agent runs, as an `ExecutionPhase`; dispatched by the registry.
- **Backend selector** — records which LLM backend the agent prefers, as a `BackendSelector`. The selector is recorded at config time but is not consulted by the registry today; agents are bound to a recorder at wiring time.
- **Execute** — runs the agent against an `AgentContext` and returns an `AgentResult` (one of three variants).

Agents are stored as `Box<dyn Agent>` in the registry.

## Execution Phases

`ExecutionPhase` declares where in the pipeline the agent runs:

- **`PreGeneration`** — no dispatcher reads it today; the pipeline iterates only `PostGeneration` agents.
- **`PostGeneration`** — runs after the main LLM call returns. `QuantifierAgent` runs here.

The pipeline shape today:

1. Load state from snapshot.
2. Generate main narration via LLM.
3. Run `PostGeneration` agents — `QuantifierAgent` analyzes the narration and returns detected NPCs + movement.
4. Apply agent results to engine state and save the snapshot.

## Result Types

An agent's `execute` returns one of three `AgentResult` variants:

- **`PromptDirective(String)`** — inject text into a future prompt. Not constructed by any registered agent today.
- **`StatePatch(StatePatch)`** — propose a state mutation. `QuantifierAgent` returns this with the NPCs it detected in the narration and any movement destination. The pipeline translates the patch back into a `QuantifierResult`.
- **`NoOp`** — the agent ran but has nothing to report.

`StatePatch` carries a `confidence` field rated `High`, `Medium`, or `Low`. This rating reflects how certain the agent's LLM call is about its detected entities.

## Agent Registry

`AgentRegistry` is constructed at startup from `AppSettings.agents` via `AgentRegistry::from_configs(&settings.agents)?`. Each agent receives a recorder bound at wiring time (see "Per-Agent Backends" below); the `agent_type` discriminator selects the implementation, and `enabled` controls registration.

If no agent config exists in `AppSettings`, the registry injects defaults for backward compatibility:

- `quantifier` agent enabled, `PostGeneration`, `UseNamed("quantifier")` backend.

## Per-Agent Backends

Each agent can use a different LLM connection. The wiring lives in `bootstrap::wiring`:

- **Main narrator** uses `narration_connection_id` from settings.
- **Quantifier** uses `quantifier_connection_id` from settings, or a custom connection named via the `UseNamed` selector.

The quantifier recorder is pre-built and passed to `AgentRegistry::from_configs_with_storage`. The agent's declared `backend_selector()` is recorded but not consulted — the registry relies on the wiring-time binding.

The `UseMain` selector exists in the enum; no agent currently selects it.

## Quantifier Prompt Architecture

The quantifier follows the same XML-sectioned instructions + XML-wrapped data pattern as the narrative prompt:

- **Instructions are XML-sectioned.** The quantifier preset is split into `role`, `instructions`, and `output_format` fields. The system-prompt builder renders the active preset (or its override) through the template engine, then appends `<available_npc_ids>` and `<available_rooms>` as additional XML blocks after the preset text.
- **Data is XML-wrapped.** The user message is built from `<CurrentRoom>`, `<PreviousRoomNpcs>` (if non-empty), `<RecentHistory>` (if non-empty), and `<LatestNarration>`. Each block is plain XML with content drawn from the current game state. The user message closes with a literal instruction reminding the model to respond only with the JSON shape specified in the system message.

The pattern matches the narrative prompt's choice of XML tags rather than object-of-analysis tags like `<Role>` or `<SystemPrompt>`.

### System Message

The system message starts with the active preset text (rendered through `{{user}}` substitution). After the preset text, the builder appends two inventory lists:

```
<available_npc_ids>
  <Npc id="..." name="..."/>
  ...
</available_npc_ids>

<available_rooms>
  <Room id="..." name="..."/>
  ...
</available_rooms>
```

The NPC list draws every known NPC's `id` and `sheet.name`; the room list draws every room's `id` and `name` from the active map. Both lists are the universe the quantifier is allowed to reference; the model's response must use ids drawn from these lists.

### User Message

The user message carries:

- **`<CurrentRoom>`** — the player's current room's name, description, and (when present) navigation description.
- **`<PreviousRoomNpcs>`** — every NPC that was in the previous room's quantifier result (only emitted if non-empty; each entry carries the NPC's `id`, `name`, and full `description` from the character sheet).
- **`<RecentHistory>`** — the four most recent history entries (only emitted if non-empty), each rendered as `<Entry sender="...">...</Entry>`.
- **`<LatestNarration>`** — the most recent scene narration, prefixed with the player name and the action text.
- **Closing prose** — a literal reminder that the decision must be based on `<LatestNarration>` (not on `<RecentHistory>`) and that the response must be only the JSON format specified in the system message.

### Response Shape

The quantifier returns a JSON object with two top-level fields:

```json
{
  "npcs_in_room": ["carla", "gabriella"],
  "movement": {
    "type": "entering",
    "destination": "entrance_hall"
  }
}
```

- **`npcs_in_room`** — list of NPC ids that are present in the current room. Ids must match the `<available_npc_ids>` list.
- **`movement.type`** — one of `"entering"`, `"leaving"`, or `null`. The parser recognises `entering` and `leaving`; other values (including the `"in"` value the prompt schema mentions) are treated as no movement.
- **`movement.destination`** — the destination room id, drawn from `<available_rooms>`. Omitted or null when no movement.

The model is expected to return only this JSON object, but the parser tolerates surrounding prose by extracting the JSON substring. Confidence is computed by the parser: a parseable JSON with the expected shape is `High` or `Medium`; a malformed or unparseable response is `Low`.

### NPC Events (Computed Client-Side)

NPC enter/leave events are not returned by the LLM. Instead, the engine compares the previous quantifier result with the current one:

- **Entered** — NPC in the current result but not in the previous result.
- **Left** — NPC in the previous result but not in the current result.

The delta computation happens at the NPC reconciliation step of the quantifier phase.

### Retry Behaviour

The quantifier orchestration runs up to two LLM calls in sequence (the retry cap lives in the quantifier orchestration source). After each attempt:

- A `Low` confidence result (parse failure, malformed JSON, or unexpected shape) triggers a retry while attempts remain.
- A `Medium` or `High` confidence result is accepted on the first attempt.

If both attempts fail, the quantifier returns a `Low` confidence fallback: `npcs_in_area` is filled from `state.scene.npcs_in_area` (or the room's static NPC list if that is empty) and `movement` is left as `Low` confidence with the parsed movement preserved. The pipeline continues with the fallback rather than failing the action.

### Quantifier Prompt Presets

The active quantifier preset id is held on `AppSettings.active_quantifier_prompt_preset_id`. The engine reads the preset from storage at quantifier-call time and passes the assembled text through `QuantifierPromptContext.quantifier_prompt_override`; callers can override the active preset for a single call via this field. No pre-assembly caching is held on `AppSettings`.

Default presets ship as `data/prompt_presets/quantifier/default.json` and are protected from edit or delete. The dashboard's Prompt Presets tab provides the create/copy/set-active surface.

## Document References

- [ADR-006: Quantifier-Driven Game Systems](../../../docs/adr/adr-006-quantifier-systems.md) — dual-LLM architecture; quantifier-driven movement + NPC detection + NPC event layer.
- [ADR-009: Agent Trait and Registry Architecture](../../../docs/adr/adr-009-agent-trait-registry.md) — the `Agent` trait + `AgentRegistry` + the extension procedure for new agents; the `PostGeneration` dispatch that hosts the quantifier.
- [ADR-027: Hexagonal Architecture Migration](../../../docs/adr/adr-027-hexagonal-architecture-migration.md)
- [`../../explanation/agent_system_design.md`](../../explanation/agent_system_design.md) — why the agent abstraction is shaped this way and which tradeoffs it encodes.
- [`../game_flow.md#trigger-evaluation`](../game_flow.md#trigger-evaluation) — uses the quantifier's NPC + movement output as the precondition for trigger evaluation.
- [`../game_flow.md#phase-flow`](../game_flow.md#phase-flow) — pipeline home for the `PostGeneration` dispatch.
- [`./prompt_system.md`](./prompt_system.md) — narrative layered prompt architecture; the quantifier is a separate secondary prompt, not part of the layered narrative stack.
- [`./narration_system.md`](./narration_system.md) — LLM transport + sanitization + forensics; the recorder the quantifier's `LlmCallRecorder` runs through.
