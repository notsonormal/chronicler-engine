# chronicler_engine/src/narrative/prompt/

## Responsibility
Prompt construction and token budget management. Builds structured LLM prompts from game state (world, room, NPCs, player, history) and ensures they fit within context window limits.

## Design Patterns
- **Builder Pattern**: `PromptBuilder` constructs prompts layer by layer (system, game state, NPC cards, player, world info, history, user message).
- **Template Method**: `render_*_layer()` methods build each prompt section.
- **Budget-Aware Truncation**: `fit_messages_to_context()` drops oldest history entries when prompts exceed token limits.

## Data & Control Flow
```
PromptContext { world, room, npcs, player, history, user_message }
  → PromptBuilder::from_context()
    → build_split()
      → render_system_layer() → system_prompt
      → render_game_state_layer() + render_npc_cards_layer() + ... → user_prompt
        → fit_messages_to_context(system, user, max_context, requested_max_tokens)
          → estimate_tokens() + truncate_to_budget()
            → (fitted_system, fitted_user, actual_max_tokens)
```

## Integration Points
- **Consumed by**: `engine/action_processing.rs` (trigger continuation prompts), `narrative/llm_client.rs`
- **Depends on**: `model/` (world, map, character, state)

## Files
| File | Purpose |
|------|---------|
| `types.rs` | `PromptBuilder`, `PromptContext`, `PromptLayer` — core types |
| `builder.rs` | `PromptBuilder` implementation with `build()` and `build_split()` |
| `context.rs` | `fit_messages_to_context()` — token budget fitting and history trimming |
| `budget.rs` | Token estimation, truncation, safety margins |
| `templates.rs` | Askama/Handlebars-style prompt templates (system, narration, quantifier) |
| `sanitize.rs` | `sanitize_for_prompt()` — cleans text for prompt insertion |
| `mod.rs` | Module exports |
