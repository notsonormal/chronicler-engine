# Plan: Fix Silent Fallbacks & Magic Values

## Changes (8 issues, ~6 files)

### 1. `load_state` silent fallback → `src/application/context.rs`
- `load_state` currently swallows all errors and returns a fresh `GameState::new()`.
- **Fix:** Log the error at `error!` level before falling back, so corrupted snapshots are visible in logs. Keep the fallback (callers aren't prepared to handle `Result` yet), but make it observable.
- **Test:** Update `test_load_state_fallback_on_snapshot_error` to verify the error is logged.

### 2. `active_swipe_index` out of bounds → `src/storage/mappers/message.rs`
- `db_message_to_model` uses `get(index)` with no fallback, leaving `text` empty.
- **Fix:** Mirror the safe pattern from `context.rs:94`: `swipes.get(index).or(swipes.first())`. Log `warn!` on fallback.
- **Test:** Add `test_active_swipe_index_out_of_bounds_fallback`.

### 3. Unknown backend defaults to OpenRouter → `src/model/llm_backend.rs`
- `From<&str>` silently maps any unknown string to `OpenRouter` (paid API).
- **Fix:** Change `_ =>` arm to panic in debug/test or return `Mock`. Better: add `warn!` log and return `Mock` for unknown strings. Update `test_unknown_returns_openrouter_default`.
- **Alternative (preferred):** Return `Mock` for unknown + log warning, avoiding accidental paid API calls.

### 4. Magic `id == 0` sentinel → `src/model/message.rs` + 4 call sites
- **Fix:** Add `Message::is_unpersisted() -> bool { self.id == 0 }`. Replace all 4 `msg.id == 0` checks.
- **Files:** `context.rs`, `bootstrap/run.rs`, `application_service.rs` (2×).

### 5. Model layer reads env vars → `src/model/settings.rs`
- `Connection::resolve_api_key()` calls `std::env::var`. Model should be pure data.
- **Fix:** Move env resolution to `bootstrap/run.rs` (where settings are loaded). Inject resolved `api_key` into `Connection`. Make `resolve_api_key()` return `self.api_key.clone()` only.
- **Note:** `resolve_base_url()` also reads `OLLAMA_BASE_URL` — move this too, or accept it as bootstrap-layer config.

### 6. LLM client hardcodes model hacks → `src/narrative/llm_client.rs` + backends
- `apply_gemma4_thinking_suffix` and `sanitize_llm_output` live in generic client.
- **Fix:** Add `preprocess_user_text(&self, text: &str) -> String` and `postprocess_response_text(&self, text: &str) -> String` to `LlmBackend` trait (default no-op). Move gemma suffix to `OllamaBackend`. Move sanitize to `OpenRouterBackend` or keep as default postprocess.
- **Files:** `backend.rs` (trait), `ollama.rs`, `openrouter.rs`, `llm_client.rs`.

### 7. Quantifier leaks `Err` → `src/narrative/agents/quantifier/core.rs`
- `quantify_room_with_llm_call` already returns `Ok(fallback)` after retries. The `Err` branch in `determine_npcs_in_room` is never hit for quantifier logic (only LLM transport failures).
- **Fix:** Change `quantify_room_with_llm_call` to return `QuantifierResult` directly. Remove `Err` branch from `determine_npcs_in_room`.

### 8. Parser cascade duplicated → `src/narrative/agents/quantifier/parser.rs`
- `parse_quantifier_response` and `parse_quantifier_response_with_movement` share identical NPC extraction logic.
- **Fix:** Extract `extract_npcs(response, known_ids) -> QuantifierParseResult` helper. Have both functions call it.

## Verification
- `cd chronicler_engine && python build.py` (fmt + clippy + tests + coverage).
- All existing tests pass; new tests added for items 1, 2.
