# Agent Scalability Assessment: Chronicler vs. Marinara

**Date:** 2026-05-09  
**Context:** User is considering adding Marinara-style multi-agent architecture to Chronicler Engine  
**Method:** Map Marinara agent patterns to Chronicler's current architecture, identify gaps

---

## Executive Summary

**Current Chronicler has ~3 "agents":**
1. **Narrator** (`LlmBackend`) — main prose generation
2. **Quantifier** (`QuantifierBackendTrait`) — NPC presence + movement detection
3. **Text Check** (`text_check`) — grammar/spelling on player input

**Marinara has ~25 agents** across 5 categories: Trackers, Narrative, Visual, Integrity, Utility.

**Verdict:**
- **3–5 more agents:** Feasible with moderate refactor. Each agent follows the Quantifier pattern.
- **10+ agents:** Requires **major restructure**. The current hardcoded pipeline in `game_service.rs` becomes unmaintainable. Need: agent registry, execution phases, sidecar support, result bus.

---

## 1. How Marinara Agents Work

### 1.1 Execution Phases

```
Player sends message
        │
        ▼
┌─────────────────┐
│  PRE-GENERATION │  Steering agents inject into prompt
│  ─────────────  │  Prose Guardian, Knowledge Router, Director
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  MAIN LLM       │  Expensive API model generates narration
│  ─────────────  │  OpenRouter, Claude, etc.
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  POST-GENERATION│  Tracker agents analyze output, update state
│  ─────────────  │  Character Tracker, World State, Quest Tracker
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  UI UPDATE      │  Visual agents trigger frontend changes
│  ─────────────  │  Expression Engine, Background, Spotify DJ
└─────────────────┘
```

### 1.2 Sidecar Model

- **Main LLM:** Expensive API model (Claude, GPT-4) for narration
- **Sidecar:** Cheap local model (Gemma-2b, Qwen-1.5b) for agents
- **Cost:** 90% of agents run on sidecar = free
- **Parallel:** Agents run while main LLM is streaming

### 1.3 Agent Outputs

Agents don't just return text. They return:
- **Prompt injections** (steering directives)
- **State patches** (JSON updates to world state)
- **UI commands** (sprite changes, background switches, music)
- **Metadata updates** (lorebook entries, quest progress)
- **Tool calls** (Spotify API, hardware control)

---

## 2. Chronicler's Current "Agent" Pattern

### 2.1 The Quantifier (Your Closest Equivalent)

```
Player sends message
        │
        ▼
┌─────────────────┐
│  parse_command  │  "look around" → Action::FreeAction
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  narrate_action │  Main LLM generates prose
│  (LlmBackend)   │  "You see Carla by the fire..."
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  QUANTIFIER     │  Analyzes narration for NPCs + movement
│  ─────────────  │  Returns: who is present, did player move
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  apply_state    │  Update scene.npcs_in_area, movement, triggers
│  ─────────────  │  execute_freeaction_impl
└─────────────────┘
```

**The Quantifier is actually 2–3 Marinara agents combined:**
- **Character Tracker** (who is present)
- **World State** (did player move / change location)
- Potentially **Combat Tracker** (if damage/HP existed)

### 2.2 Where It Lives

| Component | Marinara Location | Chronicler Location |
|-----------|-------------------|---------------------|
| Agent executor | `services/agents/agent-executor.ts` | `narrative/quantifier/core.rs` |
| Agent pipeline | `services/agents/agent-pipeline.ts` | `engine/game_service.rs` (hardcoded) |
| Agent config | `data/storage/tables/agent_configs.json` | `Cargo.toml` features + code |
| Sidecar | `services/sidecar/sidecar-inference.service.ts` | **Does not exist** |
| Result bus | Agent results patch metadata | `QuantifierResult` struct |

---

## 3. Mapping Marinara Agents to Chronicler

### 3.1 Easy Adds (Low Refactor)

These agents fit the existing Quantifier pattern:

#### Expression Engine (Post-Gen Visual)
- **What:** Detect emotions in narration → switch NPC sprites
- **Where:** New module `narrative/agents/expression.rs`
- **When:** After `commit_trigger_narration`
- **Input:** `&str` (narration text), `&[NpcCard]` (present NPCs)
- **Output:** `HashMap<String, String>` (npc_id → sprite_name)
- **Consumer:** `server/fragments.rs` → `VisualSidebarTemplate`
- **Difficulty:** Low. Server already renders sprites. Just need emotion detection.

#### Background Switcher (Post-Gen Visual)
- **What:** Detect location changes → switch room background image
- **Where:** `narrative/agents/background.rs` or inline in `handle_movement`
- **When:** After movement is applied
- **Input:** `&GameState` (current room)
- **Output:** `Option<String>` (new background image path)
- **Consumer:** `server/fragments.rs` → `VisualSidebarTemplate`
- **Difficulty:** Low. Room already has `image_path`.

#### Custom Tracker (Post-Gen State)
- **What:** Extract user-defined fields (currency, reputation) from narration
- **Where:** `engine/agents/custom_tracker.rs`
- **When:** After `apply_npc_events`
- **Input:** `&str` (narration), `&[LogEntry]` (history)
- **Output:** JSON patch to custom state
- **Consumer:** New field in `GameState`
- **Difficulty:** Low-Medium. Need new state sub-struct.

### 3.2 Medium Adds (Moderate Refactor)

These need new infrastructure:

#### Prose Guardian (Pre-Gen Steering)
- **What:** Analyze last 5 messages → ban overused words → suggest rhetorical devices
- **Where:** `narrative/agents/prose_guardian.rs`
- **When:** **Before** `backend.narrate_action()`
- **Input:** `&[LogEntry]` (recent history)
- **Output:** `String` (steering instructions)
- **Consumer:** `PromptBuilder` — inject into Layer 0 (System) or Layer 7 (PHI)
- **Difficulty:** Medium. Need to thread agent output into prompt builder. Current `PromptBuilder` has no "agent directives" layer.
- **Blocker:** `PromptBuilder` is hardcoded to 8 layers. Adding agent injection requires either:
  - New layer (Layer -1: Agent Directives)
  - Modify Layer 0 to append agent output
  - Or pass agent output into `PromptContext` and let `PromptBuilder` consume it

#### Continuity Checker (Post-Gen Validation)
- **What:** Compare narration against lore/history → flag contradictions
- **Where:** `narrative/agents/continuity.rs` or `engine/agents/continuity.rs`
- **When:** After narration generated, **before** `add_log()`
- **Input:** `&str` (narration), `&[LogEntry]` (history), `&WorldCard` (rules)
- **Output:** `Result<(), Vec<Contradiction>>`
- **Consumer:** Could reject narration (retry) or add system warning
- **Difficulty:** Medium. Need policy: reject-and-retry, or warn-and-continue?
- **Blocker:** Current pipeline has no "validation gate" between generation and logging. Adding one changes the error flow.

#### Lorebook Keeper (Post-Gen State)
- **What:** Extract durable new facts from narration → add to lorebook
- **Where:** `narrative/agents/lorebook.rs`
- **When:** After narration committed
- **Input:** `&str` (narration), `&[LogEntry]` (history)
- **Output:** `Vec<NewFact>`
- **Consumer:** New `lorebook: Vec<LoreEntry>` field in `GameState` or `WorldCard`
- **Difficulty:** Medium. Need lorebook data model + deduplication logic.

### 3.3 Hard Adds (Major Refactor)

These need infrastructure that doesn't exist:

#### Secret Plot Driver (Pre-Gen Steering)
- **What:** Manages long-term arcs, detects staleness, injects plot twists
- **Difficulty:** High. Needs:
  - Persistent plot state (overarching arc, scene direction)
  - Staleness detection (conversation loop detection)
  - Event injection (trigger system on steroids)
  - This is essentially a **second trigger system** that operates on narrative quality, not game state

#### Editor (Post-Gen Rewriting)
- **What:** Receives draft narration + all agent data → edits prose to fix errors
- **Difficulty:** High. Needs:
  - Two-phase generation: draft → edit → final
  - Agent result aggregation (all post-gen agents must run before editor)
  - Current pipeline: narrate → quantify → apply state. Adding "edit" means: narrate → edit → quantify → apply state (or narrate → quantify → edit → apply state)
  - Changes the fundamental pipeline shape

#### Knowledge Router + Retrieval (Pre-Gen Context)
- **What:** Select relevant lorebook entries → condense → inject into prompt
- **Difficulty:** High. Needs:
  - Vector DB or keyword index for lorebook
  - Relevance scoring
  - Token budget management (lore entries consume context window)
  - Current prompt builder has no dynamic lore injection layer

#### HTML Injector (Post-Gen UI)
- **What:** Detects opportunities for in-world documents → injects custom HTML/CSS
- **Difficulty:** Medium-High. Needs:
  - New fragment type in server
  - CSS/HTML sanitization
  - HTMX fragment injection into story log
  - Currently all log entries are text. HTML entries would need new `LogType` variant and rendering logic.

#### Echo Chamber / Spotify DJ / Haptic Control (External Integrations)
- **What:** Control external services based on narrative
- **Difficulty:** High. Needs:
  - External API clients (Spotify, Buttplug.io)
  - Async integration (don't block game loop)
  - New `server/` infrastructure for WebSocket or SSE push
  - Configuration UI for external services

---

## 4. The Missing Infrastructure

To support 10+ Marinara-style agents, Chronicler needs 4 things it doesn't have:

### 4.1 Agent Registry

**Current:** Agents are hardcoded in `game_service.rs`.

**Needed:**
```rust
// narrative/agents/mod.rs
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn phase(&self) -> ExecutionPhase;
    fn execute(&self, ctx: &AgentContext) -> Result<AgentResult, EngineError>;
}

pub enum ExecutionPhase {
    PreGeneration,   // Before main LLM call
    Parallel,        // During main LLM call
    PostGeneration,  // After main LLM call
}

pub struct AgentRegistry {
    agents: Vec<Box<dyn Agent>>,
}
```

**Impact:** ~100 lines of trait + registry code. Moderate.

### 4.2 Execution Phase Framework

**Current:** Linear pipeline in `game_service.rs`:
```rust
// Hardcoded steps
let narration = backend.narrate_action(&context)?;
let quantifier_result = determine_npcs_in_room(...)?;
execute_freeaction_impl(state, ...)?;
```

**Needed:**
```rust
// Phased execution
for agent in registry.pre_agents() {
    ctx = agent.execute(&ctx)?;
}

let narration = backend.narrate_action(&ctx.prompt_context)?;

for agent in registry.post_agents() {
    agent.execute(&ctx.with_narration(&narration))?;
}
```

**Impact:** Restructure `game_service.rs::execute_action`. Large.

### 4.3 Sidecar / Local Model Support

**Current:** All LLM calls go through the same `LlmBackend` trait (expensive API model).

**Needed:** Second backend for cheap local agents:
```rust
pub trait AgentBackend: Send + Sync {
    fn run_agent_prompt(&self, system: &str, user: &str) -> Result<String, EngineError>;
}

// Ollama backend for sidecar
pub struct SidecarBackend { /* local model */ }
```

**Impact:** New backend impl + configuration. Moderate.

**Cost reality:** Without a sidecar, every agent costs API tokens. 10 agents × $0.01 = $0.10 per turn. At 100 turns/day = $10/day. Marinara avoids this by running agents locally.

### 4.4 Result Bus

**Current:** Each agent returns a specific struct (`QuantifierResult`, `CheckResult`). Consumer is hardcoded.

**Needed:** Unified result type:
```rust
pub enum AgentResult {
    PromptDirective(String),        // Inject into prompt
    StatePatch(StatePatch),         // Mutate GameState
    UiCommand(UiCommand),           // Send to frontend
    ToolCall(ToolCall),             // Call external API
    NoOp,
}
```

**Impact:** New types + dispatcher. Moderate.

---

## 5. Specific Recommendations

### If You Want 3–5 More Agents (Short Term)

**Do this:**
1. **Create `narrative/agents/` module** with a simple trait:
   ```rust
   pub trait PostGenAgent {
       fn analyze(&self, narration: &str, state: &GameState) -> Result<AgentOutput, EngineError>;
   }
   ```
2. **Move Quantifier into `agents/`** as the first implementation.
3. **Add Expression Engine and Background** as lightweight post-gen agents.
4. **Add Prose Guardian** as pre-gen — pass its output into `PromptContext`.
5. **Keep hardcoded pipeline** but make it agent-aware:
   ```rust
   // game_service.rs
   let pre_outputs: Vec<String> = pre_agents.iter().map(|a| a.steer(&ctx)).collect();
   let context = make_prompt_context(..., &pre_outputs);
   let narration = backend.narrate_action(&context)?;
   let post_outputs: Vec<AgentOutput> = post_agents.iter().map(|a| a.analyze(&narration, &state)).collect();
   ```

**Cost:** ~2 weeks. Pipeline stays hardcoded but agent-shaped.

### If You Want 10+ Agents (Medium Term)

**Do this:**
1. **Implement Agent Registry** (trait + config + enable/disable)
2. **Implement Execution Phases** (pre/parallel/post framework)
3. **Add Sidecar Backend** (Ollama or local API for cheap agents)
4. **Add Result Bus** (unified agent output)
5. **Refactor `game_service.rs`** from hardcoded pipeline to phase-based executor
6. **Add agent configuration UI** (enable/disable per world, adjust prompts)

**Cost:** 1–2 months. Major refactor of narrative + engine layers.

### If You Want Full Marinara Parity (Long Term)

**Consider:**
1. **Event-driven architecture** instead of hardcoded pipeline
   - Each action emits events
   - Agents subscribe to events
   - Results are published to a bus
   - This decouples agents from each other

2. **Separate agent runtime** (sidecar process)
   - Agents run in a separate Tokio task or process
   - Communicate via channels or message queue
   - Main engine doesn't block on agents

3. **Plugin system**
   - Agents loaded from `agents/` directory
   - WASM or Lua scripts for user-defined agents
   - This matches Marinara's "Custom Tracker" concept

**Cost:** 3+ months. Effectively a v2.0 architecture.

---

## 6. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `game_service.rs` becomes unmaintainable | High (with 5+ agents) | High | Refactor to phase-based executor before adding agents |
| Agent costs become prohibitive | High (without sidecar) | High | Implement Ollama sidecar for cheap agents |
| Prompt context window overflow | Medium | High | Add token budget management for agent outputs |
| Agent conflicts (two agents contradict) | Medium | Medium | Add priority/ordering to agent registry |
| Agent latency spikes | Medium | Medium | Run agents in parallel via `spawn_blocking` |
| Agent hallucinations (wrong state updates) | High | High | Add validation layer (continuity checker) |

---

## 7. The One Thing to Do First

If you add **nothing else**, add this:

**A `PromptContext` extension point for pre-gen agents.**

Current `PromptContext`:
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

Add:
```rust
pub struct PromptContext<'a> {
    // ... existing fields ...
    pub agent_directives: Vec<String>,  // NEW
}
```

Then in `PromptBuilder`, append `agent_directives` to Layer 0 (System Prompt) or Layer 7 (PHI).

**Why this first:** Every pre-gen agent (Prose Guardian, Knowledge Router, Director, Secret Plot Driver) needs to inject text into the prompt. Without this extension point, each agent requires bespoke prompt builder changes. With it, agents become "just another input" to the prompt builder.

**Cost:** 5 lines in `PromptContext`, 3 lines in `PromptBuilder`, 1 line in `make_prompt_context`.

---

## 8. Appendix: Marinara Agent → Chronicler Mapping Table

| Marinara Agent | Chronicler Equivalent | Phase | Difficulty | Priority |
|----------------|----------------------|-------|------------|----------|
| World State | `Quantifier` (movement half) | Post | Done | — |
| Character Tracker | `Quantifier` (NPC half) | Post | Done | — |
| Prose Guardian | None | Pre | Medium | High |
| Narrative Director | None | Pre | Medium | Medium |
| Director | None | Pre | Medium | Medium |
| Expression Engine | Static sprites | Post | Low | High |
| Background | Static room image | Post | Low | Medium |
| Illustrator | None | Parallel | High | Low |
| Continuity Checker | `state_diagnostics.rs` (partial) | Post | Medium | High |
| Editor | None | Post | High | Low |
| Prompt Reviewer | None | Pre | Medium | Low |
| Lorebook Keeper | None | Post | Medium | Medium |
| Card Auditor | None | Post | High | Low |
| Quest Tracker | `Trigger` system (partial) | Post | Medium | Medium |
| Combat Tracker | None | Post | Medium | Medium |
| Persona Stats | None | Post | Medium | Low |
| Custom Tracker | None | Post | Low | Medium |
| Echo Chamber | None | Post | High | Low |
| Chat Summary | `history` truncation | Post | Medium | Low |
| Knowledge Router | None | Pre | High | Medium |
| Knowledge Retrieval | None | Pre | High | Medium |
| Schedule Planner | None | Pre | High | Low |
| Spotify DJ | None | Post | High | Low |
| HTML Injector | None | Post | High | Low |
| Haptic Control | None | Post | High | Low |
