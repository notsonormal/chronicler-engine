# Phase 3: Evolution Stress Test — Findings

**Date:** 2026-05-09  
**Scope:** Trace three hypothetical features through the current architecture to find hidden coupling and rigidity  
**Method:** Thought experiment grounded in actual code structure from Phases 1-2

---

## Executive Summary

| Scenario | Verdict | First Blocker | Estimated Refactor Size |
|----------|---------|---------------|------------------------|
| **A: Combat system** | Feasible with medium refactor | No place for ephemeral combat state | Medium (3-5 files) |
| **B: Multiplayer** | Major restructure required | `GameState` assumes single player | Large (8+ files, core redesign) |
| **C: Rules engine replacement** | Feasible with medium refactor | Quantifier is load-bearing in action pipeline | Medium (4-6 files) |

**Key insight:** The architecture is **elastic for local features** (combat, items, new actions) but **rigid for systemic changes** (multiplayer, non-LLM backends). The `GameState` decomposition helped readability but didn't improve extensibility — all sub-structs are still `pub` and accessed directly.

---

## Scenario A: Combat System

### Feature Definition
Add simple turn-based combat: HP, initiative, attack command, damage calculation, death/narration.

### Trace Through Architecture

#### 1. Model Layer — Where Does Combat State Live?

**Current `GameState`:**
```rust
pub struct GameState {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<PlayerCard>,
    pub npcs: HashMap<String, NpcCard>,
    pub movement: MovementState,
    pub narrative: NarrativeState,
    pub scene: SceneState,
    pub character_state: CharacterState,
}
```

**Problem:** No sub-struct for ephemeral encounter state. Combat is not a room, not narrative history, not movement, not scene presence.

**Options:**
- **Add to `CharacterSheet`:** HP belongs to the character, but combat is an *encounter* state (who is fighting whom, whose turn is it). Mixing encounter state into character sheets couples combat to character loading.
- **Add `CombatState` to `GameState`:** New sub-struct: `combat: Option<CombatState>`. Cleanest. But requires touching `GameState::new`, all fixtures, and any code that constructs `GameState` directly (test_support raw constructors).
- **Add to `SceneState`:** "Combat is part of the current scene." Plausible but overloading `SceneState` (already narrow: just `npcs_in_area`).

**Blocker:** `GameState` sub-structs are not extensible. Adding a new top-level field requires updating:
- `GameState::new` (initialization)
- `GameState` struct definition
- `TestGameState` all constructors (5 of them)
- Any `GameState` literal construction in tests
- Potentially serde (if `CombatState` needs persistence)

**Assessment:** Medium pain. ~50 lines of boilerplate across 4-5 files.

#### 2. Engine Layer — New Actions and State Machine

**Current `Action` enum:**
```rust
pub enum Action {
    Look,
    Inventory,
    Talk(String, Option<String>),
    FreeAction(String),
    Quit,
}
```

**Adding combat actions:** `Attack(String)`, `Defend`, `UseItem(String)`.

**Impact:**
- `engine/parser.rs` — add parsing logic (straightforward)
- `engine/game_service.rs` — add handler arms in `execute_action` (straightforward, follows existing pattern)
- `engine/action_processing.rs` — combat resolution logic. Where does it live?

**Problem:** `action_processing.rs` currently handles *narration-driven* free actions (movement, NPC events, triggers). Combat is *rules-driven* (roll initiative, check HP, apply damage). These are different shapes.

**Options:**
- **Add `process_combat` to `action_processing.rs`:** Follows existing convention but mixes narrative and combat logic in one file.
- **Create `engine/combat.rs`:** Cleaner separation. But combat resolution may need to call `add_log` (narrative), `apply_npc_events` (character_state), and `evaluate_triggers` (combat could be a trigger condition). This means `combat.rs` imports from `action_processing.rs` and `trigger_eval.rs` — manageable but new dependency web.

**Blocker:** `engine/action_processing.rs` is already the "everything else" file. Combat doesn't fit cleanly into the existing `execute_freeaction_impl` pipeline.

**Assessment:** Medium pain. Need a new module or significant expansion of action_processing.

#### 3. Trigger System — Combat as Trigger Condition

**Current `TriggerCondition`:**
```rust
pub enum TriggerCondition {
    TimesMet(ComparisonOperator, u32),
}
```

**Adding combat conditions:** `HpBelow(u32)`, `InCombat`, `PlayerHealthBelow(u32)`.

**Impact:**
- `model/trigger.rs` — add variants (straightforward)
- `engine/trigger_eval.rs` — add evaluation logic (straightforward)
- `model/character.rs` or `model/state.rs` — need to store HP somewhere

**Problem:** `TriggerCondition` is an enum. Adding variants is a breaking change for any code that pattern-matches exhaustively. In Rust, the compiler will catch this — good. But it means every match site needs updating.

**Match sites to update:**
- `engine/trigger_eval.rs::check_condition` (1 site)
- Any test code that constructs `TriggerCondition` (multiple tests)
- Serde deserialization of existing world data (backwards compat?)

**Assessment:** Low-medium pain. Enum extension is well-supported by Rust.

#### 4. Narrative Layer — Combat Narration

**Current pattern:** LLM narrates everything. Combat actions could be:
- **Narrated by LLM:** "You swing your sword at Carla..." (existing FreeAction path, no change needed)
- **Rules-driven with LLM flavor:** System calculates damage, LLM narrates the result (new path)

**Problem:** If combat is rules-driven (calculate HP, check death), the LLM must not contradict the rules. Current prompt system has no concept of "narrate this mechanical result."

**Options:**
- **FreeAction path:** Let LLM narrate combat freely. Risk: LLM hallucinates damage/death that contradicts actual HP.
- **Hybrid path:** System resolves combat, injects result into prompt ("Carla takes 5 damage. She has 3 HP remaining."), LLM narrates around the facts.

**Blocker:** The prompt builder (`narrative::prompt::PromptBuilder`) has no layer for "system-calculated combat results." Would need Layer 8+ or a special injection point.

**Assessment:** Medium-high pain if hybrid approach desired. Low pain if fully LLM-driven (but inconsistent).

#### 5. Server Layer — Combat UI

**Current templates:** Story log, action area, visual sidebar, header.

**New UI needs:**
- HP bars (player and NPCs)
- Initiative order
- Combat action buttons (Attack, Defend, Flee)

**Impact:**
- `server/templates.rs` — new templates (straightforward)
- `server/fragments.rs` — new fragment handlers (straightforward)
- `server/fragments.rs::render_action_area` — currently shows exits and status. Would need combat-mode action area.

**Assessment:** Low pain. Server layer is already set up for fragment-based UI.

### Combat Verdict

**Feasible with medium refactor.** The first blocker is `GameState` extensibility (no slot for combat state). After that, the engine and narrative layers can adapt. The trigger system is well-suited to combat conditions. The biggest design decision is whether combat is LLM-driven (easy, inconsistent) or rules-driven (harder, consistent).

---

## Scenario B: Multiplayer (2-Player Co-op)

### Feature Definition
Two players share the same world, see each other's actions, can be in different rooms.

### Trace Through Architecture

#### 1. Model Layer — Single Player Assumption

**Current `GameState`:**
```rust
pub struct GameState {
    pub player: Arc<PlayerCard>,        // ← ONE player
    pub npcs: HashMap<String, NpcCard>, // ← NPCs only
    // ...
}
```

**Problem:** `player` is singular. `npcs` is explicitly non-player characters. There is no `players: HashMap<String, PlayerCard>`.

**Options:**
- **Change `player` to `players: Vec<PlayerCard>`:** Breaks every reference to `state.player` (~48 references across 15 files). Massive blast radius.
- **Add `players: HashMap<String, PlayerCard>` alongside `player`:** Backwards compatible but creates ambiguity (which is "the" player?).
- **Create `PlayerState` sub-struct:** `players: HashMap<String, PlayerState>` where `PlayerState` contains `PlayerCard`, `current_room_id`, `inventory`, etc. This is the right design but requires moving `current_room_id` out of `MovementState` (it becomes per-player).

**Blocker:** `MovementState.current_room_id` assumes one player location. In multiplayer, each player has their own location.

**Assessment:** Large. Core data model redesign.

#### 2. Engine Layer — Action Dispatch

**Current `GameService` trait:**
```rust
pub trait GameService: Send + Sync {
    fn execute_action(&self, state: Arc<Mutex<GameState>>, input: String, player_name: String);
}
```

**Problem:** `execute_action` takes a single `player_name`. It dispatches to `Action::FreeAction(text)` with no concept of "which player."

**Impact:**
- `GameService` trait needs a player identifier parameter.
- `engine/parser.rs` — does parsing depend on player? Currently no. But if players have different commands (e.g., admin vs player), parsing becomes player-aware.
- `engine/action_processing.rs::handle_movement` — moves "the" player. In multiplayer, moves "this" player.
- `engine/action_processing.rs::execute_freeaction_impl` — `scene.npcs_in_area` is a single Vec. If players are in different rooms, "the scene" is different per player.

**Blocker:** `SceneState.npcs_in_area` is global, not per-player. Two players in different rooms need different scene states.

**Assessment:** Large. Scene state, movement state, and action dispatch all need per-player scoping.

#### 3. Concurrency — The Mutex Becomes a Bottleneck

**Current pattern:** One `Arc<Mutex<GameState>>`. Lock → work → drop.

**With two players:**
- Player A types "look around" → locks state → brief work → drops
- Player B types "go north" → locks state → brief work → drops

**Problem:** The mutex serializes ALL actions. Even if players are in different rooms, they cannot act concurrently. With 2 players this is fine. With 10+ players, the mutex becomes a severe bottleneck.

**Worse:** FreeAction involves:
1. Lock → clone data → drop
2. LLM call (1-10 seconds, no lock)
3. Lock → apply result → drop

If Player A is in step 2 (LLM call), Player B can act freely. But if both players trigger LLM calls simultaneously, the server spawns two `spawn_blocking` tasks. Each task re-locks independently in step 3. No deadlock, but the second player's step 3 waits for the first player's step 3 to release the lock.

**Blocker:** `std::sync::Mutex` is not async-aware. Two blocking tasks both trying to re-lock will contend. With many players, this creates unpredictable latency spikes.

**Options:**
- **Per-room mutexes:** Players in different rooms don't contend. But what if they move into the same room? Need lock ordering.
- **Actor model:** Each player is an actor with their own state. Room state is shared via message passing. This is the "right" design but requires replacing the entire state management layer.
- **RwLock:** Multiple readers (look, status checks), exclusive writers (movement, combat). Helps for read-heavy operations.

**Assessment:** Very large. The current architecture is fundamentally single-player.

#### 4. Narrative Layer — Shared World, Independent Views

**Current prompt context:**
```rust
pub struct PromptContext<'a> {
    pub world: &'a WorldCard,
    pub room: &'a Room,
    pub all_npcs: &'a [NpcCard],
    pub npcs_in_area: &'a [NpcCard],
    pub player: &'a PlayerCard,
    pub user_message: &'a str,
    pub history: &'a [LogEntry],
}
```

**Problem:** `player` is singular. `history` is singular. In multiplayer, each player has their own message history and their own view of the scene.

**Impact:**
- `PromptContext` needs per-player scoping.
- `history` currently alternates `Input` / `Narration` globally. With two players, history becomes interleaved: "Player A: look around", "Narration: You see...", "Player B: go north", "Narration: You walk...". The LLM must distinguish who did what.
- `QuantifierResult` is per-action. If Player A's action causes an NPC to enter, does Player B see the NPC immediately?

**Blocker:** The entire narrative pipeline assumes one player, one history, one scene.

**Assessment:** Very large. Would require redesigning `PromptContext`, `history` semantics, and quantifier scope.

#### 5. Server Layer — Multiple Connections

**Current `AppState`:**
```rust
pub struct AppState {
    pub state: Arc<std::sync::Mutex<GameState>>,
    pub game_service: Arc<dyn GameService>,
    pub settings: Arc<std::sync::RwLock<AppSettings>>,
    pub cancel_token: CancellationToken,
}
```

**Problem:** `state` is singular. Two HTTP connections share the same `AppState` and the same `GameState`.

**Impact:**
- No change needed to `AppState` structure — just the `GameState` it wraps.
- SSE / polling: both players poll `/status/generating`. The status is global, not per-player. If Player A triggers an LLM call, Player B sees "Generating..." even though they didn't do anything.

**Blocker:** `GenerationStatus` and `GenerationPhase` are global. Per-player generation tracking needed.

**Assessment:** Medium. Server layer is mostly stateless; the state it serves needs redesign.

### Multiplayer Verdict

**Major restructure required.** The first blocker is `GameState.player` being singular, but the deeper issue is that every layer assumes one player, one location, one history, one scene. The architecture would need:
- Per-player movement state
- Per-player scene state (or per-room scene state)
- Per-player generation status
- Per-player message history (or interleaved history with player attribution)
- Mutex scaling strategy (per-room, actor model, or sharding)

This is not "add a field." This is "redesign the core data model and every layer that touches it."

---

## Scenario C: Rules Engine Replacement

### Feature Definition
Replace the LLM narrator with a deterministic rules engine: parse player input → look up rules → generate templated responses. No external LLM calls.

### Trace Through Architecture

#### 1. Narrative Layer — Backend Trait Abstraction

**Current `LlmBackend` trait:**
```rust
pub trait LlmBackend: Send + Sync {
    fn generate_dialogue(&self, ctx: &PromptContext, npc: &NpcCard) -> Result<String, EngineError>;
    fn narrate_action(&self, ctx: &PromptContext) -> Result<String, EngineError>;
    fn narrate_arrival(&self, ctx: &PromptContext) -> Result<String, EngineError>;
    fn narrate_continuation(&self, system: &str, user: &str, trigger: &str, max: Option<u32>) -> Result<String, EngineError>;
    fn narrate_action_from_prompt(&self, system: &str, user: &str, max: Option<u32>) -> Result<String, EngineError>;
    fn name(&self) -> &str;
}
```

**Problem:** The trait is LLM-shaped. Every method takes a `PromptContext` (which contains world, room, NPCs, history, player) and returns a `String` (narration text).

**Can a rules engine implement this?** Yes, but awkwardly:
- `narrate_action` would ignore most of `PromptContext`, parse the player's command, look up a template, fill in variables.
- `narrate_arrival` would look up the room's arrival description template.
- `generate_dialogue` would look up the NPC's dialogue tree.

**Blocker:** The trait assumes *generation* — it takes rich context and produces prose. A rules engine wants *lookup* — it takes a key and produces text. The mismatch is semantic, not technical.

**Assessment:** Medium. Can implement the trait but would ignore most context. Need a new trait or reshape the existing one.

#### 2. Engine Layer — The Quantifier Problem

**Current flow:**
```
Player input → parse → LLM narrate → Quantifier analyze → apply state changes
```

**Problem:** The quantifier is load-bearing. It determines:
- Which NPCs are present (`QuantifierResult.npcs`)
- Whether the player moved (`QuantifierResult.movement`)

In a rules engine, these are *known*:
- NPC presence is deterministic (room config + schedule)
- Movement is explicit ("go north" → look up exit)

**But the engine pipeline assumes quantifier output:**
- `game_service.rs:219-228` calls `determine_npcs_in_room` with LLM narration text.
- `action_processing.rs:287-327` calls `execute_freeaction_impl` which expects `QuantifierResult`.
- `action_processing.rs:322` calls `compute_npc_events` to diff previous vs current NPCs.

**Blocker:** The quantifier is not a swappable backend in practice. It is embedded in the orchestration logic. Removing it requires rewriting `game_service.rs::execute_action` and `action_processing.rs::execute_freeaction_impl`.

**Assessment:** Medium-large. Need to make quantifier optional or bypassable.

#### 3. Engine Layer — PromptBuilder Coupling

**Current `action_processing.rs:150-179`:**
```rust
fn build_trigger_prompt_parts(...) -> Option<(String, String, u32)> {
    let mut pb = PromptBuilder::from_context(&trigger_ctx);
    pb.max_context_tokens = Some(max_context);
    // ...
    match pb.build_split() { ... }
}
```

**Problem:** `PromptBuilder` is imported directly into `action_processing.rs` (engine layer). Even if you replace the LLM backend with a rules engine, the *trigger continuation* path still builds an 8-layer prompt and calls `narrate_action_from_prompt`.

**If triggers are rules-driven too:**
- Trigger narration becomes template lookup, not LLM generation.
- `build_trigger_prompt_parts` becomes unnecessary.
- `TriggerContinuationRequest` becomes unnecessary.

**Blocker:** Trigger continuation assumes LLM generation. Removing the LLM means redesigning triggers.

**Assessment:** Medium. Triggers are already data-driven (condition + action). The "action" is currently an LLM prompt. Changing it to a template string is straightforward.

#### 4. Server Layer — Async Expectations

**Current behavior:**
- FreeAction → HTTP returns "Thinking..." immediately → client polls for result.
- Sync actions (Look, Inventory) → HTTP returns result immediately.

**Rules engine behavior:**
- All responses are instantaneous (no network call).
- "Thinking..." state is unnecessary.
- Polling is unnecessary.

**Impact:**
- `server/fragments.rs` can handle sync responses for FreeAction (just call `game_service.execute_action` inline, don't spawn).
- But the UI expects async behavior. Changing this requires HTMX template updates.

**Assessment:** Low-medium. Server can adapt, but UI expectations need updating.

#### 5. Model Layer — No Changes Needed

The data model (World, Map, Room, NPC, Trigger, State) is backend-agnostic. A rules engine uses the same data.

**Assessment:** None. Model layer is clean.

### Rules Engine Verdict

**Feasible with medium refactor.** The trait abstraction is real but the *orchestration* is LLM-shaped. Key changes:
1. Make quantifier optional (bypass for deterministic movement/NPC presence).
2. Reshape `LlmBackend` trait or create `NarratorBackend` that is less LLM-specific.
3. Redesign trigger continuation from LLM prompt to template string.
4. Support synchronous FreeAction in server (no spawn_blocking).

The model layer needs no changes. The engine layer needs moderate changes. The narrative layer is largely replaced.

---

## Cross-Scenario Findings

### Finding 1: `GameState` Extensibility Is Manual

All three scenarios need new state:
- Combat → `CombatState`
- Multiplayer → per-player state
- Rules engine → no new state, but optional quantifier state

**Problem:** Adding a field to `GameState` requires touching:
- `GameState` struct definition
- `GameState::new`
- All `TestGameState` constructors
- Any code that constructs `GameState` directly
- Potentially serde derives

**This is not extensible.** The decomposition into sub-structs helped readability but didn't solve the "add a new concern" problem.

### Finding 2: The Quantifier Is Load-Bearing

The quantifier is not just a "nice to have" LLM optimization. It determines:
- Player movement (was there movement in the narration?)
- NPC presence (who is in the room now?)
- NPC events (who entered/left?)

A rules engine or deterministic system doesn't need this. But the pipeline assumes it. The quantifier is deeply embedded in `game_service.rs` and `action_processing.rs`.

### Finding 3: Triggers Are Well-Designed for Extension

`TriggerCondition` is an enum. Adding `HpBelow`, `InCombat`, `HasItem` is straightforward. `TriggerAction` is data (name + narration_prompt). Changing it to a template or rule reference is straightforward.

**Assessment:** The trigger system is the most elastic part of the architecture.

### Finding 4: The Server Layer Is the Most Elastic

HTTP handlers, templates, and fragments are easy to extend. New endpoints, new templates, new UI elements — all follow existing patterns. The server doesn't care what's inside `GameState` as long as it can render it.

### Finding 5: Single-Player Is Baked In at Every Layer

Multiplayer isn't just "add more players." It's baked into:
- `GameState.player` (singular)
- `MovementState.current_room_id` (one location)
- `SceneState.npcs_in_area` (one scene)
- `NarrativeState.history` (one history)
- `GenerationState` (one generation status)
- `PromptContext.player` (one player)
- `GameService::execute_action(player_name)` (one player)

There is no "player ID" threaded through any layer. Adding multiplayer requires adding it everywhere.

---

## Recommendations

### If Combat Is on the Roadmap

1. **Add `CombatState` to `GameState`** now, even if empty (`combat: Option<CombatState>`). This establishes the extension point.
2. **Move `current_room_id` to a per-entity concept** — even for single player, model it as `entity_locations: HashMap<String, String>` (entity ID → room ID). This makes multiplayer easier later.
3. **Decide combat philosophy early:** LLM-driven (easy) or rules-driven (consistent). This determines narrative layer changes.

### If Multiplayer Is on the Roadmap

1. **Stop.** The current architecture is not the right foundation. Consider:
   - Actor model (one actor per player, one actor per room)
   - ECS (Entity-Component-System) for state
   - Event sourcing for history
2. **If proceeding anyway:** Start by threading `player_id` through every layer as a non-functional change. Then make state per-player.

### If Rules Engine Is on the Roadmap

1. **Make quantifier optional.** Add a bypass path in `game_service.rs` for deterministic movement/NPC presence.
2. **Generalize `LlmBackend` to `NarratorBackend`.** Remove `PromptContext` from the trait methods; pass only what the backend needs.
3. **Separate trigger effects from LLM prompts.** `TriggerAction.narration_prompt` should become `TriggerEffect::Narrate(String)` or `TriggerEffect::Template(String)`.

---

## Appendix: Blast Radius Summary

| Scenario | Files Touched | Core Change | Peripheral Change |
|----------|--------------|-------------|-------------------|
| **Combat** | `model/state.rs`, `engine/action.rs`, `engine/parser.rs`, `engine/action_processing.rs`, `engine/trigger_eval.rs`, `narrative/prompt/builder.rs` | Add `CombatState`, new `Action` variants, new `TriggerCondition` variants | Templates, fragments, test fixtures |
| **Multiplayer** | `model/state.rs`, `model/map.rs`, `engine/game_service.rs`, `engine/action_processing.rs`, `narrative/prompt/types.rs`, `server/fragments.rs`, `server/mod.rs`, `bootstrap.rs` | Per-player state throughout | UI, connection management, session handling |
| **Rules Engine** | `narrative/llm/mod.rs`, `engine/game_service.rs`, `engine/action_processing.rs`, `server/fragments.rs` | New backend trait, optional quantifier, sync action path | None (model unchanged) |
