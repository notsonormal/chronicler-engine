# Spec: Agent-Ready Pipeline Restructure for Chronicler Engine

**Date:** 2026-05-09  
**Status:** In Progress — Phase 1 & 2 implemented, Phase 3 partially implemented, Phase 4 pending  
**Scope:** Restructure Chronicler's hardcoded pipeline to support future agents, without implementing more than the existing narrator + quantifier  
**Related Reviews:**
- `docs/reviews/holistic-architectural-review.md`
- `docs/reviews/agent-scalability-assessment.md`
- `docs/reviews/cross-project-architectural-comparison.md`

---

## Objective

Replace Chronicler's hardcoded `parse → narrate → quantify → apply → trigger` pipeline with an extensible agent-shaped architecture. **Only two agents exist after this work:** the narrator (main LLM) and the quantifier (existing scene analyser). The goal is structural readiness — future agents can be added by implementing a trait and registering in config, without rewriting the orchestrator.

**What this is NOT:** Adding Prose Guardian, Continuity Checker, Expression Engine, or any other agents. Those come later, once the foundation is solid.

**Why do this now:**
- The current pipeline embeds the quantifier as load-bearing logic in `application/game_service/` and `action_processing.rs`
- Adding even one more agent (e.g. a pre-generation prompt injector) would require rewriting the pipeline anyway
- The GameState snapshot refactoring is needed for regeneration, reset, and diagnostics regardless of agents
- The old PromptBuilder's 8 hardcoded layers blocked dynamic prompt injection (replaced by preset-based `LayeredPromptAssembler`)

**What success looks like:**
- [x] The quantifier runs as a `dyn Agent` in a post-generation phase
- [ ] Pre-generation phase exists and is empty (ready for future agents) — *structurally exists (`NarratorAgent`) but not yet invoked in action flow*
- [x] Game state is snapshotted per turn, enabling reset and regeneration
- [x] Prompts are assembled from configurable presets, not hardcoded layers — *Phase 4 completed; `PromptBuilder` deleted, `LayeredPromptAssembler` active*
- [x] A new agent can be added later by: implementing `Agent` trait + adding config entry

---

## Assumptions

1. **Single-player only.** No multiplayer, no concurrent players.
2. **SQLite from Phase 1.** State snapshots persist to SQLite; no in-memory-only intermediate step.
3. **No backward compatibility.** `PromptBuilder` is deleted (done), `GameState` changes freely, config formats break.
4. **Only two agents implemented:** narrator (main LLM) and quantifier (existing scene analyser).
5. **Constrained JSON** for any structured LLM output. No function calling.
6. **No LLM batching.** Each agent gets its own API call. Batching is a future optimisation.
7. **A "Reset Game" button is required** — with persistent state, players need restart without server restart.
8. **Rust Edition 2024 is fixed.**
9. **Test-to-code ratio ≥ 1.5.**

---

## Tech Stack

| Layer | Current | Target | Change |
|-------|---------|--------|--------|
| Language | Rust 2024 | Rust 2024 | None |
| State | `Arc<Mutex<GameState>>` | SQLite-backed snapshots | Major — [x] implemented |
| Pipeline | Hardcoded in `application/game_service/` | Phase-based `AgentPipeline` | Major — partially implemented (bridge function used instead of formal pipeline) |
| Prompts | `PromptBuilder` (8 hardcoded layers) | Preset-based assembler (`LayeredPromptAssembler`) | Major — [x] completed |
| DB | None | SQLite (`rusqlite`) | New dependency — [x] implemented |
| Agents | Hardcoded quantifier | `dyn Agent` trait + registry | Moderate — [x] implemented |

**New dependencies:**
- [x] `rusqlite` (with `bundled` feature for zero-system-dependency builds)
- [x] `uuid` (with `v4,serde` features for snapshot IDs)

---

## Commands

```bash
# Full validation (existing)
cd chronicler_engine && python build.py

# DB setup (new)
~~cargo run -- db-migrate~~  # Superseded: migrations run automatically on DbPool::new()

# Run with tracing
RUST_LOG=info cargo run

# Reset game via CLI (for testing)
cargo run -- reset-game --world <world_name>
```

---

## Project Structure (Target)

```
chronicler_engine/
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── error.rs                          # Extended for agent errors
│   ├── model/
│   │   ├── state.rs                      # GameState (simplified)
│   │   ├── state_snapshot.rs             # [x] NEW: Snapshot types (implemented with db_id: Option<u64> instead of uuid String)
│   │   └── ... (existing)
│   ├── application/
│   │   └── game_service/                 # REFACTORED: thin orchestrator
│   │       ├── mod.rs
│   │       ├── service.rs
│   │       ├── context.rs
│   │       ├── actions.rs                # Still contains hardcoded flow; bridge for post-gen agents added
│   │       ├── retry.rs
│   │       └── helpers.rs
│   ├── engine/
│   │   ├── action_processing.rs          # [x] REFACTORED: stateless, returns TurnResult
│   │   ├── agent_pipeline.rs             # NOT CREATED: bridge function in actions.rs used instead
│   │   └── ... (existing)
│   ├── narrative/
│   │   ├── prompt/
│   │   │   ├── assembler.rs              # NOT CREATED
│   │   │   ├── preset.rs                 # NOT CREATED
│   │   │   └── assembler.rs              # Preset-based prompt assembly (Phase 4 completed)
│   │   ├── agents/
│   │   │   ├── mod.rs                    # [x] NEW: Agent trait + registry
│   │   │   └── quantifier/               # [x] MIGRATED: from quantifier/core.rs
│   │   └── ... (existing)
│   ├── storage/
│   │   ├── mod.rs                        # [x] NEW: DB abstraction
│   │   ├── db.rs                         # [x] NEW: rusqlite connection + migrations
│   │   └── snapshot_storage.rs           # [x] NEW: Snapshot CRUD (plus messages + game CRUD)
│   └── server/
│       └── ... (existing + reset endpoint) [x]
├── data/
│   └── presets/
│       └── default.json                  # NOT CREATED
└── tests/
    └── ... (existing + snapshot tests) [x]
```

---

## Code Style

### Agent Trait

[x] Implemented in `src/model/agent.rs` and `src/narrative/agents/trait_def.rs`:

```rust
// [DOC: docs/system/agent_system.md]

pub enum ExecutionPhase {
    PreGeneration,   // Before main LLM call
    PostGeneration,  // After main LLM call
}

pub enum BackendSelector {
    UseMain,           // Use the main LLM backend (fallback for quantifier: default quantifier backend)
    UseNamed(String),  // Look up backend by connection name in settings
}

pub struct AgentContext<'a> {
    pub state: &'a GameState,
    pub main_response: Option<&'a str>,  // None for pre-generation, Some(narration) for post-generation
    pub player_input: &'a str,
    pub current_room: Option<&'a Room>,  // Added during implementation
}

pub enum AgentResult {
    PromptDirective(String),    // Inject text into prompt (pre-generation only)
    StatePatch(StatePatch),     // Mutate snapshot
    NoOp,
}

pub enum StatePatch {
    Scene {
        npc_ids: Vec<String>,
        movement_destination: Option<String>,
        confidence: Confidence,
    },
}

pub enum Confidence {
    High,
    Medium,
    Low,
}

pub trait Agent: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn phase(&self) -> ExecutionPhase;
    fn backend_selector(&self) -> BackendSelector;
    fn execute(&self, ctx: &AgentContext) -> Result<AgentResult, EngineError>;
}
```

### Snapshot

`GameState` itself does **not** derive `Serialize/Deserialize` (only its sub-structs do). The snapshot stores only the mutable sub-structs; world data (`WorldCard`, `MapDef`, `PlayerCard`, `npcs`) is cached separately and re-attached on load.

~~Original spec design:~~

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameStateSnapshot {
    pub id: String,                    // uuid v4
    pub turn_id: String,
    pub swipe_index: u32,
    pub movement: MovementState,
    pub narrative: NarrativeState,
    pub scene: SceneState,
    pub npc_encounter_log: NpcEncounterLog,
    pub committed: bool,
    pub created_at: DateTime<Utc>,
}
```

[x] **Implemented design** (superseded above):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameStateSnapshot {
    pub db_id: Option<u64>,            // SQLite row ID
    pub movement: MovementState,
    pub narrative: NarrativeSnapshot,  // Persistable subset without messages
    pub scene: SceneState,
    #[serde(rename = "character_state")]
    pub npc_encounter_log: NpcEncounterLog,
    pub committed: bool,
    pub created_at: DateTime<Utc>,
}
```

*Changes:* `id` → `db_id: Option<u64>`; removed `turn_id` and `swipe_index`; `narrative` wrapped in `NarrativeSnapshot` (excludes messages, which live in a separate table).

### Turn Result

[x] Implemented in `src/engine/action_processing.rs`:

```rust
pub struct TurnResult {
    pub next_state: GameState,         // fully populated with re-attached world data
    pub narration: String,
    pub trigger_match: Option<TriggerMatch>, // raw trigger data for application to build continuation prompt
}
```

Note: `agent_results` is added in Phase 3 when the pipeline dispatcher is introduced. In Phase 2, `application/game_service/actions.rs` contains a temporary bridge that translates `AgentResult::StatePatch` back into `QuantifierResult` for `action_processing.rs`.

---

## Testing Strategy

| Level | Purpose |
|-------|---------|
| Unit | Snapshot round-tripping, preset assembly, agent trait mock |
| Integration | Full pipeline with mock backends, state assertions |
| Property | Snapshot serialisation, preset determinism |
| Guardrails | File length, arch-lint |

**Key tests:**
- [x] Pipeline with 0 agents runs main LLM only (baseline)
- [x] Quantifier agent produces identical output to old hardcoded code
- [x] Reset clears all snapshots and returns to initial state
- [x] `GameState` → snapshot → `GameState` round-trips without loss

---

## Boundaries

### Always
- Run `python build.py` before commits
- Update docs before implementing subsystems
- Add tests for every new public type
- Preserve existing HTMX dashboard when quantifier is the only agent

### Ask First
- Adding Cargo dependencies beyond `rusqlite`
- Changing `EngineError` variants
- Modifying `data/worlds/*.json` schema
- Changing `GameService` trait signature

### Never
- Commit `.db` files to repo
- Break the `model → engine → narrative → server` DAG
- Use `.unwrap()` in production code
- Make agents mandatory (engine works with zero agents)

---

## Success Criteria

1. [x] **Pipeline is agent-shaped:** The quantifier runs as `dyn Agent` in a post-generation phase.
2. [ ] **Pre-generation phase exists:** Empty but callable — adding a future agent requires no pipeline changes.  
   *Status: `NarratorAgent` exists but is not invoked in the action flow yet.*
3. [x] **State is snapshotted:** Each turn creates a new `GameStateSnapshot` row in SQLite.
4. [x] **Reset works:** `POST /reset` clears SQLite, reloads world, returns to starting room.
5. [x] **Baseline preserved:** With only quantifier enabled, output matches pre-migration.
6. [ ] **Test ratio ≥ 1.5.** — *To be verified at end of Phase 4.*
7. [ ] **Performance:** Turn with quantifier takes ≤ 110% of pre-migration time. — *To be benchmarked at end of Phase 4.*

---

# Implementation: Phased Plan

## Phase 1: SQLite + GameState Snapshots + Reset

**Goal:** Replace `Arc<Mutex<GameState>>` with SQLite-backed snapshots. Add reset endpoint.  
**Status:** [x] Implemented (with schema deviations noted below)

### Task 1.1: SQLite Setup
- [x] Add `rusqlite` with `bundled` feature to `Cargo.toml`
- [x] Create `storage/db.rs` with connection pooling and migrations
- Migration 001: `game_state_snapshots` table
  - ~~`id TEXT PRIMARY KEY`~~ — Superseded: `id INTEGER PRIMARY KEY AUTOINCREMENT`
  - ~~`turn_id TEXT NOT NULL`~~ — Superseded: not in schema
  - ~~`swipe_index INTEGER NOT NULL DEFAULT 0`~~ — Superseded: not in schema
  - [x] `movement TEXT NOT NULL` (JSON)
  - [x] `narrative TEXT NOT NULL` (JSON)
  - [x] `scene TEXT NOT NULL` (JSON)
  - [x] `npc_encounter_log TEXT NOT NULL` (JSON)
  - [x] `committed INTEGER NOT NULL DEFAULT 0`
  - [x] `created_at TEXT NOT NULL` (ISO 8601)
  - [x] `game_id INTEGER NOT NULL` + `games` table added for multi-game support
- **Files:** `src/storage/db.rs`, `src/storage/mod.rs`
- **Acceptance:**
  - [ ] ~~`cargo run -- db-migrate` creates `.db` file~~ — Superseded by auto-migration in `DbPool::new()`
  - [x] `.db` file is in `.gitignore`
  - [x] Migrations are idempotent

### Task 1.2: Snapshot Types
- [x] Add `GameStateSnapshot` in `model/state_snapshot.rs`
- [x] `from_game_state()` and `apply_to()` helpers; `TryFrom` for DB round-trips
- **Files:** `src/model/state_snapshot.rs`
- **Acceptance:**
  - [x] Round-trip property test: 100 random states → snapshot → state
  - [x] No data loss on round-trip

### Task 1.3: Snapshot Storage
- [x] Implement `SnapshotStorage` trait:
  ```rust
  pub trait SnapshotStorage: Send + Sync {
      fn save(&self, snapshot: &GameStateSnapshot) -> Result<u64, EngineError>;       // Superseded: returns row ID
      fn load_latest(&self) -> Result<Option<GameStateSnapshot>, EngineError>;         // Superseded: no turn_id param
      fn load_by_id(&self, id: u64) -> Result<Option<GameStateSnapshot>, EngineError>; // Added for retry
      fn commit(&self, snapshot_id: u64) -> Result<(), EngineError>;                    // Superseded: u64 ID
      fn reset(&self) -> Result<(), EngineError>;
      // Expanded scope: game CRUD and message storage also included
  }
  ```
- [x] SQLite implementation using `rusqlite`
- **Files:** `src/storage/snapshot_storage.rs`
- **Acceptance:**
  - [x] Save + load round-trips
  - [x] `commit` sets flag; `load_latest` prefers uncommitted for active turn
  - [x] `reset` drops all rows

### Task 1.4: Stateless Action Processing
- [x] Refactor `execute_freeaction_impl` to take `&GameState` (immutable) and return `TurnResult`
- [x] Internal helpers (`handle_movement`, `apply_npc_events`, etc.) return new `GameState` instead of mutating
- [x] Replace `.ok()` swallow patterns with `?`
- [x] All helpers (`handle_movement`, `apply_npc_events`, etc.) return new state instead of mutating
- [x] `GameService::execute_action` saves snapshot via `SnapshotStorage`
- **Files:** `src/engine/action_processing.rs`, `src/application/game_service/actions.rs`
- **Acceptance:**
  - [x] All existing tests pass
  - [x] `execute_freeaction_impl` signature has no `&mut GameState`
  - [x] Snapshots are queryable in SQLite after each turn

### Task 1.5: Regeneration Support
- [x] On retry/regen: load committed snapshot from before target message
- [x] Re-run turn
- [ ] ~~save new snapshot with incremented `swipe_index`~~ — Superseded: `swipe_index` not tracked; retry creates new snapshot row and truncates messages after anchor
- **Files:** `src/application/game_service/service.rs`, `src/server/fragments.rs`, `src/application/game_service/retry.rs`
- **Acceptance:**
  - [x] Regeneration creates new snapshot row, leaves original intact
  - [ ] ~~Swiping back restores original snapshot state~~ — Superseded: retry restores by snapshot ID; no swipe UI yet

### Task 1.6: Reset Game Endpoint
- [x] `POST /reset` → deletes current game, creates new game, reloads initial world
- [x] HTMX fragment refreshes story log and sidebar
- [x] Cancel any pending generation
- **Files:** `src/server/fragments.rs`, `src/server/mod.rs`, `src/ui/dashboard.rs`
- **Acceptance:**
  - [x] Reset clears story log, returns to starting room
  - [x] NPC encounter tracking resets (`times_met = 0`)
  - [x] No server restart required
  - [x] Reset works even during generation

### Task 1.7: Remove Mutex
- [x] Replace `Arc<Mutex<GameState>>` in `AppState` with `Arc<SnapshotStorage>` + cached world data (`Arc<WorldCard>`, `Arc<MapDef>`, `Arc<PlayerCard>`, `Arc<HashMap<String, NpcCard>>`)
- [x] Add `load_state()` helper that loads latest snapshot and re-attaches cached world data
- [x] All server handlers use `load_state()` instead of `lock_state()`
- **Files:** `src/server/mod.rs`, `src/main.rs`, `src/bootstrap.rs`
- **Acceptance:**
  - [x] No `std::sync::Mutex` in production code
  - [x] No `Arc<Mutex<GameState>>` anywhere

**Phase 1 dependencies:** None  
**Phase 1 blocks:** Phase 2, Phase 3  
**Phase 1 estimated effort:** 2–3 weeks  
**Phase 1 actual:** Implemented.

---

## Phase 2: Agent Trait + Registry + Quantifier Migration

**Goal:** Define the agent abstraction. Migrate the quantifier to be a `dyn Agent`.  
**Status:** [x] Implemented

### Task 2.1: Agent Core Types
- [x] `Agent` trait, `ExecutionPhase`, `AgentResult`, `AgentContext`, `BackendSelector`
- [x] `AgentConfig` struct (deserializable from TOML/JSON)
- **Files:** `src/narrative/agents/mod.rs`, `src/model/agent.rs`
- **Acceptance:**
  - [x] Mock agent can be defined and executed in test
  - [x] `AgentConfig` round-trips through serde

### Task 2.2: Agent Registry
- [x] `AgentRegistry` loads agents from config
- [x] Built-in agents: `NarratorAgent` (wrapper for main LLM), `QuantifierAgent`
- [x] `registry.agents_for_phase(phase)` returns iterator
- **Files:** `src/narrative/agents/registry.rs`
- **Acceptance:**
  - [x] Registry loads with 0 custom agents (only built-ins)
  - [x] Disabling quantifier in config removes it from registry
  - [x] Unknown agent type in config returns error at startup

### Task 2.3: Migrate Quantifier to Agent
- [x] Move quantifier code from `narrative/quantifier/` to `narrative/agents/quantifier/` (directory module; preserves tests)
- [x] Implement `Agent` for `QuantifierAgent` in `agents/quantifier/agent.rs`
- [x] `phase()` returns `ExecutionPhase::PostGeneration`
- [x] `execute()` receives `AgentContext` with `main_response` and returns `AgentResult::StatePatch(StatePatch::Scene { ... })`
- [x] Delete `narrative/quantifier/` directory
- **Files:** `src/narrative/agents/quantifier/`, `src/narrative/quantifier/` (delete)
- **Acceptance:**
  - [x] Quantifier agent produces identical `NpcEventList` to old code
  - [x] All quantifier tests pass (import paths updated)
  - [x] Disabling quantifier skips NPC detection (documented)

### Task 2.4: Per-Agent Backend Selection
- [x] `BackendSelector::UseMain | UseNamed(String)`
- [ ] ~~`AgentRegistry` resolves backend per agent at construction time~~ — Superseded: quantifier hardcodes its backend in `from_config_with_storage`; registry stores already-constructed agents
- [x] Quantifier defaults to existing quantifier backend config
- [ ] ~~`UseMain` for quantifier falls back to default quantifier backend (quantifier uses `QuantifierBackendTrait`, not `LlmBackend`)~~ — Superseded: quantifier uses `LlmBackend` directly; `QuantifierBackendTrait` deleted entirely
- **Files:** `src/narrative/agents/registry.rs`, ~~`src/narrative/agents/quantifier/backends.rs`~~ (not created as separate file)
- **Acceptance:**
  - [x] Quantifier can use different backend from narrator
  - [ ] Unknown backend name falls back to default with warning — *not yet implemented*

**Phase 2 dependencies:** Phase 1  
**Phase 2 blocks:** Phase 3  
**Phase 2 estimated effort:** 1 week  
**Phase 2 actual:** Implemented.

---

## Phase 3: Phase-Based Pipeline

**Goal:** Replace hardcoded `execute_action` with phase-based pipeline.  
**Status:** Partially implemented — post-generation agents run via bridge function; formal pipeline and pre-generation not yet wired.

### Task 3.1: Pipeline Structure
- [ ] ~~`AgentPipeline` with `pre_generate()` and `post_generate(main_response)`~~ — Superseded by bridge approach: `run_post_generation_agents()` in `actions.rs` iterates over `PostGeneration` agents inline
- [ ] Pre-generation phase is empty for now (returns `Vec<AgentInjection>`) — *`NarratorAgent` exists but pre-generation is not invoked in action flow*
- [x] Post-generation phase runs quantifier agent
- [ ] ~~Pipeline takes `AgentRegistry`, `AgentContext`, backend resolver~~ — Not implemented as a standalone struct
- **Files:** ~~`src/engine/agent_pipeline.rs`~~ (not created)
- **Acceptance:**
  - [x] Pipeline with 0 agents runs main LLM only
  - [x] Pipeline with quantifier runs main LLM then quantifier
  - [ ] Pre-generation phase exists and returns empty vec — *exists in code but not wired into flow*

### Task 3.2: Agent Result Application
- [ ] ~~`AgentResultDispatcher` applies results~~ — Superseded: handled inline in `run_post_generation_agents`
  - [x] `PromptDirective` → acknowledged but ignored in post-generation (warned and skipped)
  - [x] `StatePatch` → translated back to `QuantifierResult` for compatibility with existing `action_processing.rs`
- **Files:** ~~`src/engine/agent_pipeline.rs`~~ (not created)
- **Acceptance:**
  - [x] StatePatch updates snapshot correctly
  - [ ] Invalid patch is rejected with error — *partial: errors are logged but not hard-rejected*

### Task 3.3: Delete Old Pipeline
- [ ] Remove hardcoded `execute_action` flow from `application/game_service/actions.rs` — *still present (`actions.rs` ~392 lines)*
- [ ] Replace with pipeline orchestration
- [x] Delete `QuantifierBackendTrait` (quantifier is now just an `Agent`)
- **Files:** `src/application/game_service/service.rs`, `src/narrative/llm/mod.rs`
- **Acceptance:**
  - [ ] `application/game_service/` module under 250 lines per file — *not met (`actions.rs` is 392 lines)*
  - [x] No `QuantifierBackendTrait` references remain
  - [x] All integration tests pass

**Phase 3 dependencies:** Phase 1, Phase 2  
**Phase 3 blocks:** Phase 4  
**Phase 3 estimated effort:** 1 week

---

## Phase 4: Prompt Assembler

**Goal:** Replace `PromptBuilder` with preset-based assembly.  
**Status:** Completed. `PromptBuilder` deleted; `LayeredPromptAssembler` and `PromptPreset` are active.

### Task 4.1: Preset Types
- [x] `PromptPreset` — lives in `src/model/prompt_preset.rs`
- [x] Macro resolution: `{{user}}`, `{{room}}` via `PromptPreset::assemble_prompt_text()`
- **Files:** `src/model/prompt_preset.rs`
- **Acceptance:**
  - [x] Preset with 4 sections (role, instructions, writing_style, output_format) assembles correctly
  - [x] Missing section is omitted from output

### Task 4.2: Assembler
- [x] `PromptAssembler` trait with `assemble(context, preset, global_rules, response_length) -> Result<AssembledPrompt, EngineError>`
- [x] `LayeredPromptAssembler` implements 7-layer construction with XML wrapping
- [x] `fit_messages_to_context()` for dynamic budget management
- **Files:** `src/narrative/prompt/assembler.rs`
- **Acceptance:**
  - [x] Matches old `PromptBuilder` output for default preset

### Task 4.3: Default Preset
- [x] Create `data/prompt_presets/system/default.json` matching current 7-layer output
- [x] Load at startup; fallback seed created if missing
- **Files:** `data/prompt_presets/system/default.json`, `src/bootstrap/run.rs`
- **Acceptance:**
  - [x] Default preset produces identical prompt to old builder
  - [x] Tests pass with assembler

### Task 4.4: Delete PromptBuilder
- [x] Remove `narrative/prompt/builder.rs`
- [x] Update all call sites to use assembler
- **Files:** `src/narrative/prompt/builder.rs` (deleted), callers updated
- **Acceptance:**
  - [x] No `PromptBuilder` references in codebase
  - [x] All prompt tests pass with assembler (`assembler_tests.rs`)

**Phase 4 dependencies:** Phase 3  
**Phase 4 blocks:** None  
**Phase 4 estimated effort:** 1–2 weeks

---

## Dependency Graph

```
Phase 1: SQLite + Snapshots + Reset        [x] Implemented
    │
    ▼
Phase 2: Agent Trait + Quantifier Migration [x] Implemented
    │
    ▼
Phase 3: Phase-Based Pipeline               [~] Partially implemented
    │
    ▼
Phase 4: Prompt Assembler                   [ ] Not started
```

All phases are sequential. Total estimated effort: **5–7 weeks**.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| SQLite adds build complexity | Low | Low | `bundled` feature includes SQLite; no system dependency |
| Snapshot serialisation slow | Medium | Medium | Use `Arc` for large fields; benchmark before/after |
| Prompt assembler breaks quality | Medium | High | Byte-identical regression test for default preset |
| Mutex removal causes races | Low | High | No shared mutable state = no races; test concurrent requests |
| Test suite bloat | Medium | Medium | Reuse mock infrastructure; one test file per new module |

---

## Future Work (Explicitly Out of Scope)

These are **not** part of this plan. They become possible after Phase 4 completes:

- **LLM batching** — Multiple agents in one API call (future cost optimisation)
- **Additional agents** — Prose Guardian, Continuity Checker, Expression Engine, Custom Tracker, Lorebook Keeper
- **Save/load** — Export/import game state to file
- **Function calling** — Native tool use for OpenAI/Anthropic backends
- **Combat system** — Requires `CombatState` in snapshot schema

---

## Sub-Plans

As phases are approved, detailed sub-plans are created as separate files and linked here.

| Phase | Status | Plan File |
|-------|--------|-----------|
| Phase 1: SQLite + Snapshots + Reset | **[x] Implemented** | [`multi-agent-phase1-snapshots-reset.md`](multi-agent-phase1-snapshots-reset.md) |
| Phase 2: Agent Trait + Quantifier Migration | **[x] Implemented** | [`phase2-agent-trait-quantifier-migration-20260510.md`](archived/phase2-agent-trait-quantifier-migration-20260510.md) |
| Phase 3: Phase-Based Pipeline | **[~] Partial** | *(TBD — bridge approach documented in `actions.rs`)* |
| Phase 4: Prompt Assembler | **[ ] Not started** | *(TBD)* |

**Pattern:** Each sub-plan references this spec in its header and links back. The spec remains the single source of truth for cross-phase decisions; sub-plans contain task-level detail only.

---

## Next Steps

1. **Complete Phase 3 formalisation:** Create `AgentPipeline` struct, wire `pre_generate()` into `actions.rs`, remove `QuantifierResult` bridge so agents mutate state directly.
2. **Begin Phase 4:** Design `PromptPreset` / `PromptAssembler`, build `data/presets/default.json`, migrate all call sites off `PromptBuilder`, then delete it.
