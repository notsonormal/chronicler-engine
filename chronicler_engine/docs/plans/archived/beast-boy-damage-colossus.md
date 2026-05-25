# Plan: Make PromptContext Require an Assembled System Prompt

## Goal
Remove the `Option<String>` footgun from `PromptContext.assembled_system_prompt`. The engine requires a system prompt to function; the type system should enforce this at the `PromptContext` boundary. Callers (Application tier) must assemble the prompt before building the context. The builder and backends never see an `Option`.

## Files Changed

### 1. `src/narrative/prompt/types.rs`
- Rename field `assembled_system_prompt: Option<String>` → `system_prompt: String` in both `PromptContext` and `PromptBuilder`.

### 2. `src/narrative/prompt/builder.rs`
- `PromptBuilder::from_context`: copy `context.system_prompt.clone()` directly (no `Option` wrapping).
- `render_system_layer()`: return `self.system_prompt.clone()` — no fallback, no `unwrap_or_default()`.

### 3. `src/narrative/prompt/context.rs`
- `make_prompt_context`: change parameter from `assembled_system_prompt: Option<String>` to `system_prompt: String`, pass it straight into `PromptContext`.

### 4. `src/application/action_pipeline/pipeline.rs`
- `phase_narrate`: `self.ctx.active_system_prompt()` returns `Option<String>`. If `None`, return an `ActionOutcome::Error` (the engine cannot run without a system prompt). If `Some`, pass the `String` into `make_prompt_context`.
- `run_from_input` (trigger path): same pattern — handle `None` as an error before passing to `build_trigger_request`.
- `build_trigger_request`: change signature parameter from `assembled_system_prompt: Option<String>` to `system_prompt: String`.

### 5. `src/bootstrap/run.rs`
- Before spawning the arrival-narration task, create a temporary `SqlitePromptPresetStorage` from `db_pool`, look up the active preset by `settings.active_system_prompt_preset_id`, assemble it with `world.global_rules` and `response_length`, and move the resulting `String` into the closure.
- Replace `PromptContext { ..., assembled_system_prompt: None }` with `PromptContext { ..., system_prompt: assembled_string }`.

### 6. `src/narrative/llm/openrouter.rs`
- `generate_dialogue` and `narrate_arrival`: when constructing derivative `PromptContext`s, propagate the parent context’s system prompt (`context.system_prompt.clone()`).

### 7. `src/narrative/llm/ollama.rs`
- Same propagation changes as `openrouter.rs`.

### 8. `src/narrative/llm/test_support.rs`
- `make_test_context`: provide a default test system prompt (e.g. `"You are a test narrator."`) so existing call sites keep compiling.
- `make_test_context_with_npc`: same default.

### 9. `src/narrative/prompt/builder_tests.rs`
- Update all `assembled_system_prompt: None` → `system_prompt: String::new()` (or a real test string where the test explicitly exercises system-prompt content).
- Update all `assembled_system_prompt: Some(...)` → `system_prompt: ...`.

### 10. `src/narrative/llm/mock_tests.rs`
- Update all inline `PromptContext { ..., assembled_system_prompt: None }` constructions to use `system_prompt: String::new()`.

## Verification
- `cd chronicler_engine && python build.py` (fmt + clippy + tests + coverage) must pass.
- No behavioural change except that a missing system prompt now loudly errors instead of silently producing an empty string.

## Approach
There is only one sensible approach here: the review comment spells out the exact structural change. The minor implementation decisions (e.g. error wording when `active_system_prompt()` returns `None`, default test prompt text) will be handled inline during implementation.
