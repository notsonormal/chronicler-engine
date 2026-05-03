# Implementation Plan: Marinara-Style Prompt Architecture

## Overview

Refactor chronicler_engine's prompt construction to follow Marinara-Engine's proven pattern: **plain-text instructions** + **XML-wrapped data only for external context**. This fixes Gemma 4's reasoning-loop bug where self-referential XML (`<SystemPrompt>`, `<Role>`) triggers meta-analysis instead of execution.

Also ports Marinara's context-aware token budgeting and adds per-connection `max_context_tokens` configuration.

## Architecture Decisions

1. **Instructions stay plain text** — No XML tags wrapping the system prompt itself. Imperative voice only ("Your SOLE job is...").
2. **Data keeps XML wrapping** — `<GameState>`, `<KnownNpcs>`, `<ConversationHistory>` etc. are external context, not instructions.
3. **Context fitting before every LLM call** — Reserve input budget, cap `max_tokens` dynamically, trim oldest history first.
4. **Per-connection context window sizes** — Local models (Ollama) default to 8192; API models default to 32768.
5. **Keep `build()` and `build_split()`** — Both actively used; `build()` for single-message APIs, `build_split()` for OpenAI-compatible APIs.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Plain-text instructions change output quality for non-Gemma models | Medium | Test with `gemma4:e4b` and OpenRouter GPT-4o-mini before and after; keep semantic content identical |
| `build_split()` restructuring breaks LLM backends | High | Keep `call_ollama`/`call_openrouter_with_model` signatures stable; only change what strings they receive |
| `max_context_tokens` field breaks existing settings.json parsing | Medium | Use `serde(default)` with `skip_serializing_if = "Option::is_none"` |
| Context fitting over-truncates history | Low | Add tests with known token counts; verify oldest entries drop first |

## Task List

### Phase 1: Foundation — Settings + Token Budgets

- [ ] **Task 1: Add `max_context_tokens` to `Connection`**
  - **Description**: Add `max_context_tokens: Option<u32>` to `Connection` struct with serde attrs. Add default resolution: 32768 for OpenRouter/DeepSeek, 8192 for Ollama. Wire through all backend constructors (`OllamaBackend`, `OpenRouterBackend`, quantifier backends).
  - **Acceptance criteria**:
    - [ ] `Connection` has `max_context_tokens: Option<u32>` field
    - [ ] `resolve_max_context_tokens()` returns 8192 for Ollama, 32768 for OpenRouter/DeepSeek when unset
    - [ ] All backend structs store and pass `max_context_tokens`
    - [ ] Existing settings.json without `max_context_tokens` parses correctly
  - **Verification**: `cargo test --lib model::settings::tests` passes
  - **Dependencies**: None
  - **Files**: `src/model/settings.rs`, `src/narrative/llm.rs`, `src/narrative/quantifier.rs`
  - **Estimated scope**: Small (2-3 files)

- [ ] **Task 2: Bump token budget defaults**
  - **Description**: Change `DEFAULT_MAX_TOKENS` from 1024 to 2048. Add `SAFETY_MARGIN_TOKENS` (256) and `MIN_INPUT_BUDGET_TOKENS` (512) constants.
  - **Acceptance criteria**:
    - [ ] `DEFAULT_MAX_TOKENS = 2048`
    - [ ] `SAFETY_MARGIN_TOKENS = 256`
    - [ ] `MIN_INPUT_BUDGET_TOKENS = 512`
    - [ ] Connection-level `max_tokens` override still works
  - **Verification**: `cargo test --lib narrative::llm_client::tests` passes
  - **Dependencies**: None (independent of Task 1)
  - **Files**: `src/narrative/prompt.rs`, `src/narrative/llm_client.rs`
  - **Estimated scope**: XS (1-2 files)

### Checkpoint: Foundation
- [ ] `cargo test --lib` passes
- [ ] `cargo clippy` clean
- [ ] Existing settings.json loads without errors

### Phase 2: Core Prompt Restructuring

- [ ] **Task 3: Rewrite system prompt template to plain text**
  - **Description**: Convert `SYSTEM_PROMPT_TEMPLATE` from XML-wrapped sections to imperative plain text. Preserve all semantic content (role, state tracking, world dynamics, narrative rules, writing style, prohibitions). Global rules append as plain bullet list.
  - **Acceptance criteria**:
    - [ ] No `<SystemPrompt>`, `<Role>`, `<CoreRole>`, `<InputValidation>`, `<StateTracking>`, `<WorldDynamics>`, `<Narrative>`, `<Dialogue>`, `<Rules>`, `<WritingStyle>`, `<Never>`, `<Instruction>` tags in system template
    - [ ] Imperative voice throughout ("You are...", "Your job is...", "Track...", "Never...")
    - [ ] All original rules and guidance preserved
    - [ ] `render_system_layer()` returns plain text ending with global rules
  - **Verification**: Update and run `test_build_layer_0_system` — assert key phrases present, assert no `<SystemPrompt>` tag
  - **Dependencies**: None
  - **Files**: `src/narrative/prompt.rs`
  - **Estimated scope**: Small (1 file)

- [ ] **Task 4: Rewrite PHI templates to plain text**
  - **Description**: Convert `PHI_NARRATION_TEMPLATE` and `PHI_CONTINUATION_TEMPLATE` from `<AuxiliaryInstructions>` XML to plain text task instructions.
  - **Acceptance criteria**:
    - [ ] No `<AuxiliaryInstructions>` wrapper
    - [ ] Plain text instructions with explicit task framing
    - [ ] Same semantic content preserved
  - **Verification**: Update and run `test_build_split_phi_*` tests
  - **Dependencies**: Task 3
  - **Files**: `src/narrative/prompt.rs`
  - **Estimated scope**: XS (1 file)

- [ ] **Task 5: Restructure `build_split()` to separate instructions from data**
  - **Description**: `build_split()` must put plain-text instruction layers (system template, PHI) in the `system` string, and XML-wrapped data layers (game state, NPCs, player, world, history, player input) in the `user` string. `build()` concatenates all layers with a clear separator.
  - **Acceptance criteria**:
    - [ ] `build_split().0` (system) contains: plain system instructions only
    - [ ] `build_split().1` (user) contains: XML-wrapped data + plain PHI
    - [ ] `build()` produces single string with separator between instructions and data
    - [ ] `build_system_only()` and `build_user_only()` still work
  - **Verification**: Update and run `test_build_returns_all_layers`, `test_build_split_includes_phi_in_user_half`, `test_build_split_phi_narration_mode`, `test_build_split_phi_continuation_mode`
  - **Dependencies**: Tasks 3, 4
  - **Files**: `src/narrative/prompt.rs`
  - **Estimated scope**: Small (1 file)

### Checkpoint: Core Prompts
- [ ] `cargo test --lib narrative::prompt` passes
- [ ] Manual inspection: `build_split()` system half has no XML tags; user half has XML data tags
- [ ] `cargo clippy` clean

### Phase 3: Context Fitting

- [ ] **Task 6: Add `fit_messages_to_context()` helper**
  - **Description**: Port Marinara's context-fitting logic. Given system text, user text, max_context, and requested max_tokens: estimate tokens, reserve safety margin, cap max_tokens to fit, and if user text exceeds remaining budget, trim oldest history entries first.
  - **Acceptance criteria**:
    - [ ] `fit_messages_to_context()` returns `(system, user, actual_max_tokens)`
    - [ ] `actual_max_tokens <= requested_max_tokens` (or default)
    - [ ] `actual_max_tokens <= max_context - system_tokens - safety_margin - min_input_budget`
    - [ ] If user text exceeds budget, oldest history entries are dropped first
    - [ ] Returns `EngineError::ContextOverflow` only if system prompt alone exceeds budget
  - **Verification**: Add tests: `test_context_fitting_no_trim_needed`, `test_context_fitting_trims_oldest_history`, `test_context_fitting_caps_max_tokens`
  - **Dependencies**: Task 2
  - **Files**: `src/narrative/prompt.rs`
  - **Estimated scope**: Medium (1 file, new logic + tests)

- [ ] **Task 7: Wire context fitting into LLM call flow**
  - **Description**: `build_split()` calls `fit_messages_to_context()` using the connection's `max_context_tokens`. Pass the fitted `max_tokens` to `call_chat_completions()` instead of using the raw connection `max_tokens`.
  - **Acceptance criteria**:
    - [ ] `build_split()` calls `fit_messages_to_context()` before returning
    - [ ] Uses `max_context_tokens` from the active connection
    - [ ] Returns fitted `max_tokens` alongside system/user strings (or passes it through)
    - [ ] LLM backends use the context-fitted `max_tokens`
  - **Verification**: End-to-end test: build a prompt with long history, verify it fits within budget
  - **Dependencies**: Tasks 1, 5, 6
  - **Files**: `src/narrative/prompt.rs`, `src/narrative/llm.rs`
  - **Estimated scope**: Small (2 files)

### Checkpoint: Context Fitting
- [ ] `cargo test --lib narrative::prompt` passes
- [ ] `cargo test --lib narrative::llm_client` passes
- [ ] `cargo test --lib narrative::llm` passes
- [ ] `cargo clippy` clean

### Phase 4: Quantifier + Final Tests

- [ ] **Task 8: Rewrite quantifier prompts to plain text**
  - **Description**: Convert `QuantifierPromptBuilder` system prompt from XML-wrapped instructions to plain text. Keep XML wrapping for data sections (`<AvailableNpcIds>`, `<AvailableRooms>`, `<CurrentRoom>`, etc.). Convert query/task instruction to plain text.
  - **Acceptance criteria**:
    - [ ] Quantifier system prompt is plain text instructions
    - [ ] Data sections keep XML wrapping
    - [ ] Final query is plain text, not XML-wrapped
  - **Verification**: `cargo test --lib narrative::quantifier` passes
  - **Dependencies**: None (independent of narrative prompts)
  - **Files**: `src/narrative/quantifier.rs`
  - **Estimated scope**: Small (1 file)

### Checkpoint: Quantifier
- [ ] `cargo test --lib narrative::quantifier` passes
- [ ] `cargo clippy` clean

### Phase 5: Documentation

- [ ] **Task 9: Update prompt system documentation**
  - **Description**: Update `docs/system/prompt_system.md` and `docs/reference/system_prompt.md` to reflect the new architecture. Add section explaining why plain-text instructions are used for reasoning model compatibility.
  - **Acceptance criteria**:
    - [ ] Architecture docs describe plain-text instructions + XML-wrapped data pattern
    - [ ] Reference doc shows new prompt text
    - [ ] Added explanation of reasoning model compatibility
  - **Verification**: Manual review of markdown rendering
  - **Dependencies**: Tasks 3, 4, 5, 8
  - **Files**: `docs/system/prompt_system.md`, `docs/reference/system_prompt.md`
  - **Estimated scope**: Small (2 files)

### Checkpoint: Complete
- [ ] `cargo test --lib` passes with 0 failures
- [ ] `cargo clippy` clean
- [ ] `cargo fmt` clean
- [ ] Documentation updated
- [ ] No changes to `data/settings.json` required for existing setups

## Success Criteria

- [ ] `gemma4-26b:latest` (custom IQ2_XS) produces narrative content with 2048 max_tokens
- [ ] `gemma4:e4b` continues to work correctly
- [ ] All 358+ lib tests pass
- [ ] `cargo clippy` clean
- [ ] `cargo fmt` clean
- [ ] Documentation updated
- [ ] Existing settings.json loads without modification
- [ ] New `max_context_tokens` field in settings.json is optional and has sensible defaults

## Boundaries

- **Always**: Run `cargo test --lib` before marking task done
- **Always**: Update tests alongside code changes (not in a separate batch)
- **Ask first**: Adding new dependencies, changing API endpoints, modifying world data formats
- **Never**: Change `call_ollama`/`call_openrouter_with_model` signatures
- **Never**: Remove XML from data layers (only from instruction layers)
- **Never**: Break existing `Connection` JSON serialization
