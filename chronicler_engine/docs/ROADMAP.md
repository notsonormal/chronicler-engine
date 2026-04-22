# Chronicler Engine: Project Roadmap

## 🎯 Long-Term Vision
The goal of **Chronicler Engine** is to create a "Living Text Adventure" that bridges the gap between structured RPGs (like Zork or D&D) and free-form AI roleplay (like SillyTavern).

The vision is an immersive world where:
1. **Semantic Understanding**: You can type anything, and the world reacts naturally via a Game Master LLM.
2. **Spatial Awareness**: NPCs exist in specific locations, have personal goals, and remember their interactions with you.
3. **Hard Logic + Narrative Freedom**: The "Soft" LLM narration is grounded by "Hard" engine state (inventory, health, location, quests).
4. **Autonomous State Mutation**: The AI can decide to change the world state (e.g., "Carla hands you a key", and the key actually appears in your `/inventory`).

---

## 🏗 Current Architecture Status

### Phase 1: Foundations (COMPLETED)
- [x] **Core Rust Engine**: Basic execution loop and state management.
- [x] **Data-Driven World**: Loading external JSON for worlds, maps, and characters.
- [x] **LLM Integration**: OpenRouter backend with trait abstraction for testing.
- [x] **Semantic Navigation**: "Walk to Kitchen" instead of just "go north".
- [x] **GM Narration System**: Catch-all free text input routed to LLM with full context.

---

## 🚀 The Path Forward

### Phase 2: Agency & Persistence (NEXT)
The immediate focus is making the world "persistent" and allowing the LLM to actually *do* things.

- [ ] **Spec 06: World Persistence**: Save and load the `GameState` to/from JSON files (Save/Load system).
- [ ] **Spec 07: LLM Function Calling**: Allow the GM to trigger engine actions (e.g., `move_item`, `change_stat`) via JSON output.
- [ ] **Spec 08: Advanced Parser UI**: Keyboard shortcuts, command history, and better visual status bars in the terminal.

### Phase 3: Systems & Mechanics
Adding the "Gaming" elements to the Roleplay.

- [ ] **Spec 09: Item Interactions**: Picking up, dropping, examining, and using items through the GM.
- [ ] **Spec 10: Character Stats & Combat**: Health, strength, and a narrative-driven combat system.
- [ ] **Spec 11: Physical Interactions (Quantification)**: Implementation of the "Scene Quantifier" LLM to extract facts like pushing, grabbing, and movement before narration.
- [ ] **Spec 12: Time & Schedules**: NPCs moving between rooms based on a world clock.

### Phase 4: Intelligence & Memory
The focus on making NPCs feel "alive" over long sessions.

- [ ] **Spec 12: Long-Term Memory**: Integration of Vector DBs or Summary buffers so NPCs remember past conversations accurately.



---

## 🛠 Active Tasks (Current Focus)
1. **Implement Persistence**: Allow the user to type `save` or `load` to maintain their session.
2. **Draft Function Calling Spec**: Research the best way to extract "Engine Intents" from the GM's narration block.
3. **Refine Redmist Estate Data**: Fully populate the rooms with Items and interactive objects to test the GM's environmental awareness.
