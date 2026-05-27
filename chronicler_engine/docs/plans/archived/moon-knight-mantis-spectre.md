# PromptAssembler Architecture Design Plan

## 1. Executive Summary

Current architecture: `PromptBuilder` is a concrete struct (in `types.rs` / `builder.rs`) that renders XML layers and applies token budgets. It is instantiated identically inside `OpenRouterBackend` and `OllamaBackend` via a private `narrate_from_context` method. This creates duplication and couples transport logic (HTTP) with prompt assembly logic.

The `<!--POST_HISTORY-->` delimiter hack in `PromptPreset::assemble_split_text()` leaks preset-section decisions into a transport-level string, forcing `PromptBuilder::from_context()` to split the string back apart.

Goal: Introduce `PromptAssembler` as a trait-based abstraction that decouples prompt construction from LLM transport, eliminating backend duplication, removing the delimiter hack, and enabling pluggable prompt strategies.

---

## 2. File-by-File Findings

### 2.1 `src/narrative/llm/backend.rs`

**What it does:**
Defines the `LlmBackend` trait and `LlmCallResult` struct. Also provides `get_llm_backend_for` factory and `merge_single_user_message` helper.

**Methods:**
- `narrate_action(agent_name, &PromptContext)` — main narration
- `narrate_arrival(agent_name, &PromptContext)` — arrival narration
- `generate_dialogue(agent_name, &PromptContext, &NpcCard)` — dialogue (test-only in prod)
- `narrate_continuation(...)` and `complete(...)` — raw string primitives

**Interaction with PromptContext / PromptBuilder:**
The trait itself does not reference `PromptBuilder`. Coupling happens in each backend impl via private `narrate_from_context` methods.

**Changes needed:**
Remove `narrate_action`, `narrate_arrival`, `generate_dialogue` from trait. Keep only `complete` and `narrate_continuation`.

---

### 2.2 `src/application/action_pipeline/pipeline.rs`

**What it does:**
Builds `PromptContext` via `make_prompt_context` in `phase_narrate`, then calls `self.service.narrate_action(&context)`. Never touches `PromptBuilder` directly (stale import exists).

**Changes needed:**
- Remove stale `PromptBuilder` import.
- Remove `narrate_action` from `ActionPipelineBackend` trait.
- Add `assembler() -> &dyn PromptAssembler` to `ActionPipelineBackend`.
- `phase_narrate` loads preset from `self.ctx.preset_storage`, calls assembler, then calls `self.service.complete(...)`.

---

### 2.3 `src/narrative/llm/openrouter.rs`

**What it does:**
OpenRouter backend. Stores API key, model, token limits, and message storage.

**Interaction with PromptContext / PromptBuilder:**
- `narrate_from_context` (lines 59-72) is the only PromptBuilder usage:
  ```rust
  let builder = PromptBuilder::from_context(context)
      .with_max_context_tokens(self.max_context_tokens)
      .with_max_tokens(...);
  let (system, user, max) = builder.build_split()?;
  self.complete(agent_name, &system, &user, Some(max))
  ```
- `narrate_action`, `narrate_arrival`, `generate_dialogue` all delegate to `narrate_from_context`.

**Changes needed:**
- Delete `narrate_from_context`.
- Delete context methods entirely; only `complete` and `narrate_continuation` remain.
- Remove `max_context_tokens` and `max_tokens` fields (move to assembler).

---

### 2.4 `src/narrative/llm/ollama.rs`

**What it does:**
Ollama backend. Structurally identical to OpenRouter.

**Interaction with PromptContext / PromptBuilder:**
- `narrate_from_context` is a byte-for-byte duplicate of OpenRouter's version.

**Changes needed:**
- Same as OpenRouter. Deleting duplicated method eliminates copy-paste.
- Remove `max_context_tokens` and `max_tokens` fields (move to assembler).

---

### 2.5 `src/narrative/llm/mock.rs`

**What it does:**
Test backend with atomic flags for failure, delays, and per-call rotations.

**Interaction with PromptContext / PromptBuilder:**
- **Never uses PromptBuilder**. Reads `PromptContext` fields directly.
- `complete` and `narrate_continuation` operate on raw strings.

**Changes needed:**
- Remove context methods from `MockBackend`.
- Mock only needs `complete` and `narrate_continuation`.
- Existing context-method tests move to assembler test suite.

---

### 2.6 `src/model/state.rs` (lines 129-170)

**What it does:**
Defines `NarrativeState` and `SceneState`. `StoredTriggerContext` (lines 118-127) stores pre-built prompts for trigger continuations.

**Interaction with PromptContext / PromptBuilder:**
- No direct interaction. `history()` feeds `make_prompt_context`.
- `StoredTriggerContext` bypasses assembly entirely (stores raw system/user/max_tokens).

**Changes needed:**
- None. `StoredTriggerContext` is inherently independent of assembly strategy.

---

### 2.7 `src/narrative/llm/test_support.rs`

**What it does:**
Test helpers: `make_test_context`, `make_test_context_with_npc`, etc.

**Interaction with PromptContext / PromptBuilder:**
- Builds `PromptContext` with leaked static data.
- Never references `PromptBuilder`.

**Changes needed:**
- None for now. Reused directly by assembler tests.
- In Phase 2, remove `system_prompt` from `PromptContext` construction.

---

### 2.8 `src/narrative/prompt/context.rs`

**What it does:**
- `make_prompt_context`: Factory for `PromptContext`.
- `fit_messages_to_context`: Trims system/user to token budget, drops oldest history first.

**Interaction with PromptContext / PromptBuilder:**
- `fit_messages_to_context` is called only inside `PromptBuilder::build_split()` (builder.rs:80).

**Changes needed:**
- `make_prompt_context` stays for now (Phase 1); signature simplified in Phase 2.
- `fit_messages_to_context` moves into `LayeredPromptAssembler`.

---

### 2.9 `src/narrative/prompt/builder.rs`

**What it does:**
`PromptBuilder` impl with 7 XML layers, delimiter splitting, and token budgeting.

**Key methods:**
- `from_context`: Splits `system_prompt` on `POST_HISTORY_DELIMITER`.
- `build_split`: Renders layers, appends post-history prompt, calls `fit_messages_to_context`.
- `build_system_only` / `build_user_only`: Partial builds.

**Changes needed:**
- Entire file replaced by `LayeredPromptAssembler` in new `assembler.rs`.
- Layer rendering methods move into assembler impl.
- Tests become `assembler_tests.rs`.

---

### 2.10 `src/narrative/prompt/types.rs`

**What it does:**
Defines `PromptLayer` enum, `PromptContext`, and `PromptBuilder` structs.

**Changes needed:**
- `PromptContext` stays for Phase 1; `system_prompt` field removed in Phase 2.
- `PromptBuilder` struct deleted.
- `PromptAssembler` trait and `AssembledPrompt` struct added.

---

### 2.11 `src/application/context.rs`

**What it does:**
`GameServiceContext` with `active_system_prompt()` which loads preset and calls `assemble_split_text()` (the delimiter hack).

**Changes needed:**
- `active_system_prompt()` becomes unused in the pipeline (assembler loads preset directly).
- Can be removed in Phase 2.

---

### 2.12 `src/bootstrap/run.rs`

**What it does:**
Startup code. For arrival narration without scenario, builds `PromptContext` manually and calls `backend.narrate_arrival()`.

**Changes needed:**
- Build `PromptContext` without `system_prompt`.
- Load preset and call assembler directly.
- Call `backend.complete(...)` with assembled prompts.

---

## 3. Pain Points

1. **Duplicated `narrate_from_context`**: OpenRouter and Ollama share identical assembly logic.
2. **Backend owns token budget config**: `max_context_tokens` / `max_tokens` are backend fields but belong to assembly.
3. **Delimiter hack**: `<!--POST_HISTORY-->` leaks section placement decisions across the codebase.
4. **`generate_dialogue` is dead in production**: Zero non-test, non-trait-impl call sites.
5. **Trait conflates transport + assembly**: `LlmBackend` has both raw (`complete`) and high-level (`narrate_action`) methods.

---

## 4. Design Options

### Option A: Backend-Injected Assembler (Minimal Invasion)

- Add `PromptAssembler` trait.
- Add `fn assembler(&self) -> &dyn PromptAssembler` to `LlmBackend`.
- Backends delegate context methods to assembler, then call `self.complete(...)`.
- `LayeredPromptAssembler` encapsulates current logic.

**Pros:** Low blast radius; no call-site changes.
**Cons:** `LlmBackend` remains bloated; backends still know about assembly; delimiter hack persists.

---

### Option B: Service-Layer Assembler (Clean Separation) — RECOMMENDED

- **Slim `LlmBackend` to transport primitives only**:
  ```rust
  pub trait LlmBackend: Send + Sync {
      fn model(&self) -> &str;
      fn name(&self) -> &str;
      fn complete(&self, agent_name, system, user, max_tokens) -> Result<LlmCallResult, EngineError>;
      fn narrate_continuation(&self, agent_name, system, user, trigger_prompt, max_tokens) -> Result<LlmCallResult, EngineError>;
      fn save_message(&self, message: &LlmMessage);
  }
  ```
- **Remove** `narrate_action`, `narrate_arrival`, `generate_dialogue` from trait.
- **Add `PromptAssembler` trait**:
  ```rust
  pub trait PromptAssembler: Send + Sync {
      fn assemble(
          &self,
          context: &PromptContext,
          preset: &PromptPreset,
          global_rules: &[String],
          response_length: Option<&str>,
      ) -> Result<AssembledPrompt, EngineError>;
  }
  pub struct AssembledPrompt {
      pub system_prompt: String,
      pub user_prompt: String,
      pub max_tokens: u32,
  }
  ```
- `LayeredPromptAssembler` implements current XML logic + token fitting. It loads preset sections directly (no delimiter).
- `DefaultGameService` owns `Arc<dyn PromptAssembler>`.
- `ActionPipelineBackend` gets `assembler() -> &dyn PromptAssembler` and loses `narrate_action`.
- Pipeline loads preset, calls assembler, calls `complete`.
- Bootstrap loads preset, calls assembler, calls `complete`.

**Pros:** True SRP; zero backend duplication; pluggable strategies; leaner traits; eliminates delimiter hack.
**Cons:** More call sites change; mock context tests move to assembler tests.

---

## 5. Implementation Steps (Option B)

### Phase 1: Core Assembler

1. **Create `src/narrative/prompt/assembler.rs`**
   - `PromptAssembler` trait, `AssembledPrompt` struct.
   - `LayeredPromptAssembler` with:
     - `max_context_tokens` and `max_tokens` config fields.
     - `assemble()` method that:
       1. Builds `system_prompt` from preset sections (role + instructions + global_rules).
       2. Builds `post_history_prompt` from preset sections (writing_style + output_format + response_length).
       3. Renders all XML layers (game state, NPCs, player, world info, history, user input).
       4. Appends post-history prompt before user input.
       5. Calls `fit_messages_to_context` for token budgeting.
       6. Returns `AssembledPrompt`.

2. **Update `src/narrative/prompt/mod.rs`**
   - Export new types; add `assembler` module.

3. **Update `src/narrative/prompt/types.rs`**
   - Add `PromptAssembler` trait and `AssembledPrompt`.
   - Keep `PromptBuilder` for now (deleted in Phase 2).

4. **Create `src/narrative/prompt/assembler_tests.rs`**
   - Port key builder tests: layer ordering, budget trimming, empty history, NPC cards.
   - Test that assembler loads preset sections correctly.
   - Test token budget enforcement.

### Phase 2: Slim Backends

5. **Update `src/narrative/llm/backend.rs`**
   - Remove `narrate_action`, `narrate_arrival`, `generate_dialogue` from `LlmBackend` trait.
   - Keep `complete` and `narrate_continuation`.

6. **Refactor `openrouter.rs` and `ollama.rs`**
   - Delete `narrate_from_context` and all context method impls.
   - Remove `max_context_tokens` and `max_tokens` fields.
   - Keep only `complete`, `narrate_continuation`, `model`, `name`, `save_message`.

7. **Refactor `mock.rs` and mock tests**
   - Remove context methods from `MockBackend`.
   - Delete or rewrite `mock_tests.rs` context-method tests as assembler tests.

8. **Refactor `deepseek.rs` and `deepseek_tests.rs`**
   - Remove unimplemented context stubs and their tests.

### Phase 3: Service & Pipeline

9. **Update `src/application/game_service/service.rs`**
   - Add `prompt_assembler: Arc<dyn PromptAssembler>` field.
   - Construct `LayeredPromptAssembler` in `with_storage` (read token limits from settings).
   - Rewrite `narrate_action` to assemble then call `llm_backend.complete(...)`.

10. **Update `ActionPipelineBackend` trait (`pipeline.rs`)**
    - Remove `narrate_action`.
    - Add `assembler() -> &dyn PromptAssembler`.
    - Keep `complete` and `run_post_generation_agents`.

11. **Update pipeline test mocks**
    - `MockPipelineBackend` and `MockBackend` in `actions_tests.rs` implement `assembler()` returning a `MockPromptAssembler`.
    - `MockPromptAssembler` returns fixed strings for testing.

12. **Update `src/bootstrap/run.rs`**
    - Load preset from settings.
    - Build `PromptContext` without `system_prompt`.
    - Call assembler then `backend.complete(...)`.

### Phase 4: Cleanup

13. **Delete `src/narrative/prompt/builder.rs`**
    - All logic migrated to `LayeredPromptAssembler`.

14. **Delete `src/narrative/prompt/builder_tests.rs`**
    - All tests migrated to `assembler_tests.rs`.

15. **Update `PromptContext`**
    - Remove `system_prompt: String` field.
    - Update `make_prompt_context` signature.
    - Update all call sites (mechanical).

16. **Remove `active_system_prompt()`**
    - From `GameServiceContext`.

---

## 6. Migration Impact

| File | Change |
|------|--------|
| `backend.rs` | Delete 3 context methods from trait |
| `openrouter.rs` | Delete `narrate_from_context` + 3 methods; remove token config |
| `ollama.rs` | Delete `narrate_from_context` + 3 methods; remove token config |
| `mock.rs` | Delete context method impls |
| `deepseek.rs` | Delete unimplemented stubs |
| `service.rs` | Add assembler field; rewrite `narrate_action` |
| `pipeline.rs` | Remove `narrate_action` from trait; add `assembler()` |
| `bootstrap/run.rs` | Assemble before `complete` |
| `builder.rs` | **Delete** (logic → `assembler.rs`) |
| `types.rs` | Add trait + struct; keep `PromptBuilder` until Phase 4 |
| `mod.rs` | Update exports |
| `assembler.rs` | **New** — core logic |
| `assembler_tests.rs` | **New** — ported from builder_tests |
| `builder_tests.rs` | **Delete** after port |
| `mock_tests.rs` | Move context tests to assembler |
| `deepseek_tests.rs` | Delete stub tests |
| `test_support.rs` | No change in Phase 1-3; update in Phase 4 |
| `context.rs` | No change in Phase 1-3; update `make_prompt_context` in Phase 4 |
| `context.rs` (`active_system_prompt`) | Remove in Phase 4 |

---

## 7. Testing Strategy

1. **Assembler unit tests**: Layer ordering, budget trimming, preset loading, empty inputs.
2. **Pipeline integration tests**: Mock assembler + mock backend verify pipeline calls assembler then complete.
3. **Backend tests**: Only test `complete` and `narrate_continuation` with raw strings.
4. **Bootstrap tests**: If any exist, verify assembly before `complete`.
5. **Full validation**: `cd chronicler_engine && python build.py`.

---

## 8. Rollback Plan

If issues arise during implementation:
1. Revert backend changes (restore `narrate_from_context` and context methods).
2. Keep assembler as an optional wrapper around `PromptBuilder`.
3. Re-introduce delimiter temporarily if assembler has bugs.
