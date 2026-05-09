# chronicler_engine/src/narrative/

## Responsibility
The narrative tier — the interface between the synchronous engine and stochastic LLM generation. Handles prompt building, LLM backend abstraction, response parsing/sanitization, post-narration quantification, and text checking.

## Design Patterns
- **Trait-Based Backends**: `LlmBackend` trait with implementations: `OpenRouterBackend`, `OllamaBackend`, `DeepSeekBackend`, `MockBackend`.
- **Builder Pattern**: `PromptBuilder` constructs system/user prompts with token budget awareness.
- **Template Method**: Quantifier pipeline — parse → extract movement → compute NPC events.
- **Defensive Sanitization**: `sanitize_llm_output()` strips thinking artifacts and normalizes whitespace.

## Data & Control Flow
```
Engine requests narration
  → PromptBuilder::from_context() → build_split()
    → (system_prompt, user_prompt, max_tokens)
      → LlmBackend.narrate_action_from_prompt()
        → HTTP request → parse response → sanitize output
          → Return narration_text
            → QuantifierBackend.analyze(narration_text)
              → parse_quantifier_response() → MovementParseResult + NPC events
```

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `llm/` | Backend trait and implementations (OpenRouter, Ollama, DeepSeek, Mock) |
| `prompt/` | Prompt construction, token budget management, context fitting |
| `quantifier/` | Post-narration analysis — movement detection, NPC enter/leave events |
| `text_check/` | Spell/grammar checking via harper-core |

## Integration Points
- **Consumes**: `model/` (world, characters, state for prompt context)
- **Consumed by**: `engine/` (game service calls LLM and quantifier)

## Files
| File | Purpose |
|------|---------|
| `llm_client.rs` | Response parsing, content extraction, Gemma 4 thinking suffix, output sanitization |
| `mod.rs` | Module exports and test module declarations |
