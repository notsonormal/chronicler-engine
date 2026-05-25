# ADR-004: XML-Structured LLM Prompts

**Date:** 2025-04-13 (initial), 2026-04-14 (Silly Tavern v2), 2026-05-03 (plain-text instructions v3)

---

## Context

Initial prompt construction used plain text sections:

```
=== PLAYER CHARACTER ===

=== WORLD LORE ===

=== CONVERSATION HISTORY ===
```

Problems:
- **Parsing ambiguity** - LLM could misinterpret section boundaries
- **No structured content** - Hard to validate prompt completeness
- **Silly Tavern comparison** - Feature comparison showed XML advantage

---

## Decision

**Structure prompts using XML tags for clear section boundaries.**

### System Prompt Layers

8 XML-tagged sections in prompt construction:

1. `<SystemPrompt>` - Core game rules
2. `<GameState>` - Room, NPCs in area
3. `<NpcPresence>` - Dynamic NPC presence
4. `<PlayerCharacter>` - Player persona
5. `<WorldLore>` - World context
6. `<ConversationHistory>` - Full narration history
7. `<PlayerInput>` - User's current input
8. `<AuxiliaryInstructions>` - Post-history steering

### Quantifier Prompt

The quantifier prompt (for NPC detection, movement) was also updated to XML:

```xml
<QuantifierTask>Determine NPCs in the current room</QuantifierTask>
<CurrentRoom>
    <Name>Entrance Hall</Name>
    <Description>A grand entrance...</Description>
</CurrentRoom>
<RecentHistory>
    <Entry sender="narrator">You enter the hall.</Entry>
</RecentHistory>
<Query>Which NPCs are present in the room?</Query>
```

---

## Consequences

### Positive
- **Clear boundaries** - No section parsing ambiguity
- **LLM accuracy** - Structured input improves response quality
- **Extensibility** - Easy to add new sections
- **Silly Tavern compatible** - Proven pattern

### Negative
- **Token overhead** - ~200 chars for tags (negligible vs 4000 budget)
- **Learn curve** - Must maintain XML consistency

### Trade-offs
- Chose XML over JSON for readability
- Chose XML over Markdown for unambiguous boundaries

---

## v3 Update: Plain-Text Instructions + XML-Wrapped Data (2026-05-03)

**Problem discovered**: Self-referential XML tags wrapping instructions (`<SystemPrompt>`, `<Role>`, `<AuxiliaryInstructions>`, `<QuantifierTask>`) caused reasoning models (e.g., Gemma 4) to enter meta-analysis mode. The model would analyze the prompt structure instead of executing the instructions, consuming all tokens in `reasoning` fields and leaving `content` empty.

**New decision**: Separate instructions from data:
- **Instructions stay plain text** — No XML tags wrapping the system prompt or output format layer (formerly PHI). Imperative voice only ("You are...", "Your job is...", "Never...").
- **Data keeps XML wrapping** — `<GameState>`, `<KnownNpcs>`, `<ConversationHistory>`, `<CurrentRoom>`, etc. remain XML because they are external context, not instructions.
- **Output format is generic** — The output format layer contains only structural rules (anti-recap: "Do not re-narrate events that already occurred in the history above"). Tone and style instructions were moved to the customizable system prompt preset, not the generic output format template.

**Consequences of v3**:
- **Positive**: Reasoning models (Gemma 4, etc.) now execute instructions correctly without meta-analysis loops
- **Positive**: Token overhead reduced for instruction layers
- **Negative**: Slightly less visual structure in the system prompt section
- **Trade-off**: Lost the self-documenting XML structure for instructions, but gained reasoning-model compatibility

---

## Related ADRs

- [ADR-005: Layered Prompt System](./adr-005-layered-prompts.md) - Uses XML structure for data layers
- [ADR-006: Quantifier-Driven Game Systems](./adr-006-quantifier-systems.md) - Quantifier with XML data wrapping

---

## v4 Update: Sectioned XML-Wrapped Presets (2026-05-25)

**Problem discovered**: v3's plain-text system prompt worked for reasoning-model compatibility, but it merged all instructions into a single undifferentiated block. Users could not easily edit or experiment with individual aspects (role, rules, writing style, output format) without rewriting the entire prompt. The monolithic `prompt_text` field in the preset system made fine-grained customization impossible.

**New decision**: Split the system prompt into four hardcoded sections, each wrapped in XML tags:
- **`<role>`** — Identity and agency ("You are an interactive fiction author...")
- **`<instructions>`** — Behavioral rules (input validation, state tracking, narrative rules, etc.)
- **`<writing_style>`** — Prose constraints (perspective, tense, tone)
- **`<output_format>`** — Output constraints (anti-recap, GPTisms ban, response length)

These sections are assembled by `PromptPreset::assemble_prompt_text()` into a single XML-wrapped system message. Two builder-generated sections are inserted dynamically:
- **`<global_rules>`** — Injected from `world.json`, placed before `<output_format>`
- **Response length** — Appended inside `<output_format>` content from settings

**Why this reverses v3 for instruction containers**: v3 kept instructions plain-text to avoid meta-analysis. v4 wraps instruction *sections* in XML because the sections are no longer self-referential instruction tags (`<SystemPrompt>`, `<Role>`). They are content containers (`<role>`, `<instructions>`) that the LLM treats as labeled content blocks, not objects of analysis. The actual imperative text inside remains plain.

**Consequences of v4**:
- **Positive**: Users can edit individual prompt aspects via the Prompt Presets UI
- **Positive**: Section order is predictable and consistent across all presets
- **Positive**: Global rules and response length are injected automatically — no manual copy-paste
- **Negative**: Token overhead from 4–5 XML tag pairs (~150 chars, negligible)
- **Trade-off**: Instruction containers are XML-wrapped again, but with non-self-referential tags

## History

- **2025-04-13**: Initial XML refactor (prompt-xml-refactor plan)
- **Later**: v2 - Silly Tavern integration + quantifier XML (prompt-xml-refactor-v2 plan)
- **2026-05-03**: v3 - Plain-text instructions for reasoning-model compatibility (hercules-she-hulk-doctor-fate plan)
- **2026-05-19**: OutputFormat rename — PHI layer renamed to OutputFormat; anti-recap rule added to generic output format template; tone/style instructions moved to system prompt preset (see ADR-005)
- **2026-05-25**: v4 - Sectioned XML-wrapped presets (carnage-jessica-jones-magik plan)

---

## Historical Note

Initial prompt format used `=== SECTION ===` delimiters. This was later enhanced with Silly Tavern behavioral rules in v2, then refined to plain-text instructions in v3 to address reasoning-model compatibility. v4 reintroduces XML containers for instruction sections as non-self-referential content labels, splitting the monolithic prompt into editable role/instructions/writing_style/output_format fields.
