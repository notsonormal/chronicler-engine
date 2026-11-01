---
diataxis: explanation
title: Prompt System Design
---

## The two halves of a prompt

The `PromptAssembler` produces two message halves for each LLM call. The system half carries the instruction sections (`<role>`, `<instructions>`, `<global_rules>`). The user half carries the XML-wrapped data layers, the post-history splice (`<writing_style>`, `<output_format>`), and the `<PlayerInput>` when it has content. The transport sends them as the system message and the user message of a single LLM call.

OpenAI-compatible APIs expect the system role to carry imperative instructions and the user role to carry external context. The split maps the engine's directives into the role the model looks to for rules. It maps the scene into the role the model looks to for context. Reasoning models benefit from the same separation: clear imperative instructions stay in the system message while external context stays in the user message.

The assembler's output is two message halves and one shared token budget.

## Two tiers of Character awareness

The prompt carries two Character blocks. `<KnownCharacters>` is a condensed roster of every Character in the World: name, an `(in room)` / `(elsewhere)` presence marker, and a short summary. `<CharactersPresent>` is full Character cards for Characters in the current room only.

Full cards carry the detail — description, personality, scenario, relationships — the model needs to write for a Character the player is currently interacting with. Condensed cards carry enough identity and motivation that the model can reference off-screen characters or write introduction scenes when they walk in. The two tiers serve different needs. Full detail is for Characters the model is writing for right now. Condensed identity is for Characters that need to remain available.

The Swipe state keeps the Characters present in no fixed order, and the World's Characters come from a map with a random order. The renderer sorts every card list by Character id.

A random order breaks the provider cache at the first card. It also makes two runs of one setup differ in a part that no author chose. The Character id is unique and stable in a World, so it gives one order for every call.

The card lists and the history write a fallback sentence in place of an empty body. The sentence tells the model a fact: no Character is present, or the story has no history yet.

## The history cut in blocks

The history is the longest part of the prompt, and it grows with each Cycle. A provider caches the start of a prompt, up to the first change. A history start that moves on every call ends the cache at the history. A cut in blocks holds the start still between two cuts, so the cache lasts until the next cut.

The cut drops whole Messages. A cut of text can start in the middle of a Message and drop its sender label. The model then reads text with no speaker. A whole Message always keeps its label.

The space that the cut fills leaves out the Player Input and the Guide. Both change on every call. A space that depended on them would move the history start on every call. The assembler keeps a reserve for them, and a guard drops more Messages when an Input passes the reserve.

The cut records the blocks it drops. The record is the one place that states what left the prompt, so no second pass over the history is needed to find it.

## Prose constraints placed at the end of the user message

The `<writing_style>` and `<output_format>` sections are rendered after `<ConversationHistory>` and before `<PlayerInput>` in the user message, rather than with the instruction sections in the system half.

LLMs exhibit strong recency bias: content closer to the generation point carries more weight. Prose constraints and structural rules benefit from that recency when they sit at the end of the context window. That place is after all story data, and immediately before the point where the model generates. The same content in the system message at the top of the prompt carries less weight.

`<PlayerInput>` renders only when it has content. An empty tagged block near the end of the prompt reads as an empty instruction at the point where the model starts to write.

## XML sections as content labels

The assembled prompt is organised as labelled sections — `<role>`, `<instructions>`, `<global_rules>`, `<writing_style>`, `<output_format>`, and the data wrappers (`<GameState>`, `<KnownCharacters>`, `<ConversationHistory>`, `<PlayerInput>`). Each section is a content carrier. The model reads the prose or data inside the tag, in the order the assembler renders the tags.

The labels give the user an edit handle per section. `<role>` carries role definition. `<instructions>` carries imperative rules. `<global_rules>` carries the world-bound rules from `world.json`. `<output_format>` carries the structural rules the generation must follow.

Editing one aspect means editing the section that aspect lives in. The surrounding structure stays stable across edits.

The engine's tags are content labels, not self-referential tags. A self-referential tag — `<SystemPrompt>`, `<Role>`, or `<AuxiliaryInstructions>` — names the prompt-as-a-whole rather than a region of it. Reasoning models (e.g. Gemma 4) treat such tags as a cue to enter meta-analysis mode, where the model comments on the prompt instead of answering it. The engine's tags differ from these on what they name: content tags carry the section's content.

## Document References

- [`../reference/narrative/prompt_system.md`](../reference/narrative/prompt_system.md) — reference description of the prompt-system machinery (the companion this doc explains).
