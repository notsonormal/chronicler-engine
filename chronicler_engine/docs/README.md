# Chronicler Engine Documentation

This folder contains all documentation for the Chronicler Engine project.

## Folder Structure

<!-- AUTO-INDEX START -->
*Index last generated: 2026-07-02 22:00 UTC*

### Root files

- [Changelog](./CHANGELOG.md)
- [Chronicler Engine: Project Roadmap](./ROADMAP.md)

### `docs/adr/`

- [ADR-001: HTMX Web Dashboard Architecture](./adr/adr-001-htmx-web-dashboard.md)
- [ADR-002: HTTP Polling for Real-Time Updates](./adr/adr-002-http-polling.md)
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
- [ADR-019: One Table Per Storage Module](./adr/adr-019-one-table-per-storage-module.md)
- [ADR-020: Unified Storage Struct](./adr/adr-020-storage-consolidation.md)
- [ADR-021: State Patch Reducer for Post-Generation Agent Composition](./adr/adr-021-state-patch-reducer.md)
- [ADR-022: PromptAssembler Trait Decoupling](./adr/adr-022-prompt-assembler.md)
- [ADR-023: Immediate Message Persistence](./adr/adr-023-immediate-message-persistence.md)
- [ADR-024: Migrate Game Data to SQLite with Seed Pattern](./adr/adr-024-game-data-migration-to-sqlite.md)
- [ADR-025: Multi-World Data Foundation](./adr/adr-025-multi-world-data-foundation.md)
- [ADR-026: Relocate Persona Binding from World to Game](./adr/adr-026-persona-relocation-to-game.md)
- [ADR-027: Hexagonal Architecture Migration](./adr/adr-027-hexagonal-architecture-migration.md)

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

- [Plan: Abstraction Anti-Pattern Prevention via Advisory Healthcheck](./plans/abstraction-antipattern-healthcheck-plan.md)
- [Super-Plan: Abstraction-Fixes Follow-Up](./plans/abstraction-fixes-followup-superplan.md)
- [Plan: Diagnostic Decision Tree as Agent Infrastructure](./plans/diagnostic-decision-tree-plan.md)
- [Deferred arch-lint Rules — Hexagonal Reorganization](./plans/hexagonal-deferred-arch-lint-rules.md)
- [Plan: Hexagonal Architecture Reorganization](./plans/hexagonal-reorganization-plan.md)
- [Plan: Mapless Worlds via Freeform Location Names](./plans/mapless-worlds-plan.md)
- [Spec: Agent-Ready Pipeline Restructure for Chronicler Engine](./plans/multi-agent-architecture-overarching-spec.md)
- [Plan: Observability & Automated Forensics](./plans/observability-and-forensics-plan.md)
- [Plan: Phase 2 Test Quality Cleanup + Coverage Gaps](./plans/archived/phase2-test-quality-and-coverage-gaps.md)
- [Plan: Phase 2 Tests + Coverage Fixes](./plans/archived/phase2-tests-coverage-fixes.md)
- [Plan: Reliability and Cancellation](./plans/reliability-and-cancellation-plan.md)
- [AI Steering & Guided Generation](./plans/steering-and-guided-generation.md)
- [Subplan B: Quantifier `destination` field split](./plans/subplan-b-quantifier-field-split.md)
- [Subplan C: Atomic mapless enablement](./plans/subplan-c-mapless-enablement.md)
- [T1: Error Model Unification](./plans/t1-error-model-unification.md)
- [T10: Low-priority Cleanup Bundle](./plans/t10-low-priority-cleanup-bundle.md)
- [T2-ARCH: Narration Deepening](./plans/t2-arch-narration-deepening.md)
- [T5: Type Collapses (A3 + A6)](./plans/t5-type-collapses.md)
- [T6: MessageHistory Encapsulation](./plans/t6-messagehistory-encapsulation.md)
- [T9: Doc / Migration Debt](./plans/t9-doc-and-migration-debt.md)

### `docs/plans/archived/`

- [Implementation Plan: Abstraction Anti-Pattern Fixes (Corrected)](./plans/archived/abstraction-fixes-implementation-plan.md)
- [Plan: Abstraction Anti-Pattern Fixes (Tiered)](./plans/archived/abstraction-fixes-plan.md)
- [ADR-026 Follow-up: Thermo-Nuclear Review Quality Fixes](./plans/archived/adr-026-followup-quality-fixes.md)
- [Plan: Antipattern-Checker Agent Skill](./plans/archived/antipattern-checker-skill-plan.md)
- [Fix Boot Path: Restore Auto-Create Game with `--persona` CLI Flag](./plans/archived/fix-boot-and-default-game.md)
- [Plan: Phase 2 Thermonuclear Review Fixes](./plans/archived/phase2-thermonuclear-review-fixes.md)
- [Pipeline Decomposition Review Fixes (Round 3)](./plans/archived/pipeline-review-fixes-round3.md)
- [Review Fixes — Pipeline Decomposition Quality](./plans/archived/review-fixes-pipeline-quality.md)
- [Subplan A: Relocate `starting_room_id` to `StartingScenario`](./plans/archived/subplan-a-relocate-starting-room.md)
- [T10 Execution Plan: Safe Cleanup Items](./plans/archived/t10-execution-safe-cleanup-items.md)
- [T3: Service Layer Cleanup](./plans/archived/t3-service-layer-cleanup.md)
- [T4: MockBackend Modernization](./plans/archived/t4-mockbackend-modernization.md)
- [T7 Sub-Plan (Archived): Split `Backend` enum into `Backend` + `LayeredBackend`](./plans/archived/t7-storage-backend-layered-split.md)
- [Plan: Test-Police Audit Fixes — Hints Removal + Cancel Test Alignment](./plans/archived/test-police-cancel-and-hints-removal.md)

### `docs/reference/`

- [Data Layer Reference](./reference/data_layer.md)
- [Specification: Engine Data Schemas](./reference/data_schemas.md)
- [Specification: Player Persona System](./reference/persona_system.md)
- [Reference: Quantifier Prompt](./reference/quantifier_prompt.md)
- [Reference: System Prompt](./reference/system_prompt.md)
- [Specification: Testing Strategy and Architecture](./reference/testing.md)

### `docs/reviews/`

- [Chronicler Engine — Abstraction Anti-Pattern Investigation](./reviews/abstraction-antipatterns-summary.md)
- [Architectural Review: AI Agent Comprehension Challenges](./reviews/agent-comprehension-review.md)
- [Documentation Consistency Report](./reviews/docs-consistency-report.md)
- [Zone A: src/model/ — Abstraction Anti-Pattern Report](./reviews/zone-a-model.md)
- [Zone B: application/bootstrap/engine — Abstraction Anti-Pattern Report](./reviews/zone-b-app-engine.md)

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
- [Storage System](./system/storage.md)
- [Specification: Text Check System](./system/text_check.md)
- [System: Auto-Trigger & Reactive Encounters](./system/triggers.md)
- [Specification: UI Design](./system/ui_design.md)
- [Worlds Management System](./system/worlds.md)

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
