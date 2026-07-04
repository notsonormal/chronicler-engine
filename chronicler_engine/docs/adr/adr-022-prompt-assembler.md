# ADR-022: PromptAssembler Trait Decoupling

**Date:** 2026-05-31  
**Status:** Accepted  
**Replaces:** N/A  
**Superseded by:** N/A

## Context

The original architecture coupled prompt assembly logic directly into LLM transport backends (`OpenRouterBackend`, `OllamaBackend`). Both backends contained identical `narrate_from_context` methods that:

1. Instantiated `PromptBuilder` from a `PromptContext`
2. Applied token budget configuration (`max_context_tokens`, `max_tokens`)
3. Called `builder.build_split()` to render XML layers
4. Delegated to `self.complete()` with the assembled prompts

This created several problems:

### Pain Points

1. **Duplicated logic**: OpenRouter and Ollama backends shared byte-for-byte identical prompt assembly code
2. **Token budget misplacement**: Configuration lived in backend structs but belonged to assembly concerns
3. **Delimiter hack**: `PromptPreset::assemble_split_text()` used `<!--POST_HISTORY-->` delimiter to split preset sections, forcing `PromptBuilder::from_context()` to parse this delimiter and split the string back apart
4. **Bloated trait**: `LlmBackend` trait included high-level methods (`narrate_action`, `narrate_arrival`, `generate_dialogue`) that conflated transport with assembly
5. **Dead code**: `generate_dialogue` had zero production call sites outside tests

## Decision

Introduce a `PromptAssembler` trait to decouple prompt construction from LLM transport:

### New Abstraction

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

### Slimmed LlmBackend Trait

```rust
pub trait LlmBackend: Send + Sync {
    fn model(&self) -> &str;
    fn name(&self) -> &str;
    fn complete(&self, agent_name: &str, system: &str, user: &str, max_tokens: Option<u32>) 
        -> Result<LlmCallResult, EngineError>;
    fn narrate_continuation(&self, agent_name: &str, system: &str, user: &str, 
        trigger_prompt: &str, max_tokens: Option<u32>) -> Result<LlmCallResult, EngineError>;
    fn save_message(&self, message: &LlmMessage);
}
```

**Removed from trait**: `narrate_action`, `narrate_arrival`, `generate_dialogue`

### LayeredPromptAssembler

A concrete implementation `LayeredPromptAssembler` encapsulates the existing XML prompt building logic:

1. Loads preset sections directly (no delimiter hack)
2. Builds `system_prompt` from preset sections: `role_definition`, `instructions`, `global_rules`
3. Builds `post_history_prompt` from preset sections: `writing_style`, `output_format`, `response_length`
4. Renders all XML layers (game state, NPCs, player, world info, history, user input)
5. Appends post-history prompt before user input
6. Applies token budget enforcement via `fit_messages_to_context`
7. Returns `AssembledPrompt`

### Token Budget Configuration

Moved from backend structs to `LayeredPromptAssembler`:

- `max_context_tokens`: Maximum tokens for context window
- `max_tokens`: Maximum tokens for response

### Call Sites

**ActionPipeline**: Instead of `self.service.narrate_action(&context)`:
```rust
let preset = self.ctx.preset_storage.load(...)?;
let assembled = self.service.assembler().assemble(&context, &preset, ...)?;
self.service.complete(agent_name, &assembled.system_prompt, &assembled.user_prompt, Some(assembled.max_tokens))?;
```

**Bootstrap**: Load preset, call assembler directly, then call `backend.complete(...)`

## Consequences

### Positive

1. **Single Responsibility**: Each backend focuses on transport (HTTP, API keys, response parsing) without knowing about prompt assembly
2. **No Duplication**: Prompt assembly logic lives in one place (`LayeredPromptAssembler`)
3. **Pluggable Strategies**: Different assembler implementations can be injected (e.g., testing, A/B testing different prompt strategies)
4. **Cleaner Traits**: `LlmBackend` is minimal and focused on transport primitives
5. **Delimiter Eliminated**: No more `<!--POST_HISTORY-->` hack; preset sections are loaded directly
6. **Correct Configuration Ownership**: Token budgets live with the component that uses them (assembler, not backend)

### Negative

1. **Migration Cost**: Multiple files require updates (backends, pipeline, bootstrap, tests)
2. **Test Migration**: Context-based backend tests move to assembler test suite
3. **Breaking Change**: All backend implementations must be updated to remove context methods

### Trade-offs
- Chose trait decoupling over monolithic backend (clarity won; migration cost accepted as one-time)
- Chose moving token budgets to assembler over keeping on backend (configuration ownership won)
- Chose removing delimiter hack over backward compat (cleaner trait surface won)
