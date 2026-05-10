# Spec: Agent-Ready Pipeline Restructure for Chronicler Engine

**Date:** 2026-05-09
**Status:** Draft — awaiting review
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
- The current pipeline embeds the quantifier as load-bearing logic in `game_service.rs` and `action_processing.rs`
- Adding even one more agent (e.g. a pre-generation prompt injector) would require rewriting the pipeline anyway
- The GameState snapshot refactoring is needed for regeneration, reset, and diagnostics regardless of agents
- The PromptBuilder's 8 hardcoded layers block any form of dynamic prompt injection

**What success looks like:**
- The quantifier runs as a `dyn Agent` in a post-generation phase
- Pre-generation phase exists and is empty (ready for future agents)
- Game state is snapshotted per turn, enabling reset and regeneration
- Prompts are assembled from configurable presets, not hardcoded layers
- A new agent can be added later by: implementing `Agent` trait + adding config entry

---

## Assumptions

1. **Single-player only.** No multiplayer, no concurrent players.
2. **SQLite from Phase 1.** State snapshots persist to SQLite; no in-memory-only intermediate step.
3. **No backward compatibility.** `PromptBuilder` is deleted, `GameState` changes freely, config formats break.
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
| State | `Arc<Mutex<GameState>>` | SQLite-backed snapshots | Major |
| Pipeline | Hardcoded in `game_service.rs` | Phase-based `AgentPipeline` | Major |
| Prompts | `PromptBuilder` (8 hardcoded layers) | Preset-based assembler | Major |
| DB | None | SQLite (`rusqlite`) | New dependency |
| Agents | Hardcoded quantifier | `dyn Agent` trait + registry | Moderate |

**New dependencies:**
- `rusqlite` (with `bundled` feature for zero-system-dependency builds)
- `uuid` (with `v4,serde` features for snapshot IDs)

---

## Commands

```bash
# Full validation (existing)
cd chronicler_engine && python build.py

# DB setup (new)
cargo run -- db-migrate

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
│   │   ├── state_snapshot.rs             # NEW: Snapshot types
│   │   └── ... (existing)
│   ├── engine/
│   │   ├── game_service.rs               # REFACTORED: thin orchestrator
│   │   ├── action_processing.rs          # REFACTORED: stateless, returns TurnResult
│   │   ├── agent_pipeline.rs             # NEW: Phase-based orchestration
│   │   └── ... (existing)
│   ├── narrative/
│   │   ├── prompt/
│   │   │   ├── assembler.rs              # NEW: Preset-based assembly
│   │   │   ├── preset.rs                 # NEW: Preset types
│   │   │   └── builder.rs                # DELETED
│   │   ├── agents/
│   │   │   ├── mod.rs                    # NEW: Agent trait + registry
│   │   │   └── quantifier.rs             # MIGRATED: from quantifier/core.rs
│   │   └── ... (existing)
│   ├── storage/
│   │   ├── mod.rs                        # NEW: DB abstraction
│   │   ├── db.rs                         # NEW: rusqlite connection + migrations
│   │   └── snapshot_storage.rs           # NEW: Snapshot CRUD
│   └── server/
│       └── ... (existing + reset endpoint)
├── data/
│   └── presets/
│       └── default.json                  # NEW: default prompt preset
└── tests/
    └── ... (existing + snapshot + pipeline tests)
```

---

## Code Style

### Agent Trait

```rust
// [DOC: docs/system/agent_system.md]

pub enum ExecutionPhase {
    PreGeneration,   // Before main LLM call
    PostGeneration,  // After main LLM call
}

pub enum AgentResult {
    PromptDirective(String),    // Inject text into prompt
    StatePatch(StatePatch),     // Mutate snapshot
    NoOp,
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

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameStateSnapshot {
    pub id: String,                    // uuid v4
    pub message_id: String,
    pub swipe_index: u32,
    pub movement: MovementState,
    pub narrative: NarrativeState,
    pub scene: SceneState,
    pub character_state: CharacterState,
    pub committed: bool,
    pub created_at: DateTime<Utc>,
}
```

### Turn Result

```rust
pub struct TurnResult {
    pub next_state: GameState,         // fully populated with re-attached world data
    pub narration: String,
    pub trigger_continuation: Option<TriggerContinuationRequest>,
}
```

Note: `agent_results` is added in Phase 2 when the agent trait is introduced.

---

## Testing Strategy

| Level | Purpose |
|-------|---------|
| Unit | Snapshot round-tripping, preset assembly, agent trait mock |
| Integration | Full pipeline with mock backends, state assertions |
| Property | Snapshot serialisation, preset determinism |
| Guardrails | File length, arch-lint |

**Key tests:**
- Pipeline with 0 agents runs main LLM only (baseline)
- Quantifier agent produces identical output to old hardcoded code
- Reset clears all snapshots and returns to initial state
- `GameState` → snapshot → `GameState` round-trips without loss

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

1. **Pipeline is agent-shaped:** The quantifier runs as `dyn Agent` in a post-generation phase.
2. **Pre-generation phase exists:** Empty but callable — adding a future agent requires no pipeline changes.
3. **State is snapshotted:** Each turn creates a new `GameStateSnapshot` row in SQLite.
4. **Reset works:** `POST /reset` clears SQLite, reloads world, returns to starting room.
5. **Baseline preserved:** With only quantifier enabled, output matches pre-migration.
6. **Test ratio ≥ 1.5.**
7. **Performance:** Turn with quantifier takes ≤ 110% of pre-migration time.

---

# Implementation: Phased Plan

## Phase 1: SQLite + GameState Snapshots + Reset

**Goal:** Replace `Arc<Mutex<GameState>>` with SQLite-backed snapshots. Add reset endpoint.

### Task 1.1: SQLite Setup
- Add `rusqlite` with `bundled` feature to `Cargo.toml`
- Create `storage/db.rs` with connection pooling and migrations
- Migration 001: `game_state_snapshots` table
  - `id TEXT PRIMARY KEY`
  - `message_id TEXT NOT NULL`
  - `swipe_index INTEGER NOT NULL DEFAULT 0`
  - `movement TEXT NOT NULL` (JSON)
  - `narrative TEXT NOT NULL` (JSON)
  - `scene TEXT NOT NULL` (JSON)
  - `character_state TEXT NOT NULL` (JSON)
  - `committed INTEGER NOT NULL DEFAULT 0`
  - `created_at TEXT NOT NULL` (ISO 8601)
- **Files:** `src/storage/db.rs`, `src/storage/mod.rs`
- **Acceptance:**
  - [ ] `cargo run -- db-migrate` creates `.db` file
  - [ ] `.db` file is in `.gitignore`
  - [ ] Migrations are idempotent

### Task 1.2: Snapshot Types
- Add `GameStateSnapshot` in `model/state_snapshot.rs`
- `From<GameState>` and `TryFrom<GameStateSnapshot> for GameState`
- **Files:** `src/model/state_snapshot.rs`
- **Acceptance:**
  - [ ] Round-trip property test: 100 random states → snapshot → state
  - [ ] No data loss on round-trip

### Task 1.3: Snapshot Storage
- Implement `SnapshotStorage` trait:
  ```rust
  pub trait SnapshotStorage: Send + Sync {
      fn save(&self, snapshot: &GameStateSnapshot) -> Result<(), EngineError>;
      fn load_latest(&self, message_id: Option<&str>) -> Result<Option<GameStateSnapshot>, EngineError>;
      fn commit(&self, snapshot_id: &str) -> Result<(), EngineError>;
      fn reset(&self) -> Result<(), EngineError>;
  }
  ```
- SQLite implementation using `rusqlite`
- **Files:** `src/storage/snapshot_storage.rs`
- **Acceptance:**
  - [ ] Save + load round-trips
  - [ ] `commit` sets flag; `load_latest` prefers uncommitted for active turn
  - [ ] `reset` drops all rows

### Task 1.4: Stateless Action Processing
- Refactor `execute_freeaction_impl` to take `&GameState` (immutable) and return `TurnResult`
- Internal helpers (`handle_movement`, `apply_npc_events`, etc.) return new `GameState` instead of mutating
- Replace `.ok()` swallow patterns with `?`
- All helpers (`handle_movement`, `apply_npc_events`, etc.) return new state instead of mutating
- `GameService::execute_action` saves snapshot via `SnapshotStorage`
- **Files:** `src/engine/action_processing.rs`, `src/engine/game_service.rs`
- **Acceptance:**
  - [ ] All existing tests pass
  - [ ] `execute_freeaction_impl` signature has no `&mut GameState`
  - [ ] Snapshots are queryable in SQLite after each turn

### Task 1.5: Regeneration Support
- On retry/regen: load committed snapshot from before target message
- Re-run turn, save new snapshot with incremented `swipe_index`
- **Files:** `src/engine/game_service.rs`, `src/server/fragments.rs`
- **Acceptance:**
  - [ ] Regeneration creates new snapshot row, leaves original intact
  - [ ] Swiping back restores original snapshot state

### Task 1.6: Reset Game Endpoint
- `POST /reset` → calls `snapshot_storage.reset()` + reloads initial world
- HTMX fragment refreshes story log and sidebar
- Cancel any pending generation
- **Files:** `src/server/fragments.rs`, `src/server/mod.rs`, `src/ui/dashboard.rs`
- **Acceptance:**
  - [ ] Reset clears story log, returns to starting room
  - [ ] NPC encounter tracking resets (`times_met = 0`)
  - [ ] No server restart required
  - [ ] Reset works even during generation

### Task 1.7: Remove Mutex
- Replace `Arc<Mutex<GameState>>` in `AppState` with `Arc<SnapshotStorage>` + cached world data (`Arc<WorldCard>`, `Arc<MapDef>`, `Arc<PlayerCard>`, `Arc<HashMap<String, NpcCard>>`)
- Add `load_state()` helper that loads latest snapshot and re-attaches cached world data
- All server handlers use `load_state()` instead of `lock_state()`
- **Files:** `src/server/mod.rs`, `src/main.rs`, `src/bootstrap.rs`
- **Acceptance:**
  - [ ] No `std::sync::Mutex` in production code
  - [ ] No `Arc<Mutex<GameState>>` anywhere

**Phase 1 dependencies:** None
**Phase 1 blocks:** Phase 2, Phase 3
**Phase 1 estimated effort:** 2–3 weeks

---

## Phase 2: Agent Trait + Registry + Quantifier Migration

**Goal:** Define the agent abstraction. Migrate the quantifier to be a `dyn Agent`.

### Task 2.1: Agent Core Types
- `Agent` trait, `ExecutionPhase`, `AgentResult`, `AgentContext`, `BackendSelector`
- `AgentConfig` struct (deserializable from TOML/JSON)
- **Files:** `src/narrative/agents/mod.rs`, `src/model/agent.rs`
- **Acceptance:**
  - [ ] Mock agent can be defined and executed in test
  - [ ] `AgentConfig` round-trips through serde

### Task 2.2: Agent Registry
- `AgentRegistry` loads agents from config
- Built-in agents: `NarratorAgent` (wrapper for main LLM), `QuantifierAgent`
- `registry.agents_for_phase(phase)` returns iterator
- **Files:** `src/narrative/agents/registry.rs`
- **Acceptance:**
  - [ ] Registry loads with 0 custom agents (only built-ins)
  - [ ] Disabling quantifier in config removes it from registry
  - [ ] Unknown agent type in config returns error at startup

### Task 2.3: Migrate Quantifier to Agent
- Move `determine_npcs_in_room` from `narrative/quantifier/core.rs` to `narrative/agents/quantifier.rs`
- Implement `Agent` for `QuantifierAgent`
- `phase()` returns `ExecutionPhase::PostGeneration`
- `execute()` receives `AgentContext` with `main_response` and returns `AgentResult::StatePatch`
- Delete `narrative/quantifier/` directory
- **Files:** `src/narrative/agents/quantifier.rs`, `src/narrative/quantifier/` (delete)
- **Acceptance:**
  - [ ] Quantifier agent produces identical `NpcEventList` to old code
  - [ ] All quantifier tests pass without modification
  - [ ] Disabling quantifier skips NPC detection (documented)

### Task 2.4: Per-Agent Backend Selection
- `BackendSelector::UseMain | UseNamed(String)`
- `AgentRegistry` resolves backend per agent
- Quantifier defaults to existing quantifier backend config
- **Files:** `src/engine/game_service.rs`, `src/narrative/llm/mod.rs`
- **Acceptance:**
  - [ ] Quantifier can use different backend from narrator
  - [ ] Unknown backend name falls back to main with warning

**Phase 2 dependencies:** Phase 1
**Phase 2 blocks:** Phase 3
**Phase 2 estimated effort:** 1 week

---

## Phase 3: Phase-Based Pipeline

**Goal:** Replace hardcoded `execute_action` with phase-based pipeline.

### Task 3.1: Pipeline Structure
- `AgentPipeline` with `pre_generate()` and `post_generate(main_response)`
- Pre-generation phase is empty for now (returns `Vec<AgentInjection>`)
- Post-generation phase runs quantifier agent
- Pipeline takes `AgentRegistry`, `AgentContext`, backend resolver
- **Files:** `src/engine/agent_pipeline.rs`
- **Acceptance:**
  - [ ] Pipeline with 0 agents runs main LLM only
  - [ ] Pipeline with quantifier runs main LLM then quantifier
  - [ ] Pre-generation phase exists and returns empty vec

### Task 3.2: Agent Result Application
- `AgentResultDispatcher` applies results:
  - `PromptDirective` → appends to system prompt (for future pre-gen agents)
  - `StatePatch` → creates new snapshot with patch applied
- **Files:** `src/engine/agent_pipeline.rs`
- **Acceptance:**
  - [ ] StatePatch updates snapshot correctly
  - [ ] Invalid patch is rejected with error

### Task 3.3: Delete Old Pipeline
- Remove hardcoded `execute_action` flow from `game_service.rs`
- Replace with pipeline orchestration
- Delete `QuantifierBackendTrait` (quantifier is now just an `Agent`)
- **Files:** `src/engine/game_service.rs`, `src/narrative/llm/mod.rs`
- **Acceptance:**
  - [ ] `game_service.rs` under 250 lines
  - [ ] No `QuantifierBackendTrait` references remain
  - [ ] All integration tests pass

**Phase 3 dependencies:** Phase 1, Phase 2
**Phase 3 blocks:** Phase 4
**Phase 3 estimated effort:** 1 week

---

## Phase 4: Prompt Assembler

**Goal:** Replace `PromptBuilder` with preset-based assembly.

### Task 4.1: Preset Types
- `PromptPreset`, `PromptSection`, `WrapFormat`, `MarkerConfig`
- Macro resolution: `{{user}}`, `{{room}}`, `{{agent::TYPE}}`
- **Files:** `src/narrative/prompt/preset.rs`
- **Acceptance:**
  - [ ] Preset with 3 sections assembles correctly
  - [ ] Missing macro leaves placeholder

### Task 4.2: Assembler
- `assemble_prompt(preset, context) -> Vec<ChatMessage>`
- Depth injection support (sections inserted N messages deep)
- Group wrapping (XML tags)
- **Files:** `src/narrative/prompt/assembler.rs`
- **Acceptance:**
  - [ ] Matches old `PromptBuilder` output for default preset

### Task 4.3: Default Preset
- Create `data/presets/default.json` matching current 8-layer output
- Load at startup; fall back to compiled default if missing
- **Files:** `data/presets/default.json`, `src/bootstrap.rs`
- **Acceptance:**
  - [ ] Default preset produces identical prompt to old builder
  - [ ] Byte-identical regression test passes

### Task 4.4: Delete PromptBuilder
- Remove `narrative/prompt/builder.rs`
- Update all call sites to use assembler
- **Files:** `src/narrative/prompt/builder.rs` (delete), callers
- **Acceptance:**
  - [ ] No `PromptBuilder` references in codebase
  - [ ] All prompt tests pass with assembler

**Phase 4 dependencies:** Phase 3
**Phase 4 blocks:** None
**Phase 4 estimated effort:** 1–2 weeks

---

## Dependency Graph

```
Phase 1: SQLite + Snapshots + Reset
    │
    ▼
Phase 2: Agent Trait + Quantifier Migration
    │
    ▼
Phase 3: Phase-Based Pipeline
    │
    ▼
Phase 4: Prompt Assembler
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
| Phase 1: SQLite + Snapshots + Reset | **Ready** | [`multi-agent-phase1-snapshots-reset.md`](multi-agent-phase1-snapshots-reset.md) |
| Phase 2: Agent Trait + Quantifier Migration | Not started | *(TBD)* |
| Phase 3: Phase-Based Pipeline | Not started | *(TBD)* |
| Phase 4: Prompt Assembler | Not started | *(TBD)* |

**Pattern:** Each sub-plan references this spec in its header and links back. The spec remains the single source of truth for cross-phase decisions; sub-plans contain task-level detail only.

---

## Ready for Phase 1?

Phase 1 is fully specified in [`multi-agent-phase1-snapshots-reset.md`](multi-agent-phase1-snapshots-reset.md). It can begin immediately — no dependencies, no blockers.

→ **Approve Phase 1 and I'll start implementing Task 1.1.**
