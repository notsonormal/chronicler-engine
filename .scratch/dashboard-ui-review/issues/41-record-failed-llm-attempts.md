# Record failed LLM attempts and show them in LLM Messages

Type: task (AFK)
Status: resolved
Blocked by: 09

## Question

A failed LLM call leaves no row in `llm_messages`, so the one diagnostic screen cannot show the failure (finding 1.2). On the review turn the two failed Quantifier requests were absent while the engine fell back to fallback NPC IDs (finding 1.1) with nothing in the UI. Make every LLM attempt persist a row, failed or not, and render a failed row's error text in the LLM Messages fragment.

## Decisions to implement (from [Decide whether failed LLM calls appear in LLM Messages](09-decide-failed-llm-calls.md))

- Every **attempt** writes one row — success or failure. A retried-then-succeeded call leaves both rows.
- The failure marker is the existing nullable `error_message`. No status column, no migration.
- A failed row stores `EngineError::llm_error_string()` verbatim in `error_message`, and the panel renders that text — not only today's boolean `ERROR` badge.
- A failed row names its **agent** and the **backend/model** it was attempted against. Neither reaches the recorder on the failure path today.
- No attempt number, no logical-call id. Rows stay flat.
- No separate degraded-outcome record. The failure rows are the record of a fallback.
- Retention stays at the newest 50 rows. Duration and token counts are out of scope.
- The `NOT NULL` text columns take empty strings on a failure; the view model hides empty sections.
- Boundary: a 200 response with no parseable options, and a low-confidence Quantifier result, stay **success** rows with no failure marker.

## Context

- Recorder: `src/application/llm_recorder.rs::LlmCallRecorder::complete` reaches `save_fn` only after `provider.complete(...)` returns `Ok`, so an `Err` writes nothing. `LlmCallResult::to_message` (`src/application/ports/llm_provider.rs`) sets `error_message: None` unconditionally, so the column is unused in production.
- Error text: `EngineError::llm_error_string()` (`src/error.rs`) never contains the API key or `Authorization` header, and `ParseError` drops the raw response. The `Http` variant embeds the status plus up to 500 chars of provider body.
- The request payload is built in the transport (`src/adapters/driven/llm/transport/utils/request.rs`), so a failed row omits `raw_request_json`. `system_prompt` and `user_prompt` already have their own columns.
- Panel: route `GET /fragment/llm-messages` → `AppState::render_llm_messages` → `LlmMessagesTemplate` (`src/adapters/driving/http/templates.rs`), view model `LlmMessageView` (`src/adapters/driving/http/view_models.rs`).
- Failure injection for tests: `MockBackend::with_fail()` / `with_fail_first_n(n)` (`src/adapters/driven/llm/providers/mock.rs`). `tests/http/narrator_mode.rs` already reads `list_latest_llm_messages` at tier 1.

## Done when

- A failed LLM attempt persists a row naming the agent, backend and model, with `error_message` set to the error text and the response columns empty; a successful attempt still persists exactly one row.
- `GET /fragment/llm-messages` renders that error text.
- A new `docs/specs/llm_messages.md` carries the scenario, with a tier-1 test in `tests/http/` (per `tests/STRATEGY.md` rule 1), plus unit tests for the recorder's failure branch and a driven-adapter test for the persistence.
- Until ticket 31 resolves, write the Givens and Thens in `CONTEXT.md` terms, not field names.
- `python build.py` is green. Commit after user approval.

## Answer

Every LLM attempt now persists exactly one `llm_messages` row, success or failure. `LlmCallRecorder::complete` matches on the provider result; on `Err` it builds a failure row via the associated fn `failed_message` and saves it, then returns the original error. A failure row names the agent and the backend/model, stores `EngineError::llm_error_string()` verbatim in `error_message`, leaves the response columns (`raw_request_json`, `raw_response_json`, `parsed_response`) empty, and renders its failure text in `GET /fragment/llm-messages` (empty prompt/response/raw sections hidden). No status column, no migration, retention unchanged. Boundary kept: a 200 with no parseable options stays a success row with no failure marker.

**Premise correction.** The ticket said the agent and backend/model "neither reaches the recorder on the failure path today". That did not hold at the `complete` boundary: `complete` already receives `agent_name` and the resolved provider, so `provider.name()`/`provider.model()` supply both without touching `src/application/ports/llm_provider.rs` or `src/application/pipeline/**`.

Spec: new `docs/specs/llm_messages.md` with scenarios 33.1 (a failed narration attempt is recorded with its failure text and shown in the panel) and 33.2 (a later success does not overwrite an earlier failure). Givens/Thens use CONTEXT.md terms, not field names.

Tests: tier-1 HTTP `tests/http/llm_messages.rs` (33.1, 33.2); unit `src/application/llm_recorder_tests.rs` (failure branch, failed-then-succeeded, success row has no marker, provider error returned even when the failure save itself fails); driven-adapter real-SQLite `src/adapters/driven/storage/llm_messages_tests.rs`.

Judgment calls (locked decisions were ambiguous): (1) the failure row keeps `system_prompt`/`user_prompt` and empties only the response columns — the ticket's Context names `raw_request_json` as what is omitted, and prompts have their own columns; (2) if saving the failure row fails, the original provider error is still returned and the save error is logged, so callers see the true cause; (3) the driven-adapter test lives in `src/adapters/driven/storage/llm_messages_tests.rs`, which already holds a real-SQLite test for this seam.

Validation: full gate `python build.py --no-fmt` green — clippy OK, guardrails 137 passed, integration 1531 passed / 0 failed / 2 skipped, browser 31 passed / 0 failed; spec-coverage, test-structure and validate-docs OK. No commit made.
