# Plan: One chat-completions backend, the OpenCode Go provider, and no DeepSeek stub

**Date:** 2026-10-30. Reviewed with `improve-ai-plan` on 2026-10-31. Owner decisions added on 2026-10-31.
**Status:** Done on branch `opencode-go-provider` (2026-10-31). The manual dev-server checks of Phases 1, 3 and 4 passed.
**Goal:**
- One backend sends every OpenAI-compatible call, and a provider is data.
- The engine can reach OpenCode Go.
- The reasoning effort works on every provider.
- The `DeepSeek` stub is gone.
- An OpenRouter Connection can set a provider routing order.

## Origin

This plan comes from a brainstorm session with the owner (2026-10-11, continued 2026-10-30). The session compared OpenRouter, Ollama, OpenCode Go, NanoGPT, Neuralwatt and the SillyTavern source (commit `06bde939`).

NanoGPT and Neuralwatt were examples. They show that the provider shape fits more hosts, but this plan does not add them.

The reasoning-effort plan (commit `57042737`, since deleted from `old-docs/`) put two items out of scope: the `max_tokens` form field and the `supported_efforts` filter. This plan keeps them out of scope.

## Terms

- **Connection** (`CONTEXT.md`): a named way to reach one model. The code type is `LlmConnection`.
- **Provider**: the host that a Connection reaches. The code type is `LlmBackendType`, and the field is `LlmConnection::provider`. The dashboard already uses the word "provider".
- This plan does not use "preset". `PromptPreset` already owns that word.

## Facts

Labels: **known** means a live probe or the code. **docs** means the provider documentation only. **inferred** means not checked.

### The providers

OpenRouter, Ollama and OpenCode Go are in scope. The NanoGPT and Neuralwatt columns are design inputs only. They check that the provider data covers a fourth and a fifth host without new code.

| Point | OpenRouter | Ollama | OpenCode Go | NanoGPT (input) | Neuralwatt (input) |
|---|---|---|---|---|---|
| Default base URL | `https://openrouter.ai/api/v1` | `http://localhost:11434/v1` | `https://opencode.ai/zen/go/v1` | `https://api.nano-gpt.com/api/v1` | `https://api.neuralwatt.com/v1` |
| Path | `{base}/chat/completions` (known) | same (known) | same (known) | same (docs) | same (docs) |
| Auth | Bearer (known) | none (known) | Bearer (known) | Bearer (docs) | Bearer (docs) |
| Session header | none | none | `x-opencode-session`, **required**. Without it the answer is HTTP 400 `MissingSessionID` (known). It accepts a Connection id such as `conn-1793000000000-a1b2c3d4` or `deepseek-v4-flash` (known, probe of 2026-10-31) | none | `X-NW-Conversation-ID`, optional (docs) |
| Attribution headers | `X-Title`, `HTTP-Referer` (code) | none | Its docs ask for a client `User-Agent` | none | none |
| Model ID shape | `vendor/model` | tag | bare (`deepseek-v4.1-flash`) | `vendor/model` | bare |
| Flat `reasoning_effort` | works (known) | accepted (Ollama source, `openai/openai.go`) | works (known) | accepted (docs) | the documented field (docs) |
| Nested `reasoning: {effort}` | works (known) | accepted (Ollama source) | **ignored without an error** (known, probe of 2026-10-30) | accepted (docs) | not documented |
| `provider` routing object | yes (known) | no | no | yes, but it bills pay-as-you-go (docs) | no |
| Where the answer is | `choices[0].message.content` on all five | | | | |

### The current code (known, 2026-10-31)

**Backends:**

- `OpenRouterBackend` (`providers/openrouter.rs`) and `OllamaBackend` (`providers/ollama.rs`) both call `call_chat_completions`. They differ in four points:
  - OpenRouter sends a key and the title `"Chronicler Engine"`. Ollama sends neither.
  - With `single_user_message`, OpenRouter sends an empty system prompt and the merged user text. Ollama sends the system prompt **and** the merged user text, so the system prompt goes out twice.
  - Ollama applies the Gemma-4 closed-channel marker (`OllamaBackend::preprocess_user_text`).
  - `name()` returns `"OpenRouter"` or `"Ollama"`. The LLM message forensics store this value as `backend_name`. `GameViewQuery` and the debug handler show it.
- `DeepSeekBackend` (`providers/deepseek.rs`) is a stub. Its `complete` returns `EngineError::Config` ("not yet implemented").

**Request:**

- `build_request_payload` (`transport/utils/request.rs`) sends the nested `"reasoning": {"effort": …}`.
- `configure_request` sends `Authorization`. With a title it also sends `X-Title` and `HTTP-Referer`. It sends no `User-Agent`.
- `ChatCompletionsMock` (`src/test_support/chat_completions_mock.rs`) records the request body only, not the headers.

**Provider data:**

- `LlmBackendType` has `OpenRouter`, `DeepSeek`, `Mock` and `Ollama`. It carries `/// [TRIVIAL_ENUM]` and already has a method (`api_key_env_var`). The `enum variant docs` guardrail allows the marker on an enum with methods, as long as the variants are bare.
- The per-provider rules are `match` arms in four places: `LlmBackendType::api_key_env_var`, `LlmConnection::resolve_base_url`, `LlmConnection::resolve_max_context_tokens` and `provider_from_config` (`src/bootstrap/wiring.rs`).
- `DeepSeek` takes the OpenRouter env var, the OpenRouter URL and the OpenRouter context window. `check_api_key_available_with` has a guard that skips the key check for `DeepSeek`. Its comment says to delete the guard when the stub goes.
- The provider names are in three separate lists:
  - `FromStr for LlmBackendType`;
  - `SelectOptionView::providers` (`view_models.rs`);
  - a `match` in `ConnectionFormTemplate` (`templates/settings.rs`).
- The Connections panel shows the provider with `{:?}`. `check_api_key_available` also uses `{:?}`. A new variant `OpenCodeGo` would show as "OpenCodeGo".
- The stored JSON and `data/schemas/settings.schema.json` use the serde variant names (`"OpenRouter"`). The form uses the lowercase wire ids (`"openrouter"`).

**Keys:**

- `src/main.rs` and `tests/llm/flow_llm_tests.rs` call `dotenv::dotenv()`. A key in `.env` therefore reaches the engine as an environment variable.
- `OPENROUTER_API_KEY` is the env fallback of an OpenRouter Connection with no stored key. The fallback goes through `LlmConnection::resolve_api_key_with`.
- The owner put `OPENCODE_GO_API_KEY` in `.env` on 2026-10-31.

**Callers and wiring:**

- `LlmProvider::complete` takes no Game identity. Two callers call it:
  - `LlmCallRecorder::complete`, which the narrator, the Options agent and the quantifier call;
  - `ConnectionTestService::test_connection`.
- The production chain to a backend has two branches:
  - Play: `provider_from_config`, then the `ProviderResolver` in `recorder_with_storage` (`wiring.rs`), then `LlmCallRecorder::complete`.
  - The connection test: the "Test" button, then `ConnectionTestService` with its `provider_factory`.
- Connection ids: the add handler makes them with `generate_storage_id("conn")` (`conn-<millis>-<8 hex>`). Seeded Connections have readable ids (`deepseek-v4-flash`). The edit handler keeps the id.
- `edit_connection_handler` rebuilds the Connection from the form. It copies only `max_tokens` from the stored Connection, because the form does not own that field.

**The DeepSeek references (2026-10-31), excluding the `deepseek-v4-flash` model name:**

- `providers/deepseek.rs` and `providers/deepseek_tests.rs`;
- `providers/mod.rs`;
- `llm_backend.rs` and `llm_backend_tests.rs`;
- `settings.rs` and `settings_tests.rs`;
- `wiring.rs` and `wiring_tests.rs`;
- `view_models.rs`;
- `templates/settings.rs`;
- `handlers/settings_tests.rs`;
- `data/schemas/settings.schema.json`;
- `docs/diataxis/reference/architecture_system.md` and `docs/diataxis/explanation/architecture.md`;
- `tests/http/requires_migration/connections.rs` (`test_add_connection_deepseek`, and the module summary), which feeds `tests/AGENTS.md`.

None of the seven dev DBs in `target/debug/chronicler_*.db` stores a Connection with the provider `DeepSeek`. They store `OpenRouter`, `Ollama` and `Mock` only. Removing the variant therefore breaks no stored settings.

**Tests:**

- `openrouter_tests.rs`, `ollama_tests.rs` and `request_tests.rs` are seam contracts. Each test carries one `// CASE <id>` tag. `scripts/validate_seam_cases.py` refuses a duplicate case id.
- Three files each pin the reasoning field: `transport-reasoning-effort-sent`, `openrouter-reasoning-effort-sent` and `ollama-reasoning-effort-sent`, plus an `-absent` case for each.
- `tests/http/requires_migration/` is a quarantine. `scripts/validate_feature_spec.py` pins its untagged count (`REQUIRES_MIGRATION_TEST_COUNT = 68`), and the count may only go down. `docs/plans/drain-requires-migration-quarantine.md` owns that folder.
- Only five `LlmConnection` literals list every field. The other literals use `..test_connection(...)`. A new field with a serde default touches those five.
- `tests/llm/` is the only binary that calls a real provider. It runs only under `python build.py --llm-only`, and it is gated on `OPENROUTER_API_KEY`.

## What already exists

| Need | Reuse |
|---|---|
| The pattern for a closed set with names: wire id, label, list, `FromStr` | `ReasoningEffort` (`as_str`, `display_label`, `ALL`, `FromStr`) |
| A stub endpoint for the seam tests | `ChatCompletionsMock`. Phase 1 adds header capture |
| The test Connection fixture | `test_support::test_connection` |
| The provider select | `SelectOptionView` + `select_options_html` |
| The form parse that refuses a bad value with 400 | `ConnectionForm::resolved_reasoning_effort` and `resolved_max_context_tokens` |
| A stable, unique, non-secret value for the session header | `LlmConnection::id` |
| The env-var fallback and the missing-key error | `api_key_env_var`, `resolve_api_key_with`, `check_api_key_available_with` |
| Loading `.env` | `dotenv::dotenv()` in `src/main.rs` and in `tests/llm` |
| The live provider test, gated on a key | `tests/llm/flow_llm_tests.rs` (`has_llm_api_key`, `with_real_llm`) |

## Decisions

| # | Decision | Source |
|---|---|---|
| D1 | One backend, `ChatCompletionsBackend`, sends every chat-completions call. `OpenRouterBackend` and `OllamaBackend` go away. `MockBackend` stays. | Owner, 2026-10-30 |
| D2 | Each provider is data on `LlmBackendType`: wire id, label, default base URL, API-key env var, attribution title, session header, default context window, and routing support. The `match` arms and the three name lists move into these methods. | Owner, 2026-10-30. The name lists are a review fix |
| D3 | The request sends the flat `reasoning_effort`, not the nested `reasoning` object. It is the one form that OpenRouter, Ollama and OpenCode Go all honour. | Owner, 2026-10-30 |
| D4 | Every request sends `User-Agent: chronicler-engine/<CARGO_PKG_VERSION>`. | Owner, 2026-10-30 |
| D5 | Add the provider `OpenCodeGo`. NanoGPT and Neuralwatt are not added. | Owner, 2026-10-31 |
| D6 | The env fallback of an OpenCode Go Connection is `OPENCODE_GO_API_KEY`. It works the same as `OPENROUTER_API_KEY`: a stored key on the Connection wins, and the env var is used when the Connection has no key. | Owner, 2026-10-31 |
| D7 | The session header value is the Connection id (R1 option A). | Owner, 2026-10-31 |
| D8 | Delete the `DeepSeek` stub: the variant, `DeepSeekBackend`, its tests, its key-check guard, and every arm and list entry. No migration is needed: no stored Connection uses it (known). | Owner, 2026-10-31 |
| D9 | An OpenRouter Connection can store a routing order and a "use only these providers" flag. With an order set, the body gets `"provider": {"order": […], "allow_fallbacks": <not the flag>}`. With no order set, no `provider` field goes out. Only OpenRouter supports routing. | Owner, 2026-10-30. R2 is moot after D5. The flag shape is a review fix |
| D10 | The backend reads only `choices[0].message.content`. The answer rule in `parse_chat_response` does not change. | Code, already true |
| D11 | The Gemma-4 marker applies only when the provider is Ollama, as now (was O3). | Owner, 2026-10-31 |
| D12 | OpenCode Go names: serde name `OpenCodeGo`, wire id `opencode-go`, label "OpenCode Go", default context window 32768 (was O4). | Owner, 2026-10-31 |
| D13 | With `single_user_message` on, every provider sends one user message (`merge_single_user_message`) and no system message: the OpenRouter rule. Ollama no longer sends the system prompt twice (was O2). | Owner, 2026-10-31 |
| D14 | The Phase 4 routing fields always show on the Connection form, with the hint "OpenRouter only". Other providers ignore them. No script (was O5, option A). | Owner, 2026-10-31 |

## Open questions

None. The owner decided O2 to O5 on 2026-10-31 (D11 to D14). R1 became D7. R2 was dropped with D5.

## Phases

Work test first (`tdd` skill). Each phase is one branch and passes `python build.py` before the next phase starts. Phase 1 can ship alone.

### Phase 1 — Request fixes

1. Change these six cases to the flat field:
   - `transport-reasoning-effort-sent` and `transport-no-reasoning-effort-omits-field` (`request_tests.rs`);
   - `openrouter-reasoning-effort-sent` and `openrouter-reasoning-effort-absent` (`openrouter_tests.rs`);
   - `ollama-reasoning-effort-sent` and `ollama-reasoning-effort-absent` (`ollama_tests.rs`).

   `Some(effort)` gives `"reasoning_effort": "<value>"` on the wire and in `raw_request_json`, and no `reasoning` key. `None` gives neither key. Update the case descriptions.
2. Change `build_request_payload` to set `payload["reasoning_effort"]`. Update the field doc on `ChatCompletionRequest::reasoning_effort`.
3. Add `received_headers()` to `ChatCompletionsMock`. The mock already reads the header block before the body. Add the case `transport-user-agent-names-the-engine`: the request carries `User-Agent: chronicler-engine/<version>`. Then set the header in `configure_request` with `env!("CARGO_PKG_VERSION")`.
4. Run `python build.py test-pattern "reasoning" --test-target lib`, then `python build.py --no-browser`.
5. Check by hand: in the dev server, a Connection at effort `none` shows `"reasoning_effort":"none"` in the LLM message forensics.

### Phase 2 — Delete the DeepSeek stub, then one backend

1. **Delete the DeepSeek stub (D8).** Do this first, because every later step then has one arm fewer.
   - Delete `providers/deepseek.rs`, `providers/deepseek_tests.rs` and their lines in `providers/mod.rs`.
   - Remove the `DeepSeek` variant and its arms in `llm_backend.rs`, `settings.rs`, `wiring.rs`, `view_models.rs` and `templates/settings.rs`.
   - Remove the `DeepSeek` guard and its comment in `check_api_key_available_with`.
   - Remove `"DeepSeek"` from the `provider` enum in `data/schemas/settings.schema.json`.
   - Tests:
     - Delete the DeepSeek cases in `wiring_tests.rs` and `llm_backend_tests.rs`.
     - In `settings_tests.rs` and `handlers/settings_tests.rs`, change the cases that use `DeepSeek` or `"deepseek"` as "a second provider" to use `Ollama`.
     - Keep the case that `"deepseek"` now fails `FromStr`. It pins the unknown-backend error.
   - `tests/http/requires_migration/connections.rs`: delete `test_add_connection_deepseek`. The OpenRouter add test in the same file covers the add path. Lower `REQUIRES_MIGRATION_TEST_COUNT` from 68 to 67. Remove "DeepSeek" from the module summary, then regenerate `tests/AGENTS.md`.
   - Docs:
     - In `docs/diataxis/explanation/architecture.md`, delete the `deepseek` node and its edge from the diagram.
     - In `docs/diataxis/reference/architecture_system.md`, remove DeepSeek from the "Outbound LLM calls" line.
   - Check: `rg -n "DeepSeek\b|DeepSeekBackend|\"deepseek\"" src tests data docs/diataxis docs/specs` finds only the `FromStr` refusal case.
2. Give `LlmBackendType` the `ReasoningEffort` pattern. Write the unit tests in `llm_backend_tests.rs` first.
   - Add `wire_id`, `display_label`, a `SELECTABLE` list (no `Mock`), `default_base_url`, `default_max_context_tokens` and `attribution_title`.
   - `FromStr` reads `wire_id`. `SelectOptionView::providers` iterates `SELECTABLE`. The `match` in `ConnectionFormTemplate` calls `wire_id`.
   - The Connections panel and `check_api_key_available` use `display_label`, not `{:?}`.
   - `resolve_base_url` and `resolve_max_context_tokens` call the new methods.
   - Keep `[TRIVIAL_ENUM]`. The variants stay bare.
3. Write `providers/chat_completions.rs` with `ChatCompletionsBackend::from_config`.
   - Line 1 is `//! [DOC: docs/diataxis/reference/narrative/narration_system.md]`. Line 2 is the summary.
   - Register `chat_completions_tests.rs` in `providers/mod.rs` under `#[cfg(test)]`.
4. Move the cases of `openrouter_tests.rs` and `ollama_tests.rs` into `chat_completions_tests.rs`, with new `chat-completions-*` ids.
   - Drop the four provider reasoning cases. The two transport cases already pin the field, and the backend passes the value through. Keep one case `chat-completions-reasoning-effort-passed`.
   - Add one case for each difference that the provider now carries:
     - the key and the title for OpenRouter;
     - no key and no title for Ollama;
     - the `single_user_message` rule (D13): with the box on, Ollama sends no system message;
     - the Gemma-4 marker for Ollama only (D11);
     - `name()` returns `display_label`.
5. `provider_from_config` sends `OpenRouter` and `Ollama` to `ChatCompletionsBackend`. Delete `openrouter.rs`, `ollama.rs` and their test files.
6. Run `python build.py integration`, `python build.py guardrails` and `python scripts/validate_seam_cases.py`. The behaviour must not change, except for D8 and D13.

### Phase 3 — OpenCode Go

1. Add the variant `OpenCodeGo` with the D12 names. It needs an arm in every Phase 2 method:
   - `api_key_env_var` returns `Some("OPENCODE_GO_API_KEY")` (D6);
   - `default_base_url` returns `https://opencode.ai/zen/go/v1`;
   - `attribution_title` returns `None`.

   Add `"OpenCodeGo"` to the `provider` enum in the schema. Unit tests in `llm_backend_tests.rs`:
   - each selectable variant round-trips through `wire_id` and `FromStr`;
   - `OpenCodeGo` has its base URL and env var;
   - an `OpenCodeGo` Connection with no stored key resolves the key from `OPENCODE_GO_API_KEY` (through `resolve_api_key_with`, with a fake lookup);
   - a stored key wins over the env var.

   No new spec scenario: a provider list is a value, and values live in the code (`docs/AGENTS.md`, "Where a fact lives").
2. Add `LlmBackendType::session_header`: `Some("x-opencode-session")` for `OpenCodeGo` and `None` for the others. `ChatCompletionsBackend::from_config` keeps `connection.id` and sends it as the header value (D7).
3. Add seam cases with `received_headers()`:
   - `chat-completions-opencode-go-session-header`: an OpenCode Go Connection sends `x-opencode-session: <connection id>`;
   - `chat-completions-no-session-header`: an OpenRouter Connection sends no session header.
4. Live test in `tests/llm/flow_llm_tests.rs`. It runs only under `python build.py --llm-only`.
   - Add one test gated on `OPENCODE_GO_API_KEY`, after the pattern of `has_llm_api_key`. It does one `complete` call to `deepseek-v4.1-flash` at effort `none`.
   - It asserts a non-empty answer and 0 reasoning tokens in the raw response.
   - This is the outside check that the session header, the key and the flat field work on the real host.
5. Docs: in `docs/diataxis/reference/architecture_system.md`, make the "Outbound LLM calls" line read "an OpenAI-compatible chat-completions host (OpenRouter, OpenCode Go, Ollama)".
6. Check by hand: with `OPENCODE_GO_API_KEY` in `.env`, add an OpenCode Go Connection with no stored key, the model `deepseek-v4.1-flash` and effort `none`. Use "Test", play one Action, and confirm 0 reasoning tokens in the forensics.

### Phase 4 — Provider routing (OpenRouter)

1. Add `LlmBackendType::supports_routing`: true for `OpenRouter` only.
2. Add two fields to `LlmConnection` and to the schema, each with a serde default and skipped when empty:
   - `provider_order: Vec<String>`;
   - `pin_provider_order: bool`.

   The form owns both fields, so `into_config` sets them on add and on edit.
3. The checkbox is "Use only these providers" (`pin_provider_order`), not "Allow fallbacks".
   - An HTML form sends no value for an unchecked box. `CODING_STANDARDS.md` requires the form structs to accept a partial body.
   - With this name, an absent box keeps the OpenRouter default. The body sends `allow_fallbacks: !pin_provider_order`.
4. Add the two fields to `ChatCompletionRequest`. Seam cases:
   - An order on OpenRouter sends `provider.order` and `allow_fallbacks: true`.
   - A pinned order sends `allow_fallbacks: false`.
   - An empty order sends no `provider` key.
   - An order on Ollama or OpenCode Go sends no `provider` key.
5. Form: a "Provider order" text field (comma-separated; trimmed; empty entries dropped) and the checkbox, always shown, with the hint "OpenRouter only" (D14). Add spec scenarios 20.50 to 20.52 to `docs/specs/settings.md`, with their tests in `tests/http/settings_connection_form.rs`:
   - The order field stores the typed order.
   - An empty order field stores no order.
   - The form test sends the typed order and pin to the provider.
6. Check by hand on OpenRouter with the order `deepinfra`. The `provider` field of the response names the host that served the call.

## Failure modes

| Code path | Failure | Result | Handled by |
|---|---|---|---|
| Flat `reasoning_effort` | A provider ignores it without an error | The model reasons, and `TokenBudgetSpent` can return | The manual check in each phase, and the live test of Phase 3 |
| Flat `reasoning_effort` | A model refuses the value (for example, `none` on a model with mandatory reasoning) | HTTP 400 on each call | The connection test sends the effort, so "Test" fails before play |
| OpenCode Go key | No stored key and no `OPENCODE_GO_API_KEY` | `check_api_key_available` names the env var before the call | Existing code, after D6 |
| OpenCode Go key | `.env` changes while the dev server runs | **inferred:** the server reads `.env` once at start, so the new key applies only after a restart | None. The env var is read when the Connection resolves, so a restart is enough |
| OpenCode Go session (D7) | One session for all Games of a Connection | Weaker prompt caching. Nothing fails | The Game id option, later |
| OpenCode Go model only on `/v1/messages` or `/v1/responses` (Claude, MiniMax, Qwen, GPT, Grok) | Not on `/chat/completions` | An HTTP error on the call and on "Test" | Out of scope. The error text names the HTTP status |
| DeepSeek removal | A Connection stored with `"provider": "DeepSeek"` | The settings read fails | No stored Connection uses it (known, the seven dev DBs). `CODING_STANDARDS.md` asks for no backward compatibility |
| Routing order | An unknown provider slug | **inferred:** an OpenRouter 400 or 404 | Shows as a normal failure line and fails "Test" |
| Routing order, pinned | The pinned host is rate-limited | HTTP 429 and a lost generation (known, probe of 2026-10-11 with `deepinfra`) | The pin is off by default. The form hint says what it does |
| Phase 2 merge | `name()` changes | Old and new forensics rows show different backend names | The `display_label` case of Phase 2 step 4 |

## Out of scope

- The NanoGPT and Neuralwatt providers. With this plan, each one is a new variant with its arms, and Neuralwatt also gets a `session_header` value.
- A "Custom" provider with free-form extra headers and body fields (the SillyTavern `custom_include_headers` model).
- A native DeepSeek backend.
- A `max_tokens` field on the Connection form.
- Filtering the effort select by the OpenRouter `supported_efforts` metadata.
- The `/v1/messages` and `/v1/responses` endpoints, and the OpenCode Go models that only those endpoints serve.
- A session per Game (R1 option C).
- Streaming.
- Recording the routed host (the OpenRouter response `provider` field) in the forensics. The raw response already holds it.

## Done when

- `DeepSeekBackend` and the `DeepSeek` variant do not exist.
- `OpenRouterBackend` and `OllamaBackend` do not exist. `ChatCompletionsBackend` serves OpenRouter, Ollama and OpenCode Go.
- `LlmBackendType` holds the only list of provider names.
- Every request sends the flat `reasoning_effort` and the `User-Agent`.
- An OpenCode Go Connection with no stored key uses `OPENCODE_GO_API_KEY`, and it plays an Action at effort `none` with 0 reasoning tokens.
- An OpenRouter Connection can store and send a routing order.
- `python build.py` passes, and `python build.py --llm-only` passes with both keys set.

## Review notes (`improve-ai-plan`, 2026-10-31)

Fixes made in the plan:

| # | Finding | Fix |
|---|---|---|
| F1 | The plan said `LlmProvider::complete` has five call sites. It has two: `LlmCallRecorder` and `ConnectionTestService`. The three agents call `LlmCallRecorder::complete` | Facts corrected |
| F2 | The plan said the `[TRIVIAL_ENUM]` marker must go when methods are added. The guardrail allows the marker on an enum with methods | Keep the marker |
| F3 | Phase 1 missed `ollama_tests.rs`, which also pins the nested field | All six cases listed by id |
| F4 | Three separate provider-name lists, and the panel shows `{:?}` | One list on `LlmBackendType`, after the `ReasoningEffort` pattern |
| F5 | Phase 3 put the wire ids in the JSON schema. The schema uses the serde names | Schema step corrected |
| F6 | Phase 3 added a spec scenario for one provider value. A value lives in the code (`docs/AGENTS.md`) | Replaced with a unit round-trip test |
| F7 | A stored `session_id` field adds nothing over the existing Connection id | The owner chose the Connection id (D7). Go accepts it (known) |
| F8 | An "Allow fallbacks" checkbox reads as false when the form omits it | Inverted to "Use only these providers" |
| F9 | Four provider reasoning cases repeat the two transport cases | One pass-through case remains |
| F10 | The new module needs the DOC anchor lines and a test-module registration | Added to Phase 2 step 3 |
| F11 | NanoGPT routing has a billing effect | Moot: NanoGPT is out of scope (D5) |

Changes after the owner decisions of 2026-10-31:

- The scope is now OpenRouter, Ollama and OpenCode Go. NanoGPT and Neuralwatt stay in the facts table as design inputs only.
- Added D6 (`OPENCODE_GO_API_KEY`), D7 (the Connection id), and D8 with the Phase 2 step 1 deletion.
- R2 dropped. Routing is OpenRouter only.
- Added the gated live test of Phase 3. It is the outside check on the real host.

Blast radius:

- Phase 2 step 1 touches the 16 files in the DeepSeek list, plus the quarantine pin.
- Phase 2 then changes the backend that both production chains build: play and the connection test.
- Phase 3 adds one arm to every `LlmBackendType` method.
- Phase 4 touches the five full `LlmConnection` literals, `ConnectionForm`, `ConnectionFormTemplate` and the schema.
- No storage migration is needed. The settings are one JSON column, and the new fields have serde defaults.
