# Decide whether failed LLM calls appear in LLM Messages

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

The LLM Messages tab lists only successful calls. The two failed Quantifier requests from the review turn were missing (finding 1.2). Should failed calls be recorded and shown? If so, what does a failed entry hold: error text, duration, attempt number, fallback used?

## Context

- Screenshots 17, 18.
- The recorder is `src/application/llm_recorder.rs`. The record type is `src/domain/model/llm_message.rs`. Scout the save path before the grilling. Finding facts is the agent's job.
- Minor: the panel also has no duration or token counts. Decide whether they belong here or in the fog.
- Related to [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md), but separate: this is about recording, not alerting.

## Done when

- The decision is in the ticket answer. Implementation tickets are created.

## Answer

**Decided: every LLM attempt is recorded, success or failure, and a failed row's error text is shown in LLM Messages.** This closes finding 1.2 on the recording side and makes finding 1.1 visible in the panel, because a failure row names the agent and the connection attempted. Nothing alerts loudly — that stays [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md).

| # | Decision |
|---|---|
| Q1 | Record **every attempt** — success or failure — as a row in `llm_messages`. A retried-then-succeeded call leaves both rows. |
| Q2 | No duration, no token counts. Out of scope for this map; no finding asks for them. |
| Q3 | The failure marker is the existing nullable `error_message`. No status column, no migration. |
| Q4 | Retention stays at the newest 50 rows. |
| Q5 | Store `EngineError::llm_error_string()` verbatim in `error_message` and render that text in the panel. |
| Q6 | No attempt number, no logical-call id. Rows stay flat. |
| Q7 | No separate degraded-outcome record; the failure rows record the fallback. |
| Q8 | The `NOT NULL` text columns take empty strings on a failure; the view model hides empty sections. No migration. |
| Q9 | A new `docs/specs/llm_messages.md` scenario with a tier-1 test in `tests/http/`, plus unit and driven-adapter tests. |

Requirement carried into implementation: a failed row names its **agent** and the **backend/model** it was attempted against. Neither reaches the recorder on the failure path today, and without them a failure row cannot say which connection failed.

Facts that grounded the decision:

- `LlmCallRecorder::complete` reaches `save_fn` only after `provider.complete(...)` returns `Ok`; on `Err` nothing is built. `LlmCallResult::to_message` sets `error_message: None` unconditionally, so the column is unused in production.
- `EngineError::llm_error_string()` never contains the API key or the `Authorization` header; `ParseError` drops the raw response; the `Http` variant carries the status plus up to 500 chars of provider body.
- `req:N` is a process-global, transport-local per-attempt tracing counter that is never returned. No game id, generation id or attempt index reaches the recorder; `GenerationGate`'s `generation_id` never leaves the gate.
- The Quantifier's exhaustion path is a single site holding `last_error` and returning a default result as `Ok`. Options returns `Err` with no fallback value. Quantifier's low-confidence retry and Options' unparseable `Ok(None)` both write success rows.

Boundaries decided, not oversights:

- A 200 response with no parseable options, and a low-confidence Quantifier result, are **not** marked failed. Marking them would move the failure decision from the transport to the caller, which no finding needs.
- `raw_request_json` is omitted on a failed row because the transport builds it; the prompts are already stored in their own columns.
- Under the 50-row cap a repeatedly failing backend can flush older successes. Left as-is; if it bites, the fix is a retention rule on the same table, not a second store.

Graduated: [Record failed LLM attempts and show them in LLM Messages](41-record-failed-llm-attempts.md) (task, blocked by this ticket).
