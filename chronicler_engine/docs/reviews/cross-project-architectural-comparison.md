# Cross-Project Architectural Comparison: Chronicler Engine vs. Marinara Engine

**Date:** 2026-05-09
**Scope:** Comparative analysis of Chronicler Engine's architecture against Marinara Engine's solutions to the same domain problems
**Method:** Code archaeology of Marinara's agent system, prompt pipeline, state management, and game mode; mapped to Chronicler's holistic review findings

---

## Executive Summary

Both engines solve the same core problem — LLM-driven interactive fiction — but diverge on a single architectural axis: **Chronicler optimises for simplicity and correctness; Marinara optimises for extensibility and agent density.**

Marinara's solutions to Chronicler's flagged problems are not magic — they trade runtime overhead (DB queries, JSON serialisation, dynamic dispatch) for compile-time flexibility. Chronicler's current architecture is not "wrong"; it is a different point on the simplicity-extensibility spectrum. The question is whether Chronicler's target use case will ever need to move toward Marinara's end of that spectrum.

**Key insight:** Marinara's architecture is essentially what Chronicler's "10+ agents + registry + result bus" recommendation would look like if implemented in TypeScript with a SQLite backend. Many of the patterns are directly portable to Rust.

---

## 1. Agent Orchestration: Hardcoded Pipeline vs. Phase-Based Factory

### Chronicler's Problem

`engine/game_service.rs` implements a linear, hardcoded pipeline:

```
parse_command → narrate_action → determine_npcs_in_room → execute_freeaction_impl → [optional trigger continuation]
```

Each step is a direct function call. The `DefaultGameService` struct owns `Arc<dyn LlmBackend>` and `Arc<dyn QuantifierBackendTrait>` as fields. Adding a new agent requires modifying `execute_action`, cloning more fields before the LLM call, and adding mutation logic after.

**Finding from Phase 2:** "The trait abstraction is real (you can swap backends), but the *orchestration* is deeply coupled."

**Finding from Agent Assessment:** "3–5 more agents: moderate refactor. 10+ agents: major restructure (registry, execution phases, result bus)."

### Marinara's Solution

`services/agents/agent-pipeline.ts` provides a **phase-based factory**:

```typescript
export function createAgentPipeline(agents: ResolvedAgent[], baseContext: AgentContext, onResult?) {
  return {
    async preGenerate(filter?): Promise<AgentInjection[]> { ... },
    async runParallel(): Promise<AgentResult[]> { ... },
    async postGenerate(mainResponse: string): Promise<AgentResult[]> { ... },
  };
}
```

The main generation route (`routes/generate.routes.ts`) orchestrates around the main LLM call:

```
preGenerate() → [main LLM call starts] → runParallel() concurrently → [main LLM ends] → postGenerate(response)
```

Agents declare their phase in config, not code:

```typescript
export type AgentPhase = "pre_generation" | "parallel" | "post_processing";
```

**Batched execution:** Within a phase, agents sharing the same `provider+model` are combined into a single LLM call using XML-delimited tasks. A batch of 5 agents becomes one API request with `<result agent="world-state">...</result>` blocks. If parsing fails, individual retry falls back to separate calls.

### Comparison

| Aspect | Chronicler | Marinara |
|--------|-----------|----------|
| Pipeline shape | Hardcoded function chain | Phase-based factory |
| Agent insertion point | Modify `game_service.rs` | Add config entry |
| Execution order | Fixed in code | Configurable per agent |
| Parallelism | None (sequential) | Intra-phase parallel + inter-phase concurrent |
| API call efficiency | 1 call per agent step | 1 call per (phase, provider, model) group |

### Portable Pattern

The phase-based factory is directly portable to Rust:

```rust
pub trait Agent: Send + Sync {
    fn phase(&self) -> ExecutionPhase;
    fn execute(&self, ctx: &AgentContext) -> Result<AgentResult, EngineError>;
}

pub struct AgentPipeline { /* ... */ }

impl AgentPipeline {
    pub async fn pre_generate(&self) -> Vec<AgentInjection> { ... }
    pub async fn run_parallel(&self) -> Vec<AgentResult> { ... }
    pub async fn post_generate(&self, main_response: &str) -> Vec<AgentResult> { ... }
}
```

The batching optimisation is harder in Rust due to async trait object constraints, but the phase structure alone solves the maintainability ceiling.

---

## 2. State Management: Mutable Central State vs. Immutable Snapshots

### Chronicler's Problem

Single `Arc<std::sync::Mutex<GameState>>` held by `AppState`. All mutations go through this lock.

**Phase 2 finding:** "`DefaultGameService` must clone 8+ fields out of `GameState` before calling `backend.narrate_action()`, then re-lock to apply results."

**Phase 3 finding:** "Adding a field to `GameState` requires touching `GameState` struct definition, `GameState::new`, all `TestGameState` constructors, and any code that constructs `GameState` directly."

**Phase 2 finding:** "`GeneratingGuard` lives in `model/state.rs` but is conceptually engine orchestration."

### Marinara's Solution

No in-memory shared state for generation. Generation is **stateless** relative to the HTTP request:

1. Load chat messages from SQLite
2. Load latest committed game state snapshot from SQLite
3. Build `AgentContext` from DB data
4. Run agents (each reads DB, none hold locks)
5. Save new game state snapshot to DB
6. Save assistant message to DB

```typescript
// packages/shared/src/types/game-state.ts
export interface GameState {
  id: string;
  chatId: string;
  messageId: string;
  swipeIndex: number;
  date: string | null;
  time: string | null;
  location: string | null;
  weather: string | null;
  presentCharacters: PresentCharacter[];
  recentEvents: string[];
  playerStats: PlayerStats | null;
  personaStats: CharacterStat[] | null;
  manualOverrides?: Record<string, string> | null;
  committed?: boolean;
}
```

**Committed/uncommitted pattern:** When a user sends a new message, the last assistant message's game state is `commit()`ted. Swipes/regenerations create new snapshots without touching committed state.

**Per-character extensibility:** `PresentCharacter` carries `customFields: Record<string, string>` and `stats: CharacterStat[]`. The Custom Tracker agent writes arbitrary key-value pairs without code changes.

### Comparison

| Aspect | Chronicler | Marinara |
|--------|-----------|----------|
| State lifetime | Session duration (in-memory) | Per-message (DB-backed) |
| Mutability | `&mut GameState` under mutex | Immutable snapshots, new row per turn |
| Concurrency control | `std::sync::Mutex` + poison recovery | DB transactions + generation guard |
| Regeneration/swipes | Complex (reset state, guard poison) | Trivial (new snapshot row, old one preserved) |
| Extensibility | Edit struct, edit constructors, edit tests | Edit interface, edit storage layer |
| Testability | Mock `Arc<Mutex<GameState>>` | Mock storage service, pure functions |

### Portable Pattern

The snapshot pattern is portable but would require Chronicler to adopt a persistence layer (even SQLite). The simpler portable pattern is **value-based state threading**:

```rust
// Instead of &mut GameState, return a new GameState
pub fn execute_turn(state: GameState, action: Action) -> Result<TurnResult, EngineError> {
    let new_state = state.clone(); // or structural sharing
    // ... mutations on new_state ...
    Ok(TurnResult { state: new_state, narration, agent_results: ... })
}
```

This eliminates the mutex and makes swipes/regenerations trivial at the cost of clone overhead (mitigated by `Arc` for large fields).

---

## 3. Prompt Construction: Hardcoded Layers vs. Composable Presets

### Chronicler's Problem

`PromptBuilder` has 8 hardcoded layers (0–7). Adding a pre-gen agent requires modifying the builder. The assessment's #1 recommendation was adding `agent_directives: Vec<String>` to `PromptContext`.

**Phase 1 finding:** "`Prompt` means full prompt, single layer, quantifier prompt, or trigger prompt." "`Layer` means prompt layer 0-7 OR architecture tier."

**Phase 3 finding:** "The prompt builder has no concept of 'narrate this mechanical result.' Would need Layer 8+ or a special injection point."

### Marinara's Solution

`services/prompt/assembler.ts` builds prompts from **user-editable presets** consisting of:

- **Sections** — ordered, each with `role`, `content`, `enabled`, `isMarker`
- **Groups** — wrapping sections in XML tags
- **Markers** — dynamic injection points (`chat_history`, `agent_data`, `chat_summary`, etc.)
- **Depth injection** — lorebook entries inserted N messages deep in history
- **Macro resolution** — `{{user}}`, `{{char}}`, `{{agent::world-state}}` expanded at assembly time

Agent injection is just another section:

```typescript
if (markerConfig.type === "agent_data" && section.content) {
  ctx.macroCtx.agentData = {
    ...ctx.macroCtx.agentData,
    [agentType]: expanded.content,
  };
}
```

Pre-gen agents produce `AgentInjection { agentType, text }`. The assembler wraps them as XML/markdown and injects via macros. No builder modification needed — agents are data.

### Comparison

| Aspect | Chronicler | Marinara |
|--------|-----------|----------|
| Prompt structure | Code (8 hardcoded layers) | Data (user-editable preset JSON) |
| Agent injection | Modify `PromptBuilder` | Add preset section with `agent_data` marker |
| Layer ambiguity | "Layer" = prompt layer OR architecture tier | "Section" = prompt part; "Layer" = not used |
| Reordering | Impossible without code change | Drag-and-drop in UI |
| Testability | Test `PromptBuilder` logic | Test preset assembly with mock sections |

### Portable Pattern

Chronicler could adopt a **preset-based assembler** without a UI:

```rust
pub struct PromptPreset {
    pub sections: Vec<PromptSection>,
    pub groups: Vec<PromptGroup>,
    pub wrap_format: WrapFormat,
}

pub struct PromptSection {
    pub id: String,
    pub role: PromptRole,
    pub content: String,        // may contain macros
    pub enabled: bool,
    pub marker: Option<MarkerConfig>,
    pub injection_depth: Option<usize>,
}
```

The `agent_directives: Vec<String>` recommendation becomes `sections: Vec<PromptSection>` where one section has `marker: Some(MarkerConfig::AgentData("prose-guardian"))`.

---

## 4. Error Handling: Structure Loss vs. Structured Agent Results

### Chronicler's Problem

`map_llm_error` flattens `EngineError` to `String` for display in the UI.

**Phase 2 finding:** "When the LLM returns unparseable JSON, the user sees 'unexpected response format' but the actual model output (the only debug evidence) is **gone**."

**Agent Assessment finding:** "`GenerationStatus::Error(String)` loses all structured error data by server layer. `map_llm_error` is the bottleneck."

### Marinara's Solution

Every agent returns the same structured shape:

```typescript
export interface AgentResult {
  agentId: string;
  agentType: string;
  type: AgentResultType;  // "game_state_update" | "text_rewrite" | "sprite_change" | ...
  data: unknown;
  tokensUsed: number;
  durationMs: number;
  success: boolean;
  error: string | null;
}
```

On failure:
- `success: false`
- `error: "JSON parse failed: ..."`
- `data: null`

Raw response preserved in debug logs:

```typescript
logger.debug(`[agent] ${config.type} raw response: ${responseText.slice(0, 500)}`);
```

**Batch failure recovery:** If batch parsing fails for one agent, retry individually with same context. The batch raw response is still available for inspection.

**Tool calling eliminates parse failures:** Agents that mutate state use function calling (`update_game_state`, `set_expression`). The LLM emits JSON tool calls, not free text. No regex/parse step, no `raw_response` loss.

### Comparison

| Aspect | Chronicler | Marinara |
|--------|-----------|----------|
| Error type | `EngineError` enum, flattened to `String` | `AgentResult` with `success` + `error` fields |
| Raw response preservation | Lost in `map_llm_error` | Logged at debug level |
| Parse failures | Undebuggable from UI | Batch retry + individual fallback |
| Structured output | Manual regex/JSON parse | Function calling (tool use) |

### Portable Pattern

Replace `GenerationStatus::Error(String)` with structured status:

```rust
pub enum GenerationStatus {
    Idle,
    Generating,
    Error(GenerationError),
}

pub struct GenerationError {
    pub message: String,
    pub raw_response: Option<String>,
    pub agent_results: Vec<AgentResult>, // which agents ran, what they produced
}
```

And adopt tool calling for agents that need structured output:

```rust
pub trait ToolCallingAgent {
    fn tools(&self) -> Vec<ToolDefinition>;
    fn execute(&self, ctx: &AgentContext, tool_calls: Vec<ToolCall>) -> Result<AgentResult, EngineError>;
}
```

---

## 5. Backend Routing: Global Traits vs. Per-Agent Connections

### Chronicler's Problem

All LLM calls go through the same `LlmBackend` trait. The quantifier can use a different backend, but this is handled as a special case (`QuantifierBackendTrait`).

**Agent Assessment finding:** "Without a sidecar, every agent costs API tokens. 10 agents × $0.01 = $0.10 per turn."

### Marinara's Solution

Per-agent connection override:

```typescript
export interface AgentConfig {
  // ...
  connectionId: string | null;  // "use this Ollama connection for this agent"
}
```

The agent executor resolves the provider per-agent:

```typescript
const resolvedAgents: ResolvedAgent[] = await Promise.all(
  enabledAgents.map(async (agent) => {
    const conn = agent.connectionId 
      ? await connections.getWithKey(agent.connectionId)
      : defaultConnection;
    const provider = createLLMProvider(conn);
    return { ...agent, provider, model: conn.model };
  })
);
```

**Batching by provider+model:** Agents with the same cheap local connection are batched together. Agents with expensive API connections run separately.

**Local sidecar:** `LOCAL_SIDECAR_CONNECTION_ID` maps to a local Ollama/Gemma instance. Most tracker agents default to this. The main narration uses the expensive API model.

### Comparison

| Aspect | Chronicler | Marinara |
|--------|-----------|----------|
| Backend abstraction | `LlmBackend` trait (global) | `BaseLLMProvider` per agent |
| Quantifier backend | Special `QuantifierBackendTrait` | Just another agent with `connectionId` |
| Cost control | Feature-gated mock backend | Per-agent connection selection + batching |
| Sidecar support | Ollama backend exists but underused | First-class local sidecar with auto-batching |

### Portable Pattern

Chronicler already has separate LLM configs. The gap is wiring them to individual pipeline steps:

```rust
pub struct AgentConfig {
    pub agent_type: String,
    pub backend: BackendSelector,  // UseMain | UseQuantifier | Named(String)
}

pub enum BackendSelector {
    UseMain,           // Use the main narration backend
    UseQuantifier,     // Use the quantifier backend
    Named(String),     // Use a named backend from config
}
```

---

## 6. Layer Boundaries: Tight Coupling vs. Storage-Service Decoupling

### Chronicler's Problem

Engine imports 9 narrative types. `narrative/quantifier/core.rs` imports `engine::logic::get_current_room` — an upward violation.

**Phase 2 finding:** "`engine/` is supposed to be above `narrative/` in the stack. But `engine/` imports 9 narrative types, and `narrative/quantifier/core.rs` imports `engine::logic::get_current_room`."

**Phase 4 finding:** "`engine/` imports 33 narrative references. `narrative/` imports 1 engine reference."

### Marinara's Solution

No "engine" vs "narrative" layer separation. Architecture is:

```
Routes (HTTP handlers)
    ↓
Services (business logic: agents, prompt, lorebook, game)
    ↓
Storage (DB abstraction: chats, game-state, agents, connections)
    ↓
DB (SQLite with Drizzle ORM)
```

Agents live in `services/agents/` but import:
- `BaseLLMProvider` (for LLM calls)
- `AgentContext` (shared types)
- Storage services (for reading/writing state)

No upward dependencies. The quantifier equivalent (world-state agent) receives `gameState` and `mainResponse` in its context. It does not call `get_current_room` — the room is already in the snapshot.

### Comparison

| Aspect | Chronicler | Marinara |
|--------|-----------|----------|
| Layer model | `model → engine → narrative → server` | `routes → services → storage → db` |
| Cross-layer imports | 33 engine→narrative, 1 narrative→engine | Minimal; services compose horizontally |
| State access | Direct struct field mutation | Via storage services with typed interfaces |
| Upward violations | `quantifier/core.rs → engine::logic` | None; context is passed downward |

### Portable Pattern

Chronicler could flatten its layer model for the agent system:

```
server (HTTP handlers — Axum)
    ↓
engine (orchestration: game service, agent pipeline)
    ↓
narrative (LLM interaction: prompt building, backends)
    ↓
model (pure data types)
```

The key change: **agents are not in `narrative/` or `engine/`. They are peers.** The agent trait lives at the `engine` level. Implementations may call `narrative` (for LLM) and `model` (for state), but the pipeline orchestrator only knows the trait.

---

## 7. The Quantifier as Load-Bearing Component

### Chronicler's Problem

The quantifier determines NPC presence and movement. It's embedded in `game_service.rs` and `action_processing.rs`. The pipeline assumes quantifier output.

**Phase 3 finding:** "The quantifier is not just a 'nice to have' LLM optimization. It determines player movement, NPC presence, NPC events."

**Agent Assessment finding:** "The Quantifier is actually 2–3 Marinara agents combined (Character Tracker + World State). But it's embedded in the orchestration logic."

### Marinara's Solution

The world-state agent is not special:

```typescript
{
  id: "world-state",
  name: "World State",
  phase: "post_processing",
  enabledByDefault: false,
  defaultInjectAsSection: true,
  category: "tracker",
}
```

It runs after generation, receives narration text in `mainResponse`, and returns `game_state_update` results. If disabled, the pipeline skips it. The next agent (character-tracker) reads the last committed snapshot — not the world-state agent's output.

**No agent is load-bearing.** The pipeline assumes nothing about which agents are enabled. The game state storage layer handles "no snapshot exists yet" by cloning the latest committed snapshot.

### Comparison

| Aspect | Chronicler | Marinara |
|--------|-----------|----------|
| Quantifier status | Mandatory, load-bearing | Optional agent (world-state) |
| NPC tracking | Hardcoded in pipeline | Agent-produced, optional |
| Movement detection | Required for correctness | Optional; state carried forward if disabled |
| Failure mode | Pipeline fails | Agent fails gracefully, state unchanged |

---

## 8. Tool Calling: Unstructured Output vs. Function Calling

### Chronicler's Problem

Agents (quantifier, future agents) must parse free-text LLM output to extract structured data. This is brittle and error-prone.

### Marinara's Solution

Agents that need structured state mutations use **function calling** (OpenAI/Anthropic tool use):

```typescript
export const BUILT_IN_TOOLS: ToolDefinition[] = [
  {
    name: "update_game_state",
    description: "Update the current game state — character stats, inventory, quest progress, etc.",
    parameters: {
      type: "object",
      properties: {
        type: { enum: ["stat_change", "inventory_add", "quest_update", "location_change"] },
        target: { type: "string" },
        key: { type: "string" },
        value: { type: "string" },
        description: { type: "string" },
      },
      required: ["type", "target", "key", "value"],
    },
  },
  {
    name: "set_expression",
    description: "Set a character's sprite expression for visual novel display.",
    parameters: {
      type: "object",
      properties: {
        characterName: { type: "string" },
        expression: { type: "string" },
      },
      required: ["characterName", "expression"],
    },
  },
  // ... Spotify, lorebook search, dice rolling, etc.
];
```

The agent executor runs a **tool loop** (up to 5 rounds):

```typescript
for (let round = 0; round < MAX_TOOL_ROUNDS; round++) {
  const result = await provider.chatComplete(messages, { tools });
  if (!result.toolCalls?.length) break; // No tools = final response
  
  // Execute each tool call and append results to message history
  for (const tc of result.toolCalls) {
    const toolResult = await toolContext.executeToolCall(tc);
    messages.push({ role: "tool", content: toolResult, tool_call_id: tc.id });
  }
}
```

### Portable Pattern

Tool calling is backend-dependent (OpenAI/Anthropic format). Chronicler could adopt a lighter pattern: **constrained JSON output**.

```rust
pub trait StructuredAgent {
    fn output_schema(&self) -> serde_json::Value; // JSON schema
    fn execute(&self, ctx: &AgentContext) -> Result<serde_json::Value, EngineError>;
}
```

The agent prompt includes: "Respond with valid JSON matching this schema. No markdown fences." This is less reliable than native tool calling but backend-agnostic.

---

## 9. Testing Philosophy: Mock Backends vs. Storage Mocking

### Chronicler's Problem

Heavy mock usage. Property tests delegate to `assert_state_consistency` rather than independently verifying numeric invariants.

**Phase 4 finding:** "Error-path coverage is thin. No dedicated error-path tests."

### Marinara's Solution

Tests focus on:
- **Storage layer mocking** (in-memory SQLite)
- **Pure function testing** (agent executors with mock providers)
- **Integration testing** (full generation pipeline with seeded DB)

The agent executor is pure: given `AgentExecConfig`, `AgentContext`, and `BaseLLMProvider`, it returns `AgentResult`. Mocking the provider is trivial.

### Comparison

| Aspect | Chronicler | Marinara |
|--------|-----------|----------|
| Mock target | `LlmBackend` trait | `BaseLLMProvider` |
| State mocking | Clone `GameState`, patch fields | Seed SQLite, query via storage |
| Error tests | Weak | Agent executor tests cover failure paths |
| Property tests | 7 proptest properties | Not heavily used; integration tests preferred |

---

## 10. Recommendations for Chronicler

### If Chronicler Stays at 3–5 Agents (Current Trajectory)

1. **Implement `agent_directives: Vec<String>` in `PromptContext`** — 5-line change enabling all pre-gen steering agents.
2. **Add per-agent backend selection** — wire existing LLM configs to individual pipeline steps.
3. **Fix `map_llm_error`** — preserve `ParseError.raw_response` before adding any new agents.
4. **Keep hardcoded pipeline** — but extract it into a single `execute_pipeline` function with clear phase comments.

### If Chronicler Wants 10+ Agents (Marinara-Style Density)

1. **Adopt the phase-based factory pattern** — `AgentPipeline` with `pre_generate` / `run_parallel` / `post_generate`.
2. **Make game state snapshot-based** — new snapshot per turn, committed on user follow-up. This is the biggest architectural shift.
3. **Replace `PromptBuilder` with preset-based assembly** — even without a UI, a data-driven prompt structure enables agent injection without code changes.
4. **Add structured agent results** — unified `AgentResult` enum instead of `QuantifierResult`, `CheckResult`, etc.
5. **Consider SQLite for state snapshots** — eliminates mutex complexity, enables swipe/regeneration trivially, and provides audit trail.

### What Chronicler Should NOT Copy from Marinara

1. **TypeScript dynamic typing** — Chronicler's Rust type safety is a strength. Marinara's `data: unknown` on `AgentResult` would be a `serde_json::Value` in Rust, which is ergonomic enough.
2. **Everything-is-config UI** — Marinara exposes preset editing, agent configuration, and lorebook management through a complex UI. Chronicler's HTMX dashboard is simpler by design.
3. **Per-chat persona system** — Marinara's multi-persona, multi-character conversation model is overkill for Chronicler's single-player text adventure focus.
4. **External integrations** — Spotify, haptic devices, Discord webhooks are domain-specific to Marinara's user base.

---

## Appendix A: File Mapping

| Chronicler File | Marinara Equivalent | Notes |
|-----------------|---------------------|-------|
| `engine/game_service.rs` | `routes/generate.routes.ts` | Main orchestrator; Marinara's is 8x larger but modular |
| `engine/action_processing.rs` | `services/agents/agent-pipeline.ts` + `services/game/*` | State mutation split across agents and game services |
| `narrative/prompt/builder.rs` | `services/prompt/assembler.ts` | Data-driven presets vs. hardcoded layers |
| `narrative/quantifier/core.rs` | `services/agents/agent-executor.ts` (world-state agent) | Just another agent; not special-cased |
| `model/state.rs` | `packages/shared/src/types/game-state.ts` + `services/storage/game-state.storage.ts` | DB snapshots vs. in-memory struct |
| `server/fragments.rs` | `routes/generate.routes.ts` (SSE streaming) | Server-sent events vs. HTMX fragments |
| `error.rs` | `packages/shared/src/types/agent.ts` (`AgentResult`) | Structured results vs. flattened errors |

## Appendix B: Architectural Decision Record

### ADR: Snapshot-Based State vs. In-Memory Mutable State

**Context:** Chronicler uses `Arc<Mutex<GameState>>` for session state. Marinara uses per-message DB snapshots. The agent scalability assessment identified `GameState` extensibility as a blocker.

**Option A: Keep in-memory mutable state**
- Pros: Zero DB overhead, simple reasoning, fast tests
- Cons: Extensibility pain, mutex complexity, regeneration difficulty

**Option B: Adopt snapshot-based persistence**I 
- Pros: Trivial regeneration/swipes, audit trail, agent-friendly, no mutex
- Cons: DB dependency, serialisation overhead, requires migration

**Decision for Chronicler:** **Option A for now.** Chronicler's current use case (single-player text adventure, ~3 agents) does not justify the DB complexity. Revisit if:
- Combat system is implemented (needs ephemeral state tracking)
- Multiplayer is seriously considered
- Agent count exceeds 8

**Mitigation:** Add `GameState::snapshot(&self) -> GameStateSnapshot` and `GameState::restore(snapshot) -> Self` methods now. This establishes the snapshot pattern without committing to a DB layer.

---

*All Chronicler findings verified against:*
- `cargo nextest run --features diagnostics` — all tests pass
- `python build.py` — fmt + clippy + guardrails + tests pass

*All Marinara findings verified against:*
- Source code from `D:\John\DevContainer\Marinara-Engine` (TypeScript/SQLite)
- `packages/server/src/services/agents/agent-executor.ts`
- `packages/server/src/services/agents/agent-pipeline.ts`
- `packages/server/src/services/prompt/assembler.ts`
- `packages/shared/src/types/agent.ts`
- `packages/shared/src/types/game-state.ts`
- `packages/server/src/routes/generate.routes.ts`
- `packages/server/src/services/storage/game-state.storage.ts`
