# Reasoning effort per connection

**Date:** 2026-10-22
**Status:** Ready to implement. The owner decided all design questions in the grilling session of 2026-10-22, and the review questions R1 and R2 of the `improve-ai-plan` review.
**Source:** the owner's report: `deepseek/deepseek-v4-flash` sometimes spends its whole token budget on reasoning. The research is in `tmp/research/reasoning_params.md` (git-ignored).

## Summary

Each connection gets an optional **reasoning effort**. When the field is set, the OpenRouter and Ollama backends send `"reasoning": {"effort": "<value>"}` in the chat-completions body. When it is not set, the engine sends no reasoning field, and the model default applies. You set the effort in `data/settings.json` or with a select on the Settings connection form.

## Why

- The engine sends no reasoning field now. The body holds only `model`, `messages`, `stream` and `max_tokens` (`src/adapters/driven/llm/transport/utils/request.rs`).
- On OpenRouter, `deepseek/deepseek-v4-flash` reasons by default at effort `high`. Its reasoning counts against `max_tokens`. When the reasoning uses the full budget, the answer is empty, and the Job fails with `LlmFailure::TokenBudgetSpent`.
- In `data/settings.json`, `deepseek-v4-flash` is an **OpenRouter** connection. The `DeepSeek` backend is a stub that only returns an error.

## Facts from the research

Each fact is known from the provider docs or the live OpenRouter API, unless it has a label.

- **OpenRouter:**
  - The field is `reasoning: {effort}`. The effort values are `none`, `minimal`, `low`, `medium`, `high`, `xhigh` and `max`.
  - `none` turns reasoning off. A model with `mandatory: true` refuses `none`.
- **No field sent:** Each model uses its own default. No global default exists.
- **`deepseek/deepseek-v4-flash`:** The model accepts `xhigh`, `high` and `none`. Its default is `high`.
- **A value that the model does not accept:** OpenRouter has no single documented rule. The value can be ignored or mapped, or the request can fail with a 400. This is **inferred** and not tested.
- **Ollama `/v1/chat/completions`:**
  - The endpoint accepts the same `reasoning.effort` field, and `none` turns thinking off.
  - On a model without thinking, the current Ollama source logs a warning and ignores the field. Older releases may return a 400. This is **inferred** from the `main` source, not from a release note.
- **Native DeepSeek API:** It uses different fields: `thinking: {type}` and `reasoning_effort`. This plan does not touch it.

## Decisions

| # | Decision |
|---|---|
| D1 | The field is on each connection (`LlmProviderConfig`), next to `max_tokens` and `single_user_message`. To use one model at two efforts, make two connections. |
| D2 | The field is optional. If it is not set, the engine sends no reasoning field. Existing connections keep their current behaviour. |
| D3 | OpenRouter and Ollama send `"reasoning": {"effort": "<value>"}`. `Mock` and the `DeepSeek` stub ignore the field. |
| D4 | You can set it in `data/settings.json` and with a select on the Settings connection form. |
| D5 | The name is "reasoning effort": `reasoning_effort` in the code and the JSON, and "Reasoning effort" in the UI. It gets no `CONTEXT.md` entry, because that glossary holds only game-domain terms. |
| D6 | The select offers "Provider default" (empty value) plus all seven OpenRouter values. The engine does not check whether the model accepts the value. |
| D7 | The connection test sends the effort of the connection. A value that the model refuses then fails in the test, not during play. |
| D8 | Effort only. There is no `reasoning.max_tokens` budget field. |
| D9 | The spent-budget failure line becomes "Reasoning used the whole budget. Lower the reasoning effort." (59 characters). The owner first chose a longer text; review question R1 shortened it to fit the two-line clamp of the status column. |
| D10 | The seed connection `deepseek-v4-flash` in `data/settings.json` gets `"reasoning_effort": "none"`. |

No ADR: the change is easy to reverse.

## Design

### Domain: `src/domain/model/settings.rs`

- Add the enum `ReasoningEffort`, with the variants `Off`, `Minimal`, `Low`, `Medium`, `High`, `XHigh` and `Max`.
  - Serde uses the lowercase provider values: `none`, `minimal`, `low`, `medium`, `high`, `xhigh`, `max`. `Off` serializes as `"none"`. The variant is not named `None`, so that it does not read like `Option::None`.
  - Give it `as_str`, `FromStr` (an unknown value is an error), `display_label` and `ALL`, after the pattern of `NarratorMode`.
  - Give each variant a `///` doc. The guardrail `enum variant docs` (`tests/infrastructure/guardrails/enums.rs`) requires it, because the enum has methods and is not a `[TRIVIAL_ENUM]`.
- Add `#[serde(default, skip_serializing_if = "Option::is_none")] pub reasoning_effort: Option<ReasoningEffort>` to `LlmProviderConfig`.
- The stored settings keep connections as a JSON column. An old row has no field and reads as `None`. No migration is necessary.
- An unknown value in the stored JSON or in `data/settings.json` fails the settings read, the same as an unknown `provider` does now. The schema check of the gate catches it in `data/settings.json`. The form refuses it before it is stored (see the Settings form section).
- Add the new field to every `LlmProviderConfig` literal: `new`, `AppSettings::default`, and about 46 literals in 20 files in `src/` and `tests/`. Most of them get `reasoning_effort: None`. The literals include `tests/test_utils/stub_server.rs`, which also renders `ConnectionFormTemplate` for the browser tier.

### Transport: `src/adapters/driven/llm/transport/utils/`

- `request.rs`: `build_request_payload` takes `reasoning_effort: Option<ReasoningEffort>`. When the value is `Some`, the payload gets `"reasoning": {"effort": effort.as_str()}`. When it is `None`, the payload has no `reasoning` key.
- `client.rs`: `call_chat_completions`, `call_openrouter_with_model` and `call_ollama` pass the value through. Add a `debug` log line for the effort, next to the max-tokens line.
- The `raw_request_json` holds the field, so the LLM message forensics show the effort with no other change.

### Providers: `src/adapters/driven/llm/providers/`

- `OpenRouterBackend` and `OllamaBackend` store `reasoning_effort` from `from_config` and pass it to the transport.
- `MockBackend` and `DeepSeekBackend` do not change.
- Keep the Gemma-4 closed-channel workaround in `ollama.rs`. It is a separate fix, and an effort does not replace it.

### Connection test: `src/application/connection_test_service.rs`

- No code change is expected. The service builds the provider from the connection through `provider_factory`, so the provider carries the effort (D7). The test of the "Test" button on unsaved form values uses `ConnectionForm::into_config`, which must copy the field (see the next section).

### Settings form: `src/adapters/driving/http/settings/`

- `handlers/settings.rs`:
  - Add `#[serde(default)] pub conn_reasoning_effort: String` to `ConnectionForm`.
  - Add `resolved_reasoning_effort()`, after the pattern of `resolved_max_context_tokens()`. An empty value gives `None`. A known value gives `Some`. Any other value is refused with 400 and the message "Reasoning effort must be one of the listed values, or empty for the provider default".
  - `into_config` and `edit_connection_handler` set the field.
- `templates/settings.rs`: in `ConnectionFormTemplate`, add a "Reasoning effort" select after "Context window", with the options from a new `SelectOptionView::reasoning_efforts(selected)` in `src/adapters/driving/http/view_models.rs`. The first option is "Provider default" with an empty value. Render it with `select_options_html`, the same as the provider select.

### Failure line: `src/domain/model/state/generation_status.rs`

- Change the `TokenBudgetSpent` text to the D9 text. Also update the assertion in `tests/http/failure_display.rs:314`.
- **Risk:** The comment on `line` says the 240px status column clamps the line to two lines. The D9 text is about as long as the current text (58 characters), so it is expected to fit (**inferred** from `assets/styles.css`). Check the line in the browser (step 7). If the line ends in an ellipsis, stop and ask the owner.

### Data and schema

- `data/schemas/settings.schema.json`: add `reasoning_effort` to the connection properties, as a `string` with the seven values in an `enum` and a description.
- `data/settings.json`: add `"reasoning_effort": "none"` to `deepseek-v4-flash` (D10).
- **Known limit:** The seed only creates the settings row when the DB has none (`src/adapters/driven/storage/settings.rs`). An existing dev DB keeps its row. On that DB, set the effort once in the Settings form.

### Docs

- `docs/diataxis/reference/narrative/narration_system.md`, line 12: add a sentence about the reasoning effort next to the context-window sentence. Include the rule "empty sends no field, so the model default applies".

## Steps

Work test first (`tdd` skill). Write each spec scenario before its test.

1. **Domain enum and field.** Add unit tests in `src/domain/model/settings_tests.rs`:
   - Serde round trip of each value, and `Off` <-> `"none"`.
   - An old connection JSON with no field reads as `None`.
   - A `None` field does not serialize.
   - `FromStr` refuses an unknown value.

   Then add the enum and the field, and fix all literals. Run `python build.py clippy`.
2. **Payload.** In `request_tests.rs`, add one test that checks `Some(High)` gives `"reasoning": {"effort": "high"}` and one test that checks `None` gives no `reasoning` key. Then change the transport and the two providers.
3. **Provider wiring test.** The OpenRouter URL is a constant, so only the Ollama path can be tested end to end. In `ollama_tests.rs`, start a `std::net::TcpListener` stub on `127.0.0.1:0`. The stub reads one request and returns a fixed chat-completions answer. Point an Ollama connection with an effort at the stub, call `complete`, and assert that `raw_request_json` holds the effort. Both backends use the same `call_chat_completions`, so this also covers most of the OpenRouter path. **Known:** no test in the repo stubs an LLM endpoint yet. The tokio listeners in `tests/test_utils/` serve the engine, not a provider. Review question R2 decided to keep this step: a missed field is silent, and the stub is the only automated proof that a provider passes it. Keep the stub inside `ollama_tests.rs`, and use no new dependency.
4. **Settings form.** Add scenarios to `docs/specs/settings.md`, after 20.28, under "POST /connections/{id}/edit — update":
   - **20.29** The reasoning effort field stores the chosen effort: an edit with `high` returns 200, the stored connection holds `High`, and a following `GET /fragment/connections/{id}/edit` renders `high` as the selected option.
   - **20.30** An empty reasoning effort field stores no effort: an edit of a connection with effort `high` and an empty field stores `None`, and the edit form selects "Provider default".
   - **20.31** An unknown reasoning effort is refused: an edit or an add with `turbo` returns 400 with the refusal message of the Settings form section, and the stored connections do not change.

   Write the tests in `tests/http/settings.rs` with the `// [docs/specs/settings.md] SCENARIO: 20.xx` tags. Then change the handler, the view model and the template.
5. **Failure line.** Update Scenario 38.7 in `docs/specs/failure_display.md` if its words no longer fit. It already says that the line "names ... only an action the dashboard offers", and the Settings form now offers this action. Then change the text and the assertion.
6. **Data, schema and docs.** Edit the two JSON files and the reference doc. Run `python build.py validate-docs` and the data check of the full gate.
7. **Gate and check.**
   - Run `python build.py --no-browser`, then `python build.py`.
   - In the dev server (`python build.py run`), set `none` on the `deepseek-v4-flash` connection. Use "Test", then play one Action, and confirm that the LLM message forensics show `"reasoning":{"effort":"none"}`.
   - Look at the D9 failure line in the status column (see the risk above).

## Review notes (`improve-ai-plan`, 2026-10-22)

**Scope.** The plan touches about 15 files, which is more than the 8-file smell line of the review. A smaller first change (domain, transport and seed only) does not fix the owner's problem, because the existing dev DB ignores the seed and needs the form. Most of the files get only one mechanical `reasoning_effort: None` line.

**What already exists, and the plan reuses it:**
- `NarratorMode` is the pattern for the enum (`as_str`, `FromStr`, `display_label`).
- `ConnectionForm::resolved_max_context_tokens` is the pattern for the form parse and its 400 refusal.
- `SelectOptionView::providers` and `select_options_html` are the pattern for the select.
- `ConnectionTestService` already builds the provider from the connection, so D7 needs no service change.
- The forensics `raw_request_json` already shows the full request body.

**Failure modes:**

| Code path | Failure | Result |
|---|---|---|
| Form parse | Unknown value | 400 refusal. Nothing is stored. |
| Settings read | Unknown value in the stored JSON or the seed | The settings read fails, the same as an unknown `provider`. The schema check catches it in the seed. |
| OpenRouter call | The model refuses the value | **Inferred:** a 400 or a silent ignore. A 400 shows as a normal failure line and fails the connection test (D7). A silent ignore shows only in the forensics. |
| Ollama call | An old Ollama release and a model without thinking | **Inferred:** a 400, which shows as a normal failure line. |
| Provider wiring | The provider does not pass the field | A silent miss. Only step 3 (if kept) and the manual check of step 7 catch it. |

**Unresolved decisions:** none. R1 and R2 are decided (see below).

## Out of scope

- The native DeepSeek API mapping (`thinking: {type}`, `reasoning_effort`). It comes with the real `DeepSeek` backend.
- Reading `supported_efforts` from the OpenRouter `/models` API to filter the select.
- A reasoning token budget (`reasoning.max_tokens`).
- A `max_tokens` input on the connection form.

## Review questions

- **R1.** Decided: Option A, the shorter D9 text.
- **R2.** Decided: Option A, keep step 3 (the Ollama stub test).
