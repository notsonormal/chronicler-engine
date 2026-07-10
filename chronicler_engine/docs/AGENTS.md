# Chronicler Engine Documentation

This folder contains all documentation for the Chronicler Engine project.

For general engine principles, workflow, and conventions, see [`../AGENTS.md`](../AGENTS.md).

## Keeping Documentation Clean

**Plan authoring convention:** reference doc issues by quotable phrase, never line numbers — line numbers rot. Use the exact sentence (or a quoted fragment of it) as the anchor.

### The Per-Edit Gate (for doc edits)

Docs in this repo are a **Specification**, not a conversation. They state contracts — what the system guarantees — not implementations. Code references in prose are sediment unless they pass the non-removable test.

Spec-Driven Implementation (SDI) means the code reflects the spec. It does **not** mean restating code in the docs. Symbols map 1-to-1 to concepts (SDI principle), so naming the concept in prose IS naming the symbol — no need to also quote the function/type/file.

Before committing any edit to a `docs/system/*.md`, `docs/reference/*.md`, `docs/architecture/*.md`, or `docs/diagnostics/*.md` file, apply this test to every code reference (type, function, file, module path, line number) in your added or changed prose:

> If this reference is removed, does the reader lose contract information?

- **Yes** — keep it. State the contract alongside; never let a reference stand alone as the substance.
- **No** — remove it. The code is the verification. The doc verifies the contract, not the implementation.

XML/domain markups (e.g. `<ConversationHistory>`, `<PlayerInput>`) are domain tags, not code references — they don't trigger this test.

Numerical anchor budgets ("max N per section") don't work — the writer is also the counter. Use the non-removable test instead.

Accumulated violations in existing docs: invoke the [`chronicler-docs-hygiene`](../../.agents/skills/chronicler-docs-hygiene/SKILL.md) skill.

## Folder Structure

<!-- AUTO-INDEX START -->
*Index last generated: 2026-07-10 14:41 UTC*

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
- [ADR-030: is_generating Dual-Source Invariant — AtomicBool Is Cached View of Persisted Status](./adr/adr-030-is-generating-invariant.md)
- [ADR-031: OpContext Absorption Trade-offs](./adr/adr-031-opcontext-absorption-tradeoffs.md)
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
- [Plan: Mapless Worlds via Freeform Location Names](./plans/mapless-worlds-plan.md)
- [Plan: Reliability and Cancellation](./plans/reliability-and-cancellation-plan.md)
- [Super-Plan: `simpler-hexagon` Pre-Merge Cleanup](./plans/simpler-hexagon-pre-merge-superplan.md)
- [AI Steering & Guided Generation](./plans/steering-and-guided-generation.md)
- [Subplan B: Quantifier `destination` field split](./plans/subplan-b-quantifier-field-split.md)
- [Subplan C: Atomic mapless enablement](./plans/subplan-c-mapless-enablement.md)
- [T1: Error Model Unification](./plans/t1-error-model-unification.md)
- [T10: Low-priority Cleanup Bundle](./plans/t10-low-priority-cleanup-bundle.md)
- [T11: Documentation Hygiene Skill Hardening](./plans/t11-documentation-hygiene-skill-hardening.md)
- [T2-ARCH: Narration Deepening](./plans/t2-arch-narration-deepening.md)
- [T5: Type Collapses (A3 + A6)](./plans/t5-type-collapses.md)
- [T6: MessageHistory Encapsulation](./plans/t6-messagehistory-encapsulation.md)
- [Title: Delete `LlmMessageRepository` Port, Closure-Substitute Recorder Save Seam, Update ADR-027](./plans/title-delete-llmmessagerepository-port-closure-substitute-re.md)

### `docs/reference/`

- [Data Layer Reference](./reference/data_layer.md)
- [Specification: Engine Data Schemas](./reference/data_schemas.md)
- [Specification: Player Persona System](./reference/persona_system.md)
- [Reference: Quantifier Prompt](./reference/quantifier_prompt.md)
- [Reference: System Prompt](./reference/system_prompt.md)
- [Test Support Reference](./reference/test_support.md)
- [Testing Policy](./reference/testing.md)

### `docs/system/`

- [Action Pipeline](./system/action_pipeline.md)
- [Agent System](./system/agent_system.md)
- [System: Character State & Persistence](./system/character_state.md)
- [Specification: Dashboard UI](./system/dashboard.md)
- [System: Dynamic Pseudo-Rooms](./system/dynamic_rooms.md)
- [Specification: Game Flow](./system/game_flow.md)
- [Specification: LLM Processing & Integration](./system/llm_processing.md)
- [Message Model](./system/message_model.md)
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
