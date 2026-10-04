# 17 — Flatten LLM transport/utils and share the result mapping

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we flatten `adapters/driven/llm/transport/utils/` into `transport/` and
share one `ChatCompletionResult → LlmCallResult` mapping between the
OpenRouter and Ollama adapters — or is the churn not worth it?

## Background

This is **candidate I** of the 2026-10-04 review, rated **Speculative**. See
`assets/architecture-review-2026-10-04.html`, card I.

The friction:

- `transport/mod.rs` (8 lines) and `transport/utils/mod.rs` (10 lines) are
  re-export modules. The only external surface is two functions in
  `utils/client.rs`. `request.rs` and `response.rs` are already pure-function
  modules.
- `OpenRouterBackend::complete` (`providers/openrouter.rs`) and
  `OllamaBackend::complete` (`providers/ollama.rs`) are identical except for
  the `name()` / `model()` accessors. Each maps `ChatCompletionResult` to
  `LlmCallResult` field by field.
- `ChatCompletionResult` carries `system_prompt` / `user_prompt`, which
  `response.rs` fills and the providers then overwrite. The scout found no
  reader.

The `LlmProvider` seam itself is real (4 adapters: OpenRouter, Ollama, Mock,
DeepSeek stub) and is not in question.

## What this ticket resolves

- **Commit or reject.** This is the weakest new candidate. Seriously consider
  **reject**. If rejected for a load-bearing reason, offer an ADR.
- **Shape if committed.** The flattened module layout; where the shared
  mapping lives; whether the unused prompt fields are dropped.

## Constraints

- Must keep `LlmProvider` as the accepted port
  (`docs/diataxis/explanation/architecture.md`).
- Module moves regenerate the AGENTS.md structure index (pre-commit hook).
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling`.
