# Chronicler Engine Documentation

This folder contains all documentation for the Chronicler Engine project.

For general engine principles, workflow, and conventions, see [`../AGENTS.md`](../AGENTS.md).

## Keeping Documentation Clean

**Plan authoring convention:** reference doc issues by quotable phrase, never line numbers — line numbers rot. Use the exact sentence (or a quoted fragment of it) as the anchor.

**Sediment:** Layers of old content that settle in the docs and are never cleared, because adding feels safe and removing feels risky — so stale and irrelevant lines accumulate and you must core down through them to find what is still live. The default fate of any documentation without a pruning discipline; the slow erosion of relevance, as opposed to duplication's repeated meaning.

**Duplication:** The same meaning given more than one single source of truth. It costs maintenance (change one place, you must change the others), costs tokens, and inflates prominence — repeating a meaning weights it on the ladder past its real rank. The accidental inverse of a leading word, which raises attention on purpose by repeating a token, never the meaning.

## Folder Structure

<!-- AUTO-INDEX START -->
*Index last generated: 2026-07-05 01:28 UTC*

### Root files

- [Changelog](./CHANGELOG.md)
- [Chronicler Engine: Project Roadmap](./ROADMAP.md)

### `docs/adr/`

- [ADR-NNN: Title (imperative or declarative, e.g. "Use SQLite for Game State")](./adr/adr-000-template.md)
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
- [ADR-014: Action Pipeline Architecture](./adr/adr-014-action-pipeline.md)
- [ADR-015: Prompt Presets System](./adr/adr-015-prompt-presets.md)
- [ADR-016: Multi-Game Support](./adr/adr-016-multi-game-support.md)
- [ADR-017: Message Swipes](./adr/adr-017-message-swipes.md)
- [ADR-020: Unified Storage Struct](./adr/adr-020-storage-consolidation.md)
- [ADR-022: PromptAssembler Trait Decoupling](./adr/adr-022-prompt-assembler.md)
- [ADR-024: Migrate Game Data to SQLite with Seed Pattern](./adr/adr-024-game-data-migration-to-sqlite.md)
- [ADR-025: Multi-World Data Foundation](./adr/adr-025-multi-world-data-foundation.md)
- [ADR-026: Relocate Persona Binding from World to Game](./adr/adr-026-persona-relocation-to-game.md)
- [ADR-027: Hexagonal Architecture Migration](./adr/adr-027-hexagonal-architecture-migration.md)
- [ADR-028: Test Module Header Convention](./adr/adr-028-test-module-header-convention.md)
- [ADR Standards](./adr/README.md)

### `docs/architecture/`

- [Architecture Guardrails](./architecture/guardrails.md)
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
- [Plan: Documentation Hygiene Skill](./plans/docs-hygiene-skill-plan.md)
- [Deferred arch-lint Rules — Hexagonal Reorganization](./plans/hexagonal-deferred-arch-lint-rules.md)
- [Plan: Mapless Worlds via Freeform Location Names](./plans/mapless-worlds-plan.md)
- [Spec: Agent-Ready Pipeline Restructure for Chronicler Engine](./plans/multi-agent-architecture-overarching-spec.md)
- [Plan: Observability & Automated Forensics](./plans/observability-and-forensics-plan.md)
- [Plan: Reliability and Cancellation](./plans/reliability-and-cancellation-plan.md)
- [AI Steering & Guided Generation](./plans/steering-and-guided-generation.md)
- [Plan: Pi Subagent Guardrails Extension](./plans/subagent-guardrails-extension-plan.md)
- [Subplan B: Quantifier `destination` field split](./plans/subplan-b-quantifier-field-split.md)
- [Subplan C: Atomic mapless enablement](./plans/subplan-c-mapless-enablement.md)
- [T1: Error Model Unification](./plans/t1-error-model-unification.md)
- [T10: Low-priority Cleanup Bundle](./plans/t10-low-priority-cleanup-bundle.md)
- [T2-ARCH: Narration Deepening](./plans/t2-arch-narration-deepening.md)
- [T5: Type Collapses (A3 + A6)](./plans/t5-type-collapses.md)
- [T6: MessageHistory Encapsulation](./plans/t6-messagehistory-encapsulation.md)
- [T9: Doc / Migration Debt](./plans/t9-doc-and-migration-debt.md)

### `docs/reference/`

- [Data Layer Reference](./reference/data_layer.md)
- [Specification: Engine Data Schemas](./reference/data_schemas.md)
- [Specification: Player Persona System](./reference/persona_system.md)
- [Reference: Quantifier Prompt](./reference/quantifier_prompt.md)
- [Reference: System Prompt](./reference/system_prompt.md)
- [Test Support Reference](./reference/test_support.md)
- [Testing Policy](./reference/testing.md)

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
