# Chronicler Engine Documentation

This folder contains all documentation for the Chronicler Engine project.

## Folder Structure

### `docs/architecture/`
**System definition** - The single source of truth for the current system structure.

- `system.md` - Core architecture, module tiers, file mapping, UI specification, and change log

### `docs/system/`
**Domain documentation** - Describes specific subsystems and features.

- `dashboard.md` - HTMX web dashboard specification
- `navigation.md` - Semantic navigation system
- `narration_engine.md` - Game Master narration system
- `llm_processing.md` - LLM integration

### `docs/plans/`
**Implementation plans** - Blueprints for features (active or archived).

- `archived/` - Completed or abandoned plans
  - `hx_migration.md` - HTMX migration (superseded by architecture/system.md)
  - `tui_migration.md` - Original TUI spec (superseded)
  - `scene_quantification.md` - Deferred feature

### `docs/adr/`
**Architecture Decision Records** - Key architectural decisions with context, rationale, and consequences.

See [ADR-001](adr/adr-001-polling-for-realtime-updates.md) for example format.

### `docs/reference/`
**Reference documentation** - Data schemas, API specs, testing strategy.

- `data_schemas.md` - Map topology and character normalization
- `persona_system.md` - Player persona system
- `testing.md` - Testing strategy and LLM abstraction

### `docs/ROADMAP.md`
**Project roadmap** - Long-term vision and planned phases.

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
2. **Update architecture** - Modify `architecture/system.md` to reflect changes
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