# Chronicler Engine Documentation

This folder contains all documentation for the Chronicler Engine project.

## Folder Structure

<!-- AUTO-INDEX START -->
*Index last generated: 2026-05-31 13:22 UTC*

### Root files

- [Changelog](./CHANGELOG.md)
- [Chronicler Engine: Project Roadmap](./ROADMAP.md)

### `docs/adr/`

- [ADR-001: HTMX Web Dashboard Architecture](./adr/adr-001-htmx-web-dashboard.md)
- [ADR-002: HTTP Polling for Real-Time Updates](./adr/adr-002-sse-realtime-updates.md)
- [ADR-003: Askama Template Engine for HTML Rendering](./adr/adr-003-askama-templates.md)
- [ADR-004: XML-Structured LLM Prompts](./adr/adr-004-xml-prompt-format.md)
- [ADR-005: SillyTavern-Style Layered Prompt System](./adr/adr-005-layered-prompts.md)
- [ADR-006: Quantifier-Driven Game Systems](./adr/adr-006-quantifier-systems.md)
- [ADR-007: Settings System Architecture](./adr/adr-007-settings-system.md)
- [ADR-008: SQLite Snapshot Persistence](./adr/adr-008-sqlite-snapshot-persistence.md)
- [ADR-009: Agent Trait and Registry Architecture](./adr/adr-009-agent-trait-registry.md)
- [ADR-010: Concurrency and Generation Gate Model](./adr/adr-010-concurrency-generation-gate.md)
- [ADR-011: Text Check Integration](./adr/adr-011-text-check-integration.md)
- [ADR-012: LLM Call Logging and Forensics](./adr/adr-012-llm-message-logging.md)
- [ADR-013: Message Domain Model](./adr/adr-013-message-domain-model.md)
- [ADR-014: Action Pipeline Architecture](./adr/adr-014-action-pipeline.md)
- [ADR-015: Prompt Presets System](./adr/adr-015-prompt-presets.md)
- [ADR-016: Multi-Game Support](./adr/adr-016-multi-game-support.md)
- [ADR-017: Message Swipes](./adr/adr-017-message-swipes.md)
- [ADR-018: Application Service Layer](./adr/adr-018-application-service.md)
- [ADR-019: One Table Per Storage Module](./adr/adr-019-one-table-per-storage-module.md)
- [ADR-020: Unified Storage Struct](./adr/adr-020-storage-consolidation.md)
- [ADR-021: State Patch Reducer for Post-Generation Agent Composition](./adr/adr-021-state-patch-reducer.md)
- [ADR-022: PromptAssembler Trait Decoupling](./adr/adr-022-prompt-assembler.md)
- [ADR-023: Immediate Message Persistence](./adr/adr-023-immediate-message-persistence.md)

### `docs/architecture/`

- [Architecture Guardrails](./architecture/guardrails.md)
- [Runtime Invariants](./architecture/invariants.md)
- [Specification: Core Architecture (Modular)](./architecture/system.md)

### `docs/diagnostics/`

- [Debugging Guide](./diagnostics/DEBUGGING.md)
- [Error Catalog](./diagnostics/error_catalog.md)

### `docs/external_applications/`

- [Marinara-Engine Reference](./external_applications/marinara_engine.md)
- [Marinara Engine — Default System Prompt](./external_applications/marinara_engine_system_prompt.md)
- [SillyTavern Chat Window Reference](./external_applications/sillytavern_chat_window.md)
- [SillyTavern Prompt System Reference](./external_applications/sillytavern_prompt_system.md)

### `docs/plans/`

- [Plan: Diagnostic Decision Tree as Agent Infrastructure](./plans/diagnostic-decision-tree-plan.md)
- [LLM Infrastructure Improvements](./plans/llm-infrastructure-improvements.md)
- [Spec: Agent-Ready Pipeline Restructure for Chronicler Engine](./plans/multi-agent-architecture-overarching-spec.md)
- [Plan: Observability & Automated Forensics](./plans/observability-and-forensics-plan.md)
- [AI Steering & Guided Generation](./plans/steering-and-guided-generation.md)
- [Plan: Trigger Identity: Index → UUID](./plans/trigger-identity-uuid-plan.md)

### `docs/plans/archived/`

- [Refactor handle_movement: Split Mixed Responsibilities](./plans/archived/handle-movement-refactor-2026-05-31.md)
- [Plan: Observability & Automated Forensics](./plans/archived/observability-and-forensics-plan-2026-05-31.md)
- [Remove Identity Wrapper Functions from GameService](./plans/archived/remove-identity-wrapper-functions-2026-05-31.md)

### `docs/reference/`

- [Data Layer Reference](./reference/data_layer.md)
- [Specification: Engine Data Schemas](./reference/data_schemas.md)
- [Specification: Player Persona System](./reference/persona_system.md)
- [Reference: Quantifier Prompt](./reference/quantifier_prompt.md)
- [Reference: System Prompt](./reference/system_prompt.md)
- [Specification: Testing Strategy and Architecture](./reference/testing.md)

### `docs/reviews/`

- [Architectural Review: AI Agent Comprehension Challenges](./reviews/agent-comprehension-review.md)

### `docs/reviews/archived/`

- [Agent Scalability Assessment: Chronicler vs. Marinara](./reviews/archived/agent-scalability-assessment.md)
- [Cross-Project Architectural Comparison: Chronicler Engine vs. Marinara Engine](./reviews/archived/cross-project-architectural-comparison.md)
- [Architectural Review: Defensive Architecture & Invariant Enforcement](./reviews/archived/defensive-architecture-review.md)
- [Holistic Architectural Review: Chronicler Engine](./reviews/archived/holistic-architectural-review.md)
- [Phase 1: Domain Alignment — Findings](./reviews/archived/holistic-review-phase1-domain-alignment.md)
- [Phase 2: Structural Forces — Findings](./reviews/archived/holistic-review-phase2-structural-forces.md)
- [Phase 3: Evolution Stress Test — Findings](./reviews/archived/holistic-review-phase3-evolution-stress.md)
- [Phase 4: Health Metrics — Baseline](./reviews/archived/holistic-review-phase4-health-metrics.md)

### `docs/system/`

- [Agent System](./system/agent_system.md)
- [System: Character State & Persistence](./system/character_state.md)
- [Specification: Dashboard UI](./system/dashboard.md)
- [System: Dynamic Pseudo-Rooms](./system/dynamic_rooms.md)
- [Specification: Game Flow](./system/game_flow.md)
- [Specification: LLM Processing & Integration](./system/llm_processing.md)
- [Specification: Game Master Narration System](./system/narration_engine.md)
- [Specification: Semantic Navigation](./system/navigation.md)
- [Chronicler Engine Prompt System](./system/prompt_system.md)
- [Engine Startup & Initialization](./system/startup.md)
- [Specification: Text Check System](./system/text_check.md)
- [System: Auto-Trigger & Reactive Encounters](./system/triggers.md)
- [Specification: UI Design](./system/ui_design.md)

<!-- AUTO-INDEX END -->

---

## Key Principles

1. **Architecture is the single source of truth** - Any system-level change should be reflected in `architecture/system.md`
2. **Plans update the system first** - Before implementing, update the architecture document
3. **Domain docs explain "why"** - System docs explain subsystems, not every implementation detail
4. **Reference docs are stable** - Data schemas and APIs don't change often

---

## Workflow

When adding a new feature:

1. **Create a plan** in `docs/plans/` (or update existing)
2. **Update architecture** - Modify `docs/architecture/system.md` to reflect changes
3. **Update all the other docs as needed** - Read `docs/*`
4. **Implement** - Write the code
5. **Validate** - Run the full build and test suite:
   ```bash
   python build.py  # Or manually: cargo fmt && cargo clippy && cargo nextest run
   ```
6. **Archive** - Move completed plans to `plans/archived/`

---

## Quick Reference

| Question | Document |
|----------|----------|
| What modules/tiers exist? | [`architecture/system.md`](./architecture/system.md) |
| How do I start the engine? | [`system/startup.md`](./system/startup.md) |
| How does movement/navigation work? | [`system/navigation.md`](./system/navigation.md) |
| How are LLM prompts built? | [`system/prompt_system.md`](./system/prompt_system.md) |
| How does the dashboard UI work? | [`system/dashboard.md`](./system/dashboard.md) |
| How do NPC triggers/encounters work? | [`system/triggers.md`](./system/triggers.md) |
| How does game state persist (snapshots)? | [`system/game_flow.md`](./system/game_flow.md) + [ADR-008](./adr/adr-008-sqlite-snapshot-persistence.md) |
| How do I configure an LLM connection? | [`system/llm_processing.md`](./system/llm_processing.md) |
| What `data/` JSON schemas are used? | [`reference/data_schemas.md`](./reference/data_schemas.md) |
| How do I run tests? | [`reference/testing.md`](./reference/testing.md) |
| Why was X designed this way? | [`docs/adr/`](./adr/) |
| What's the current roadmap? | [`ROADMAP.md`](./ROADMAP.md) |
