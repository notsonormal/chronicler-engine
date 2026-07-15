---
diataxis: explanation
title: Prompt System Design
---

> **Diátaxis mode:** Explanation. The reader problem solved here is *understanding*: the shape of the prompt system — the system/user split, two-tier NPC cards, and post-history placement of prose constraints — and the tradeoffs that shape encodes. Companion to `../reference/prompt_system.md`, which describes the prompt machinery as it is.

## The two halves of a prompt

The `PromptAssembler` produces two message halves for each LLM call: a system half carrying the instruction sections (`<role>`, `<instructions>`, `<global_rules>`) and a user half carrying the XML-wrapped data layers, the post-history splice (`<writing_style>`, `<output_format>`), and the `<PlayerInput>`. The transport sends them as the system message and the user message of a single LLM call.

OpenAI-compatible APIs expect the system role to carry imperative instructions and the user role to carry external context. The split maps the engine's directives into the role the model looks to for rules and the scene into the role the model looks to for context. Reasoning models benefit from the same separation: clear imperative instructions stay in the system message while external context stays in the user message.

The assembler's output is two message halves and one shared token budget.

## Two tiers of NPC awareness

The prompt carries two NPC blocks. `<KnownNpcs>` is a condensed roster of every NPC the player has met: name, an `(in room)` / `(elsewhere)` presence marker, and a short summary. `<NpcsInRoom>` is full character cards for NPCs in the current room only.

Full cards carry the detail — description, personality, scenario, relationships — the model needs to write for an NPC the player is currently interacting with. Condensed cards carry enough identity and motivation that the model can reference off-screen characters or write introduction scenes when they walk in. The two tiers serve different needs: full detail for NPCs the model is writing for right now, condensed identity for NPCs that need to remain available.

The design pays for two render paths and a presence-marker computation. Off-screen awareness lives at the condensed tier; full-card detail is reserved for the NPCs the player is interacting with.

## Prose constraints placed at the end of the user message

The `<writing_style>` and `<output_format>` sections are rendered after `<ConversationHistory>` and before `<PlayerInput>` in the user message, rather than with the instruction sections in the system half.

LLMs exhibit strong recency bias: content closer to the generation point carries more weight. Placing prose constraints and structural rules at the end of the context window — after all story data but immediately before the point where the model generates — buys the leverage of that recency. The same content in the system message at the top of the prompt carries less weight.

The assembler holds the two sections as a single splice string and renders them between history and player input.

## Document References

- [ADR-004: XML-Structured LLM Prompts](../../docs/adr/adr-004-xml-prompt-format.md) — XML-sectioned instructions + XML-wrapped data; tags not objects of analysis.
- [ADR-005: SillyTavern-Style Layered Prompt System](../../docs/adr/adr-005-layered-prompts.md) — layered prompt architecture.
- `../reference/prompt_system.md` — reference description of the prompt-system machinery (the companion this doc explains).
- `../reference/system_prompt.md` — the assembled system prompt structure.
- `../reference/quantifier_prompt.md` — the quantifier as a separate secondary prompt.
