# Blueprint: Scene Quantification Architecture (Dual-LLM)

> [!NOTE]
> This is a **Hypothetical Specification** stored in the `/blueprints/` directory. It defines the proposed architecture for Phase 3 of the roadmap and is currently NOT implemented.

## Objective
To enable complex physical world interactions (Pushing, Grabbing, Brawling) by separating natural language understanding from deterministic state changes.

## The Dual-LLM Pipeline
Instead of a single Narration engine, the system will use a two-stage pipeline:

### 1. The Intent Layer (The "Quantifier")
- **Model**: Ultra-fast, lightweight LLM (e.g., Llama-3-8B or GPT-4o-mini).
- **Mission**: Map the user's free-text input to a structured **Intent Enum**.
- **Example**: 
    - *Input*: "I shove Carla into the courtyard."
    - *Output*: `{"intent": "PHYSICAL_MOVE", "target": "carla", "destination": "courtyard"}`.

### 2. The Verification Layer (The Engine)
- **Role**: Validates the intent against the `GameState`.
- **Logic**:
    - Is the `target` in the same room as the player?
    - Is the `destination` adjacent to the current room?
    - Is the `target` incapacitated?
- **Result**: The engine performs the move or rejects the intent with an `EngineError`.

### 3. The Narrative Layer (The "Storyteller")
- **Model**: High-fidelity LLM (e.g., Claude 3.5 Sonnet or GLM-4.5).
- **Mission**: Write the immersive description based on the **Hard Result** from the engine.
- **Example**:
    - *Context*: "Attempt: Push Carla. Result: Success. State: Carla is now in Courtyard."
    - *Output*: "With a sudden, powerful lunge, you catch Carla off-guard. She stumbles back through the archway, her boots skidding on the courtyard gravel as the gate clangs behind her."

## Benefits
- **Deterministic Simulation**: The world state remains 100% correct.
- **Zero-Hallucination Actions**: NPCs cannot "teleport" via loose AI narration; they only move if the engine logic allows it.
- **Combat Readiness**: This architecture provides the foundation for a "Narrative Combat" system with hard health and damage stats.
