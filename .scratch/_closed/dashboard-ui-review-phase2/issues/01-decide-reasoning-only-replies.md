# Decide when a reasoning-only reply counts as an answer

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

When a provider returns no `content` but does return `reasoning` or `reasoning_content`, should the engine accept that text as the answer, or record the attempt as a failure?

## Context

- Source: finding R6 (P1) in the [re-review](../../dashboard-ui-review/re-review-2026-10-09.md), screenshot `tmp/t24/12-reasoning-as-player-message.png`.
- Retry on a player input called `deepseek-v4-flash` on OpenRouter. The reply hit `max_tokens` (`finish_reason: "length"`, 2048 completion tokens) with `content: null` and 8771 characters of `reasoning`. `extract_content_from_response` (`src/adapters/driven/llm/transport/utils/response.rs`) falls back to `reasoning`, then `reasoning_content`. The chain-of-thought became Swipe 2/2 of the player's Message. LLM Messages shows a success, role health stays Healthy, no banner.
- The fallback is deliberate: commit `1d4aeace` added it and `response_tests.rs` pins it. Reversing it is a design change, so decide before building.
- The fallback serves every role: narration, quantifier, options, impersonate, user-message regeneration.
- Inputs to weigh: does any configured model (OpenRouter, DeepSeek, Ollama) put its real answer only in a reasoning field? Should `finish_reason: "length"` with empty content always fail? Should the failure use the typed failure kinds from [Close the four-review findings on `dashboard-ui-issues-2`](../../dashboard-ui-review/issues/71-close-review-findings.md), so the status line and the banner report it?
- Recommendation to grill: treat empty `content` as a failure of a new typed kind ("the model returned no answer"), keep the reasoning text in the stored raw response only, and never show reasoning as story text.

## Done when

- The rule is decided and recorded here.
- The implementation is graduated into a task ticket and added to the `Blocked by:` of [Re-review the dashboard after phase 2](07-re-review-after-phase-2.md).

## Answer

Decided with the user in a grilling session. Research asset: [01-reasoning-replies-research.md](../assets/01-reasoning-replies-research.md).

**Evidence.** SillyTavern and Marinara Engine never use reasoning text as the reply. Both keep it in a separate channel. Marinara shows empty `content` as a failure that names the token budget. The OpenRouter reasoning-tokens docs describe R6 exactly: a model that spends all of `max_tokens` on reasoning returns `finish_reason: "length"` with empty `content`. No provider documents a model that puts its final answer only in a reasoning field. The "GLM answers in reasoning" claim behind commit `1d4aeace` comes only from third-party bug reports (inferred: the same truncation case, misread).

**Rule.**

1. **Never use reasoning text as the answer.** Empty or null `content` is always a failure, whatever `reasoning`, `reasoning_content` or `finish_reason` hold. The fallback chain in `extract_content_from_response` goes, and so do the `response_tests.rs` cases that check it.
2. **Every role, one place.** The rule lives in the transport (`src/adapters/driven/llm/transport/utils/response.rs`). OpenRouter and Ollama both parse through it, so narration, quantifier, options, impersonate and user-message regeneration all follow it.
3. **A new failure kind for a spent token budget.** When `finish_reason` is `"length"` and `content` is empty, the failure gets its own `GenerationFailureKind`. Its sentence tells the user to raise Max Tokens or to choose a model that does less reasoning. Every other empty reply stays `UnreadableAnswer`. Token counts go in the details, not in the one-line sentence.
4. **Failed rows keep the raw response.** When the engine received a body, the failure carries it, and the LLM Messages row stores it in `raw_response_json`. Reasoning text is stored only there and never goes into the story log.
5. **Truncated replies that have content are not decided here.** A `"length"` reply with non-empty `content` still counts as an answer. Moved to the map's **Not yet specified**.

**Out of scope.** A separate "thinking" block in the story log (the SillyTavern pattern) is a new feature, not a fix for R6.

Implementation: [Fail a reply that has no content](08-fail-reply-without-content.md).
