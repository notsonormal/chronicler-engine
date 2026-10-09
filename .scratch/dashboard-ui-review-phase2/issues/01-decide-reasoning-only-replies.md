# Decide when a reasoning-only reply counts as an answer

Type: grilling (HITL)
Status: open
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
