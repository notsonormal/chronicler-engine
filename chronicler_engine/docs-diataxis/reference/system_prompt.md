---
diataxis: reference
title: System Prompt
---

> **Diátaxis mode:** Reference. This document describes the assembled system prompt as it is: which sections appear, where each section's content comes from, and how the two dynamic injection points (`global_rules` from world data and response length from settings) are placed. The problem it solves for the reader is *look-up*: given the active preset and the current game state, what the system message looks like. The seven-layer architecture lives in `./prompt_system.md`; verbatim preset text lives in `data/prompt_presets/system/default.json`.

## Overview

The system message is rendered by the assembler from four editable preset sections plus two dynamically injected blocks. Only three of the four preset sections land in the system message; the fourth (`writing_style`) is part of the post-history splice in the user message and is documented under `./prompt_system.md`. Empty sections are omitted from the assembled prompt; if no sections are present, the assembler produces an empty system message.

## Section Definitions

The four preset sections and the two dynamic blocks:

| Section | Source | XML Tag | System / User | Required |
|---------|--------|---------|---------------|----------|
| **Role** | Preset `role` field | `<role>` | System | No |
| **Instructions** | Preset `instructions` field | `<instructions>` | System | No |
| **Writing Style** | Preset `writing_style` field | `<writing_style>` | User (post-history splice) | No |
| **Global Rules** | `world.json` `global_rules` array | `<global_rules>` | System | Dynamic |
| **Output Format** | Preset `output_format` field + response length | `<output_format>` | User (post-history splice) | No |

The system half of the message carries `<role>`, `<instructions>`, and (when present) `<global_rules>`. The user half carries the data layers plus the post-history splice for `<writing_style>` and `<output_format>` — see `./prompt_system.md` for the splice position.

## Assembled Shape

The system message has the following shape (per-layer prose is from `data/prompt_presets/system/default.json`; reproduce verbatim by opening that file):

```xml
<role>
    You are an interactive fiction author with your own free will...
</role>

<instructions>
    Input validation rules:
    - ...

    State tracking rules:
    - ...

    Narrative rules:
    - ...
</instructions>

<global_rules>
    - Rule from world.json
    - Another rule from world.json
</global_rules>
```

Empty sections are dropped. The `<role>` and `<instructions>` sections render the preset fields through the template engine (which substitutes `{{user}}` — see `./prompt_system.md`) before wrapping.

## Dynamic Injection: Global Rules

Rules from the world's `global_rules` array are formatted as bullet points and wrapped in `<global_rules>`. They are inserted between `<instructions>` and the post-history splice in the rendered system prompt (i.e. after `<instructions>`, before `<output_format>` is rendered — the actual placement is "the third section of the system half"). An empty `global_rules` array produces no `<global_rules>` block.

## Dynamic Injection: Response Length

The user's selected response length from `AppSettings.response_length` is appended to the `<output_format>` content before wrapping:

```xml
<output_format>
    ...preset content...

    Response Length:
    <configured guidance text>
</output_format>
```

The default value shipped with the engine's default settings is the scene-adaptive guidance. The injection happens in the post-history splice; see `./prompt_system.md` for the splice position.

## Prompt Presets

The four editable sections are stored on `PromptPreset` records. The active preset id is held on `AppSettings.active_system_prompt_preset_id`. At assembly time, the assembler reads the preset fresh from storage; `AppSettings` holds only the active-id reference.

Default presets ship as `data/prompt_presets/system/default.json` and are protected from edit or delete. The dashboard's Prompt Presets tab provides the create/copy/set-active surface.

## Document References

- [ADR-005: SillyTavern-Style Layered Prompt System](../../docs/adr/adr-005-layered-prompts.md) — post-history splice rationale + preset-driven system prompt.
- [ADR-022: PromptAssembler Trait Decoupling](../../docs/adr/adr-022-prompt-assembler.md) — assembly decoupled from transport.
- [`./prompt_system.md`](./prompt_system.md) — seven-layer architecture + post-history splice position + token budgets + system/user separation.
- [`./llm_processing.md`](./llm_processing.md) — transport + sanitization + response sanitization (runs after the system message is built).
- [`./narration_engine.md`](./narration_engine.md) — how the Game Master uses the assembled system prompt in the FreeAction flow.