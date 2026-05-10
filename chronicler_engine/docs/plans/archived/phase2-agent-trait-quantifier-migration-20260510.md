# Plan: Phase 2 — Agent Trait, Registry, and Quantifier Migration

**Date:** 2026-05-10
**Status:** Planned
**Parent Spec:** `docs/plans/multi-agent-architecture-overarching-spec.md`
**Goal:** Define the `Agent` trait, build the `AgentRegistry`, and migrate the quantifier to run as a `dyn Agent` in a post-generation phase.
**Estimated Effort:** 1 week

---

## Overview

Phase 1 replaced `Arc<Mutex<GameState>>` with SQLite-backed snapshots. Phase 2 defines the agent abstraction that future agents will plug into.

**What changes:**
- New `Agent` trait with `ExecutionPhase`, `AgentResult`, `AgentContext`, and `BackendSelector`.
- `AgentRegistry` loads built-in and custom agents from config.
- `QuantifierAgent` implements `Agent`; old `narrative/quantifier/` directory is deleted.
- `DefaultGameService` calls the quantifier through the `Agent` trait instead of directly.

**What does NOT change:**
- Prompt assembly (Phase 4).
- Pipeline orchestration structure (Phase 3 — the hardcoded `execute_action` flow stays for now, but calls the quantifier via trait).
- `action_processing.rs` signature (`FreeActionContext` still takes `QuantifierResult`).
- Narrative quality, quantifier behaviour, or HTMX dashboard.

**Critical insight from codebase:** The quantifier currently runs between narration and `execute_freeaction_impl`, receiving the AI-generated text and returning `QuantifierResult`. In the new architecture, it becomes a post-generation `Agent` that returns `AgentResult::StatePatch`. `game_service.rs` translates that patch back into `QuantifierResult` for `action_processing.rs` in Phase 2; Phase 3 will replace that bridge with generic dispatch.

---

## Architecture Decisions

1. **`StatePatch` is an enum with per-domain variants.** Initially only `Scene { npc_ids, movement_destination }`. Future agents add variants (e.g., `Narrative`, `Character`) without changing the `AgentResult` enum shape.
2. **Agents own their backends at construction time.** `AgentRegistry` constructs each agent with its resolved backend (`Arc<dyn QuantifierBackendTrait>` for quantifier). `BackendSelector` is metadata for registry construction, not a runtime resolver.
3. **`NarratorAgent` is a stub.** The main LLM call is not yet part of the agent pipeline (that happens in Phase 3). `NarratorAgent` exists in the registry as a built-in for completeness and future use.
4. **Quantifier code moves to `agents/quantifier/` module.** Types, parser, prompt, and backends move as a unit to preserve existing tests with minimal import changes.
5. **`AgentContext` uses the full `GameState` reference.** Post-generation agents need room info, NPCs, history, and the `main_response`. Pre-generation agents (future) will use the same context.

---

## Task 2.1: Agent Core Types

**Goal:** Define the `Agent` trait and all supporting types.

### Context

The trait lives in `src/narrative/agents/mod.rs`. Supporting types (`ExecutionPhase`, `AgentResult`, `AgentContext`, `BackendSelector`, `StatePatch`) can live there or in `src/model/agent.rs` depending on dependency direction.

Because `AgentContext` holds `&GameState` and `AgentResult::StatePatch` references model types, the clean split is:
- `src/model/agent.rs` — data enums and structs (`ExecutionPhase`, `AgentResult`, `AgentContext`, `BackendSelector`, `StatePatch`, `AgentConfig`)
- `src/narrative/agents/mod.rs` — the `Agent` trait plus module declarations

This keeps `model/` as the dependency root and avoids circular imports between `narrative/` and `model/`.

### Steps

1. Create `src/model/agent.rs`:
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum ExecutionPhase {
       PreGeneration,
       PostGeneration,
   }

   impl Default for ExecutionPhase {
       fn default() -> Self {
           ExecutionPhase::PostGeneration
       }
   }

   #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case", tag = "type", content = "value")]
   pub enum BackendSelector {
       UseMain,
       UseNamed(String),
   }

   impl Default for BackendSelector {
       fn default() -> Self {
           BackendSelector::UseMain
       }
   }

   #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
   pub struct AgentConfig {
       pub name: String,
       pub agent_type: String,
       pub enabled: bool,
       #[serde(default)]
       pub backend: BackendSelector,
       #[serde(default)]
       pub phase: ExecutionPhase,
   }

   #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum Confidence {
       High,
       Medium,
       Low,
   }

   #[derive(Debug, Clone, PartialEq)]
   pub enum StatePatch {
       Scene {
           npc_ids: Vec<String>,
           movement_destination: Option<String>,
           confidence: Confidence,
       },
   }

   #[derive(Debug, Clone, PartialEq)]
   pub enum AgentResult {
       PromptDirective(String),
       StatePatch(StatePatch),
       NoOp,
   }

   pub struct AgentContext<'a> {
       pub state: &'a crate::model::state::GameState,
       pub main_response: Option<&'a str>,
       pub player_input: &'a str,
   }
   ```

2. Add `pub mod agent;` to `src/model/mod.rs`.

3. Create `src/narrative/agents/mod.rs`:
   ```rust
   pub mod quantifier;
   pub mod registry;

   pub use crate::model::agent::{
       AgentConfig, AgentContext, AgentResult, BackendSelector, ExecutionPhase, StatePatch,
   };

   pub trait Agent: Send + Sync + std::fmt::Debug {
       fn name(&self) -> &str;
       fn phase(&self) -> ExecutionPhase;
       fn backend_selector(&self) -> BackendSelector;
       fn execute(&self, ctx: &AgentContext) -> crate::error::Result<AgentResult>;
   }
   ```

4. Add `pub mod agents;` to `src/narrative/mod.rs`.

**Files:**
- `src/model/agent.rs` (new)
- `src/model/mod.rs`
- `src/narrative/agents/mod.rs` (new)
- `src/narrative/mod.rs`

**Acceptance Criteria:**
- [ ] `cargo check` passes with no new warnings.
- [ ] Mock agent can be defined in a test file and executed:
   ```rust
   #[derive(Debug)]
   struct MockAgent;
   impl Agent for MockAgent { ... }
   ```
- [ ] `AgentConfig` round-trips through `serde_json` (all fields preserved).
- [ ] `AgentConfig` round-trips through `toml` crate (all fields preserved).
- [ ] `AgentContext` lifetime `'a` allows borrowing from `GameState` without clone.

---

## Task 2.2: Agent Registry

**Goal:** Build `AgentRegistry` that loads agents from config and iterates by phase.

### Context

The registry is constructed at startup (in `bootstrap.rs` or `main.rs`) and passed into `DefaultGameService`. It holds `Vec<Box<dyn Agent>>`.

Built-in agents:
- `NarratorAgent` — stub; returns `AgentResult::NoOp`. Phase = `PreGeneration`.
- `QuantifierAgent` — migrated quantifier. Phase = `PostGeneration`.

Custom agents are out of scope for this phase, but the registry design must support them later.

### Steps

1. Create `src/narrative/agents/registry.rs`:
   ```rust
   use crate::error::EngineError;
   use crate::model::agent::{AgentConfig, BackendSelector, ExecutionPhase};
   use crate::narrative::agents::{Agent, quantifier::QuantifierAgent};

   #[derive(Debug, Default)]
   pub struct AgentRegistry {
       agents: Vec<Box<dyn Agent>>,
   }

   impl AgentRegistry {
       pub fn from_configs(configs: &[AgentConfig]) -> Result<Self, EngineError> {
           let mut registry = Self::default();

           // If no agent configs exist, inject defaults for backward compatibility.
           // This ensures existing settings.toml files without an [agents] section
           // still get the quantifier enabled.
           let default_configs = default_agent_configs();
           let effective_configs = if configs.is_empty() {
               &default_configs[..]
           } else {
               configs
           };

           for config in effective_configs {
               if !config.enabled {
                   continue;
               }
               let agent: Box<dyn Agent> = match config.agent_type.as_str() {
                   "quantifier" => Box::new(QuantifierAgent::from_config(config)?),
                   "narrator" => Box::new(NarratorAgent::new(config.name.clone())),
                   other => {
                       return Err(EngineError::Config(format!(
                           "Unknown agent type: {other}"
                       )));
                   }
               };
               registry.agents.push(agent);
           }
           Ok(registry)
       }

       pub fn with_agent(agent: Box<dyn Agent>) -> Self {
           Self {
               agents: vec![agent],
           }
       }

       pub fn add_agent(&mut self, agent: Box<dyn Agent>) {
           self.agents.push(agent);
       }

       pub fn agents_for_phase(
           &self,
           phase: ExecutionPhase,
       ) -> impl Iterator<Item = &dyn Agent> {
           self.agents
               .iter()
               .filter(move |a| a.phase() == phase)
               .map(|a| a.as_ref())
       }

       pub fn is_empty(&self) -> bool {
           self.agents.is_empty()
       }
   }
   ```

2. Implement `NarratorAgent` in `src/narrative/agents/mod.rs` (or a new `src/narrative/agents/narrator.rs`):
   ```rust
   #[derive(Debug)]
   pub struct NarratorAgent {
       name: String,
   }

   impl NarratorAgent {
       pub fn new(name: String) -> Self {
           Self { name }
       }
   }

   impl Agent for NarratorAgent {
       fn name(&self) -> &str {
           &self.name
       }
       fn phase(&self) -> ExecutionPhase {
           ExecutionPhase::PreGeneration
       }
       fn backend_selector(&self) -> BackendSelector {
           BackendSelector::UseMain
       }
       fn execute(&self, _ctx: &AgentContext) -> crate::error::Result<AgentResult> {
           Ok(AgentResult::NoOp)
       }
   }
   ```

3. Add default agent config to settings. Update `src/model/settings.rs` (or create an `agents.toml` / extend existing settings):
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   pub struct AppSettings {
       // ... existing fields ...
       #[serde(default)]
       pub agents: Vec<AgentConfig>,
   }
   ```

   Provide a default so existing `settings.toml` files without `[agents]` continue to work:
   ```rust
   fn default_agent_configs() -> Vec<AgentConfig> {
       vec![
           AgentConfig {
               name: "quantifier".to_string(),
               agent_type: "quantifier".to_string(),
               enabled: true,
               backend: BackendSelector::UseNamed("quantifier".to_string()),
               phase: ExecutionPhase::PostGeneration,
           },
       ]
   }

   impl Default for AppSettings {
       fn default() -> Self {
           Self {
               // ... existing defaults ...
               agents: default_agent_configs(),
           }
       }
   }
   ```

4. In `bootstrap.rs` or `main.rs`, construct the registry and pass it to `DefaultGameService::with_backends`.

**Files:**
- `src/narrative/agents/registry.rs` (new)
- `src/narrative/agents/mod.rs`
- `src/model/settings.rs`
- `src/bootstrap.rs`

**Acceptance Criteria:**
- [ ] Registry loads with 0 custom agents (only built-ins) when no agent config is present.
- [ ] Disabling quantifier in config removes it from registry (`agents_for_phase(PostGeneration)` yields empty).
- [ ] Unknown `agent_type` in config returns `EngineError::Config` at startup (fail-fast).
- [ ] `agents_for_phase(PreGeneration)` and `agents_for_phase(PostGeneration)` return correct subsets.
- [ ] `NarratorAgent` stub returns `NoOp` for any context.

---

## Task 2.3: Migrate Quantifier to Agent

**Goal:** Move quantifier code to `agents/quantifier/` and implement `Agent` for `QuantifierAgent`.

### Context

Current quantifier lives in `src/narrative/quantifier/` (11 files). The spec says delete this directory. The new home is `src/narrative/agents/quantifier/` (directory module) so that types, parser, prompt, backends, and tests can move as a unit without file-size bloat.

`determine_npcs_in_room` becomes the core of `QuantifierAgent::execute`. It receives `AgentContext`, extracts the narration text from `main_response`, runs the existing quantifier logic, and returns `AgentResult::StatePatch(StatePatch::Scene { ... })`.

### Steps

1. Create directory `src/narrative/agents/quantifier/`.

2. Move files from `src/narrative/quantifier/` to `src/narrative/agents/quantifier/`:
   - `types.rs` → unchanged
   - `parser.rs` → unchanged
   - `prompt.rs` → unchanged
   - `backends.rs` → unchanged
   - `core.rs` → absorb `determine_npcs_in_room` into `quantifier.rs`; keep `quantify_room_with_llm_call` and helpers as free functions
   - `test_support.rs` → unchanged
   - All `*_tests.rs` files → unchanged except update `use` paths

3. Create `src/narrative/agents/quantifier/mod.rs`:
   ```rust
   pub mod backends;
   pub mod core;
   pub mod parser;
   pub mod prompt;
   pub mod types;

   pub use backends::{
       MockQuantifierBackend, QuantifierBackendTrait, RealQuantifierBackend, get_quantifier_backend,
       get_quantifier_backend_for,
   };
   pub use core::{action_boundary_contains, quantify_room_with_llm_call};
   pub use parser::{
       compute_npc_events, extract_movement_from_text, parse_quantifier_response,
       parse_quantifier_response_with_movement,
   };
   pub use prompt::QuantifierPromptBuilder;
   pub use types::{
       MovementParseResult, MovementType, NpcEvent, NpcEventList, NpcEventType, QuantifierConfidence,
       QuantifierParseResult, QuantifierPromptContext, QuantifierResult, RoomInfo,
   };

   #[cfg(test)]
   pub(crate) mod test_support;
   #[cfg(test)]
   mod backends_tests;
   #[cfg(test)]
   mod core_tests;
   #[cfg(test)]
   mod parser_tests;
   #[cfg(test)]
   mod prompt_tests;
   ```

4. Create `src/narrative/agents/quantifier.rs` (parent module file for `agents::quantifier`):
   ```rust
   pub mod quantifier;
   pub use quantifier::*;
   ```
   Wait — if we use `agents/quantifier/` as a directory, the parent `agents/mod.rs` declares `pub mod quantifier;` which loads `agents/quantifier/mod.rs`. We don't need a separate `quantifier.rs` file. The spec's target structure shows `quantifier.rs` as a single file, but a directory module is more practical.

   **Decision:** Use `agents/quantifier/` directory module. Update `agents/mod.rs` to `pub mod quantifier;`.

5. Implement `QuantifierAgent` in `src/narrative/agents/quantifier/agent.rs` (new file) or inline in `quantifier/mod.rs`:
   ```rust
   use crate::error::EngineError;
   use crate::model::agent::{AgentContext, AgentResult, BackendSelector, ExecutionPhase, StatePatch};
   use crate::narrative::agents::Agent;
   use crate::narrative::agents::quantifier::{
       QuantifierBackendTrait, QuantifierConfidence, QuantifierResult, determine_npcs_in_room,
   };
   use std::sync::Arc;

   #[derive(Debug)]
   pub struct QuantifierAgent {
       name: String,
       backend: Arc<dyn QuantifierBackendTrait>,
   }

   impl QuantifierAgent {
       pub fn from_config(
           config: &crate::model::agent::AgentConfig,
       ) -> Result<Self, EngineError> {
           let backend = match &config.backend {
               BackendSelector::UseMain => {
                   // Quantifier has no "main" backend; fall back to default quantifier backend
                   Arc::from(crate::narrative::agents::quantifier::get_quantifier_backend())
                       as Arc<dyn QuantifierBackendTrait>
               }
               BackendSelector::UseNamed(name) => {
                   // Future: resolve named backend from settings
                   // For now, always use default quantifier backend
                   log::warn!("Named quantifier backend '{name}' not yet supported; using default");
                   Arc::from(crate::narrative::agents::quantifier::get_quantifier_backend())
                       as Arc<dyn QuantifierBackendTrait>
               }
           };
           Ok(Self {
               name: config.name.clone(),
               backend,
           })
       }
   }

   impl Agent for QuantifierAgent {
       fn name(&self) -> &str {
           &self.name
       }
       fn phase(&self) -> ExecutionPhase {
           ExecutionPhase::PostGeneration
       }
       fn backend_selector(&self) -> BackendSelector {
           BackendSelector::UseNamed("quantifier".to_string())
       }
       fn execute(&self, ctx: &AgentContext) -> Result<AgentResult, EngineError> {
           let main_response = ctx
               .main_response
               .ok_or_else(|| EngineError::Config("Quantifier requires main_response".into()))?;

           let state = ctx.state;
           let room_npc_ids = crate::engine::logic::get_current_room(state)
               .map(|r| r.npcs.clone())
               .unwrap_or_default();
           let previous_room_npcs: Vec<_> = state.scene.npcs_in_area.clone();

           let result = determine_npcs_in_room(
               state,
               &room_npc_ids,
               &previous_room_npcs,
               main_response,
               self.backend.as_ref(),
           );

           let confidence = match result.npcs.confidence {
               crate::narrative::agents::quantifier::QuantifierConfidence::High => Confidence::High,
               crate::narrative::agents::quantifier::QuantifierConfidence::Medium => Confidence::Medium,
               crate::narrative::agents::quantifier::QuantifierConfidence::Low => Confidence::Low,
           };

           Ok(AgentResult::StatePatch(StatePatch::Scene {
               npc_ids: result.npcs.npc_ids,
               movement_destination: result.movement.destination,
               confidence,
           }))
       }
   }
   ```

6. Update `src/narrative/quantifier/mod.rs` to re-export from new location (temporary re-export for backward compatibility during migration), OR delete the directory entirely and update all `use` statements in the codebase.

   **Decision:** Delete `src/narrative/quantifier/` entirely. Update `use` statements in:
   - `src/engine/game_service.rs`
   - `src/engine/action_processing.rs`
   - Any other files referencing `crate::narrative::quantifier::...`

   Find all references with `grep -rn "narrative::quantifier" src/`.

7. Update quantifier test files:
   - Replace `use crate::narrative::quantifier::...` with `use crate::narrative::agents::quantifier::...`
   - Test logic and assertions stay unchanged.

**Files:**
- `src/narrative/agents/quantifier/` (new directory + files)
- `src/narrative/quantifier/` (deleted)
- `src/engine/game_service.rs`
- `src/engine/action_processing.rs`
- `src/engine/action_processing_tests.rs`
- `tests/game_service_tests.rs`
- `tests/diagnostic_benchmark.rs`
- `src/server/mod_tests.rs` (if it imports quantifier types)
- Any other files with `narrative::quantifier` imports

**Acceptance Criteria:**
- [ ] `src/narrative/quantifier/` directory no longer exists.
- [ ] All quantifier tests pass (`cargo test quantifier` or `cargo test` if test module paths are correct).
- [ ] `QuantifierAgent::execute` returns `AgentResult::StatePatch` with correct `npc_ids`, `movement_destination`, and `confidence`.
- [ ] Disabling quantifier in config means `game_service.rs` skips NPC detection (documented in code comment).
- [ ] `cargo test --features diagnostics` passes.

---

## Task 2.4: Integrate Agent into GameService

**Goal:** `DefaultGameService` calls the quantifier through `AgentRegistry` instead of directly.

### Context

Currently, `DefaultGameService` holds `quantifier_backend: Arc<dyn QuantifierBackendTrait>` and calls `determine_npcs_in_room` directly.

After this task, it holds `agent_registry: AgentRegistry`. It queries the registry for post-generation agents, runs the quantifier agent, receives `AgentResult::StatePatch`, translates the patch into `QuantifierResult`, and passes that to `execute_freeaction_impl`.

This is a bridge: in Phase 3, the pipeline will replace this manual orchestration with `AgentPipeline`.

### Steps

1. Update `DefaultGameService`:
   ```rust
   pub struct DefaultGameService {
       llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
       agent_registry: crate::narrative::agents::registry::AgentRegistry,
   }
   ```

2. Update constructors:
   ```rust
   impl DefaultGameService {
       pub fn new() -> Self {
           let settings = crate::settings::load_settings().unwrap_or_default();
           let registry = crate::narrative::agents::registry::AgentRegistry::from_configs(&settings.agents)
               .unwrap_or_default();
           Self {
               llm_backend: Arc::from(crate::narrative::llm::get_llm_backend()),
               agent_registry: registry,
           }
       }

       pub fn with_backends(
           llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
           agent_registry: crate::narrative::agents::registry::AgentRegistry,
       ) -> Self {
           Self {
               llm_backend,
               agent_registry,
           }
       }

       /// Convenience constructor for tests that only need a mock quantifier.
       pub fn with_mock_quantifier(
           llm_backend: Arc<dyn crate::narrative::llm::LlmBackend>,
           quantifier_backend: Arc<dyn crate::narrative::agents::quantifier::QuantifierBackendTrait>,
       ) -> Self {
           let agent = crate::narrative::agents::quantifier::QuantifierAgent::with_backend(
               "quantifier".to_string(),
               quantifier_backend,
           );
           let registry = crate::narrative::agents::registry::AgentRegistry::with_agent(
               Box::new(agent),
           );
           Self {
               llm_backend,
               agent_registry: registry,
           }
       }
   }
   ```

3. In `execute_action` for `Action::FreeAction`, replace the direct quantifier call with agent registry iteration:
   ```rust
   // After narration generation...
   let mut quantifier_result = QuantifierResult {
       npcs: QuantifierParseResult {
           npc_ids: room.npcs.clone(),
           confidence: QuantifierConfidence::Low,
       },
       movement: MovementParseResult {
           movement_type: None,
           destination: None,
           confidence: QuantifierConfidence::Low,
       },
   };

   let agent_ctx = crate::model::agent::AgentContext {
       state: &state,
       main_response: Some(&narration_text),
       player_input: &text,
   };

   for agent in self.agent_registry.agents_for_phase(ExecutionPhase::PostGeneration) {
       match agent.execute(&agent_ctx) {
           Ok(crate::model::agent::AgentResult::StatePatch(patch)) => {
               match patch {
                   crate::model::agent::StatePatch::Scene { npc_ids, movement_destination, confidence } => {
                       quantifier_result.npcs.npc_ids = npc_ids;
                       quantifier_result.movement.destination = movement_destination;
                       quantifier_result.npcs.confidence = match confidence {
                           crate::model::agent::Confidence::High => QuantifierConfidence::High,
                           crate::model::agent::Confidence::Medium => QuantifierConfidence::Medium,
                           crate::model::agent::Confidence::Low => QuantifierConfidence::Low,
                       };
                   }
               }
           }
           Ok(crate::model::agent::AgentResult::NoOp) => {}
           Ok(crate::model::agent::AgentResult::PromptDirective(_)) => {
               log::warn!("Post-generation agent returned PromptDirective; ignoring");
           }
           Err(e) => {
               log::warn!("Agent {} failed: {e}", agent.name());
           }
       }
   }
   ```

   Note: If no post-generation agents are registered, `quantifier_result` defaults to room static NPCs (fallback), preserving baseline behaviour.

4. Update `GameServiceContext` if needed (no changes required for Phase 2; `agent_registry` lives on `DefaultGameService`, not the context).

5. Update test helpers and call sites:
   - Replace `DefaultGameService::with_backends(llm, quantifier)` calls in tests with `DefaultGameService::with_mock_quantifier(llm, quantifier)`.
   - Update `tests/game_service_tests.rs` imports:
     ```rust
     use chronicler_engine::narrative::agents::quantifier::{
         MockQuantifierBackend, MovementParseResult, MovementType, QuantifierConfidence,
     };
     ```
   - Update `tests/diagnostic_benchmark.rs` imports similarly.
   - Update `src/engine/action_processing_tests.rs` imports:
     ```rust
     use crate::narrative::agents::quantifier::{
         MovementParseResult, MovementType, QuantifierConfidence, QuantifierParseResult, QuantifierResult,
     };
     ```
   - `DefaultGameService::new()` calls in tests remain unchanged (they load real settings; acceptable for integration tests that don't mock the backend).

6. Remove `QuantifierBackendTrait` from `game_service.rs` imports and any direct backend references.

**Files:**
- `src/engine/game_service.rs`
- Test files that construct `DefaultGameService`

**Acceptance Criteria:**
- [ ] Quantifier produces identical `NpcEventList` to old code (byte-level or struct-level equality test).
- [ ] With quantifier disabled in registry, `execute_freeaction_impl` receives fallback static NPCs.
- [ ] All integration tests pass.
- [ ] `game_service.rs` contains no direct `determine_npcs_in_room` call.
- [ ] No `QuantifierBackendTrait` references remain in `game_service.rs`.

---

## Task 2.5: Per-Agent Backend Selection

**Goal:** Each agent can resolve its own backend; quantifier defaults to existing quantifier backend config.

### Context

The spec defines `BackendSelector::UseMain | UseNamed(String)`. `UseMain` means "use whatever the main LLM backend is" (for future pre-generation agents). `UseNamed` looks up a backend by name in settings.

For Phase 2, only the quantifier needs backend resolution, and it already has `get_quantifier_backend()` which reads settings. The extension point is: if settings define multiple quantifier connections, `UseNamed("my-quantifier")` should resolve to that connection.

### Steps

1. Add backend resolution to `AgentRegistry::from_configs`:
   ```rust
   fn resolve_backend(
       selector: &BackendSelector,
       settings: &AppSettings,
   ) -> Result<Arc<dyn QuantifierBackendTrait>, EngineError> {
       match selector {
           BackendSelector::UseMain => {
               // No main quantifier backend; use default
               Ok(Arc::from(get_quantifier_backend()))
           }
           BackendSelector::UseNamed(name) => {
               if let Some(conn) = settings.connections.iter().find(|c| c.name == *name) {
                   Ok(Arc::from(get_quantifier_backend_for(conn)))
               } else {
                   log::warn!("Backend '{name}' not found; falling back to default quantifier backend");
                   Ok(Arc::from(get_quantifier_backend()))
               }
           }
       }
   }
   ```

2. Update `QuantifierAgent::from_config` to accept a pre-resolved backend instead of calling `get_quantifier_backend()` internally:
   ```rust
   pub fn with_backend(
       name: String,
       backend: Arc<dyn QuantifierBackendTrait>,
   ) -> Self {
       Self { name, backend }
   }
   ```

   Then `AgentRegistry::from_configs` resolves the backend first, then passes it to `QuantifierAgent::with_backend`.

3. Ensure `get_quantifier_backend_for` is pub and accessible from `registry.rs`.

**Files:**
- `src/narrative/agents/registry.rs`
- `src/narrative/agents/quantifier/agent.rs` (or wherever `QuantifierAgent` lives)
- `src/narrative/agents/quantifier/backends.rs` (export visibility)

**Acceptance Criteria:**
- [ ] Quantifier can use a different backend from narrator (verified by test with two mock backends).
- [ ] Unknown backend name logs a warning and falls back to default.
- [ ] `BackendSelector::UseMain` for quantifier falls back to default quantifier backend.

---

## Dependencies

| Task | Depends On | Blocks |
|------|-----------|--------|
| 2.1 Agent Core Types | — | 2.2, 2.3 |
| 2.2 Agent Registry | 2.1 | 2.3, 2.4 |
| 2.3 Migrate Quantifier | 2.1, 2.2 | 2.4 |
| 2.4 Integrate into GameService | 2.2, 2.3 | — |
| 2.5 Per-Agent Backend | 2.2, 2.3 | — |

**Parallelisable:** Task 2.1 can be done in isolation. Tasks 2.2 and 2.3 can be developed in parallel once 2.1 is done (2.2 uses the trait, 2.3 implements it). Tasks 2.4 and 2.5 are sequential integration steps.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Moving quantifier tests breaks import paths | High | Low | Move files as a unit; bulk-replace `use crate::narrative::quantifier` → `use crate::narrative::agents::quantifier` |
| `StatePatch` design doesn't generalise to future agents | Medium | Medium | Keep enum open; only add variants when a concrete agent needs them. Document that `StatePatch` will evolve. |
| `GameService` tests rely on direct `MockQuantifierBackend` | Medium | Medium | Create helper `registry_with_mock_quantifier(backend)` in test support; update call sites |
| `AgentContext` lifetime causes borrow-checker issues | Low | Medium | Use `&'a GameState` (owned by caller); clone only what the agent needs internally |
| Backend resolution adds startup complexity | Low | Low | Fail-fast: unknown agent type or missing backend logs warning and uses default |

---

## Verification

### Automated Tests

```bash
cd chronicler_engine
python build.py
```

| Behaviour | Test Level | How |
|-----------|-----------|-----|
| Mock agent compiles and executes | Unit | Define `MockAgent` in test module, call `execute`, assert `NoOp` |
| `AgentConfig` serde round-trip | Unit | Serialize to JSON/TOML, deserialize, assert equality |
| Registry loads built-ins only | Unit | `AgentRegistry::from_configs(&[])` → assert contains no agents OR `from_configs` with defaults |
| Registry rejects unknown type | Unit | Pass config with `agent_type = "bogus"` → assert `Err` |
| Registry filters by phase | Unit | Add narrator + quantifier, assert `agents_for_phase(Pre).count() == 1` and `agents_for_phase(Post).count() == 1` |
| Quantifier agent returns `StatePatch` | Unit | Construct `QuantifierAgent` with `MockQuantifierBackend`, call `execute` with `AgentContext`, assert `StatePatch::Scene` |
| Quantifier agent output matches old `determine_npcs_in_room` | Integration | Same input → old function and new agent produce identical `npc_ids` and `movement_destination` |
| GameService uses registry | Integration | Mock backend in registry → assert `execute_action` calls it |
| Fallback when quantifier disabled | Integration | Empty registry → assert `execute_freeaction_impl` receives static room NPCs |
| All existing quantifier unit tests | Unit | `cargo test quantifier` (parser, core, backends, prompt) |

### Spot-Checks

```bash
# Verify no references to old quantifier module path
grep -rn "narrative::quantifier" src/
# Expected: no matches (except possibly in docs/plans/)

# Verify no direct determine_npcs_in_room in game_service
grep -n "determine_npcs_in_room" src/engine/game_service.rs
# Expected: no matches

# Verify QuantifierBackendTrait not imported in game_service
grep -n "QuantifierBackendTrait" src/engine/game_service.rs
# Expected: no matches
```

---

## Success Criteria

1. `Agent` trait exists with `name`, `phase`, `backend_selector`, `execute` methods.
2. `AgentRegistry` loads from config, returns agents by phase, fails fast on unknown types.
3. `QuantifierAgent` implements `Agent` and returns `AgentResult::StatePatch`.
4. `DefaultGameService` calls quantifier through `AgentRegistry`, not directly.
5. With quantifier enabled, output matches pre-migration (identical `NpcEventList`).
6. With quantifier disabled, fallback static NPCs are used.
7. `python build.py` passes (fmt + clippy + guardrails + tests).
8. Test-to-code ratio remains ≥ 1.5.

---

## Decisions

- **`StatePatch` carries generic `Confidence`.** A new `Confidence` enum lives in `model/agent.rs` and is included in `StatePatch::Scene`. The quantifier module keeps its existing `QuantifierConfidence` type; `QuantifierAgent::execute` maps between them. This avoids a disruptive rename across the quantifier module while still letting the agent abstraction carry confidence data.
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(rename_all = "snake_case")]
  pub enum Confidence {
      High,
      Medium,
      Low,
  }
  ```

- **`AgentConfig` lives in existing `AppSettings`.** Add `pub agents: Vec<AgentConfig>` to `src/model/settings.rs` under the existing `AppSettings` struct. Use `#[serde(default)]` so existing `settings.toml` files without an `agents` key continue to work. No separate `agents.toml` file.

---

## Next Phase

Phase 3 (Phase-Based Pipeline) will:
- Introduce `AgentPipeline` with `pre_generate()` and `post_generate()`.
- Replace the manual agent loop in `game_service.rs` with pipeline orchestration.
- Add `AgentResultDispatcher` for generic `StatePatch` application.
- Delete `QuantifierBackendTrait` (quantifier is now just an `Agent`).

---

## Alignment with Overarching Spec

The following deviations from `multi-agent-architecture-overarching-spec.md` were identified during review. These are practical adjustments, not scope changes.

| Spec Item | Plan Deviation | Rationale |
|-----------|---------------|-----------|
| `quantifier.rs` single file | `quantifier/` directory module | Quantifier has 11 files (~1000+ lines). Consolidating into one file breaks test organisation and creates an unmaintainable 800+ line file. |
| `AgentContext` undefined | Defined with `state`, `main_response`, `player_input` | Needed for `QuantifierAgent::execute` to reproduce current `determine_npcs_in_room` behaviour. |
| `StatePatch` undefined | Defined as `StatePatch::Scene { npc_ids, movement_destination, confidence }` | Captures everything the quantifier currently produces. Open enum for future agents. |
| `agent_results` added to `TurnResult` in Phase 2 | Deferred to Phase 3 | Phase 2 uses a temporary bridge in `game_service.rs` (`StatePatch` → `QuantifierResult`). `TurnResult` is unchanged. `agent_results` belongs with the pipeline dispatcher in Phase 3. |
| Task 2.4 files: `game_service.rs`, `llm/mod.rs` | Plan uses `registry.rs`, `quantifier/backends.rs` | Backend resolution lives in the registry/quantifier, not the LLM module. `game_service.rs` is touched in Task 2.4 for integration, not backend resolution. |
| `NarratorAgent` as pipeline agent | `NarratorAgent` is a registry stub only | The narrator runs at generation time, not in a pre/post phase. It returns `NoOp` and is not executed by the pipeline. Future work may redesign this. |
| `BackendSelector::UseMain` for quantifier | Falls back to default quantifier backend | `UseMain` implies the main `LlmBackend`, but the quantifier needs `QuantifierBackendTrait`. These are incompatible traits. A future phase may unify backend traits. |

### Recommended Spec Updates

1. **Target structure diagram**: Change `quantifier.rs` → `quantifier/` directory.
2. **Agent trait code block**: Add `AgentContext`, `StatePatch`, `Confidence`, and `BackendSelector` definitions.
3. **TurnResult note**: Change "added in Phase 2" → "added in Phase 3".
4. **Task 2.3 files**: Change `quantifier.rs` → `quantifier/`.
5. **Task 2.4 files**: Remove `narrative/llm/mod.rs`; add `agents/registry.rs` and `agents/quantifier/backends.rs`.
6. **Add explicit note**: Phase 2 includes a temporary `StatePatch` → `QuantifierResult` bridge in `game_service.rs` because `action_processing.rs` is not refactored until Phase 3.

→ **Approve this plan and I'll start implementing Task 2.1.**
