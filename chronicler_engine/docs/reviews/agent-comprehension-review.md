# Architectural Review: AI Agent Comprehension Challenges

**Reviewer:** Kimi Code CLI  
**Date:** 2026-05-30  
**Scope:** Entire Chronicler Engine codebase  
**Related:** `local://agent-comprehension-review.md` (investigation plan)

---

## Executive Summary

The Chronicler Engine has **high documentation quality** but **significant comprehension friction** in areas requiring domain knowledge to navigate. Without explicit understanding of these patterns, an AI agent will misunderstand, make incorrect changes, or miss load-bearing invariants.

### Severity Matrix

| Area | Comprehension Difficulty | Severity |
|------|--------------------------|----------|
| Error Handling | Three-layer hierarchy, one-way boundary, dual load_state functions | CRITICAL |
| Trigger System | Counterintuitive timing (evaluate BEFORE increment), index-based identity | CRITICAL |
| Game Service | Strict mutation ordering enforced by structure, not logic | CRITICAL |
| State Management | Nested sub-states, snapshot exclusion, Message/Swipe duality | HIGH |
| Web Server | Dual rendering paradigm, scattered HTMX attributes, monolithic router | HIGH |
| Storage | Three-layer abstraction, dual representation mapping, game-scoped isolation | MEDIUM |
| Bootstrap | Initialization ordering, settings threading | MEDIUM |
| LLM Integration | Three-tier architecture, quantifier dual role, token budgeting | MEDIUM |

---

## 1. Error System Architecture

### Finding: One-Way Error Boundary

The error system implements a one-way boundary: `EngineError` can convert to `ApplicationError::Engine`, but not the reverse.

**Architecture:**
```
EngineError (core domain)
    └─ From<LlmFailure>     (LLM API errors)
    └─ From<NarrativeFailure> (prompt/narration errors)
    └─ InternalError        (invariant violations)

ApplicationError (HTTP layer)
    └─ ApplicationError::Engine(EngineError)  ← ONE WAY ONLY
    └─ ApplicationError::InvalidInput(String)
    └─ ApplicationError::GenerationInProgress
```

**Why it exists:** Engine errors are infrastructure concerns (LLM timeout, parse failure, invariant violation). Application errors are user-facing (bad input, generation in flight). The reverse conversion would leak infrastructure details to the user.

**What breaks if violated:** An `ApplicationError` with internal context could escape to HTTP handlers that shouldn't see LLM error details. Trust boundaries collapse.

### Finding: Dual load_state Functions

Two state-loading functions exist with opposite failure semantics:

| Function | Returns | Use Case |
|----------|---------|----------|
| `try_load_state(ctx)` | `Result<GameState, EngineError>` | Tests — must handle failure explicitly |
| `load_state(ctx)` | `GameState` (unwraps) | Production — graceful degradation |

**Implementation difference:**
```rust
// src/application/context.rs
pub fn try_load_state(ctx: &GameServiceContext) -> Result<GameState, EngineError> {
    // Explicit: returns Result, caller must handle
}

pub fn load_state(ctx: &GameServiceContext) -> GameState {
    // Graceful: on error, returns fresh state instead of failing
    try_load_state(ctx).unwrap_or_else(|_| GameState::default())
}
```

**What breaks if confused:** Tests that use `load_state()` won't catch snapshot corruption — they'll silently get a fresh state. Production code that uses `try_load_state()` will panic on any error instead of degrading.

### Finding: internal_error() Helper

`EngineError::Internal` wraps invariant violations via a helper function:

```rust
pub fn internal_error(invariant: impl Into<String>) -> InternalError {
    InternalError { invariant: invariant.into() }
}
```

**Why it matters:** This is the idiomatic way to signal that a precondition was violated. Direct `EngineError::Internal` construction is verbose and error-prone.

---

## 2. State Management

### Finding: Nested Sub-State Structure

```rust
pub struct GameState {
    pub world: Arc<WorldCard>,           // World data (loaded from JSON)
    pub map: Arc<MapDef>,                // Map definition (loaded from JSON)
    pub player: Arc<PlayerCard>,         // Player persona (loaded from JSON)
    pub movement: MovementState,         // current_room_id, visited_rooms
    pub narrative: NarrativeState,        // input_buffer, history, last_trigger
    pub scene: SceneState,               // npcs_in_area, current_context
    pub npcs: HashMap<String, NpcCard>,   // All NPC cards (loaded from JSON)
    pub npc_encounter_log: NpcEncounterLog,// Trigger encounter tracking (JSON field: "character_state")
}
```

**Design rationale:** Decomposition improves readability and enables targeted accessor traits (planned). Each sub-state has distinct mutation patterns:

- `movement`: Changes on room transition
- `narrative`: Changes on LLM response or user input
- `scene`: Changes on quantifier NPC resolution
- `npc_encounter_log`: Changes on trigger evaluation

### Finding: Partial Snapshot Composition

Snapshots (`GameStateSnapshot`) do NOT contain everything — they exclude large data loaded at startup:

```rust
pub struct GameStateSnapshot {
    pub movement: MovementState,
    pub narrative: NarrativeSnapshot,    // Excludes message history
    pub scene: SceneState,
    pub npc_encounter_log: NpcEncounterLog,
    pub created_at: DateTime<Utc>,
}
```

**What's excluded and why:**
- `world`, `map`, `player`, `npcs`: Loaded from JSON at startup, never change during game
- `narrative.history`: Stored separately in `llm_message_storage` table for efficient retrieval

**Why it's confusing:** An AI agent might assume `from_game_state(snapshot.apply_to(state))` restores full state. It doesn't — messages must be loaded separately via `load_messages_with_swipes()`.

### Finding: Message/Swipe Duality

`Message` carries both active content and swipe history:

```rust
pub struct Message {
    pub id: u64,
    pub text: String,                   // Active swipe text
    pub active_swipe_index: usize,      // Which swipe is active
    pub swipes: Vec<Swipe>,            // All generations
    // ... metadata fields also reflect active swipe
}

pub struct Swipe {
    pub text: String,
    pub snapshot_id: Option<u64>,       // State captured when this swipe was created
    pub location_header: Option<String>,
    pub event_header: Option<String>,
}
```

**Runtime behavior:** When `set_active_swipe()` is called, the Message's top-level fields (`text`, `location_header`, `event_header`, `snapshot_id`) are updated to reflect the selected swipe. Inactive swipes live in the `swipes` vector.

**Design rationale:** Efficient rendering — most UI only needs the active swipe. Swipe history is available for navigation without re-parsing.

**What breaks if misunderstood:** Writing to `message.text` directly (instead of via `set_active_swipe()`) creates inconsistent state — the active swipe and the top-level fields diverge.

---

## 3. Trigger System

### Finding: Counterintuitive Evaluation Timing

Trigger evaluation happens BEFORE `times_met` increment. This is load-bearing.

**The flow:**
1. Quantifier detects NPCs in narration
2. **Evaluate triggers** (at this point, `times_met` still = 0)
3. Trigger fires (e.g., `TimesMet Eq 0` is TRUE because counter hasn't incremented)
4. **Increment times_met** (now becomes 1)

**Why it exists:** If step 4 happened before step 2, `TimesMet Eq 0` would immediately become false, and first-encounter triggers would never fire.

**What breaks if order is swapped:** Introduction triggers never fire. `times_met` starts at 0, player encounters NPC, counter increments to 1, trigger evaluation sees `times_met = 1`, `TimesMet Eq 0` is false. The trigger never fires.

**Code location:** `src/engine/trigger_eval.rs` — `evaluate_triggers()` is called before `apply_npc_events()` which calls `increment_times_met()`.

### Finding: Index-Based Trigger Identity

Triggers are identified by their INDEX into the NPC's trigger vector, not by a unique ID:

```rust
pub struct NpcEncounterState {
    pub times_met: u32,
    pub trigger_fired: HashMap<usize, bool>,  // INDEX, not UUID
    pub currently_meeting: bool,
}
```

**Why it exists:** Triggers are defined in JSON NPC cards, not in the database. There's no persistent ID — the index is derived from the array order in the card file.

**What breaks if triggers are reordered:** If an NPC's trigger array is modified mid-game (new trigger inserted at index 2, shifting existing indices), the `trigger_fired` HashMap references stale indices. A trigger that was "fired" maps to a different trigger after edit.

**Risk:** Editing NPC cards mid-game corrupts encounter tracking. Document this limitation.

### Finding: Two-Level State Tracking

The trigger system tracks encounter cycles via two variables:

```rust
pub struct NpcEncounterState {
    pub times_met: u32,              // Completed encounter cycles (enter → exit → enter)
    pub trigger_fired: HashMap<usize, bool>,  // Non-repeatable triggers fired
    pub currently_meeting: bool,      // Player is currently in same room
}
```

**Cycle semantics:**
- `times_met` increments when `currently_meeting` transitions false→true
- `currently_meeting` is set true on room entry, false on room exit
- One cycle = entering room with NPC → leaving → re-entering

**Why two variables:** `currently_meeting` handles the "are they here now" state for same-room re-evaluation. `times_met` handles "how many times has this happened" for trigger requirements.

### Finding: Repeat Semantics

| `repeat` | Behavior | `trigger_fired` growth |
|----------|----------|------------------------|
| `true` | Can fire multiple times | Never grows |
| `false` | Fires once, then marked in `trigger_fired` | Grows indefinitely |

**Non-obvious consequence:** `repeat: false` entries accumulate in `trigger_fired` HashMap across a game session. No cleanup occurs — the HashMap grows until the NPC's trigger array is modified or the game ends.

---

## 4. Game Service Orchestration

### Finding: Load-Bearing Mutation Order

The action pipeline mutates state in a strict, enforced order. This is documented in `docs/system/triggers.md` and `docs/system/game_flow.md`:

| Step | Operation | Why it must come here |
|------|-----------|----------------------|
| 1 | `handle_movement()` — updates `movement.current_room_id` | Room must be current before NPCs are resolved |
| 2 | Resolve current NPCs from quantifier result | Uses updated `movement.current_room_id` from step 1 |
| 3 | `state.add_log(narration_text)` | Narration must be in history before triggers read it |
| 4a | `evaluate_triggers()` + build prompt | Reads `state.narrative.history()` (step 3) to build continuation prompt |
| 4b | Trigger LLM call | Runs outside state lock (frontend can poll main narration) |
| 4c | `commit_trigger_narration()` | Re-acquires lock to add trigger logs and mark fired |
| 5 | `apply_npc_events()` — mutates `npc_encounter_log` | `times_met` increments AFTER trigger evaluation |

**How it's enforced:** Code structure, not runtime checks. Violations compile but break behavior silently.

**What breaks if reordered:**
- Steps 3 and 4a swapped: Triggers generate without current narration as context
- Steps 4a and 5 swapped: `TimesMet Eq 0` never fires (see Trigger Timing finding)
- Step 1 moved after step 3: Narration logged against old room, then room changes — inconsistent state
- Step 4b moved inside lock: Frontend cannot poll main narration until trigger LLM completes

### Finding: Lock Release Before LLM Call

The state lock is released during LLM calls, then re-acquired for commit:

```rust
// Inside execute_freeaction_impl or ActionPipeline
let prompt = evaluate_triggers(state);  // Lock held
drop(state_guard);                    // Lock released

let result = llm_call(prompt).await;   // LLM call, no lock

let mut state_guard = ctx.state.lock().await;  // Lock re-acquired
commit_trigger_narration(&mut state_guard, result);  // Commit, lock held
```

**Why it exists:** HTMX polling can fetch main narration while trigger continuation is generating. The frontend sees updates sooner without waiting for the entire pipeline.

**What breaks if changed:** Polling returns "still generating" until trigger completes, even if main narration is ready. User sees delay with no progress indication.

---

## 5. Web Server Architecture

### Finding: Dual Rendering Paradigm Coexists

Two rendering approaches are used:

| Paradigm | Module | Characteristics |
|----------|--------|-----------------|
| Askama templates | `src/server/templates.rs`, `server/fragments/` | Type-safe, compiled, consistent |
| String concatenation | Inline HTML in handlers | Flexible, verbose, error-prone |

**Why both exist:** Askama is used for page-level templates and consistent fragment rendering. String concat is used for dynamic HTML construction in handlers that need conditional logic or inline data embedding.

**Risk:** No clear rule for which paradigm applies where. An AI agent might use the wrong approach or mix them inconsistently.

### Finding: Monolithic Router

`build_router()` in `src/server/mod.rs` contains 50+ routes in one function:

```rust
pub fn build_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", GET index_handler)
        // ... 40+ more routes
        .route("/action", POST action_handler)
        .route("/action/process", POST process_action_handler)
        // ... fragment routes
        .with_state(app_state)
}
```

**Why it's monolithic:** Routes are registered at startup; grouping by feature requires refactoring. The current structure prioritizes simplicity over organization.

**Comprehension challenge:** Finding where a specific route is registered requires scanning 150+ lines of route definitions.

### Finding: GenerationGuard RAII Pattern

`GenerationGuard` uses RAII to ensure flag release on panic:

```rust
struct GenerationGuard(Arc<AtomicBool>);

impl Drop for GenerationGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
```

**Why RAII:** If the guard is dropped due to panic (e.g., LLM call fails and unwrap panics), the flag still releases. Manual flag management would require `finally` blocks or scoped guard patterns.

**What breaks if simplified:** A non-RAII pattern that doesn't handle panic would leave the generation flag set, blocking future actions until server restart.

---

## 6. Storage Architecture

### Finding: Three-Layer Abstraction

Storage uses three distinct layers:

```
Domain Models (src/model/)     → Pure business logic types
    ↓
DB Models (src/storage/models/) → Flat table-row structs, JSON columns
    ↓
Mappers (src/storage/mappers/)  → Bidirectional conversion
    ↓
Backend (src/storage/backend/)  → SQLite/InMemory/Test dispatch
```

**Example mapping:**
```rust
// Domain
pub struct Message { pub id: u64, pub text: String, pub swipes: Vec<Swipe>, ... }

// DB Model
pub struct DbMessage { pub id: i64, pub text: String, pub swipes_json: String, ... }

// Mapper
impl MessageMapper {
    fn to_db(msg: &Message) -> DbMessage { ... }
    fn from_db(db: DbMessage) -> Message { ... }
}
```

**Why it exists:** SQLite stores flat rows with JSON columns. Domain models use nested structs. Mappers handle the translation, enabling:
- Storage to evolve independently of domain
- InMemory backend for testing without SQLite
- JSON columns to store complex nested data without schema changes

**What breaks if not followed:** Adding a field to the domain model requires DB model and mapper updates. Skipping any layer causes data loss or corruption.

### Finding: Game-Scoped Isolation via Atomic Counter

Game scope isolation uses an `AtomicU64` counter passed through `with_backend_mut`:

```rust
pub struct Storage {
    pub backend: Arc<Mutex<Box<dyn StorageBackend>>>,
    pub game_counter: Arc<AtomicU64>,  // Current game ID
}

pub fn with_backend_mut<T>(
    storage: &Storage,
    f: impl FnOnce(&mut dyn StorageBackend) -> T,
) -> T {
    let counter = storage.game_counter.load(Ordering::SeqCst);
    let mut guard = storage.backend.lock().unwrap();
    // ... dispatch with counter
}
```

**Why it exists:** Multiple games can run against the same storage backend without interference. Each operation is tagged with the current game's counter value.

**What breaks if bypassed:** Operations would execute without game context, potentially reading/writing cross-game data.

---

## 7. LLM Integration

### Finding: Three-Tier Architecture

The LLM system has three distinct tiers:

```
Backend (src/narrative/llm/backend.rs)
    └─ LlmBackend trait: send_message(), narrate_action(), narrate_arrival()
    
Prompt (src/narrative/prompt/)
    └─ LayeredPromptAssembler: builds 8-layer prompts with token budgeting
    
Agent (src/narrative/agents/)
    └─ AgentRegistry: dispatches to quantifier, narrator, trigger agents
```

**Why three tiers:** Each has distinct responsibility:
- Backend: HTTP transport, API differences (OpenRouter vs Ollama)
- Prompt: Token budgeting, content organization, layer assembly
- Agent: Role-specific logic, response parsing, tool invocation

### Finding: Quantifier Dual Role

The quantifier agent has both agent and parser responsibilities:

```rust
pub struct QuantifierAgent {
    // Agent role: decides NPC movements and actions
    // Parser role: parses LLM response into structured QuantifierResult
}
```

**Why it exists:** Quantifier output is parsed directly into game state changes. Combining agent and parser reduces abstraction layers and simplifies the flow.

**Risk:** If LLM output format changes, both agent and parser must be updated together.

### Finding: Token Budgeting Complexity

Prompt assembly includes token budget enforcement:

```rust
pub fn assemble_prompt(...) -> Result<String, NarrativeFailure> {
    // 1. Build all layers
    // 2. Check total against max_tokens (default 8192)
    // 3. If overflow: truncate from oldest entries
    // 4. Return assembled prompt
}
```

**Why it's complex:** Different LLMs have different context windows. The assembler must budget tokens across 8 layers while preserving critical information (system prompt, current input, latest history).

---

## 8. Documentation Gaps

### What Exists

| Document | Coverage |
|----------|----------|
| `docs/system/triggers.md` | Trigger timing, mutation order, room scoping, requirements |
| `docs/system/game_flow.md` | Full game loop, retry flow, polling architecture |
| `docs/system/llm_processing.md` | Three-tier architecture, backend selection |
| `docs/system/startup.md` | Initialization sequence, world loading |
| `docs/architecture/system.md` | High-level architecture, module boundaries |
| `docs/adr/` | Design decisions (13 ADRs) |

### What's Missing

| Gap | Impact |
|-----|--------|
| No "why" for snapshot exclusion | AI assumes snapshots contain everything |
| No explicit doc for error boundary direction | AI might attempt reverse conversion |
| No storage mapping guide | AI doesn't know about 3-layer requirement |
| AGENTS.md is verbose (300+ lines) | Critical patterns buried in detail |
| No quick-reference for mutation order | AI might reorder critical steps |
| No trigger index stability warning | Mid-game NPC edits corrupt encounter tracking |

### Documentation Quality Assessment

**WHAT/WHERE coverage:** Good — AGENTS.md, WHERE TO LOOK table  
**WHY coverage:** Partial — triggers.md and game_flow.md explain timing, but error boundary and snapshot composition lack rationale  
**Invariant coverage:** Scattered — mutation order in triggers.md, error handling in game_flow.md, no single reference

---

## 9. Recommendations

The problems documented above are **code architecture issues, not documentation gaps**. Adding tests or comments to broken design just documents the brokenness. The fixes should make incorrect usage **impossible**.

### Refactoring Opportunities

#### 1. Trigger Identity: Index → UUID

**Problem:** Triggers identified by array index. Mid-game edits corrupt `trigger_fired` HashMap.

**Current:**
```rust
pub struct NpcEncounterState {
    pub trigger_fired: HashMap<usize, bool>,  // INDEX, not stable
}
```

**Fix:** Use UUID assigned at parse time:
```rust
pub struct Trigger {
    pub id: Uuid,  // Stable identity regardless of array position
    pub requirement: TriggerRequirement,
    pub narration: TriggerNarration,
    pub repeat: bool,
    pub room_id: Option<String>,
}

pub struct NpcEncounterState {
    pub trigger_fired: HashMap<Uuid, bool>,  // Stable across reorders
}
```

**Why:** Edits to NPC cards mid-game won't corrupt tracking. Index shifts don't matter.

#### 2. Mutation Order: Builder Pattern

**Problem:** Steps must execute in specific order, but violations compile and break silently.

**Current:**
```rust
// All public functions - nothing enforces order
handle_movement(...);
add_message(...);
evaluate_triggers(...);
apply_npc_events(...);
```

**Fix:** Chain that makes wrong order impossible:
```rust
let result = ActionPipeline::new(state)
    .handle_movement(dest, npc_ids)?
    .resolve_npcs()?
    .add_narration(narration_text)?
    .evaluate_triggers()?
    .apply_npc_events()?
    .finish();
```

**Why:** Can't call `evaluate_triggers()` without going through `add_narration()` first. Type system enforces order.

#### 3. load_state: Single Clear Function

**Problem:** `try_load_state` vs `load_state` have opposite failure semantics. Easy to use wrong one.

**Current:**
```rust
pub fn try_load_state(...) -> Result<GameState, EngineError>  // explicit
pub fn load_state(...) -> GameState  // graceful, never fails
```

**Fix:** One function with explicit contract:
```rust
/// Loads state from storage, falling back to fresh state if snapshot corrupted.
/// Use this for production. For tests that must catch corruption, use load_state_strict.
pub fn load_or_fresh(ctx: &GameServiceContext) -> GameState {
    match try_load_state(ctx) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("State load failed ({e}), returning fresh state");
            GameState::new(...)
        }
    }
}

/// Loads state or fails. Use when you need to detect corruption.
pub fn load_expecting_valid_state(...) -> Result<GameState, EngineError>
```

**Why:** Names convey behavior. Tests use strict variant, production uses graceful.

#### 4. Snapshot Composition: Type-Level Distinction

**Problem:** Snapshots are partial but type doesn't reflect this. AI assumes full restore.

**Current:**
```rust
pub struct GameStateSnapshot { ... }
// Nothing distinguishes this from "full state"
```

**Fix:** Explicit types:
```rust
/// Partial snapshot - does NOT include world/map/player/npcs or messages.
/// Must be combined with message loading for full state.
pub struct PartialGameSnapshot { ... }

/// Full game state for in-memory operations.
pub struct GameState { ... }

/// Creates full state from partial snapshot plus message loading.
pub fn restore_full_state(
    partial: PartialGameSnapshot,
    messages: Vec<Message>,
    world: Arc<WorldCard>,
    map: Arc<MapDef>,
) -> GameState { ... }
```

**Why:** Type signature makes it impossible to think `apply_to()` restores everything.

#### 5. Error Boundary: Explicit Wrapping

**Problem:** `From<EngineError>` makes boundary feel automatic. AI might try reverse.

**Current:**
```rust
impl From<EngineError> for ApplicationError {
    fn from(e: EngineError) -> Self { Self::Engine(e) }
}
```

**Fix:** Explicit method, no `From` impl:
```rust
impl ApplicationError {
    /// Wraps an engine error. ONE-WAY: Engine errors cannot escape to user-facing handlers.
    /// Use this when crossing from domain to HTTP layer. Do NOT use for the reverse direction.
    pub fn from_engine(e: EngineError) -> Self { Self::Engine(e) }
}
```

**Why:** `from_engine(e)` reads as intentional wrapping, not automatic conversion.

#### 6. Message/Swipe: Enforced Consistency

**Problem:** Writing `message.text` directly bypasses `set_active_swipe()`, creating inconsistency.

**Current:**
```rust
pub struct Message {
    pub text: String,  // Writable directly - bypasses swipe tracking
    pub swipes: Vec<Swipe>,
}
```

**Fix:** Private fields, controlled mutation:
```rust
pub struct Message {
    text: String,  // private - must use set_active_swipe()
    swipes: Vec<Swipe>,
    active_swipe_index: usize,
}

impl Message {
    pub fn set_active_swipe(&mut self, index: usize) {
        // Updates text, location_header, event_header, snapshot_id
        // All in one atomic operation
    }
    
    pub fn text(&self) -> &str { &self.text }  // read-only accessor
}
```

**Why:** Compiler prevents direct mutation. Consistency guaranteed.

#### 7. Lock Pattern: Scoped Guard

**Problem:** Lock release/reacquire pattern is easy to forget.

**Current:**
```rust
let prompt = evaluate_triggers(state);
drop(state_guard);                    // easy to miss
let result = llm_call(prompt).await;
let mut state_guard = ctx.state.lock().await;  // easy to miss
commit_trigger_narration(&mut state_guard, result);
```

**Fix:** RAII guard that handles lock release:
```rust
let result = {
    let _llm_scope = LlmScope::new(&state_guard);
    llm_call(prompt).await
};  // lock re-acquired when _llm_scope drops

commit_trigger_narration(&mut state_guard, result);
```

**Why:** Lock management happens in one place. Drop handles reacquire.

---

## Summary

These aren't documentation problems — they're design problems. The refactors make incorrect usage **impossible by construction**, not just **unlikely without tests**.

| Problem | Band-aid Fix | Real Fix |
|---------|---------------|----------|
| Trigger index | "add a test" | Use UUID |
| Mutation order | "add a test" | Builder pattern |
| load_state | "add a test" | Single function + names |
| Snapshot | "add a test" | Type distinction |
| Error boundary | "add a comment" | Explicit method |
| Message/Swipe | N/A | Private fields |
| Lock release | N/A | Scoped guard |

The goal: **make the code teach the correct usage by making incorrect usage fail to compile.**

---

*Review generated from investigation plan: `local://agent-comprehension-review.md`*