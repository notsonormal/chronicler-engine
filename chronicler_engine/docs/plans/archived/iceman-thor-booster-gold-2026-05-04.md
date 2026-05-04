# Implementation Plan: Fix Gemma 4 Thinking Suffix Corruption

## Overview

The `apply_gemma4_thinking_suffix()` function in `llm_client.rs` is malformed compared to the SillyTavern preset it was derived from. It injects raw template tokens into user message content for **all backends** (OpenRouter and Ollama), causing Gemma 4 models to emit corrupted output: `<channel|>` prefixes, massive `<thought>` blocks, and broken narrative prose.

This plan fixes the suffix format, scopes it to Ollama only (where the passthrough template actually needs it), and adds output sanitization as a safety net.

## Architecture Decisions

1. **Ollama-only suffix injection** — OpenRouter applies native chat templates; injecting raw `<|turn>` tokens into user message content fights the template and produces garbage. The suffix belongs only in the Ollama passthrough path.
2. **Exact SillyTavern `last_output_sequence` format** — Remove the erroneous leading `<turn|>` line. The correct prefill is `<|turn>model\n<|channel>thought\n<channel|>`.
3. **Output sanitization for all backends** — A post-processing layer strips leaked thinking artifacts (`<channel|>`, `<thought>`, `<|channel>thought`) from every response. This is a defensive safety net, not a substitute for the prompt-level fix.

## Task List

### Task 1: Fix thinking suffix format and scope to Ollama

**Description:** Move `apply_gemma4_thinking_suffix` out of the shared `call_chat_completions` path and into `call_ollama` only. Correct the suffix to match SillyTavern's `last_output_sequence` exactly.

**Acceptance criteria:**
- [ ] `call_chat_completions` no longer applies any thinking suffix
- [ ] `call_ollama` applies the corrected suffix: `<|turn>model\n<|channel>thought\n<channel|>`
- [ ] `call_openrouter_with_model` remains unchanged (no suffix)
- [ ] Existing tests for `call_openrouter_*` and `call_ollama_*` still pass
- [ ] New test verifies suffix is appended only in `call_ollama` with Gemma 4 model names

**Files touched:**
- `chronicler_engine/src/narrative/llm_client.rs`

**Estimated scope:** Small (1 file, ~15 lines changed)

---

### Task 2: Add output sanitization for thinking artifacts

**Description:** Add a `sanitize_llm_output` function that strips common thinking/reasoning tag leaks from raw LLM responses, and wire it into `parse_chat_response` so every backend benefits.

**Patterns to strip:**
- Leading `<channel|>` prefix
- `<thought>...</thought>` blocks (and content between them)
- `<|channel>thought...<channel|>` blocks
- `<|turn>model`, `<turn|>`, `<|turn>` orphan tokens

**Acceptance criteria:**
- [ ] `sanitize_llm_output` exists and handles all patterns above
- [ ] `parse_chat_response` calls sanitization on extracted `content` before returning
- [ ] Unit tests cover each pattern individually and in combination
- [ ] Empty-string and already-clean inputs pass through unchanged

**Files touched:**
- `chronicler_engine/src/narrative/llm_client.rs`

**Estimated scope:** Small (1 file, ~40 lines added + tests)

---

### Task 3: Update documentation

**Description:** Update `docs/system/llm_processing.md`, `CHANGELOG.md`, and `docs/reference/marinara_engine.md` to reflect the corrected suffix format, Ollama-only scope, and new output sanitization layer.

**Acceptance criteria:**
- [ ] `llm_processing.md` §8 shows the corrected suffix format and notes "Ollama only"
- [ ] `CHANGELOG.md` unreleased section notes the fix
- [ ] `marinara_engine.md` cross-reference updated if needed

**Files touched:**
- `chronicler_engine/docs/system/llm_processing.md`
- `chronicler_engine/docs/CHANGELOG.md`
- `chronicler_engine/docs/reference/marinara_engine.md`

**Estimated scope:** Small (3 files, ~10 lines changed)

---

## Checkpoint: After Tasks 1–3

- [ ] `cargo test` passes in `chronicler_engine/`
- [ ] `cargo clippy` is clean
- [ ] All new tests pass
- [ ] Documentation is consistent with code

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Fixed suffix still confuses some Gemma 4 quants | Medium | Output sanitization catches leaks; can iterate on suffix format later |
| Moving suffix from shared path breaks other callers | Low | Only `call_ollama` and `call_openrouter_with_model` call `call_chat_completions`; verified by grep |
| Output sanitizer is too aggressive | Low | Regex targets specific tag patterns; narrative prose without these exact strings is untouched |

## Open Questions

None — the approach is validated by SillyTavern preset research and Marinara-Engine reference analysis.
