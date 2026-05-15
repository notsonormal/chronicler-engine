# Chronicler Engine Documentation

This folder contains all documentation for the Chronicler Engine project.

## Folder Structure

<!-- AUTO-INDEX START -->
*Index last generated: 2026-05-15 22:56 UTC*

### Root files

- [Changelog](./CHANGELOG.md)
- [Chronicler Engine: Project Roadmap](./ROADMAP.md)

### `docs/adr/`

- [ADR-001: HTMX Web Dashboard Architecture](./adr/adr-001-htmx-web-dashboard.md)
- [ADR-002: Server-Sent Events for Real-Time Updates](./adr/adr-002-sse-realtime-updates.md)
- [ADR-003: Askama Template Engine for HTML Rendering](./adr/adr-003-askama-templates.md)
- [ADR-004: XML-Structured LLM Prompts](./adr/adr-004-xml-prompt-format.md)
- [ADR-005: SillyTavern-Style Layered Prompt System](./adr/adr-005-layered-prompts.md)
- [ADR-006: Quantifier-Driven Game Systems](./adr/adr-006-quantifier-systems.md)
- [ADR-007: Settings System Architecture](./adr/adr-007-settings-system.md)
- [ADR-008: SQLite Snapshot Persistence](./adr/adr-008-sqlite-snapshot-persistence.md)
- [ADR-009: Agent Trait and Registry Architecture](./adr/adr-009-agent-trait-registry.md)
- [ADR-010: Concurrency and Generation Gate Model](./adr/adr-010-concurrency-generation-gate.md)
- [ADR-011: Text Check Integration](./adr/adr-011-text-check-integration.md)
- [ADR-012: Turn + Swipe Domain Model](./adr/adr-012-turn-swipe-model.md)
- [ADR-013: LLM Call Logging and Forensics](./adr/adr-013-llm-message-logging.md)
- [ADR-014: Message Domain Model](./adr/adr-014-message-swipe-model.md)

### `docs/architecture/`

- [Architecture Guardrails](./architecture/guardrails.md)
- [Chronicler Engine Runtime Invariants](./architecture/invariants.md)
- [Specification: Core Architecture (Modular)](./architecture/system.md)

### `docs/diagnostics/`

- [Error Catalog](./diagnostics/error_catalog.md)

### `docs/external_applications/`

- [Marinara-Engine Reference](./external_applications/marinara_engine.md)
- [Marinara Engine — Default System Prompt](./external_applications/marinara_engine_system_prompt.md)
- [SillyTavern Chat Window Reference](./external_applications/sillytavern_chat_window.md)
- [SillyTavern Prompt System Reference](./external_applications/sillytavern_prompt_system.md)

### `docs/plans/`

- [Plan: Diagnostic Decision Tree as Agent Infrastructure](./plans/diagnostic-decision-tree-plan.md)
- [Implementation Plan: Fix Diagnostic Signal Quality for All 12 Scenarios](./plans/diagnostic_fixes_plan.md)
- [Plan: Fast-Fail Build & Test Localization](./plans/fast-fail-build-test-localization-plan.md)
- [LLM Infrastructure Improvements](./plans/llm-infrastructure-improvements.md)
- [Spec: Agent-Ready Pipeline Restructure for Chronicler Engine](./plans/multi-agent-architecture-overarching-spec.md)
- [Plan: Observability & Automated Forensics](./plans/observability-and-forensics-plan.md)
- [AI Steering & Guided Generation](./plans/steering-and-guided-generation.md)

### `docs/plans/archived/`

- [Implementation Plan: Restrict Message Deletion & Inline Location/Event Headers](./plans/archived/cyborg-obsidian-riri-williams.md)
- [Implementation Plan: LLM Messages Tab](./plans/archived/dagger-wiccan-martian-manhunter-20260514.md)
- [Plan: Fix Story Log Button Visibility & Text Bolding](./plans/archived/fix-story-log-buttons-and-bolding.md)
- [Implementation Plan: Redmist Estate Data Overhaul](./plans/archived/ice-winter-soldier-bobbi-morse.md)
- [Plan: Message+Swipe Storage (Marinara/SillyTavern Model)](./plans/archived/jakeem-thunder-hal-jordan-dazzler-20260515.md)
- [Plan: Migrate Chronicler Engine to Turn + Swipe Model](./plans/archived/jericho-huntress-devil-dinosaur-20260513.md)
- [Plan: Restrict Message Deletion & Rethink Location/Event Headers](./plans/archived/lightray-thor-hulk.md)
- [Implementation Plan: Test Suite Improvements](./plans/archived/test-fix-plan.md)
- [Plan: Address Code Review Findings](./plans/archived/wonder-woman-star-lord-war-machine.md)

### `docs/reference/`

- [Specification: Engine Data Schemas](./reference/data_schemas.md)
- [Specification: Player Persona System](./reference/persona_system.md)
- [Reference: Quantifier Prompt](./reference/quantifier_prompt.md)
- [Reference: Normal System Prompt](./reference/system_prompt.md)
- [Specification: Testing Strategy and Architecture](./reference/testing.md)

### `docs/reviews/`

- [Agent Scalability Assessment: Chronicler vs. Marinara](./reviews/agent-scalability-assessment.md)
- [Cross-Project Architectural Comparison: Chronicler Engine vs. Marinara Engine](./reviews/cross-project-architectural-comparison.md)
- [Architectural Review: Defensive Architecture & Invariant Enforcement](./reviews/defensive-architecture-review.md)
- [Holistic Architectural Review: Chronicler Engine](./reviews/holistic-architectural-review.md)
- [Phase 1: Domain Alignment — Findings](./reviews/holistic-review-phase1-domain-alignment.md)
- [Phase 2: Structural Forces — Findings](./reviews/holistic-review-phase2-structural-forces.md)
- [Phase 3: Evolution Stress Test — Findings](./reviews/holistic-review-phase3-evolution-stress.md)
- [Phase 4: Health Metrics — Baseline](./reviews/holistic-review-phase4-health-metrics.md)

### `docs/system/`

- [Agent System](./system/agent_system.md)
- [System: Character State & Persistence](./system/character_state.md)
- [Specification: Dashboard UI](./system/dashboard.md)
- [System: Dynamic Pseudo-Rooms](./system/dynamic_rooms.md)
- [Specification: Game Flow](./system/game_flow.md)
- [Llm Processing](./system/llm_processing.md)
- [Specification: Game Master Narration System](./system/narration_engine.md)
- [Specification: Semantic Navigation](./system/navigation.md)
- [Chronicler Engine Prompt System](./system/prompt_system.md)
- [Engine Startup & Initialization](./system/startup.md)
- [Testing Strategy](./system/testing.md)
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
| How do I run tests? | [`system/testing.md`](./system/testing.md) |
| Why was X designed this way? | [`docs/adr/`](./adr/) |
| What's the current roadmap? | [`ROADMAP.md`](./ROADMAP.md) |
