# Implementation Plan: LLM Messages Tab

## Overview
Add a new "LLM Messages" tab to the Chronicler Engine dashboard that displays the last 50 LLM calls (agent, model, full request/response). Data is persisted in SQLite with a strict global cap of 50 rows.

## Architecture Decisions

1. **Unified logging at the HTTP client level** — `call_chat_completions` in `llm_client.rs` is the single chokepoint for all LLM traffic (narrator + quantifier). We'll add `agent_name`, `backend_name`, and an optional `LlmMessageStorage` reference here. This captures everything consistently without duplicating logic across 4 backend implementations and the quantifier path.

2. **`LlmBackend` trait returns `LlmCallResult`** — As chosen in the grilling session, the trait gains `agent_name: &str` and returns `LlmCallResult { text, system_prompt, user_prompt, raw_request_json, raw_response_json }`. Callers extract `.text` and the backend logs via the shared HTTP client layer.

3. **Quantifier logged separately** — The quantifier calls `llm_client.rs` directly (not via `LlmBackend`). We'll thread the same logging parameters through the quantifier closure.

4. **SQLite auto-pruning** — `SqliteLlmMessageStorage::save()` wraps insert + delete-oldest in a transaction. No background jobs needed.

5. **Flat global log** — No `turn_id` foreign key. Survives resets. Supports future multi-game.

## Task List

### Phase 1: Foundation (Data Model + Storage)

#### Task 1: Create `LlmMessage` model
**Description:** Add the core data structure for an LLM call record.

**Files touched:**
- `src/model/llm_message.rs` (new)
- `src/model/mod.rs`

**Acceptance criteria:**
- [ ] `LlmMessage` struct with fields: `id: i64`, `agent_name: String`, `backend_name: String`, `model_name: String`, `system_prompt: String`, `user_prompt: String`, `raw_request_json: String`, `raw_response_json: String`, `parsed_response: String`, `error_message: Option<String>`, `created_at: DateTime<Utc>`
- [ ] Module registered in `model/mod.rs`

**Estimated scope:** Small (1-2 files)

---

#### Task 2: Add `llm_messages` table + `LlmMessageStorage` trait
**Description:** Create the SQLite table with migrations and define the storage trait.

**Files touched:**
- `src/storage/db.rs`
- `src/storage/llm_message_storage.rs` (new)
- `src/storage/mod.rs` (if exists, or update `src/storage/mod.rs` reference)

**Acceptance criteria:**
- [ ] `llm_messages` table created in `run_migrations` with all columns
- [ ] Index on `created_at DESC`
- [ ] `LlmMessageStorage` trait with `save(&self, message: &LlmMessage) -> Result<(), EngineError>` and `list_latest(&self, limit: usize) -> Result<Vec<LlmMessage>, EngineError>`
- [ ] `SqliteLlmMessageStorage` implementing the trait
- [ ] `save()` wraps insert + "DELETE FROM llm_messages WHERE id NOT IN (SELECT id FROM llm_messages ORDER BY created_at DESC LIMIT 50)" in a transaction
- [ ] `InMemoryLlmMessageStorage` for tests (ring buffer of 50)

**Estimated scope:** Medium (3-5 files)

---

### Checkpoint: After Phase 1
- [ ] `cargo check` passes
- [ ] Unit test for `InMemoryLlmMessageStorage` ring buffer works

---

### Phase 2: HTTP Client + LlmBackend Trait

#### Task 3: Create `LlmCallResult` / `ChatCompletionResult` types
**Description:** Define the rich return types for LLM calls.

**Files touched:**
- `src/narrative/llm_client.rs`
- `src/narrative/llm/backend.rs`

**Acceptance criteria:**
- [ ] `ChatCompletionResult` struct in `llm_client.rs`: `text: String`, `system_prompt: String`, `user_prompt: String`, `raw_request_json: String`, `raw_response_json: String`
- [ ] `LlmCallResult` struct in `llm/backend.rs`: wraps `ChatCompletionResult` + `backend_name: String`, `model_name: String`, `agent_name: String`

**Estimated scope:** Small (1-2 files)

---

#### Task 4: Change `call_chat_completions` to return `ChatCompletionResult`
**Description:** The raw HTTP client now captures and returns all metadata needed for logging.

**Files touched:**
- `src/narrative/llm_client.rs`

**Acceptance criteria:**
- [ ] `call_chat_completions` returns `Result<ChatCompletionResult, EngineError>`
- [ ] Function constructs `raw_request_json` from the `payload` variable before sending
- [ ] On success, captures `raw_response` body and parsed text
- [ ] On error, still returns a `ChatCompletionResult` with `error_message` set and empty `text` (or use a separate error path)
- [ ] `call_openrouter_with_model` and `call_ollama` updated to return `ChatCompletionResult`

**Estimated scope:** Small (1 file)

---

#### Task 5: Update `LlmBackend` trait + all implementations
**Description:** Add `agent_name` parameter and `LlmCallResult` return type. Each backend constructs the result from `ChatCompletionResult`.

**Files touched:**
- `src/narrative/llm/backend.rs`
- `src/narrative/llm/openrouter.rs`
- `src/narrative/llm/deepseek.rs`
- `src/narrative/llm/ollama.rs`
- `src/narrative/llm/mock.rs`

**Acceptance criteria:**
- [ ] Trait updated: `fn narrate_action(&self, agent_name: &str, context: &PromptContext) -> Result<LlmCallResult, EngineError>`
- [ ] Same for `narrate_arrival`, `narrate_continuation`, `narrate_action_from_prompt`, `generate_dialogue`
- [ ] All 4 backend implementations updated
- [ ] Backends add `storage: Option<Arc<dyn LlmMessageStorage>>` field
- [ ] `get_llm_backend_for` updated to accept optional storage and pass it to backend constructors
- [ ] `get_llm_backend` / `get_llm_backend_with_settings` updated
- [ ] Mock backend returns synthetic `LlmCallResult` with sensible defaults

**Estimated scope:** Medium (5 files, mechanical)

---

### Checkpoint: After Phase 2
- [ ] `cargo check` passes
- [ ] `cargo test` passes (mock tests updated for new signature)

---

### Phase 3: Call Sites + Quantifier

#### Task 6: Update all LlmBackend call sites
**Description:** Every caller must pass an agent name and extract `.text` from `LlmCallResult`.

**Files touched:**
- `src/engine/game_service/actions.rs`
- `src/engine/game_service/retry.rs`
- `src/engine/action_processing.rs`
- `src/bootstrap/run.rs`
- `src/test_support/context.rs` (if it constructs backends)

**Acceptance criteria:**
- [ ] `actions.rs`: `"narrator"` for `narrate_action`, `"trigger"` for `narrate_action_from_prompt`
- [ ] `retry.rs`: `"trigger"` for retry event continuation
- [ ] `action_processing.rs`: `"trigger"` for trigger evaluation
- [ ] `bootstrap/run.rs`: `"narrator"` for arrival narration
- [ ] All call sites extract `.text` from `LlmCallResult`
- [ ] `GameServiceContext` gains `llm_message_storage: Arc<dyn LlmMessageStorage>`
- [ ] `AppState` gains `llm_message_storage: Arc<dyn LlmMessageStorage>`
- [ ] `bootstrap/run.rs` creates both storages from shared `DbPool`

**Estimated scope:** Medium (5-6 files)

---

#### Task 7: Update quantifier to log LLM calls
**Description:** The quantifier calls `llm_client.rs` directly via `call_openrouter_with_model` / `call_ollama`. Thread logging through.

**Files touched:**
- `src/narrative/agents/quantifier/backends.rs`
- `src/narrative/agents/quantifier/core.rs`
- `src/engine/game_service/actions.rs` (pass storage to quantifier)

**Acceptance criteria:**
- [ ] Quantifier backend structs gain `storage: Option<Arc<dyn LlmMessageStorage>>`
- [ ] `get_quantifier_backend_for` accepts optional storage
- [ ] `quantify_room_with_llm_call` closure returns `ChatCompletionResult` instead of `String`
- [ ] Inside `quantify_room_with_llm_call`, after successful LLM call, construct `LlmMessage` with `agent_name="quantifier"` and log it
- [ ] `get_quantifier_backend` updated

**Estimated scope:** Medium (3 files)

---

### Checkpoint: After Phase 3
- [ ] `cargo test` passes
- [ ] Manual: run engine, submit an action, verify `llm_messages` table has rows

---

### Phase 4: Server Fragment + UI

#### Task 8: Add server endpoint + Askama template
**Description:** New HTMX fragment that renders the LLM messages list.

**Files touched:**
- `src/server/fragments/endpoints.rs`
- `src/server/fragments/renderers.rs`
- `src/server/templates.rs`
- `src/server/mod.rs`

**Acceptance criteria:**
- [ ] `LlmMessageView` struct in `templates.rs` (formatted for display)
- [ ] `LlmMessagesTemplate` Askama template: compact list, expandable rows, raw JSON collapsible, newest-last
- [ ] `render_llm_messages` in `renderers.rs` loads latest 50 from storage
- [ ] `llm_messages_fragment` handler in `endpoints.rs`
- [ ] Route `/fragment/llm-messages` registered in `server/mod.rs`

**Estimated scope:** Medium (4 files)

---

#### Task 9: Update `index.html` + `styles.css`
**Description:** Add the new tab shell and styling.

**Files touched:**
- `assets/index.html`
- `assets/styles.css`

**Acceptance criteria:**
- [ ] New tab button `<button class="tab" data-tab="llm-messages">LLM Messages</button>`
- [ ] New tab content `<div class="tab-content" id="llm-messages-tab">` with HTMX polling `hx-trigger="load, every 4s"`
- [ ] CSS for `.llm-message-list`, `.llm-message-card`, `.llm-message-header`, `.llm-message-raw` (collapsible)
- [ ] Vanilla JS toggle for expand/collapse raw JSON

**Estimated scope:** Small (2 files)

---

### Checkpoint: After Phase 4
- [ ] `cargo check` passes
- [ ] Full `python build.py` passes
- [ ] Manual: open UI, click LLM Messages tab, see list

---

### Phase 5: Validation

#### Task 10: Run full validation
**Description:** Ensure everything compiles, tests pass, and formatting is clean.

**Verification:**
- [ ] `cd chronicler_engine && python build.py` passes (fmt + clippy + tests + coverage)
- [ ] Screenshot review of LLM Messages tab in browser

**Estimated scope:** Small (verification only)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Quantifier logging misses the "latest 50" cap if logged outside the shared storage transaction | Low | Use the same `SqliteLlmMessageStorage::save()` which wraps insert + prune |
| Changing `LlmBackend` trait breaks many tests | Medium | Update mock backend + all test call sites in same PR; compiler will flag every breakage |
| HTMX polling every 4s is too aggressive for large payloads | Low | 50 JSON payloads could be large; template should truncate raw JSON display to ~500 chars with "show more" |
| `agent_name` string mismatch between callers | Low | Use constants: `AGENT_NARRATOR`, `AGENT_QUANTIFIER`, `AGENT_TRIGGER`, `AGENT_DIALOGUE` |

## Open Questions
- None — all resolved in grilling session.
