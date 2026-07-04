# ADR-004: XML-Structured LLM Prompts

**Date:** 2025-04-13
**Status:** Accepted
**Drivers:** Parsing clarity, reasoning-model compatibility, prompt customizability

## Context

Initial prompt construction used plain text sections delimited by `=== SECTION ===` markers. This caused three problems:

1. **Parsing ambiguity** — LLMs could misinterpret section boundaries
2. **No structured content** — Hard to validate prompt completeness
3. **SillyTavern comparison** — Feature comparison showed XML advantage

Later, two follow-on problems emerged:

- Self-referential XML tags wrapping instructions (`<SystemPrompt>`, `<Role>`, `<AuxiliaryInstructions>`) caused reasoning models (Gemma 4, etc.) to enter meta-analysis mode, consuming all tokens in reasoning fields and leaving `content` empty.
- A monolithic `prompt_text` field made fine-grained customization impossible — users could not edit individual aspects (role, rules, writing style, output format) without rewriting the entire prompt.

## Decision

**Structure LLM prompts using XML tags for clear section boundaries, separating instructions (plain text) from data (XML-wrapped) and sectioning the system prompt into four editable parts.**

### Current Format

**Data layers remain XML-wrapped** — external context, not instructions:
- `<GameState>`, `<KnownNpcs>`, `<ConversationHistory>`, `<CurrentRoom>`, etc.
- Quantifier prompt uses XML for `<QuantifierTask>`, `<CurrentRoom>`, `<RecentHistory>`, `<Query>`

**Instructions stay plain text within XML content containers** — non-self-referential section tags (`<role>`, `<instructions>`) that the LLM treats as labeled content blocks, not objects of analysis. The actual imperative text inside each section remains plain prose.

**System prompt split into four editable sections**, each wrapped in its own XML tag:
1. `<role>` — Identity and agency ("You are an interactive fiction author...")
2. `<instructions>` — Behavioral rules
3. `<writing_style>` — Prose constraints (perspective, tense, tone)
4. `<output_format>` — Output constraints (anti-recap, GPTisms ban)

Two sections are injected dynamically at assembly time:
- `<global_rules>` — From `world.json`, placed before `<output_format>`
- Response length — Appended inside `<output_format>` from settings

The assembled prompts live in `default.json` seed files under `data/prompt_presets/{system,quantifier}/`, stored in the `prompt_presets` DB table, and are assembled by `PromptPreset::assemble_prompt_text()` (see ADR-015).

## Consequences

### Positive
- Clear section boundaries — no parsing ambiguity
- Reasoning-model compatible — instructions don't trigger meta-analysis loops
- Fine-grained customization — users edit individual prompt aspects via the Prompt Presets UI
- Predictable section order across all presets
- Global rules and response length injected automatically — no manual copy-paste

### Negative
- Token overhead — 4–5 XML tag pairs per section (~150–200 chars; negligible vs 4000+ budget)
- Askama-specific template syntax to maintain
- Instruction containers are XML-wrapped, requiring non-self-referential tag discipline

### Trade-offs
- Chose XML over JSON for readability
- Chose XML over Markdown for unambiguous section boundaries
- Chose plain-text instructions inside XML containers over fully XML-wrapped instructions (reasoning-model compatibility)
- Chose four editable sections over a monolithic `prompt_text` (customizability)

## Related ADRs

- [ADR-005: SillyTavern-Style Layered Prompt System](./adr-005-layered-prompts.md) — Layer system using this format
- [ADR-006: Quantifier-Driven Game Systems](./adr-006-quantifier-systems.md) — Quantifier prompt uses XML data wrapping
- [ADR-015: Prompt Presets System](./adr-015-prompt-presets.md) — DB-backed preset storage and assembly

## History

- **2025-04-13**: Initial XML refactor — 8 XML-tagged sections replacing `=== SECTION ===` delimiters
- **v2**: SillyTavern integration + quantifier XML added
- **2026-05-03 (v3)**: Plain-text instructions inside XML data wrappers — fixes Gemma 4 reasoning-loop bug
- **2026-05-25 (v4)**: Sectioned XML-wrapped presets — monolithic `prompt_text` split into `role`/`instructions`/`writing_style`/`output_format` for UI editability
