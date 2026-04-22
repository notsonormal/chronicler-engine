# ADR-004: XML-Structured LLM Prompts

**Date:** 2025-04-13 (initial), 2026-04-14 (Silly Tavern v2)

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
2. `<GameState>` - Room, inventory, NPCs in area
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

## Related ADRs

- [ADR-005: Layered Prompt System](./adr-005-layered-prompts.md) - Uses XML structure
- [ADR-006: Quantifier-Driven Game Systems](./adr-006-quantifier-systems.md) - Quantifier with XML

---

## History

- **2025-04-13**: Initial XML refactor (prompt-xml-refactor plan)
- **Later**: v2 - Silly Tavern integration + quantifier XML (prompt-xml-refactor-v2 plan)

---

## Historical Note

Initial prompt format used `=== SECTION ===` delimiters. This was later enhanced with Silly Tavern behavioral rules in v2.