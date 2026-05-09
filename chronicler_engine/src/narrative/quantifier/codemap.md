# chronicler_engine/src/narrative/quantifier/

## Responsibility
Post-narration analysis. Uses a secondary LLM pass to detect movement intent and which NPCs are present in the current room after narration. Bridges the gap between free-form LLM output and structured game state updates.

## Design Patterns
- **Trait-Based Backends**: `QuantifierBackendTrait` with `RealQuantifierBackend` and `MockQuantifierBackend`.
- **Retry with Fallback**: `quantify_room_with_llm_call()` attempts parsing twice before falling back to static room NPCs.
- **Confidence Grading**: `QuantifierConfidence` (`High`/`Medium`/`Low`) determines how much trust to place in results.

## Data & Control Flow
```
narration_text + game_state
  → QuantifierPromptBuilder::build() → (system_prompt, user_prompt)
    → LLM call (quantifier model)
      → parse_quantifier_response_with_movement()
        → JSON parse → npc_ids + movement
          → fallback: regex text extraction if JSON fails
            → QuantifierResult { npcs, movement }
              → compute_npc_events(previous, current) → NpcEventList
```

## Integration Points
- **Consumed by**: `engine/game_service.rs` (post-narration quantification)
- **Depends on**: `model/` (room, NPCs, state), `narrative/llm/` (for LLM calls)

## Files
| File | Purpose |
|------|---------|
| `types.rs` | `QuantifierResult`, `MovementParseResult`, `NpcEvent`, `QuantifierConfidence` |
| `core.rs` | `determine_npcs_in_room()` — main quantifier orchestration with retry logic |
| `parser.rs` | `parse_quantifier_response()`, `compute_npc_events()`, movement extraction |
| `prompt.rs` | `QuantifierPromptBuilder` — constructs quantifier-specific prompts |
| `backends.rs` | `QuantifierBackendTrait`, `RealQuantifierBackend`, `MockQuantifierBackend` |
| `mod.rs` | Module exports and test module declarations |
