# ADR-005: SillyTavern-Style Layered Prompt System

**Date:** 2025-04-13

> **Reference**: Full prompt layer specs, token budgets, and implementation details are in [`docs/system/prompt_system.md`](../system/prompt_system.md).

---

## Context

Pre-layered prompt system sent isolated prompts without comprehensive context:
- No conversation history sent to LLM
- Fragmented game state (room, inventory, NPCs in separate calls)
- No world info triggers
- Limited token budget management

The team wanted SillyTavern-style comprehensive prompting for better narrative quality.

---

## Decision

**Adopt an 8-layer prompt system sending full context to the LLM on every action.**

### Key choices and why

**Hard truncation over summarization** — Simpler and more reliable. Summarization requires a second LLM call, adds latency, and can hallucinate. Oldest history entries are trimmed first.

**Keyword triggers over RAG** — No vector database dependency. For a game with bounded lore, string matching against history is fast and sufficient. RAG adds operational complexity with marginal benefit at this scale.

**Full history over a sliding window** — The LLM needs full context for coherent long-form narrative. A fixed window risks cutting off plot-critical earlier entries arbitrarily.

**Single unified OutputFormat layer** — Previously `PhiMode::Continuation` existed as a separate variant; removed 2026-05-03. Continuation-specific instructions moved to the trigger user message (Layer 6) instead. The PHI layer was later renamed to **OutputFormat** (2026-05-19) to better reflect its purpose: structural output rules (anti-recap, perspective, tense) rather than post-history steering.

---

## OutputFormat Separation and Hardcoded Template Removal (2026-05-19)

**Problem**: The system prompt and output format (formerly PHI) were tightly coupled. Writing style instructions (third-person limited, past tense, literary fiction style) were baked into the hardcoded `SYSTEM_PROMPT_TEMPLATE` in `templates.rs`. This meant:
1. Users could not customize writing style without code changes
2. The hardcoded template served as an invisible fallback, masking when DB preset loading failed
3. The output format layer contained tone dictates that belonged in the customizable system prompt

**Decision**:
- **Delete `templates.rs`** — The hardcoded `SYSTEM_PROMPT_TEMPLATE` is removed entirely. System prompts are now sourced exclusively from DB-driven presets (see ADR-015).
- **Move writing style to system prompt preset** — Tone, perspective, and tense instructions are part of the customizable `default.json` seed and user presets.
- **OutputFormat becomes generic** — The inline `OUTPUT_FORMAT_TEMPLATE` in `builder.rs` contains only structural rules: perspective/tense reminders (generic, not stylistic), the "narrate what happens now" instruction, and an anti-recap rule ("Do not re-narrate events that already occurred in the history above").
- **Startup gap closed** — On bootstrap, the active system preset is loaded from the DB and cached into `AppSettings.active_system_prompt` (a `#[serde(skip)]` transient field). This guarantees the system prompt is populated even though the cached text is not persisted to `settings.json`.

### Why this separation matters

| Layer | Purpose | Customizable |
|-------|---------|-------------|
| **System Prompt** | World rules, identity, writing style, tone | Yes (via Prompt Presets) |
| **Output Format** | Structural rules: anti-recap, perspective, tense, narration scope | No (generic template) |

This aligns with the Marinara architecture pattern: system prompt sets the "who and how," output format sets the "what structure."

---

## Consequences

### Positive
- Context-rich prompts produce meaningfully better narrative quality
- Token budget management is deterministic (no LLM-in-the-loop summarization)
- Prompt injection sanitization via `{{variable}}` pattern filtering
- System prompt is now fully customizable via the Prompt Presets UI
- OutputFormat is generic and stable — no risk of tone drift from hardcoded templates
- Removing the hardcoded fallback surfaces DB loading failures immediately

### Negative
- More tokens per request → higher latency and API cost
- Full history means no compression — context window eventually limits session length
- System prompt cached in `AppSettings` is transient (`#[serde(skip)]`); requires DB lookup on every startup

### Trade-offs
- Chose hard truncation over summarization (simpler, no recursive LLM cost)
- Chose keyword triggers over RAG (no vector DB to run/maintain)
- Chose full history over sliding window (better narrative coherence)
- Chose generic OutputFormat over customizable one (simplicity and consistency across presets)

---

## Related ADRs

- [ADR-004: XML-Structured LLM Prompts](./adr-004-xml-prompt-format.md) — Format used within each layer

---

## History

- **2025-04-13**: Layered prompt system implemented; conversation history added to LLM calls for the first time
- **2026-05-03**: `PhiMode::Continuation` removed; PHI unified to a single universal template
- **2026-05-03**: System prompt and PHI converted to plain-text instructions (Marinara pattern); XML retained only for external data layers. Fixes Gemma 4 reasoning-loop bug. See ADR-004 v3 for full context.
- **2026-05-19**: PHI renamed to OutputFormat; hardcoded `SYSTEM_PROMPT_TEMPLATE` removed; writing style moved to customizable system prompt preset; generic anti-recap rule added to OutputFormat; startup loading of active preset into `AppSettings` closes the cached-prompt gap