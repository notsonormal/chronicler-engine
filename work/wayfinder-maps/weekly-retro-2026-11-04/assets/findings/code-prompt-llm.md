# Weekly audit 2026-11-04: area prompt-llm

## Summary

The area is in good health. The week replaced three providers with one chat-completions backend. It put the Connection slots into one table and decided each layout change by a measured trial. Each setting now has one main owner. But three values can still be set or reached by two paths: reasoning effort, the response length and the Connection of a call. The assembler is still one pipeline (render, cut, render again, fit). It has three special cases, and each came from one ticket. The model-workaround code is not behind one seam, and no trial or test says when any of it can go.

## Findings

### 1. Medium: an event Narration sends its Narration twice. A string compare finds the Input in the history
- Evidence (known): `narrator.rs:191` `"Previous narration:\n{previous}\n\nTrigger direction: …"`. `previous` is `history.last()`. `GenerationOrder::event_after` uses `Place::NewMessage` (`generation.rs:79`), so the window ends with that Narration. The model gets it once as `<LastMessage>` and once in `<PlayerInput>`, beside "Do NOT repeat". `narrator_tests.rs:1989` pins the duplicate without a comment.
- Evidence (known): `assembler.rs:296` `let input_in_history = last_kept.is_some_and(|entry| … entry.text.trim() == self.user_message.trim())`. The Job saved the Input, so it knows this fact. The assembler guesses it from the text.
- Why review missed it: ticket 15 (`1989d3fe`) added the drop rule for Inputs only. The continuation text dates from before the window.
- Fix class: structure. Make `Generation` (or `NarrationValues`) say whether the request text is already the last entry. Give the event continuation only the Trigger direction.

### 2. Medium: the Job and the recorder each resolve the Connection of one call
- Evidence (known): `job.rs:476` `settings.connection_for(preset_type)` gives the budget and `max_tokens`. `wiring.rs:50` `settings.connection_for(preset_type)` reads the settings again and gives the provider. A settings edit between the two reads mixes the window of one Connection with the model of another.
- This split already caused one bug. `856e36a5` removed a hook path that "the recorder reports a missing Connection" covered.
- Why review missed it: tickets 21 and 22 each added one half.
- Fix class: structure. The Job resolves the Connection once and passes it (or the provider) to `LlmCallRecorder::complete`.

### 3. Medium: two places set reasoning effort, routing and session
- Evidence (known): `generation_parameters.rs:11` `ENGINE_KEYS = ["model", "messages", "stream", "max_tokens"]`, with `TYPED_KEYS` for the samplers. `request.rs:76` starts the body from `custom` and then writes `session_id`, `reasoning_effort` and `provider` (`:88-94`) only when the Connection sets them. So a preset can send `reasoning_effort` through `custom`, and the value that wins depends on the Connection. A `temperature` in `custom` is refused.
- Why review missed it: ticket 22 made the refusal rule. The keys came earlier, from `bf3b87a5`.
- Fix class: check. Keep one list of the keys that `build_request_payload` writes. `refused_custom_key` reads that list, and a test checks that each written key is on it.

### 4. Medium: two copies of the agent prompt, and a second history format
- Evidence (known): jscpd finds 67 duplicated lines between `options/prompt.rs:71-94` and `quantifier/prompt.rs:84-107` (`<RecentHistory>`, `<CurrentRoom>`, `<CharactersPresent>`). Ticket 19 (`3d3a61ed`) made the same edit in both files: `in_id_order`, `vars.render(description)` and the Persona name. The agents still send history as one block with sender labels (`MessageType::Narration => "Narrator"`). The narrator sends one chat message for each entry, with no label.
- Why review missed it: each diff changed both copies the same way, so each copy looked correct.
- Fix class: structure. Make one agent-context renderer that both builders call.

### 5. Medium: the filter for player text no longer reaches the typed Input, and the tests hide this
- Evidence (known): `sanitize_for_prompt` runs only in `render_user_layer` (`assembler.rs:470`). Since ticket 15, a typed Input is the last history entry, so the engine drops that layer. `prompt_system.md:89` says "So the filter does not touch that Input." The Guide (`:484`) is never filtered. No template renders player text, so the filter protects nothing in the engine (inferred).
- Evidence (known): the default `Fixture` history ends with a Narration (`narrator_tests.rs:133-139`), and its `tail` ends with an Input. The Job never sends that shape for a typed-Input Narration. About 60 call sites use it, and the 8 `narrator-input-injection-*` CASEs pass only because of it.
- Why review missed it: ticket 15 wrote the gap into the doc and did not delete the filter or move it.
- Fix class: one-off fix (delete the filter and its 8 CASEs, or filter the history entries). Then make the default fixture end with the Input, as production does.

### 6. Medium: byte slices of a string can panic on the one shared transport path (pre-existing)
- Evidence (known): `response.rs:140` `format!("{}...", &error_body[..500])` runs on each non-2xx body over 500 bytes. If byte 500 falls inside a multi-byte character, the Job thread panics. `quantifier/utils/orchestration.rs:110` `&response[..response.len().min(200)]` has the same fault, but only with debug logging on.
- Why review missed it: both lines are from May. `bf3b87a5` sent all providers through this path.
- Fix class: check. Turn on `clippy::string_slice` for `src/`.

### 7. Medium: `trial.py` writes the settings with SQL and holds a second copy of the slot rule
- Evidence (known): `trial.py:459` `conn.execute("update settings set connections = ?", …)`. Its docstring says "the edit form would clear the key and the ceiling". The ceiling claim is wrong: `settings.rs:271` keeps it, and has done so since `1359b375`. `trial.py:467` copies the slot fallback "as `AppSettings::connection_for` does". `validate_data.py:13-16` lists the four slot fields again. So the slot table exists in three places.
- Why review missed it: `trial.py` changed in 9 commits. Each diff saw only its own reason.
- Fix class: check. A Python test compares the slot fields with `CONNECTION_SLOTS`. Or give `trial.py` an engine route to call.

### 8. Low: two session rules for one purpose
- Evidence (known): `llm_backend.rs:92` `sends_game_session` (OpenRouter: one id for each Game) and `:97` `session_header` (OpenCode Go: one value for each Game and agent). Layout-trials ticket 10 measured the gain of a session for each agent only on OpenCode Go: the Quantifier went from 3.8 % to 34.3 % cached. Whether OpenRouter gains the same is guessed.
- Fix class: structure. Use one session key (Game, agent). The provider chooses only how the key is sent.

### 9. Low: the response length is carried twice
- Evidence (known): `CallPlan` holds `max_tokens` in `settings` and in `budget` (`job.rs:479-483`). The narrator replaces the first value (`narrator.rs:143` `max_tokens: assembled.max_tokens`). The agents use the first and ignore the second. `max_tokens_ceiling` has no form field, and only a special case in the edit handler keeps it (`settings.rs:271-273`).
- Fix class: structure. Leave `max_tokens` out of `CallSettings` on the narrator path.

### 10. Low: leftovers of earlier states
- `TokenBudget::fallback()` and `MAX_CONTEXT_TOKENS` (`token_budget.rs:7,36`): only `narrator_tests.rs` calls them (11 call sites). They repeat the 32768 of `llm_backend.rs:72`.
- Nothing writes `MessageType::System` (inferred from a search). Ticket 15 still gave it a system role in the middle of the history (`assembler.rs:453`) and a CASE (`narrator_tests.rs:720`).
- The placeholder `"Empty uses the default (65,536)"` (settings template `:164`) repeats `DEFAULT_MAX_HISTORY_TOKENS`.
- Fix class: one-off fix.

### 11. Low: one test copy for each Connection field
- Evidence (known): `tests/http/settings_connection_form.rs` has the same group of tests (sets it, empty stores none, invalid refused, the form test passes it on) for the context window, history size, reasoning effort and provider order. That is 15 tests over one `into_config` mapping.
- Fix class: rule. Test a new optional field of a form with one row in a table-driven test.

## Answers to the brief

- **Settings owners:** each setting has one main owner. The exceptions are findings 2, 3 and 9. The fields that only JSON can set (the ceiling, the preset parameters, the Impersonate and Options slots) are an owner decision on the watch list (`prompt-flexibility/map.md:62`), not drift.
- **Assembler:** it is one pipeline with no leftovers of the one-block layout in the docs. Its special cases are in finding 1 and finding 10.
- **`narrator_tests.rs`:** the growth adds confidence, not copies. The file went from 43 to 72 tests, and each new CASE is a different clause. The flat `user_prompt()` helper survives layout changes, so churn is low. The cost is the fixture shape in finding 5. Also, 16 of the 72 tests check two string functions through the whole narrator.

## Bitter lesson

1. **The workarounds are spread out, and nothing tells when to delete them.** `single_user_message` lives in `application/prompting/prompt_merge.rs`, and the adapter calls it (`chat_completions.rs:69`). Every shipped Connection sets it to `false`, and `trial.py:878` refuses such packs, so no measurement can show it is unused. The Gemma 4 fix has three parts: markers added by a match on the model name (`chat_completions.rs:124`), a strip in `sanitize.rs`, and a call in `llm_recorder.rs:81` that runs on every answer of every model. Move them behind one per-Connection seam, and record a count each time a fix-up changes the text.
2. **Parsers accept several shapes, and the hit rate is never measured.** The Quantifier tries three JSON forms and then a substring match (`parser.rs:134` `contains(&id.to_lowercase())`). It retries on low confidence (`orchestration.rs:129`). The Options parser uses three regexes that are tied to the preset shapes. The confidence goes only to `tracing`. Since ticket 22, a preset `custom` can send `response_format`, so structured output is now possible through data. Nothing tries it. Store the confidence in the LLM message, and let a trial count it.
3. **The token constants are tuned by hand, and the measurement is already collected.** `token_budget.rs` holds `CHARS_PER_TOKEN` 4, `MESSAGE_OVERHEAD_TOKENS` 4, a safety margin of 256 and a reserve of 1024/8. `trial.py:303-307` already reads `usage.prompt_tokens` for each call, but nothing compares it with `estimate_tokens`. One line in the pack could calibrate these constants.
4. **The trial harness is the general method, and it gets its investment.** `trial.py`, `prefix_measure.py` and the ad hoc LLM judge decided tickets 09, 10 and 15 with numbers. Their limit is one setup and one model ("No second model", `layout-trials/map.md:17`). So no trial can reach the Gemma and single-message workarounds. Add a second model to the harness, or delete the workarounds for models that no trial runs.
5. **Some prompt text is in Rust.** The event continuation (`narrator.rs:191`) and the last line of the Options prompt are not in presets. A trial cannot change them without a code change.

## What went well

- One Connection slot table (`settings.rs:511-536`, `d6daa238`) serves the lookup, the refusal of a removal and the clear of an optional slot.
- Layout changes are kept or dropped by a measured trial, and the numbers are in the commit (`1989d3fe`: 63.9 % to 86.0 % cached). The map says "An inconsistency inherent to the model is accepted, not engineered around" (`layout-trials/map.md:29`).
- One `ChatCompletionsBackend` replaced three providers (`deepseek.rs`, `ollama.rs` and `openrouter.rs` deleted). Its provider differences are methods on `LlmBackendType`, and `raw_request_json` is the one record of what the engine sent.
