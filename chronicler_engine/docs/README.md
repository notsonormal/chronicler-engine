# Chronicler Engine Documentation

This folder contains all documentation for the Chronicler Engine project.

## Folder Structure

<!-- AUTO-INDEX START -->
*Index last generated: 2026-05-05 23:33 UTC*

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

### `docs/architecture/`

- [Architecture Guardrails](./architecture/guardrails.md)
- [Chronicler Engine Runtime Invariants](./architecture/invariants.md)
- [Specification: Core Architecture (Modular)](./architecture/system.md)

### `docs/plans/`

- [LLM Infrastructure Improvements](./plans/llm-infrastructure-improvements.md)
- [Structured Error Taxonomy](./plans/structured-error-taxonomy.md)

### `docs/plans/archived/`

- [Plan: Raise `game_service.rs` Coverage to 80%+ via Refactoring](./plans/archived/coverage-game-service-archived.md)
- [Plan: Improve Quantifier Prompt for Movement Certainty](./plans/archived/donna-troy-hercules-warpath-2026-05-03.md)
- [Plan: Dependency-Inject LLM/Quantifier Backends into DefaultGameService](./plans/archived/drax-psylocke-gamora-2026-05-04.md)
- [Spec: Event Header Entries](./plans/archived/event-header-entries-archived.md)
- [Plan: Granular Status Phases for LLM Pipeline](./plans/archived/granular-status-phases-archived.md)
- [Implementation Plan: Marinara-Style Prompt Architecture](./plans/archived/hercules-she-hulk-doctor-fate-20260503.md)
- [Implementation Plan: Fix Gemma 4 Thinking Suffix Corruption](./plans/archived/iceman-thor-booster-gold-2026-05-04.md)
- [Implementation Plan: Isolate Slow LLM Tests](./plans/archived/lockjaw-aquaman-sam-alexander-2026-05-03.md)
- [Plan: Unify PHI Layer — Remove PhiMode::Continuation](./plans/archived/luke-cage-pantha-morbius-2026-05-03.md)
- [Plan: Unify PHI Layer — Remove PhiMode::Continuation](./plans/archived/luke-cage-pantha-morbius-20260503.md)
- [Plan: Auto-Generated Index for `chronicler_engine/docs`](./plans/archived/obsidian-doctor-mid-nite-kid-flash-2026-05-05.md)
- [Async Concurrency & Codebase Hygiene](./plans/archived/polaris-steel-sentry-2026-05-05.md)
- [Spec: Align chronicler_engine Prompts with Marinara Engine Battle-Tested Patterns](./plans/archived/prompt-alignment-with-marinara-2026-05-04.md)
- [Plan: Phase 4 — Replace std::thread::spawn with Tokio](./plans/archived/rocket-silver-surfer-orphan-2026-05-05.md)
- [Issue Tracker Implementation Plan](./plans/archived/spider-man-impulse-aquaman-2026-05-05.md)

### `docs/reference/`

- [Specification: Redmist Estate Map and Data Parsing](./reference/data_schemas.md)
- [Marinara-Engine Reference](./reference/marinara_engine.md)
- [Marinara Engine — Default System Prompt](./reference/marinara_engine_system_prompt.md)
- [Specification: Player Persona System](./reference/persona_system.md)
- [Reference: Quantifier Prompt](./reference/quantifier_prompt.md)
- [SillyTavern Chat Window Reference](./reference/sillytavern_chat_window.md)
- [SillyTavern Prompt System Reference](./reference/sillytavern_prompt_system.md)
- [Reference: Normal System Prompt](./reference/system_prompt.md)
- [Specification: Testing Strategy and Architecture](./reference/testing.md)

### `docs/system/`

- [System: Character State & Persistence](./system/character_state.md)
- [Specification: Dashboard UI](./system/dashboard.md)
- [System: Dynamic Pseudo-Rooms](./system/dynamic_rooms.md)
- [Specification: Game Flow](./system/game_flow.md)
- [Specification: LLM Processing & Integration](./system/llm_processing.md)
- [Specification: Game Master Narration System](./system/narration_engine.md)
- [Specification: Semantic Navigation](./system/navigation.md)
- [Chronicler Engine Prompt System](./system/prompt_system.md)
- [Engine Startup & Initialization](./system/startup.md)
- [Testing Strategy](./system/testing.md)
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
3. **Update all the other docs as needed* - Read  `docs/*`
3. **Implement** - Write the code
4. **Validate** - Run the full build and test suite:
   ```bash
   python build.py  # Or manually: cargo fmt && cargo clippy && cargo test
   ```
5. **Archive** - Move completed plans to `plans/archived/`

---

## Quick Reference

| Question | Answer |
|----------|--------|
| What modules exist? | `architecture/system.md` |
| How does navigation work? | `system/navigation.md` |
| What data formats are used? | `reference/data_schemas.md` |
| What's the current roadmap? | `ROADMAP.md` |
