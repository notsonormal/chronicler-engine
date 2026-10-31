# Put the call parameters on the presets

Type: task (AFK)
Status: open
Blocked by: 21

## Question

Give each preset its generation parameters, and give the Options preset its count. Make the Connection `max_tokens` a ceiling. Carry the parameters and the `GameId` on every call, and send an OpenRouter `session_id`.

Sources: [Decide the parameters of each call](07-call-parameters.md), and fix 4 of [Decide which parts of a Quantifier or Options call an author can change](10-quantifier-options-layout.md) (the Options count).

It waits for [Add the connection slots and remove the global response length](21-connection-slots-and-response-length.md). That ticket makes `Job::budget` read the Connection of each call's slot, and the ceiling comes from that Connection.

This is the largest engine ticket. If it does not fit one session, stop after step 4 (the preset fields are stored and validated), record the state in the answer, and open a follow-up ticket for steps 5 to 12 (the fields are sent).

Done when each preset sends its parameters, the Options preset sets the number of Options, OpenRouter calls carry a `session_id`, and `python build.py` passes.

## Decisions

From [Decide the parameters of each call](07-call-parameters.md):

- `max_tokens`, typed `temperature` and `top_p`, and a free object for other samplers go on each preset kind. A value that is not set is not sent.
- The Connection keeps the model, the window, `reasoning_effort`, provider routing and a `max_tokens` ceiling.
- The free object is refused when it holds `model`, `messages`, `stream` or `max_tokens`. The error names the key.
- Every OpenRouter call sends a `session_id`, a hash of the `GameId`, taken from the Job and never from `ActiveGame`.
- No override for each Game, and no cache setting.

From ticket 10: `option_count` on the Options preset, default 3, used by `{{option_count}}` and by the parser.

Choices made when the work was ticketed ([Write the build plan for the engine changes](09-build-plan.md)), corrected by the plan review of 2026-10-31:

- **Shape.** `PromptPreset` gets `parameters` (`GenerationParameters { max_tokens, temperature, top_p, custom }`, where `custom` is a `serde_json::Map`) and `option_count: Option<u32>`.
  - One JSON column holds `parameters` (ME keeps one `parameters` object too). `option_count` is its own column.
  - `parameters` has `#[serde(default)]`, because a seed file that sets no parameters means "no parameters". It is not for old stored data.
  - A new `src/domain/model/generation_parameters.rs` holds `GenerationParameters` and `CallSettings { max_tokens, temperature, top_p, custom, game: Option<GameId> }`. `CallSettings` derives `Default`. They live in the domain because the domain `AgentSettings` holds them, and `arch-lint` forbids `domain/model/` from using the application layer.
- **Validation.** `PromptPreset::validate` refuses the four `custom` keys, an `option_count` of 0, and an `option_count` on a preset that is not an Options preset. The seed and `PromptPresetService::save_preset` both call it.
- **A refused seed file** keeps today's behaviour: startup logs a warning and goes on. The warning names the file and the refused key. A fatal start would change the startup contract for every data error, which is not this ticket's goal.
- **Merge order.** The request writes `custom` first and the engine keys after it, so an engine key always wins.
- **The ceiling (owner, 2026-10-31, O5 (a): a cap).** `LlmConnection::max_tokens` becomes `max_tokens_ceiling`. `LlmConnection::max_tokens_for(preset: Option<u32>) -> u32` is the lower of the preset value (or 2048) and the ceiling (or no limit). One domain constant holds 2048, next to `max_tokens_for`. The transport and `fit_messages_to_context` import it, and the second constant goes.
- **What each call sends as `max_tokens`:**
  - a Narrator-role call: `max_tokens_for` of its preset, then the clamp to the free context space. The clamp can lower the preset value: the preset value is a request, not a guarantee;
  - the Quantifier and Options: `max_tokens_for` of their preset, with no clamp, as today;
  - the connection test: 8, and nothing else, so it stays cheap.
- **The call.** `CallSettings` replaces the `max_tokens` argument of `LlmProvider::complete` and `LlmCallRecorder::complete`. `Narrator::narrate` takes the preset's parameters and the `GameId`.
- **`AgentSettings`** becomes `{ preset: Option<String>, call: CallSettings, option_count: Option<u32> }`.
- **The Options count reaches the text** through `TemplateVars::with_option_count(count)`, used in `options/prompt.rs`. `generate_options` takes `settings.option_count`, else `DEFAULT_OPTION_COUNT`, for the text and for the parser.
- **`session_id`** is a UUID v5 of the `GameId` with one fixed namespace (the `v5` feature of the `uuid` crate). It is computed by a method in the chat-completions adapter, not a free function. Only OpenRouter sends it: `LlmBackendType::sends_game_session` is a separate method from `supports_routing`, because "a host reads a body session" and "a host reads a provider order" are two rules that agree only today. A call with no Game sends none. Trade-off: the `v5` feature adds one small SHA-1 crate.
- **The form.** An edit through the preset form already keeps the fields that the form does not own. Scenario 21.14 pins this for the allowed modes. It gains one line for the generation parameters. A UI for the new fields belongs to the map's fog item "A UI for the new changeable fields".

## Facts

Known on 2026-10-31 (inferred where marked).

Presets:

- `PromptPreset` is one flat struct for all four kinds. The `prompt_presets` table has one column per field: two SELECT column lists and the INSERT/UPSERT in `storage/presets.rs`, and `DbPromptPreset::from_row` and `into_preset` in `storage/models/prompt_preset.rs`. No preset schema exists in `data/schemas/`, and `validate_data.py` does not read `data/prompt_presets/`.
- `PromptPreset` has 24 literals that list every field, in 10 files, plus `Self { … }` in its `Default` impl: `tests/storage/preset_storage.rs` (7), `fixtures.rs` (3), `bootstrap/run_tests.rs` (3), `narrator_tests.rs` (3), `builders/presets_tests.rs` (2), `tests/test_utils/stub_server.rs` (2), and one each in `bootstrap/run.rs`, `prompt_presets/handlers/prompt_presets.rs`, `storage/models/prompt_preset.rs` and `storage/presets_tests.rs`.
- `edit_preset` (`prompt_preset_service.rs`) starts from the stored preset and changes only the form fields, so a field that the form does not own is kept.
- `process_preset_file` (`bootstrap/run.rs`) calls `Storage::save_preset` past the service. When `ensure_presets` fails, startup logs "Failed to seed prompt presets" and goes on. Only a JSON parse error names the file today.
- `PromptPresetService::hook_text` returns a `String`. `PromptPresetService::for_narration` returns the whole `PromptPreset`, and `PromptPreset::assemble_text` is public. `AgentSettings` (`domain/model/agent.rs`, derives `Default`) is `{ preset: Option<String>, max_tokens: Option<u32> }`, and `Job::hook_settings` sets `max_tokens: None`. So the Quantifier and Options always send 2048, and no context clamp applies to them.
- `{{option_count}}` already renders in the Options preset text and in the fallback, always as 3: `options/prompt.rs` builds `TemplateVars::new`, which uses `DEFAULT_OPTION_COUNT`. `generate_options` sets `OptionsPromptContext.option_count` to `DEFAULT_OPTION_COUNT`, and the parser keeps at most that many. The dock renders any number.

Connections and calls:

- The Connection `max_tokens` is read in `Job::budget` and copied back in `edit_connection_handler`. The connection form does not own it. `data/settings.json` sets it on two Ollama connections: 8192 on `ollama-gemma-4-26B` and 2048 on `ollama-gemma-4-e2b`. The rename touches 13 sites: `settings.rs` (2), `settings/handlers/settings.rs` (4), the domain `settings_tests.rs` (2), the storage `settings_tests.rs` (4), `data/settings.json` (2) and `data/schemas/settings.schema.json` (1).
- Two constants mean "send 2048 when nothing is set": `DEFAULT_MAX_TOKENS` (`transport/utils/request.rs`) and `MAX_RESPONSE_TOKENS` (`token_budget.rs`, used by `fit_messages_to_context`).
- `LlmProvider::complete` and `LlmCallRecorder::complete` take no `GameId`. The Job holds it (`self.writer.game()`).
- `LlmProvider` has 10 implementations: `ChatCompletionsBackend`, `MockBackend`, `ScriptedModel`, and 7 test fakes (`quantifier/utils/orchestration_tests.rs` 5, `llm_recorder_tests.rs` 1, `connection_test_service_tests.rs` 1). 57 sites call `.complete(`, 53 of them in test code, one in `tests/llm/flow_llm_tests.rs`.
- `Narrator::narrate` has one production caller (`Job::narrate`) and 4 test callers in `narrator_tests.rs`.
- `ChatCompletionRequest` is `Copy`, with borrowed fields. It is built at 3 sites (`chat_completions.rs`, `request_tests.rs` 2). `client.rs` destructures it with `..`.
- `ScriptedModel` records a `ModelCall` (`role`, `system_prompt`, `user_prompt`, `max_tokens`). One test reads `max_tokens` from it (`narrator_tests.rs`).
- `LlmBackendType::supports_routing` is `self == Self::OpenRouter`, and gates the provider order in `ChatCompletionsBackend`. The existing `x-opencode-session` header is another thing: a header for OpenCode Go, not a body field.
- `Cargo.toml` has no hash crate. `uuid` has the `v4` and `serde` features only.
- `GameId` is `pub struct GameId(pub u64)` in `domain/model/game.rs`. No architecture rule stops `domain/model/` from using `serde_json`.
- The free-function guardrail forbids a top-level free function in `domain/model/` and in the adapter files.
- `tests/storage/preset_storage.rs` carries no `CASE` tags. One tag would make it a declaring file, and then every test in it would need one.
- The next free scenario numbers: 21.44 in `docs/specs/prompt_presets.md`, 24.25 in `docs/specs/options.md`. A Gherkin step uses glossary nouns, not field names (`tests/STRATEGY.md`, "Spec vocabulary").
- `agent_system.md`, "What an Agent gets", describes `AgentSettings`.

## What already exists

| Need | Reuse |
|---|---|
| A read of the whole preset for a hook | `PromptPresetService::for_narration` and `PromptPreset::assemble_text`. If `hook_text` is only these two, the Job calls them and `hook_text` goes |
| The default count | `DEFAULT_OPTION_COUNT` |
| The two ends of the Options count | `TemplateVars` (the text) and `parse_options(text, count)` (the cap) |
| An OpenRouter-only predicate | `supports_routing`, as the model for `sends_game_session` |
| "An edit keeps a stored field the form does not own" | Scenario 21.14 and its test in `tests/http/prompt_presets.rs` |
| A new column in a migration | `column_exists` and `ALTER TABLE … ADD COLUMN` in `storage/utils/plumbing.rs` |
| The cheap connection test | `TEST_MAX_TOKENS` in `connection_test_service.rs` |

## Steps

1. **The domain.**
   - Add `generation_parameters.rs` (line 1 `//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]`, line 2 a summary) with `GenerationParameters` and `CallSettings`, and its test file. Register both in `src/domain/model/mod.rs`.
   - Add `parameters` and `option_count` to `PromptPreset`, and `PromptPreset::validate`, with tests in the domain `prompt_preset_tests.rs`.
   - Update the 25 full `PromptPreset` literals in one commit.
2. **Storage.** A migration adds `parameters TEXT NOT NULL DEFAULT '{}'` and `option_count INTEGER` to `prompt_presets` (the map's migration rule: next free step, `column_exists`, raise `LATEST_VERSION`, a test in `plumbing_tests.rs`). Change the column lists, `from_row` and `into_preset`. Add a round-trip test, with no `CASE` tag, to `tests/storage/preset_storage.rs`.
3. **The seed.** `process_preset_file` reads both fields and calls `validate`. A refusal names the file and the key. Extend `test_ensure_presets_all_fields_mapped` in `run_tests.rs`.
4. **The service and the form.** `save_preset` calls `validate`. Add "And it preserves the stored generation parameters" to scenario 21.14 in `docs/specs/prompt_presets.md`, with one more assertion in its test. `edit_preset` needs no change.
5. **The call.**
   - `CallSettings` replaces the `max_tokens` argument of both `complete` methods. Update the 10 implementations and the 57 call sites mechanically, in place.
   - `ScriptedModel`'s `ModelCall` carries the `CallSettings`. Change the one test that reads `max_tokens` from it.
   - `ConnectionTestService` sends `CallSettings { max_tokens: Some(8), ..Default::default() }`.
   - `ChatCompletionRequest` gets the three new values (`custom` as a `&'a Map`). `build_request_payload` writes `custom` first, then the engine keys.
   - Seam cases in `request_tests.rs`, each with its `// CASE` line: `transport-samplers-sent`, `transport-unset-samplers-omitted`, `transport-custom-parameters-sent` and `transport-engine-keys-win-over-custom`. In `chat_completions_tests.rs`: `chat-completions-call-settings-passed`.
6. **The ceiling.** Rename the Connection field to `max_tokens_ceiling` at the 13 sites. Add `max_tokens_for`, with tests in the domain `settings_tests.rs`, including a Connection with no ceiling. Move the 2048 constant into the domain, and delete the other one.
7. **The Job and the agents.**
   - `Job::budget` takes the preset of the call and uses `connection.max_tokens_for(preset.parameters.max_tokens)`.
   - `Job::narrate` passes the preset's parameters and the `GameId` to `Narrator::narrate` (1 production and 4 test call sites). The Narrator builds `CallSettings` with the clamped `max_tokens`.
   - `Job::hook_settings` reads the whole preset (see "What already exists") and fills `AgentSettings`: the text, `CallSettings` with `max_tokens_for` of the hook's Connection and preset, the samplers, the `GameId`, and `option_count`.
   - Options: add `TemplateVars::with_option_count`, use it in `options/prompt.rs`, and make `generate_options` take `settings.option_count`, else `DEFAULT_OPTION_COUNT`.
   - Seam cases: `narrator-sends-the-preset-parameters`, `quantifier-sends-the-preset-parameters`, `options-sends-the-preset-parameters`, and `options-count-from-the-preset` (`{{option_count}}` renders the count, and the parser keeps at most that many).
8. **The dock.** Add scenario 24.25 to `docs/specs/options.md`, in glossary words, for example "The Options preset sets how many Options the dock offers". Its test in `tests/http/options.rs` uses a preset with a count of 2 and a model that answers with 4 suggestions.
9. **The OpenRouter session.** Add the `v5` feature to `uuid`. Add `LlmBackendType::sends_game_session` and `ChatCompletionRequest.session_id`. `ChatCompletionsBackend` computes it from `CallSettings.game`. Seam cases in `chat_completions_tests.rs`:
   - `chat-completions-openrouter-game-session`: the same `GameId` gives the same value on two calls, and two Games give two values;
   - `chat-completions-no-game-session-elsewhere`: Ollama and OpenCode Go send no `session_id`;
   - `chat-completions-no-game-no-session`: a call with no Game sends none.
10. **Docs.**
    - `prompt_system.md`, "Prompt Presets": a preset carries its generation parameters, and a value that is not set is not sent. The free object cannot set the model, the messages, the stream or `max_tokens`. An Options preset can set the number of Options.
    - `agent_system.md`, "What an Agent gets": an Agent gets the preset text, the call settings and the Options count.
    - `architecture_system.md`: the `PromptPresetService` row adds the refusal.
11. **Check by hand.** On the dev server, play one Action on an OpenRouter Connection. The LLM message shows `session_id` in `raw_request_json`, and the call succeeds. If no OpenRouter key is available, the answer says so, and the owner makes this check.
12. Run `python build.py`.

## Failure modes

| Code path | Failure | Result | Handled by |
|---|---|---|---|
| Free object guard | A seed file holds `"model"` in `custom` | Startup logs a warning that names the file and the key, and goes on. That preset is not seeded | Step 3 |
| Free object guard | A future caller saves a preset with a refused key | The save is refused, and the error names the key | `validate` in `save_preset` |
| Stored row | The `parameters` column holds bad JSON | `into_preset` fails: the presets panel renders empty, and a hook that reads the preset fails its Job | Accepted. Only a hand edit of the DB can write it |
| Preset edit | The form sends no `parameters` or `option_count` | The edit keeps the stored values | Scenario 21.14 |
| New fields on an existing database | A changed seed file with the same id | It does not reach the stored preset | Accepted (O3). Use a new preset id or a new database |
| Ceiling | A Connection with no ceiling | The preset value or 2048 is sent | `max_tokens_for` test |
| Ceiling rename | A stored Connection has `max_tokens` | The value is dropped | Accepted (ticket 07: "it can rename or reset the field") |
| Narrator clamp | The free context space is smaller than the preset value | A lower `max_tokens` is sent | Accepted. The preset value is a request |
| `session_id` | OpenRouter refuses the field | Every OpenRouter call fails | The check by hand |
| `session_id` | Two databases have a Game with the same id | They share an OpenRouter session. Only the routing is shared | Accepted |

## Not in scope

- A UI for the parameters and the count.
- A fatal start on a bad seed file.
- `CASE` tags for the rest of `tests/storage/preset_storage.rs`.
- A per-Game override of a parameter, and a cache setting (ticket 07).
- A JSON schema for `data/prompt_presets/`.

## Output

Resolved when the check by hand passes (or the answer hands it to the owner) and `python build.py` passes. The answer records the CASE ids, the scenario ids and the migration step.
