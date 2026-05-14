# ADR-005: SillyTavern-Style Layered Prompt System

**Date:** 2025-04-13

> **Reference**: Full prompt layer specs, token budgets, and implementation details are in [`docs/system/prompt_system.md`](../system/prompt_system.md).

---

## Context

Pre-layered prompt system sent single-turn prompts without comprehensive context:
- No conversation history sent to LLM
- Fragmented game state (room, inventory, NPCs in separate calls)
- No world info triggers
- Limited token budget management

The team wanted SillyTavern-style comprehensive prompting for better narrative quality.

---

## Decision

**Adopt an 8-layer prompt system sending full context to the LLM on every turn.**

### Key choices and why

**Hard truncation over summarization** — Simpler and more reliable. Summarization requires a second LLM call, adds latency, and can hallucinate. Oldest history entries are trimmed first.

**Keyword triggers over RAG** — No vector database dependency. For a game with bounded lore, string matching against history is fast and sufficient. RAG adds operational complexity with marginal benefit at this scale.

**Full history over a sliding window** — The LLM needs full context for coherent long-form narrative. A fixed window risks cutting off plot-critical earlier entries arbitrarily.

**Single unified PHI layer** — Previously `PhiMode::Continuation` existed as a separate variant; removed 2026-05-03. Continuation-specific instructions moved to the trigger user message (Layer 6) instead.

---

## Consequences

### Positive
- Context-rich prompts produce meaningfully better narrative quality
- Token budget management is deterministic (no LLM-in-the-loop summarization)
- Prompt injection sanitization via `{{variable}}` pattern filtering

### Negative
- More tokens per request → higher latency and API cost
- Full history means no compression — context window eventually limits session length

### Trade-offs
- Chose hard truncation over summarization (simpler, no recursive LLM cost)
- Chose keyword triggers over RAG (no vector DB to run/maintain)
- Chose full history over sliding window (better narrative coherence)

---

## Related ADRs

- [ADR-004: XML-Structured LLM Prompts](./adr-004-xml-prompt-format.md) — Format used within each layer

---

## History

- **2025-04-13**: Layered prompt system implemented; conversation history added to LLM calls for the first time
- **2026-05-03**: `PhiMode::Continuation` removed; PHI unified to a single universal template
- **2026-05-03**: System prompt and PHI converted to plain-text instructions (Marinara pattern); XML retained only for external data layers. Fixes Gemma 4 reasoning-loop bug. See ADR-004 v3 for full context.