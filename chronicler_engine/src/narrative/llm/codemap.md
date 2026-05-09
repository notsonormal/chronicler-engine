# chronicler_engine/src/narrative/llm/

## Responsibility
LLM backend abstraction and implementations. Provides a uniform `LlmBackend` trait for generating dialogue, narration, and arrival scenes across multiple providers (OpenRouter, Ollama, DeepSeek) plus a mock for testing.

## Design Patterns
- **Trait Abstraction**: `LlmBackend` trait defines all narrative generation methods.
- **Factory Pattern**: `get_llm_backend_for()` creates the correct backend from `Connection` config.
- **Connection-Driven Routing**: Backend selection is driven by `settings.json` connection definitions, not hardcoded model names.

## Data & Control Flow
```
Settings → get_narration_connection() → Connection
  → get_llm_backend_for(connection)
    → match provider:
      → OpenRouter → OpenRouterBackend::from_connection()
      → Ollama → OllamaBackend::from_connection()
      → DeepSeek → DeepSeekBackend::from_connection()
      → Mock → MockBackend::default()
    → backend.narrate_action_from_prompt(system, user, max_tokens)
      → HTTP request → parse response → sanitize → return String
```

## Integration Points
- **Consumed by**: `engine/game_service.rs`, `engine/action_processing.rs`
- **Depends on**: `model/llm_backend.rs` (`LlmBackendType`, `Connection`)

## Files
| File | Purpose |
|------|---------|
| `backend.rs` | `LlmBackend` trait, `get_llm_backend()`, `get_llm_backend_for()` |
| `openrouter.rs` | `OpenRouterBackend` — OpenRouter API client |
| `ollama.rs` | `OllamaBackend` — local Ollama inference client |
| `deepseek.rs` | `DeepSeekBackend` — DeepSeek API client |
| `mock.rs` | `MockBackend` — deterministic test responses |
| `mod.rs` | Module exports and test module declarations |
