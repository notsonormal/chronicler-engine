# Chronicler Engine: Project Roadmap

## Long-Term Vision
The goal of **Chronicler Engine** is to create a "Living Text Adventure" that bridges the gap between structured RPGs (like Zork or D&D) and free-form AI roleplay (like SillyTavern).

The vision is an immersive world where:
1. **Semantic Understanding**: You can type anything, and the world reacts naturally via a Game Master LLM.
2. **Spatial Awareness**: NPCs exist in specific locations, have personal goals, and remember their interactions with you.
3. **Hard Logic + Narrative Freedom**: The "Soft" LLM narration is grounded by "Hard" engine state (inventory, health, location, quests).
4. **Autonomous State Mutation**: The AI can decide to change the world state (e.g., "Carla hands you a key", and the key actually appears in your `/inventory`).

---

## Current Architecture Status

### Phase 1: Foundations (COMPLETED)
- [x] Core Rust Engine: Basic execution loop and state management.
- [x] Data-Driven World: Loading external JSON for worlds, maps, and characters.
- [x] LLM Integration: OpenRouter, Ollama, DeepSeek backends with trait abstraction for testing.
- [x] Semantic Navigation: "Walk to Kitchen" instead of just "go north".
- [x] GM Narration System: Catch-all free text input routed to LLM with full context.
- [x] HTMX Web Dashboard: Real-time UI with polling-based updates.

### Phase 2: Agency & Persistence (COMPLETED)
- [x] SQLite Snapshots: Save and load `GameState` per turn; reset game support.
- [x] Agent Trait + Registry: Extensible `dyn Agent` architecture with quantifier as post-generation agent.
- [x] Structured Error Taxonomy: `EngineError` with typed `LlmFailure`, `NarrativeFailure`, and `InternalError` variants.
- [x] Text Check Integration: Spell/grammar checking via harper-core with configurable modes.
- [x] Settings System: Persistent JSON-based connection profiles with UI management.
- [x] Granular Retry Logic: Pre-generation snapshots for main narration and trigger continuations.

---

## The Path Forward

### Phase 3: Systems & Mechanics
Adding the "Gaming" elements to the roleplay.

- [ ] **Item Interactions**: Picking up, dropping, examining, and using items through the GM.
- [ ] **Character Stats & Combat**: Health, strength, and a narrative-driven combat system.
- [x] **Physical Interactions (Quantification)**: Scene Quantifier LLM extracts movement and presence from narration.
- [ ] **Time & Schedules**: NPCs moving between rooms based on a world clock.
- [ ] **LLM Function Calling**: Allow the GM to trigger engine actions (e.g., `move_item`, `change_stat`) via structured output.

### Phase 4: Intelligence & Memory
Making NPCs feel "alive" over long sessions.

- [ ] **Long-Term Memory**: Integration of vector DBs or summary buffers so NPCs remember past conversations accurately.

---

## Active Tasks
1. **Refine Redmist Estate Data**: Fully populate rooms with items and interactive objects to test GM environmental awareness.
2. **Draft Function Calling Spec**: Research the best way to extract engine intents from GM narration.
