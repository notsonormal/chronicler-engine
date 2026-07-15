---
diataxis: reference
title: Quantifier Prompt
---

> **Diátaxis mode:** Reference. This document describes the quantifier prompt architecture as it is: the XML-sectioned instructions + XML-wrapped data pattern, the response JSON shape, the client-side NPC event computation, and the low-confidence retry behaviour. The problem it solves for the reader is *look-up*: given the active quantifier preset and the latest narration, what the engine sends to the quantifier LLM and what it does with the response. Verbatim preset text lives in `data/prompt_presets/quantifier/default.json`; transport and forensics live in `./llm_processing.md`.

## Overview

The quantifier is a separate secondary prompt used for post-narration scene analysis. It runs as a `PostGeneration` agent (`QuantifierAgent`) with its own LLM connection (`AppSettings.quantifier_connection_id`). Its job is structured extraction over a fixed schema: which NPCs are present in the current room and whether the player moved. It is not part of the layered narrative prompt stack — `./prompt_system.md` describes the narrative prompt; this document describes the quantifier's own shape.

## Prompt Architecture

The quantifier follows the same XML-sectioned instructions + XML-wrapped data pattern as the narrative prompt:

- **Instructions are XML-sectioned.** The quantifier preset is split into `role`, `instructions`, and `output_format` fields. The system-prompt builder renders the active preset (or its override) through the template engine, then appends `<available_npc_ids>` and `<available_rooms>` as additional XML blocks after the preset text.
- **Data is XML-wrapped.** The user message is built from `<CurrentRoom>`, `<PreviousRoomNpcs>` (if non-empty), `<RecentHistory>` (if non-empty), and `<LatestNarration>`. Each block is plain XML with content drawn from the current game state. The user message closes with a literal instruction reminding the model to respond only with the JSON shape specified in the system message.

The pattern matches the narrative prompt's choice of XML tags rather than object-of-analysis tags like `<Role>` or `<SystemPrompt>`.

## System Message

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

## User Message

The user message carries:

- **`<CurrentRoom>`** — the player's current room's name, description, and (when present) navigation description.
- **`<PreviousRoomNpcs>`** — every NPC that was in the previous room's quantifier result (only emitted if non-empty; each entry carries the NPC's `id`, `name`, and full `description` from the character sheet).
- **`<RecentHistory>`** — the four most recent history entries (only emitted if non-empty), each rendered as `<Entry sender="...">...</Entry>`.
- **`<LatestNarration>`** — the most recent scene narration, prefixed with the player name and the action text.
- **Closing prose** — a literal reminder that the decision must be based on `<LatestNarration>` (not on `<RecentHistory>`) and that the response must be only the JSON format specified in the system message.

## Response Shape

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

## NPC Events (Computed Client-Side)

NPC enter/leave events are not returned by the LLM. Instead, the engine compares the previous quantifier result with the current one:

- **Entered** — NPC in the current result but not in the previous result.
- **Left** — NPC in the previous result but not in the current result.

The delta computation lives in the pipeline's NPC reconciliation step (see `./triggers.md`).

## Retry Behaviour

The quantifier orchestration runs up to two LLM calls in sequence (the retry cap lives in the quantifier orchestration source). After each attempt:

- A `Low` confidence result (parse failure, malformed JSON, or unexpected shape) triggers a retry while attempts remain.
- A `Medium` or `High` confidence result is accepted on the first attempt.

If both attempts fail, the quantifier returns a `Low` confidence fallback: `npcs_in_area` is filled from `state.scene.npcs_in_area` (or the room's static NPC list if that is empty) and `movement` is left as `Low` confidence with the parsed movement preserved. The pipeline continues with the fallback rather than failing the action.

## Prompt Presets

The active quantifier preset id is held on `AppSettings.active_quantifier_prompt_preset_id`. The engine reads the preset from storage at quantifier-call time and passes the assembled text through `QuantifierPromptContext.quantifier_prompt_override`; callers can override the active preset for a single call via this field. No pre-assembly caching is held on `AppSettings`.

Default presets ship as `data/prompt_presets/quantifier/default.json` and are protected from edit or delete. The dashboard's Prompt Presets tab provides the create/copy/set-active surface.

## Document References

- [ADR-006: Quantifier-Driven Game Systems](../../docs/adr/adr-006-quantifier-systems.md) — dual-LLM architecture; quantifier-driven movement + NPC detection + NPC event layer.
- [ADR-009: Agent Trait and Registry Architecture](../../docs/adr/adr-009-agent-trait-registry.md) — the `PostGeneration` dispatch that hosts the quantifier.
- [`./prompt_system.md`](./prompt_system.md) — narrative layered prompt architecture; the quantifier is a separate secondary prompt, not part of the layered narrative stack.
- [`./agent_system.md`](./agent_system.md) — the `QuantifierAgent` that drives this prompt and returns `StatePatch` results to the pipeline.
- [`./llm_processing.md`](./llm_processing.md) — transport + sanitization + forensics; the recorder the quantifier's `LlmCallRecorder` runs through.
- [`./triggers.md`](./triggers.md) — uses the quantifier's NPC + movement output as the precondition for trigger evaluation and the source of NPC enter/leave deltas.