---
diataxis: explanation
title: Prompt System Design
---

## The two halves of a prompt

OpenAI-compatible APIs expect the system role to carry instructions and the user role to carry context. So the preset's directives go to the system message, and the scene goes to the user message.

## Two tiers of Character awareness

`<CharactersPresent>` holds full cards, and `<KnownCharacters>` holds a roster of every Character in the World. Full cards carry the description, personality, scenario and relationships that the model needs for a Character in the Room. The roster carries enough identity for the model to name an absent Character or write its entrance.

## Prompt order for the provider cache

The history is the largest part of the prompt, and it grows with each Cycle. A provider caches the start of a prompt, up to the first change. So the parts before the history must stay the same from call to call, and the history start must stay still.

Every card list renders in Character id order. A random order breaks the cache at the first card. It also makes two runs of one setup differ in a part that no author chose. The Character id is unique and stable in a World, so it gives one order for every call.

The blocks that stay the same during a story come before the history. The blocks that follow the Room, `<GameState>` and `<CharactersPresent>`, come after it. A Room change or a Character that enters or leaves then ends the cache after the history. For the same reason, the `<KnownCharacters>` roster holds only what stays the same: the name and the summary.

The history cut drops Messages in blocks. A cut that moved on every call would end the cache at the history. A cut in blocks holds the history start still between two cuts while the history space stays the same.

The history space leaves out the `<PlayerInput>` layer and the Guide. The Input changes on every call, and the Guide shows only on a Guided Generation. A space that depended on them would move the history start on every call. A reserve for them keeps the history start still on an ordinary Input.

## Whole Messages in the history

The cut drops whole Messages. A cut of text can start in the middle of a Message and drop its sender label. The model then reads text with no speaker. A whole Message always keeps its label.

## The end of the user message

The `<writing_style>` and `<output_format>` sections come after the story data and before `<PlayerInput>`, in the user message. LLMs show a strong recency bias: content closer to the generation point carries more weight. Prose constraints and structural rules get that weight at the end of the prompt. In the system message at the top, the same content carries less weight.

An empty block near the end of the prompt reads as an empty instruction at the point where the model starts to write. So `<PlayerInput>` and the Guide render only when they have content. A data block states a fact in place of an empty body. A card list says that it has no Character. The history says that the story has no history yet.

## Content labels

Each tag names the content of its region, so an author has an edit handle for each section. To change one aspect, an author edits the section that holds it, and the structure around it stays the same.

A tag that names the prompt as a whole, such as `<SystemPrompt>` or `<Role>`, makes some reasoning models comment on the prompt in place of answering it.

## Document References

- [`../reference/narrative/prompt_system.md`](../reference/narrative/prompt_system.md) — reference description of the prompt-system machinery (the companion this doc explains).
