# ADR-005: SillyTavern-Style Layered Prompt System

**Date:** 2025-04-13
**Status:** Accepted
**Drivers:** Narrative quality, token budget determinism, full context per action

> **Reference**: Full prompt layer specs, token budgets, and implementation details are in [`docs/system/prompt_system.md`](../system/prompt_system.md).

## Context

Pre-layered prompt system sent isolated prompts without comprehensive context:
- No conversation history sent to LLM
- Fragmented game state (room, inventory, NPCs in separate calls)
- No world info triggers
- Limited token budget management

The team wanted SillyTavern-style comprehensive prompting for better narrative quality.

## Decision

**Adopt a layered prompt system sending full context to the LLM on every action, with hard truncation, keyword triggers, and full history.**

### Key choices

- **Hard truncation over summarization** — Simpler and more reliable. Summarization requires a second LLM call, adds latency, can hallucinate. Oldest history entries trimmed first.
- **Keyword triggers over RAG** — No vector database dependency. For a game with bounded lore, string matching against history is fast and sufficient. RAG adds operational complexity with marginal benefit at this scale.
- **Full history over a sliding window** — The LLM needs full context for coherent long-form narrative. A fixed window risks cutting off plot-critical earlier entries arbitrarily.
- **Single unified OutputFormat layer** — Previously `PhiMode::Continuation` existed as a separate variant; removed 2026-05-03. Continuation-specific instructions moved to the trigger user message instead. OutputFormat holds structural rules (anti-recap, perspective, tense), not tone dictates.
- **System prompt is preset-driven** — Writing style, role, instructions, and output format live in DB-backed prompt presets (see ADR-015), not hardcoded templates. The hardcoded `SYSTEM_PROMPT_TEMPLATE` was removed.
- **Post-history section placement** — `writing_style` and `output_format` moved to the post-history user message (after `<ConversationHistory>`) so style/format constraints are not drowned out by massive context data. Matches Marinara Engine's proven architecture.
- **`PromptAssembler` trait decouples assembly from transport** — `LayeredPromptAssembler` owns the layer assembly logic; `LlmBackend` is pure transport. See ADR-022.

## Consequences

### Positive
- Context-rich prompts produce meaningfully better narrative quality
- Token budget management is deterministic (no LLM-in-the-loop summarization)
- Prompt injection sanitization via `{{variable}}` pattern filtering
- System prompt fully customizable via the Prompt Presets UI
- OutputFormat is generic and stable — no risk of tone drift from hardcoded templates
- Assembly logic is testable in isolation from LLM transport

### Negative
- More tokens per request → higher latency and API cost
- Full history means no compression — context window eventually limits session length
- System prompt cached in `AppSettings` is transient (`#[serde(skip)]`); requires DB lookup on every startup

### Trade-offs
- Chose hard truncation over summarization (simpler, no recursive LLM cost)
- Chose keyword triggers over RAG (no vector DB to run/maintain)
- Chose full history over sliding window (better narrative coherence)
- Chose preset-driven over hardcoded system prompt (customizability, no invisible fallback)
- Chose post-history placement of style/format (recency-bias avoidance)

## Related ADRs

- [ADR-004: XML-Structured LLM Prompts](./adr-004-xml-prompt-format.md) — Format used within each layer
- [ADR-015: Prompt Presets System](./adr-015-prompt-presets.md) — DB-backed prompt storage and assembly
- [ADR-022: PromptAssembler Trait Decoupling](./adr-022-prompt-assembler.md) — Assembly decoupled from transport

## History

- **2025-04-13**: Layered prompt system implemented; conversation history added to LLM calls
- **2026-05-03**: `PhiMode::Continuation` removed; PHI unified to single universal template. Plain-text instructions inside XML data layers for reasoning-model compatibility
- **2026-05-19**: PHI renamed to OutputFormat; hardcoded `SYSTEM_PROMPT_TEMPLATE` removed; writing style moved to customizable system prompt preset; generic anti-recap rule added to OutputFormat
- **2026-05-25**: `prompt_text` split into `role`/`instructions`/`writing_style`/`output_format`; `assemble_prompt_text()` added
- **2026-05-26**: `writing_style` and `output_format` moved to post-history user message; `PromptBuilder` replaced by `PromptAssembler` trait and `LayeredPromptAssembler` (ADR-022)
