---
diataxis: explanation
title: Prompt System Design
---

> **Diátaxis mode:** Explanation. This document is *understanding-oriented*: it explains why the Chronicler Engine's prompt system is shaped this way — why the system and user halves are split, why NPC cards are rendered in two tiers, and why prose constraints sit after the conversation history. It is the companion to `../reference/prompt_system.md`, which describes the prompt machinery as it is. The reader problem solved here is *understanding*: the tradeoffs the prompt-system shape encodes.

# Prompt System Design

## Why system and user are separated

The `PromptAssembler` produces two separate message halves: a system half carrying the instruction sections (`<role>`, `<instructions>`, `<global_rules>`) and a user half carrying the XML-wrapped data layers, the post-history splice (`<writing_style>`, `<output_format>`), and the `<PlayerInput>`. The transport sends them as the system message and the user message of a single LLM call.

The reason for the split is compatibility with OpenAI-compatible APIs, which expect the system role to carry imperative instructions and the user role to carry external context. The separation ensures reasoning models receive clear imperative instructions in the system role, while all external context stays in the user role.

The tradeoff is that the assembler maintains two message halves and fits both to the token budget, rather than emitting a single prompt string. The design pays that orchestration cost in exchange for compatibility with the system/user message-role convention.

## Why NPC cards are two-tier

The prompt carries two NPC blocks: `<KnownNpcs>`, a condensed roster of every NPC the player has met (name, an `(in room)`/`(elsewhere)` presence marker, and a short summary); and `<NpcsInRoom>`, full character cards for NPCs in the current room only.

The reason for the two-tier split is that the LLM needs two different kinds of NPC awareness. Full cards carry the detail — description, personality, scenario, relationships — the model needs to write for an NPC the player is currently interacting with; condensed cards carry enough identity and motivation that the model can reference off-screen characters or write introduction scenes when they walk in. Full cards for every NPC the player has ever met would bloat the prompt without paying off — most are not in the scene, and a condensed card preserves identity and motivation at a fraction of the token cost.

The tradeoff is two render paths and a presence-marker computation rather than one unified NPC block. The design pays that cost in exchange for off-screen awareness at low token cost and full-card detail reserved for the NPCs the player can actually interact with.

## Why prose constraints sit after history

The `<writing_style>` and `<output_format>` sections are rendered after `<ConversationHistory>` and before `<PlayerInput>` in the user message, rather than with the instruction sections in the system half.

The reason for the post-history placement is that LLMs exhibit strong recency bias: content closer to the generation point carries more weight. Placing prose constraints and structural rules at the end of the context window — after all story data but immediately before the point where the model generates — makes them more effective than the same content buried at the top of the prompt in the system message.

The tradeoff is that the assembler renders these sections as a separate string and splices them between the history and user-input layers, rather than emitting them inline with the system prompt. The design pays that rendering cost in exchange for the recency-bias leverage that late placement buys.

## Document References

- [ADR-004: XML-Structured LLM Prompts](../../docs/adr/adr-004-xml-prompt-format.md) — XML-sectioned instructions + XML-wrapped data; tags not objects of analysis.
- [ADR-005: SillyTavern-Style Layered Prompt System](../../docs/adr/adr-005-layered-prompts.md) — layered prompt architecture.
- [`../reference/prompt_system.md`](../reference/prompt_system.md) — reference description of the prompt-system machinery (the companion this doc explains).
- [`../reference/system_prompt.md`](../reference/system_prompt.md) — the assembled system prompt structure.
- [`../reference/quantifier_prompt.md`](../reference/quantifier_prompt.md) — the quantifier as a separate secondary prompt.