# Fail a reply that has no content

Type: task (AFK)
Status: resolved
Blocked by: 01

## Question

Make the engine follow the rule decided in [Decide when a reasoning-only reply counts as an answer](01-decide-reasoning-only-replies.md): a provider reply with null or empty `content` is a failure, and its reasoning text never becomes story text.

## Context

- Source: finding R6 (P1) in the [re-review](../../dashboard-ui-review/re-review-2026-10-09.md), screenshot `tmp/t24/12-reasoning-as-player-message.png`. `deepseek/deepseek-v4-flash` on OpenRouter returned `finish_reason: "length"`, `content: null` and 8771 characters of `reasoning`. The reasoning became Swipe 2/2 of the player's Message.
- Research: [01-reasoning-replies-research.md](../assets/01-reasoning-replies-research.md). Marinara's `empty-response-reason.ts` is a reference for the failure wording.
- Current code (known):
  - `extract_content_from_response` (`src/adapters/driven/llm/transport/utils/response.rs`) falls back from `content` to `reasoning` to `reasoning_content`, but only when `content` is JSON `null` or missing. An empty-string `content` passes through, and OpenRouter and Ollama then raise `LlmFailure::EmptyResponse` (`providers/openrouter.rs`, `providers/ollama.rs`).
  - `GenerationFailure::from_engine_error` (`src/domain/model/state/generation_status.rs`) maps `EmptyResponse` and `ParseError` to `GenerationFailureKind::UnreadableAnswer`. The sentence comes from `src/adapters/driving/http/utils/error.rs`.
  - `LlmRecorder::failed_message` (`src/application/llm_recorder.rs`) blanks `raw_response_json` because the error does not carry the body.
  - No code reads `finish_reason`.

## Scope

1. Remove the reasoning fallback. Null, missing or empty `content` is a failure in the transport, for every provider and role.
2. Read `finish_reason`. `"length"` with empty `content` gives a new `LlmFailure` variant and a new `GenerationFailureKind`. Its one-line sentence tells the user that the model used its whole token budget before it wrote an answer, and to raise Max Tokens or choose a model that does less reasoning. Other empty replies stay `UnreadableAnswer`. Token counts (`usage.completion_tokens`, `usage.completion_tokens_details.reasoning_tokens`) go in the raw details when present.
3. The empty-content failures carry the raw response body. `failed_message` stores it in `raw_response_json`, so the LLM Messages row shows it.
4. Replace the `response_tests.rs` cases that check the fallback. Check that role health and the failure banner report the new failure (the "Failure and health states" section of `docs/diataxis/reference/frontend/dashboard.md`), and update that doc and `prompt_system.md` where they describe the fallback.

Not in scope: a `"length"` reply that has content (map fog), and a "thinking" block in the story log (out of scope).

## Done when

- A reply with no content never becomes story text, for any role. The status line, the banner and role health report the failure. The LLM Messages row keeps the raw body.
- Tests are placed by `tests/STRATEGY.md`, and the answer names the tier.
- `python build.py` is green, the user reviews the diff, and the change is committed through `/commit-and-push`.

## Answer

Built. The rule lives in the transport, one place for every provider and role.

**Where the rule sits.** `parse_chat_response` (`src/adapters/driven/llm/transport/utils/response.rs`) now reads `choices[0].message.content` inline and nothing else; `extract_content_from_response` and the `reasoning` / `reasoning_content` fallback are gone. Blank, null or missing `content` is a failure. The dead `result.text.trim().is_empty()` guards are removed from `providers/openrouter.rs` and `providers/ollama.rs`, along with their now-unused `LlmFailure` imports. A body with no `choices[0].message` (missing, empty, or `null` message) is still `ParseError` — the provider never shaped a reply — so `unexpected_structure` and `token_spend` are private helpers beside it.

**New failure.**

| Surface | Value |
| --- | --- |
| `LlmFailure` variant | `TokenBudgetSpent { raw_response, completion_tokens, reasoning_tokens }`, for `finish_reason: "length"` with an empty answer |
| `GenerationFailureKind` variant | `TokenBudgetSpent` |
| Status line | "The model used its whole token budget before writing an answer — raise Max Tokens or choose a model that does less reasoning." |
| Display / details disclosure | `LLM returned no answer after spending its whole token budget (2048 completion tokens, 2041 of them reasoning)` — the parenthetical is trimmed to the counts the provider reported, and absent when it reported none |
| `llm_error_string()` | `LLM Error: the model spent its whole token budget before writing an answer (…)`; `EmptyResponse` keeps `LLM Error: empty response` |

`EmptyResponse` gained a `raw_response` field. Every other empty reply stays `EmptyResponse` → `UnreadableAnswer`. A `"length"` reply that does carry content stays an answer (map fog, untouched). Role health and the banner needed no new code: both read the row's `error_message`, and the recorder test pins that column for the empty-content path.

**Raw body.** `LlmFailure::raw_response_body()` returns the body for the three body-carrying kinds (`EmptyResponse`, `TokenBudgetSpent`, `ParseError`), and `LlmCallRecorder::failed_message` stores it in `raw_response_json`. `raw_request_json` stays blank — the transport returns the request payload only on success. Filling the row for `ParseError` too is the same rule with no special case; the ticket named only the empty-content kinds, and this is the one deliberate extension.

**Tests, by tier.**

- **Unit** (`src/**/*_tests.rs`): `response_tests.rs` rewritten around `parse_chat_response` (22 cases) — content returned verbatim, including a `"length"` reply with content; null content with `reasoning` and with `reasoning_content` → `EmptyResponse` carrying the exact input body; `""`, whitespace-only and missing content → `EmptyResponse`; `"length"` with empty content → `TokenBudgetSpent` with `(2048, 2041)`, with `None`/`None` when `usage` is absent and `(Some(300), None)` when `reasoning_tokens` is absent; `"stop"` and missing `finish_reason` → `EmptyResponse`; missing/empty `choices`, missing/`null` `message`, malformed JSON, `{}`, whitespace-only input → `ParseError`; an `error` body → `Http`. Also `error_tests.rs` (Display for the three count shapes, `raw_response_body` for every kind, `llm_error_string` rows), `generation_status_tests.rs` (`test_spent_token_budget_is_its_own_kind`), and `llm_recorder_tests.rs` (`complete_records_the_raw_body_of_an_empty_response`, with an in-file provider double — the row keeps the body, blanks the request, and has an empty `parsed_response`).
- **Tier 1** (HTTP E2E): `tests/http/failure_display.rs::test_spent_token_budget_clamps_with_its_own_line`, new scenario `38.7` in `docs/specs/failure_display.md` — the new kind renders its own line, not the unreadable-answer line or the general one, with the raw text once and only inside the disclosure.

**Docs.** `docs/diataxis/reference/narrative/narration_system.md` gains a "Reasoning text is never the answer" paragraph beside the Gemma 4 one. The ticket's other two candidates describe no fallback: `dashboard.md` says only "one sentence per `GenerationFailureKind`" without listing kinds (still true), and `prompt_system.md` has no response-parsing section. Inventing a mention would have been untrue, so neither was touched. No `CONTEXT.md` term covers an empty answer.

**Accepted deviations from the brief.** `extract_content_from_response` was deleted rather than made private; `expected_format` for the unexpected-structure error is now `choices[0].message.content` instead of the removed `content or reasoning`; `{"choices":[{"message":null}]}` stays `ParseError`.

**Behaviour note (inferred).** A `content` that is neither a string, `null` nor absent — an empty array or an Anthropic-style block array, both real OpenRouter shapes — is reported as an empty answer, not a parse error. The user-visible kind is identical either way (`UnreadableAnswer`), the body is kept, and the brief's literal rule does not distinguish the case. Support for that shape is not a fix for R6 and is not recorded as fog.

**Not fixed.** `docs/external_applications/marinara_engine.md:138` names `narrative::llm_client::extract_content_from_response()`, which this ticket deleted, and its Notes column ("chronicler handles JSON fields") described the reasoning-field fallback that this ticket removed. The symbol path was already stale — the whole "chronicler_engine equivalent" column of that table still uses the pre-hexagonal layer names. A docs-hygiene pass owns it, not this ticket.

**Review follow-ups.** The Settings connection test calls the provider with an 8-token probe, so a reasoning model would now fail it with `TokenBudgetSpent` while it is reachable. `ConnectionTestService::test_connection` counts that failure as reachable (unit test `test_spent_token_budget_still_proves_the_provider_is_reachable`). The recorder unit test `complete_records_a_spent_token_budget_for_role_health` pins the `error_message` that role health and the banner read. Smaller clean-ups: `EngineError::raw_response_body`, an exhaustive `LlmFailure::raw_response_body`, renamed helpers, and one overlapping test removed.

**Open question for the user.** The status line says "raise Max Tokens", as the Scope asks, but no dashboard control sets `max_tokens`: the Settings handler saves every connection with `max_tokens: None`, so every call uses the 2048 default.

**Gate.** `python build.py` green after the review follow-ups (`logs/build_20261010_154734.log`): all steps OK; integration `1604 passed, 0 failed, 2 skipped`; browser `27 passed, 0 failed`.

Remaining: the user reviews the diff, then one commit through `/commit-and-push`.
