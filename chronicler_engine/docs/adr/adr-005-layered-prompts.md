# ADR-005: SillyTavern-Style Layered Prompt System

**Date:** 2025-04-13

---

## Context

Pre-layered prompt system sent single-turn prompts without comprehensive context:
- No conversation history sent to LLM
- Fragmented game state (room, inventory, NPCs)
- No world info triggers
- Limited token budget management

The team wanted SillyTavern-style comprehensive prompting for better narrative quality.

---

## Decision

**Adopt layered prompt system sending full context to LLM.**

### 8-Layer Prompt Structure

Based on SillyTavern's Prompt Manager:

| Layer | Content | Token Budget |
|-------|---------|--------------|
| Layer 0 | System prompt (game rules, role) | ~500 |
| Layer 1 | Game state (room, inventory, NPC states) | ~500 |
| Layer 2 | NPC cards (only in-room NPCs) | ~1500 |
| Layer 3 | Player persona | ~300 |
| Layer 4 | World info (keyword-triggered) | ~500 |
| Layer 5 | Full narration history | ~3000 |
| Layer 6 | User message | ~200 |
| Layer 7 | PHI (Post-History Instructions) — universal behavioral constraints | ~300 |

### Implementation Features

1. **Token budget**: MAX_CONTEXT_TOKENS = 32768 (fallback), context-aware fitting via `fit_messages_to_context()` — dynamically caps `max_tokens` and trims oldest history first
2. **Per-connection context windows**: `max_context_tokens` configurable per connection (8192 for Ollama, 32768 for API models)
3. **Prompt injection sanitization**: Filter `{{variable}}` patterns, instruction overrides
4. **World Info triggers**: Keyword matching from history (not RAG)
5. **Full history**: All conversation retained and sent (no summarization)

### Prompt Classes

```rust
pub struct PromptContext {
    pub current_room: Option<Room>,
    pub npcs_in_area: Vec<NpcCard>,
    pub player: &Player,
    pub history: Vec<LogEntry>,
    pub user_input: &str,
    // ...
}
```

---

## Consequences

### Positive
- **Context-rich**: LLM receives full game state + history
- **Narrative quality**: Better responses with comprehensive context
- **Token management**: Hard truncation prevents overflow
- **Security**: Prompt injection sanitization

### Negative
- **Token overhead**: More context = more tokens used
- **Latency**: Longer prompts = longer LLM processing
- **Cost**: More tokens = higher API costs

### Trade-offs
- Chose hard truncation over summarization (simpler, more reliable)
- Chose full history over sliding window (better context)
- Chose keyword triggers over RAG (simpler, no vector DB)

---

## Related ADRs

- [ADR-004: XML-Structured LLM Prompts](./adr-004-xml-prompt-format.md) - Format used for layer boundaries
- **Note**: These ADRs complement each other - ADR-004 defines *how* to format (XML tags), ADR-005 defines *what* to include (8 layers of content)

---

## History

- **2025-04-13**: Prompt builder system implemented

---

## Alternative Considerations

### Rejected Approaches
- **Summarization**: Chose hard truncation (simpler, no LLMsummaries-LLM)
- **RAG for World Info**: Chose keyword matching (simpler, no vector DB)
- **Per-NPC history**: Chose single unified history (simpler)

## History

- **2025-04-13**: Prompt builder system implemented
- **2026-05-03**: PHI layer unified — `PhiMode::Continuation` removed; PHI is now a single universal template. Continuation-specific instructions moved to the trigger user message (Layer 6).
- **2026-05-03**: Marinara-style prompt overhaul — system prompt and PHI converted to plain-text instructions. XML tags retained only for external data (`<GameState>`, `<KnownNpcs>`, etc.). Fixes Gemma 4 reasoning-loop bug triggered by self-referential XML.

## Historical Note

This was the first major prompt restructuring, introducing conversation history to LLM calls.